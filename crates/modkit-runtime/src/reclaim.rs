//! Non-blocking reclamation of dropped callback state.
//!
//! Dropping a subscription disables future callback starts without blocking
//! the caller, then defers plugin-context reclamation until all callbacks
//! already in flight have drained. Each reclaimable item carries a release
//! closure that the host invokes exactly once through this queue.

use std::{collections::VecDeque, sync::Mutex};

/// A plugin-owned context plus the release closure that frees it.
///
/// The release closure is invoked exactly once, either by [`Reclaimable::release`]
/// or by [`DeferredReclamation::run_pending`]. Calling it twice is impossible
/// because the closure is taken out of the item.
pub struct Reclaimable {
    release: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl Reclaimable {
    /// Wraps a release closure that frees plugin-owned callback state.
    pub fn new(release: impl FnOnce() + Send + 'static) -> Self {
        Self {
            release: Some(Box::new(release)),
        }
    }

    /// Invokes the release closure exactly once, consuming the item.
    pub fn release(mut self) {
        if let Some(release) = self.release.take() {
            release();
        }
    }
}

/// A non-blocking queue of reclaimable callback state.
///
/// The host enqueues dropped subscription state here and runs pending items
/// only after the associated in-flight callbacks have drained, so the release
/// closure never runs while a callback is still executing.
pub struct DeferredReclamation {
    state: Mutex<VecDeque<Reclaimable>>,
}

impl DeferredReclamation {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(VecDeque::new()),
        }
    }

    /// Queues an item for later reclamation. Never blocks.
    pub fn enqueue(&self, item: Reclaimable) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push_back(item);
    }

    /// Releases every pending item exactly once and empties the queue.
    pub fn run_pending(&self) {
        let pending = {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            std::mem::take(&mut *state)
        };
        for item in pending {
            item.release();
        }
    }

    /// Reports whether any item is awaiting reclamation.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_empty()
    }
}

impl Default for DeferredReclamation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{DeferredReclamation, Reclaimable};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn run_pending_releases_each_item_exactly_once() {
        let releases = Arc::new(AtomicUsize::new(0));
        let queue = DeferredReclamation::new();
        for _ in 0..3 {
            let releases = Arc::clone(&releases);
            queue.enqueue(Reclaimable::new(move || {
                releases.fetch_add(1, Ordering::SeqCst);
            }));
        }
        assert!(!queue.is_empty());
        queue.run_pending();
        assert!(queue.is_empty());
        assert_eq!(releases.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn release_is_exactly_once_even_when_called_directly() {
        let releases = Arc::new(AtomicUsize::new(0));
        let releases_clone = Arc::clone(&releases);
        let item = Reclaimable::new(move || {
            releases_clone.fetch_add(1, Ordering::SeqCst);
        });
        item.release();
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn enqueue_never_releases_until_run_pending() {
        let releases = Arc::new(AtomicUsize::new(0));
        let queue = DeferredReclamation::new();
        let releases_clone = Arc::clone(&releases);
        queue.enqueue(Reclaimable::new(move || {
            releases_clone.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        queue.run_pending();
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }
}
