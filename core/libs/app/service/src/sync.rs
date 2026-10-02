//! Mutex helpers shared by the Kernel and supervised Tasks.

use std::sync::{Mutex, MutexGuard};

/// Locks `mutex`, or takes the guarded value if a prior holder panicked (D-29).
pub(crate) fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::lock_unpoisoned;

    #[test]
    fn poisoned_lock_does_not_block_the_next_holder() {
        let mutex = Arc::new(Mutex::new(0));
        let mutex_for_panic = Arc::clone(&mutex);
        let panic_result = std::panic::catch_unwind(move || {
            let _guard = mutex_for_panic.lock().unwrap();
            panic!("holder panicked");
        });
        assert!(panic_result.is_err());
        assert!(mutex.is_poisoned());
        *lock_unpoisoned(&mutex) = 1;
        assert_eq!(*lock_unpoisoned(&mutex), 1);
    }
}
