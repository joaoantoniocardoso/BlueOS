//! In-process Projections: pure functions of the Snapshot on deduplicated typed `watch` receivers (D-27).

use std::sync::Arc;

use tokio::sync::watch;

use blueos_domain::Domain;

type ProjectValue<D, T> = Arc<dyn Fn(&<D as Domain>::Snapshot) -> T + Send + Sync>;

/// Refreshes one Projection from the current Snapshot.
pub(crate) type RefreshProjection<D> = Box<dyn Fn(&<D as Domain>::Snapshot) + Send + Sync>;

/// A typed Projection declared in `build`. Clone it into the service Context for Tasks.
pub struct Projection<T> {
    receiver: watch::Receiver<T>,
}

pub(crate) struct ProjectionRegistry<D: Domain> {
    projections: Vec<RefreshProjection<D>>,
}

impl<T> Clone for Projection<T> {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Projection<T> {
    /// Subscribes to updates. Unchanged values are not redelivered.
    pub fn subscribe(&self) -> watch::Receiver<T> {
        self.receiver.clone()
    }
}

impl<D: Domain> ProjectionRegistry<D> {
    pub(crate) fn new(projections: Vec<RefreshProjection<D>>) -> Self {
        Self { projections }
    }

    pub(crate) fn refresh(&self, snapshot: &D::Snapshot) {
        for projection in &self.projections {
            projection(snapshot);
        }
    }
}

pub(crate) fn register_projection<D, T>(
    project: impl Fn(&D::Snapshot) -> T + Send + Sync + 'static,
    snapshot: &D::Snapshot,
) -> (Projection<T>, RefreshProjection<D>)
where
    D: Domain,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let project: ProjectValue<D, T> = Arc::new(project);
    let initial = project(snapshot);
    let (sender, receiver) = watch::channel(initial);
    let refresh = Box::new(move |next_snapshot: &D::Snapshot| {
        let next = project(next_snapshot);
        sender.send_if_modified(|current| {
            if *current == next {
                return false;
            }
            *current = next;
            true
        });
    });
    (Projection { receiver }, refresh)
}
