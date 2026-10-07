//! L1 property tests: Domain and Block invariants through sequences of Commands.

#[macro_use]
extern crate proptest_state_machine;

mod common;

use core::time::Duration;

use blueos_domain::{Command, Domain, Outcome};
use proptest::prelude::*;
use proptest_state_machine::{ReferenceStateMachine, StateMachineTest};

use common::{Counter, CounterRequest, NOW, Pump, Tank, TankQuery, TankRequest, TankSnapshot};

#[derive(Clone, Debug, Eq, PartialEq)]
enum RefPump {
    Idle,
    Running,
}

struct CounterStateMachine;

#[derive(Clone, Debug)]
enum CounterTransition {
    Increment,
}

impl ReferenceStateMachine for CounterStateMachine {
    type State = u32;
    type Transition = CounterTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        Just(0u32).boxed()
    }

    fn transitions(_state: &Self::State) -> BoxedStrategy<Self::Transition> {
        Just(CounterTransition::Increment).boxed()
    }

    fn apply(state: Self::State, transition: &Self::Transition) -> Self::State {
        match transition {
            CounterTransition::Increment => state + 1,
        }
    }
}

struct CounterUnderTest {
    snapshot: u32,
}

impl StateMachineTest for CounterUnderTest {
    type SystemUnderTest = Self;
    type Reference = CounterStateMachine;

    fn init_test(
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) -> Self::SystemUnderTest {
        Self {
            snapshot: *ref_state,
        }
    }

    fn apply(
        mut state: Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        transition: CounterTransition,
    ) -> Self::SystemUnderTest {
        let decision = match transition {
            CounterTransition::Increment => Counter::handle(
                &mut state.snapshot,
                Command::Request(CounterRequest::Increment),
                NOW,
            ),
        };
        assert!(
            matches!(decision, Outcome::Applied { .. }),
            "increment must always apply, got {decision:?}"
        );
        assert_eq!(state.snapshot, *ref_state);
        state
    }

    fn check_invariants(
        state: &Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) {
        assert_eq!(state.snapshot, *ref_state);
    }
}

struct PumpStateMachine;

#[derive(Clone, Debug)]
enum PumpTransition {
    Start { run_time: Duration },
    Stop,
}

impl ReferenceStateMachine for PumpStateMachine {
    type State = RefPump;
    type Transition = PumpTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        Just(RefPump::Idle).boxed()
    }

    fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> {
        match state {
            RefPump::Idle => prop_oneof![
                2 => (1u64..=3_600).prop_map(|seconds| PumpTransition::Start {
                    run_time: Duration::from_secs(seconds),
                }),
            ]
            .boxed(),
            RefPump::Running => prop_oneof![
                1 => Just(PumpTransition::Stop),
                1 => (1u64..=3_600).prop_map(|seconds| PumpTransition::Start {
                    run_time: Duration::from_secs(seconds),
                }),
            ]
            .boxed(),
        }
    }

    fn apply(state: Self::State, transition: &Self::Transition) -> Self::State {
        match (state, transition) {
            (_, PumpTransition::Start { .. }) => RefPump::Running,
            (RefPump::Running, PumpTransition::Stop) => RefPump::Idle,
            (RefPump::Idle, PumpTransition::Stop) => RefPump::Idle,
        }
    }

    fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
        match transition {
            PumpTransition::Stop => matches!(state, RefPump::Running),
            PumpTransition::Start { .. } => true,
        }
    }
}

struct PumpUnderTest {
    pump: Pump,
}

impl StateMachineTest for PumpUnderTest {
    type SystemUnderTest = Self;
    type Reference = PumpStateMachine;

    fn init_test(
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) -> Self::SystemUnderTest {
        let pump = match ref_state {
            RefPump::Idle => Pump::Idle,
            RefPump::Running => Pump::Running {
                since: NOW.monotonic,
            },
        };
        Self { pump }
    }

