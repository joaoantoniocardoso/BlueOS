use core::time::Duration;

use tokio::time;

const STEP_DELAY_MILLIS: u64 = 5;

/// Steps mirrored from the domain job graph. The app maps [`example_pump_logic::PumpJobSpec`] here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimulatedPumpStep {
    VerifyOff,
    RampUp,
    VerifyOn,
}

/// Pretends to drive hardware (delay + success). Adapters may not import `logic/` (D-02); keep this crate IO-only.
pub async fn run_step(step: SimulatedPumpStep) -> bool {
    let _ = step;
    time::sleep(Duration::from_millis(STEP_DELAY_MILLIS)).await;
    true
}
