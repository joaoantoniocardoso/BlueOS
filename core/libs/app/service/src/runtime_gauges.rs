//! The numbers Tokio keeps about the runtime a Service runs on (D-35), as gauges in the Service's metrics.

use metrics::Gauge;
use tokio::runtime::Handle;

use crate::metrics_registry::MetricsRegistry;

/// How many worker threads the runtime has.
const WORKERS: &str = "tokio_workers";
/// How many tasks the runtime has spawned that have not finished.
const ALIVE_TASKS: &str = "tokio_alive_tasks";
/// How many tasks wait in the runtime's global queue.
const GLOBAL_QUEUE_DEPTH: &str = "tokio_global_queue_depth";

/// The gauges of the runtime the Kernel runs on. Only values that stay still while a Service is idle are here, so an
/// idle Service does not republish its `metrics` (D-35).
pub(crate) struct RuntimeGauges {
    handle: Handle,
    workers: Gauge,
    alive_tasks: Gauge,
    global_queue_depth: Gauge,
}

impl RuntimeGauges {
    /// The gauges of the runtime this is called on, recorded in `registry`.
    pub(crate) fn new(registry: &MetricsRegistry) -> Self {
        metrics::with_local_recorder(registry, || Self {
            handle: Handle::current(),
            workers: metrics::gauge!(WORKERS),
            alive_tasks: metrics::gauge!(ALIVE_TASKS),
            global_queue_depth: metrics::gauge!(GLOBAL_QUEUE_DEPTH),
        })
    }

    /// Reads the runtime's numbers into the gauges.
    pub(crate) fn sample(&self) {
        let runtime = self.handle.metrics();
        self.workers.set(runtime.num_workers() as f64);
        self.alive_tasks.set(runtime.num_alive_tasks() as f64);
        self.global_queue_depth
            .set(runtime.global_queue_depth() as f64);
    }
}