    fn apply(
        mut state: Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        transition: PumpTransition,
    ) -> Self::SystemUnderTest {
        let decision = match transition {
            PumpTransition::Start { run_time } => state.pump.start(run_time, NOW),
            PumpTransition::Stop => state.pump.stop(NOW),
        };
        assert!(
            matches!(decision, Outcome::Applied { .. }),
            "transition {:?} must apply for reference {:?}, got {decision:?}",
            transition,
            ref_state
        );
        state
    }

    fn check_invariants(
        state: &Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) {
        match (ref_state, &state.pump) {
            (RefPump::Idle, Pump::Idle) => {}
            (RefPump::Running, Pump::Running { .. }) => {}
            (reference, pump) => {
                panic!(
                    "pump state {:?} does not match reference {:?}",
                    pump, reference
                );
            }
        }
    }
}

struct TankStateMachine;

#[derive(Clone, Debug)]
enum TankTransition {
    StartPump { run_time: Duration },
    StopPump,
}

impl ReferenceStateMachine for TankStateMachine {
    type State = RefPump;
    type Transition = TankTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        Just(RefPump::Idle).boxed()
    }

    fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> {
        match state {
            RefPump::Idle => prop_oneof![
                2 => (1u64..=3_600).prop_map(|seconds| TankTransition::StartPump {
                    run_time: Duration::from_secs(seconds),
                }),
            ]
            .boxed(),
            RefPump::Running => prop_oneof![
                1 => Just(TankTransition::StopPump),
                1 => (1u64..=3_600).prop_map(|seconds| TankTransition::StartPump {
                    run_time: Duration::from_secs(seconds),
                }),
            ]
            .boxed(),
        }
    }

    fn apply(state: Self::State, transition: &Self::Transition) -> Self::State {
        match (state, transition) {
            (_, TankTransition::StartPump { .. }) => RefPump::Running,
            (RefPump::Running, TankTransition::StopPump) => RefPump::Idle,
            (RefPump::Idle, TankTransition::StopPump) => RefPump::Idle,
        }
    }

    fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
        match transition {
            TankTransition::StopPump => matches!(state, RefPump::Running),
            TankTransition::StartPump { .. } => true,
        }
    }
}

struct TankUnderTest {
    snapshot: TankSnapshot,
}

impl StateMachineTest for TankUnderTest {
    type SystemUnderTest = Self;
    type Reference = TankStateMachine;

    fn init_test(
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) -> Self::SystemUnderTest {
        let pump = match ref_state {
            RefPump::Idle => Pump::Idle,
            RefPump::Running => Pump::Running {
                since: NOW.monotonic,
            },
        };
        Self {
            snapshot: TankSnapshot { pump },
        }
    }

    fn apply(
        mut state: Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        transition: TankTransition,
    ) -> Self::SystemUnderTest {
        let command = match transition {
            TankTransition::StartPump { run_time } => {
                Command::Request(TankRequest::StartPump { run_time })
            }
            TankTransition::StopPump => Command::Request(TankRequest::StopPump),
        };
        let decision = Tank::handle(&mut state.snapshot, command, NOW);
        assert!(
            matches!(decision, Outcome::Applied { .. }),
            "transition {:?} must apply for reference {:?}, got {decision:?}",
            transition,
            ref_state
        );
        state
    }

    fn check_invariants(
        state: &Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) {
        match (ref_state, &state.snapshot.pump) {
            (RefPump::Idle, Pump::Idle) => {}
            (RefPump::Running, Pump::Running { .. }) => {}
            (reference, pump) => {
                panic!(
                    "tank pump {:?} does not match reference {:?}",
                    pump, reference
                );
            }
        }
    }
}

prop_state_machine! {
    #[test]
    fn counter_domain_commands(sequential 1..32 => CounterUnderTest);
}

prop_state_machine! {
    #[test]
    fn pump_block_commands(sequential 1..32 => PumpUnderTest);
}

prop_state_machine! {
    #[test]
    fn tank_domain_commands(sequential 1..32 => TankUnderTest);
}

#[test]
fn tank_query_variants_are_constructible() {
    assert!(matches!(TankQuery::PumpRunTime, TankQuery::PumpRunTime));
}
