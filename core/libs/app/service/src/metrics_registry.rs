//! The metrics of one Service (D-35): counters, gauges and histograms recorded through the `metrics` facade, kept in
//! process until the Kernel publishes them as the `metrics` State.

use core::{future::Future, sync::atomic::Ordering};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use metrics::{
    Counter, Gauge, Histogram, HistogramFn, Key, KeyName, Metadata, Recorder, SharedString, Unit,
    atomics::AtomicU64,
};
use tracing::warn;

use blueos_idl::msg::blueos_msgs::{
    MetricCounter, MetricGauge, MetricHistogram, MetricLabel, ServiceMetrics,
};

use crate::sync::lock_unpoisoned;

/// The upper bound of each histogram bucket. They suit durations recorded in seconds, from a microsecond to ten
/// seconds; the Message carries them, so a histogram with another range can get its own bounds later.
const BUCKET_BOUNDS: [f64; 22] = [
    0.000_001,
    0.000_002_5,
    0.000_005,
    0.000_01,
    0.000_025,
    0.000_05,
    0.000_1,
    0.000_25,
    0.000_5,
    0.001,
    0.002_5,
    0.005,
    0.01,
    0.025,
    0.05,
    0.1,
    0.25,
    0.5,
    1.0,
    2.5,
    5.0,
    10.0,
];

/// Every metric one Service recorded. Each Kernel owns one, so Kernels in one process never share values; clones
/// share the values.
#[derive(Clone, Default)]
pub(crate) struct MetricsRegistry {
    metrics: Arc<Mutex<Metrics>>,
}

/// The value of each metric, by name and labels.
#[derive(Default)]
struct Metrics {
    counters: BTreeMap<Key, Arc<AtomicU64>>,
    /// The bits of each gauge's `f64`.
    gauges: BTreeMap<Key, Arc<AtomicU64>>,
    histograms: BTreeMap<Key, Arc<Buckets>>,
}

/// One histogram's values so far.
#[derive(Default)]
struct Buckets(Mutex<Summary>);

#[derive(Default)]
struct Summary {
    count: u64,
    sum: f64,
    /// One count per entry of [`BUCKET_BOUNDS`], then the count of values above the last bound.
    bucket_counts: [u64; BUCKET_BOUNDS.len() + 1],
}

impl Recorder for MetricsRegistry {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        Counter::from_arc(registered(
            &mut lock_unpoisoned(&self.metrics).counters,
            key,
        ))
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        Gauge::from_arc(registered(&mut lock_unpoisoned(&self.metrics).gauges, key))
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        Histogram::from_arc(registered(
            &mut lock_unpoisoned(&self.metrics).histograms,
            key,
        ))
    }
}

impl MetricsRegistry {
    /// Makes this registry the process-wide recorder of the `metrics` facade, so a value recorded outside the
    /// Kernel's tasks, on a blocking thread for example, still lands in it. Only the shipped entry calls it: one
    /// process runs one Service.
    pub(crate) fn install(&self) {
        let registry = Self {
            metrics: Arc::clone(&self.metrics),
        };
        if metrics::set_global_recorder(registry).is_err() {
            warn!(
                "A metrics recorder was already installed, so values recorded off the Kernel's tasks are lost"
            );
        }
    }

    /// Runs `future` with this registry as the facade's recorder while it is polled, so what a Task or IO code
    /// records lands in its own Service's registry, even with many Kernels in one test process.
    // ponytail: a task that `future` spawns itself does not inherit the registry; in the shipped process the global
    // recorder catches it, in a test only a handle registered here (`metrics::counter!` kept in a variable) does.
    pub(crate) fn scope<F: Future>(&self, future: F) -> impl Future<Output = F::Output> + use<F> {
        let registry = Self {
            metrics: Arc::clone(&self.metrics),
        };
        let mut future = Box::pin(future);
        core::future::poll_fn(move |context| {
            let _recorder = metrics::set_default_local_recorder(&registry);
            future.as_mut().poll(context)
        })
    }

    /// Every metric recorded so far, each list sorted by name and then labels, so an unchanged registry always
    /// encodes to the same bytes.
    pub(crate) fn service_metrics(&self) -> ServiceMetrics {
        let metrics = lock_unpoisoned(&self.metrics);
        ServiceMetrics {
            counters: metrics
                .counters
                .iter()
                .map(|(key, value)| MetricCounter {
                    name: key.name().to_owned(),
                    labels: labels(key),
                    value: value.load(Ordering::Acquire),
                })
                .collect(),
            gauges: metrics
                .gauges
                .iter()
                .map(|(key, bits)| MetricGauge {
                    name: key.name().to_owned(),
                    labels: labels(key),
                    value: f64::from_bits(bits.load(Ordering::Acquire)),
                })
                .collect(),
            histograms: metrics
                .histograms
                .iter()
                .map(|(key, buckets)| {
                    let summary = lock_unpoisoned(&buckets.0);
                    MetricHistogram {
                        name: key.name().to_owned(),
                        labels: labels(key),
                        count: summary.count,
                        sum: summary.sum,
                        bucket_bounds: BUCKET_BOUNDS.to_vec(),
                        bucket_counts: summary.bucket_counts.to_vec(),
                    }
                })
                .collect(),
        }
    }
}

impl HistogramFn for Buckets {
    fn record(&self, value: f64) {
        let mut summary = lock_unpoisoned(&self.0);
        summary.count = summary.count.saturating_add(1);
        summary.sum += value;
        let bucket = BUCKET_BOUNDS.partition_point(|bound| *bound < value);
        if let Some(count) = summary.bucket_counts.get_mut(bucket) {
            *count = count.saturating_add(1);
        }
    }
}

/// The value registered under `key`, added at zero the first time.
fn registered<T: Default>(values: &mut BTreeMap<Key, Arc<T>>, key: &Key) -> Arc<T> {
    if let Some(value) = values.get(key) {
        return Arc::clone(value);
    }
    Arc::clone(values.entry(key.clone()).or_default())
}

fn labels(key: &Key) -> Vec<MetricLabel> {
    key.labels()
        .map(|label| MetricLabel {
            name: label.key().to_owned(),
            value: label.value().to_owned(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::Ordering;
    use std::{collections::BTreeMap, sync::Arc};

    use metrics::{Key, atomics::AtomicU64};

    use super::{BUCKET_BOUNDS, MetricsRegistry, registered};

    #[test]
    fn registering_a_key_again_keeps_its_value() {
        let mut values = BTreeMap::new();
        let key = Key::from_name("pings_sent");
        let first: Arc<AtomicU64> = registered(&mut values, &key);
        first.store(3, Ordering::Release);

        assert_eq!(registered(&mut values, &key).load(Ordering::Acquire), 3);
    }

    #[test]
    fn a_histogram_counts_each_value_in_the_first_bucket_whose_bound_holds_it() {
        let registry = MetricsRegistry::default();
        let histogram = metrics::with_local_recorder(&registry, || metrics::histogram!("step"));

        for value in [0.000_001, 0.000_001_5, 0.003, 11.0] {
            histogram.record(value);
        }

        let [recorded] = registry.service_metrics().histograms.try_into().unwrap();
        let mut expected_counts = vec![0; BUCKET_BOUNDS.len() + 1];
        expected_counts[0] = 1;
        expected_counts[1] = 1;
        expected_counts[11] = 1;
        expected_counts[22] = 1;
        assert_eq!(recorded.count, 4);
        assert_eq!(recorded.bucket_counts, expected_counts);
    }
}
