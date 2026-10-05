//! Service harness startup.

use core::time::Duration;
use std::path::Path;

use blueos_recorder_app::{IndexWalker, RecorderContext, RecorderService};
use blueos_service::testing::Harness;

use super::super::{install_index_walker, recorder_arguments};

pub(crate) async fn start_harness(path: &Path) -> Harness<RecorderService> {
    start_harness_with(path, |_context| {}).await
}

pub(crate) async fn start_harness_with(
    path: &Path,
    change: impl FnOnce(&mut RecorderContext),
) -> Harness<RecorderService> {
    Harness::start_with(recorder_arguments(path), change)
        .await
        .expect("harness")
}

pub(crate) async fn start_harness_with_index_walker(
    path: &Path,
    walk_timeout: Duration,
    walker: IndexWalker,
) -> Harness<RecorderService> {
    start_harness_with(path, |context| {
        install_index_walker(context, walk_timeout, walker)
    })
    .await
}
