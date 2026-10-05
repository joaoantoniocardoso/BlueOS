//! Shutdown drain while IO is in flight.

mod effects_fixture;

use effects_fixture::helpers::{
    run_async_shutdown_abandon_after_five_seconds, run_async_shutdown_waits_for_in_flight_io,
    run_blocking_shutdown_waits_for_in_flight_io,
};

#[tokio::test(start_paused = true)]
async fn shutdown_waits_for_in_flight_io_before_returning() {
    run_async_shutdown_waits_for_in_flight_io().await;
}

#[tokio::test(start_paused = true)]
async fn shutdown_abandons_in_flight_io_after_five_seconds() {
    run_async_shutdown_abandon_after_five_seconds().await;
}

#[tokio::test(start_paused = true)]
async fn shutdown_waits_for_in_flight_blocking_io_before_returning() {
    run_blocking_shutdown_waits_for_in_flight_io().await;
}
