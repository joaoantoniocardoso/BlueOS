//! A deadlock lockbud must report, so CI can tell an analysis that ran from one that skipped every crate.

use std::sync::Mutex;

/// Locks `counter` again while its first guard is alive, which deadlocks.
pub fn double_lock(counter: &Mutex<u32>) {
    match *counter.lock().unwrap() {
        0 => {}
        _ => *counter.lock().unwrap() += 1,
    }
}
