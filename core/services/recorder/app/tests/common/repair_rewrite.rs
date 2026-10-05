//! Test doubles that wrap the recorder rewrite Port.

use core::sync::atomic::{AtomicBool, Ordering};
use std::{
    fs,
    sync::{Arc, Mutex, mpsc},
};

use tokio::sync::Notify;

use blueos_recorder_app::RecorderContext;
use blueos_service::testing::lock_unpoisoned;

/// Holds the real rewrite Port mid-flight until the rewrite is cancelled or the test ends.
pub(crate) struct HeldRewrite {
    entered: Arc<Notify>,
    release: Arc<AtomicBool>,
    returned: Arc<AtomicBool>,
}

/// Wraps the real rewrite Port so the test sets each read offset it reports before it runs the real rewrite.
pub(crate) struct SteppedRewrite {
    /// A read offset to report, or `None` to run the real rewrite.
    sender: mpsc::Sender<Option<u64>>,
    receiver: Arc<Mutex<mpsc::Receiver<Option<u64>>>>,
}

impl Drop for HeldRewrite {
    fn drop(&mut self) {
        self.release.store(true, Ordering::Relaxed);
    }
}

impl HeldRewrite {
    pub(crate) fn new() -> Self {
        Self {
            entered: Arc::new(Notify::new()),
            release: Arc::new(AtomicBool::new(false)),
            returned: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wraps the rewrite Port in `context`: it waits for a cancel or the release, then runs the real rewrite.
    pub(crate) fn wrap(&self, context: &mut RecorderContext) {
        let rewriter = Arc::clone(&context.rewriter);
        let entered = Arc::clone(&self.entered);
        let release = Arc::clone(&self.release);
        let returned = Arc::clone(&self.returned);
        context.rewriter = Arc::new(move |source, output, progress, cancel| {
            entered.notify_one();
            while !release.load(Ordering::Relaxed) && !cancel.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
            let rewritten = rewriter(source, output, progress, cancel);
            returned.store(true, Ordering::Relaxed);
            rewritten
        });
    }

    pub(crate) fn entered(&self) -> &Notify {
        &self.entered
    }

    pub(crate) fn returned(&self) -> &AtomicBool {
        &self.returned
    }
}

impl SteppedRewrite {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Wraps the rewrite Port in `context`: it reports each offset the test sends, then runs the real rewrite.
    pub(crate) fn wrap(&self, context: &mut RecorderContext) {
        let rewriter = Arc::clone(&context.rewriter);
        let receiver = Arc::clone(&self.receiver);
        context.rewriter = Arc::new(move |source, output, progress, cancel| {
            let total_bytes = fs::metadata(source).map_or(0, |metadata| metadata.len());
            while let Ok(Some(bytes_processed)) = lock_unpoisoned(&receiver).recv() {
                progress(bytes_processed, total_bytes);
            }
            rewriter(source, output, progress, cancel)
        });
    }

    pub(crate) fn report(&self, bytes_processed: u64) {
        self.sender
            .send(Some(bytes_processed))
            .expect("the rewrite is waiting for a step");
    }

    pub(crate) fn finish(&self) {
        self.sender
            .send(None)
            .expect("the rewrite is waiting for a step");
    }
}
