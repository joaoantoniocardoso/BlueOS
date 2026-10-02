//! The contract between a Service's pure logic and the Kernel that runs it.
//!
//! A [`Domain`] receives one [`Command`] at a time, changes its Snapshot and returns a [`Decision`]: domain events
//! and [`Effect`]s for the Kernel to carry out. It never does IO, never waits and never reads a clock; waiting is
//! data (an [`Effect`]) and the time is [`Now`], passed in. A Block is a plain type with its own part of the Snapshot
//! whose methods return an [`Outcome`]; the Domain lifts it into its own types with `Outcome::map`.

#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{error::Error, time::Duration};

/// The Decision of a Domain: the [`Outcome`] in the Domain's own types.
pub type Decision<D> = Outcome<
    <D as Domain>::Event,
    <D as Domain>::Tick,
    <D as Domain>::IoRequest,
    <D as Domain>::TimerKey,
>;

/// An input to a Domain, grouped by where it came from, so a client can only ever send a Request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command<Request, IoResult, Tick, ObservedFact> {
    /// Sent by a client through a Command endpoint, or by in-process code through the Kernel.
    Request(Request),
    /// Reports how an [`Effect::Io`] ended.
    IoResult(IoResult),
    /// Delivered by an [`Effect::Schedule`] when its time comes.
    Tick(Tick),
    /// Reports something a Task saw in the outside world. It carries the full current value, not a change, so
    /// handling it twice leaves the same Snapshot.
    ObservedFact(ObservedFact),
}

/// An order to the Kernel to do something the Domain cannot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect<Tick, IoRequest, TimerKey> {
    /// Runs the IO request. The IO Effects of one Decision run in order, and each one reports its own result.
    Io(IoRequest),
    /// Delivers `command` as a Tick once `after` has passed. Scheduling a key that is already armed re-arms it.
    Schedule {
        /// How long to wait, counted from the Command that returned this Effect.
        after: Duration,
        /// Names the timer, so it can be re-armed or cancelled.
        key: TimerKey,
        /// The Tick to deliver.
        command: Tick,
    },
    /// Disarms the timer with this key. Cancelling a timer that is not armed does nothing.
    Cancel(TimerKey),
}

/// What a Block or a Domain returns for one Command.
#[derive(Debug)]
pub enum Outcome<Event, Tick, IoRequest, TimerKey> {
    /// The Command was applied: domain events to publish and Effects to carry out, in order.
    Applied {
        /// Facts about the changes the Command made.
        events: Vec<Event>,
        /// Orders for the Kernel.
        effects: Vec<Effect<Tick, IoRequest, TimerKey>>,
    },
    /// The Command was invalid for the Snapshot: nothing changed and nothing is published. The Kernel sends the
    /// reason to the client with the rejection.
    Rejected {
        /// Why the Command was rejected, as the Block's or the Domain's own error type.
        reason: Box<dyn Error + Send + Sync>,
    },
}

/// The time the Kernel read from its Clock once for the Command being handled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Now {
    /// Calendar time since the Unix epoch. It can jump when the system clock is set, so never measure with it.
    pub wall: Duration,
    /// Time since the Kernel started. It never goes backwards, so measure elapsed time with it.
    pub monotonic: Duration,
}

impl<Event, Tick, IoRequest, TimerKey> Outcome<Event, Tick, IoRequest, TimerKey> {
    /// Rejects the Command. Decide before changing the Snapshot: a rejected Command changes nothing.
    pub fn reject(reason: impl Error + Send + Sync + 'static) -> Self {
        Self::Rejected {
            reason: Box::new(reason),
        }
    }

    /// Lifts a Block's Outcome into the Domain that composes it, usually by passing the Domain's enum constructors:
    /// `.map(Event::Pump, Tick::Pump, IoRequest::Pump, TimerKey::Pump)`. A type the Block leaves uninhabited is
    /// lifted with `|never| match never {}`.
    pub fn map<MappedEvent, MappedTick, MappedIoRequest, MappedTimerKey>(
        self,
        mut into_event: impl FnMut(Event) -> MappedEvent,
        mut into_tick: impl FnMut(Tick) -> MappedTick,
        mut into_io_request: impl FnMut(IoRequest) -> MappedIoRequest,
        mut into_timer_key: impl FnMut(TimerKey) -> MappedTimerKey,
    ) -> Outcome<MappedEvent, MappedTick, MappedIoRequest, MappedTimerKey> {
        match self {
            Self::Applied { events, effects } => Outcome::Applied {
                events: events.into_iter().map(&mut into_event).collect(),
                effects: effects
                    .into_iter()
                    .map(|effect| match effect {
                        Effect::Io(request) => Effect::Io(into_io_request(request)),
                        Effect::Schedule {
                            after,
                            key,
                            command,
                        } => Effect::Schedule {
                            after,
                            key: into_timer_key(key),
                            command: into_tick(command),
                        },
                        Effect::Cancel(key) => Effect::Cancel(into_timer_key(key)),
                    })
                    .collect(),
            },
            Self::Rejected { reason } => Outcome::Rejected { reason },
        }
    }
}

/// The pure logic of a Service.
///
/// A type the Domain has no use for is an uninhabited type ([`core::convert::Infallible`] or an empty enum), never
/// a placeholder, so a `match` needs no arm for it.
pub trait Domain {
    /// The data the Domain keeps between Commands. The Kernel clones it before every Command and restores the clone
    /// if the Command fails, so keep large, rarely changed parts behind an `Arc`.
    type Snapshot: Clone;
    /// The Commands a client can send.
    type Request;
    /// The Commands that report how an IO request ended.
    type IoResult;
    /// The Commands that a timer delivers.
    type Tick;
    /// The Commands that report what a Task saw.
    type ObservedFact;
    /// The domain events, published as Events after the Command is acknowledged.
    type Event;
    /// The IO the Domain asks the Kernel to run.
    type IoRequest;
    /// The names of the Domain's timers.
    type TimerKey;

    /// Applies one Command to the Snapshot and decides what happens next. A Command that is invalid for the
    /// Snapshot is rejected, never accepted and ignored.
    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        now: Now,
    ) -> Decision<Self>;
}
