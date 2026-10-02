//! In-process Projections: pure functions of the Snapshot on deduplicated typed `watch` receivers (D-27).

use std::sync::Arc;

use tokio::sync::watch;

use blueos_domain::Domain;

type ProjectValue<D, T> = Arc<dyn Fn(&<D as Domain>::Snapshot) -> T + Send + Sync>;

/// A typed Projection declared in `build`. Clone it into the service Context for Tasks.
pub struct Projection<T> {
    receiver: watch::Receiver<T>,
}

pub(crate) trait RefreshProjection<D: Domain> {
    fn refresh(&self, snapshot: &D::Snapshot);
}

pub(crate) struct ProjectionRegistry<D: Domain> {
    projections: Vec<Box<dyn RefreshProjection<D> + Send + Sync>>,
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
    pub(crate) fn new(projections: Vec<Box<dyn RefreshProjection<D> + Send + Sync>>) -> Self {
        Self { projections }
    }

    pub(crate) fn refresh(&self, snapshot: &D::Snapshot) {
        for projection in &self.projections {
            projection.refresh(snapshot);
        }
    }
}

struct TypedRefresh<D: Domain, T> {
    project: ProjectValue<D, T>,
    sender: watch::Sender<T>,
}

impl<D, T> RefreshProjection<D> for TypedRefresh<D, T>
where
    D: Domain,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    fn refresh(&self, snapshot: &D::Snapshot) {
        let next = (self.project)(snapshot);
        self.sender.send_if_modified(|current| {
            if *current == next {
                return false;
            }
            *current = next;
            true
        });
    }
}

pub(crate) fn register_projection<D, T>(
    project: impl Fn(&D::Snapshot) -> T + Send + Sync + 'static,
    snapshot: &D::Snapshot,
) -> (Projection<T>, Box<dyn RefreshProjection<D> + Send + Sync>)
where
    D: Domain,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let project: ProjectValue<D, T> = Arc::new(project);
    let initial = project(snapshot);
    let (sender, receiver) = watch::channel(initial);
    let refresh = Box::new(TypedRefresh { project, sender });
    (Projection { receiver }, refresh)
}
