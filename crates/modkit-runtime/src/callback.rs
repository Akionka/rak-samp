//! Generic callback lifecycle primitives.
//!
//! These primitives are shared by every host service that dispatches plugin
//! callbacks. They track callback activity so the host can reject blocking
//! waits from inside a callback, serialize dispatch to one thread, and wait
//! for in-flight callbacks to drain before reclaiming plugin-owned state.

use std::{
    cell::Cell,
    sync::{Condvar, Mutex},
    thread::{self, ThreadId},
};

/// Marks "inside a host callback" on the current thread for wait rejection.
///
/// A plugin callback must not block for a game command, so the command ABI
/// rejects timed waits while this context is active on the current thread.
pub struct CallbackContext;

thread_local! {
    static CALLBACK_DEPTH: Cell<usize> = const { Cell::new(0) };
}

impl CallbackContext {
    /// Enters a callback context on the current thread. The returned guard
    /// leaves the context when dropped.
    pub fn enter() -> CallbackContextGuard {
        CALLBACK_DEPTH.with(|depth| depth.set(depth.get() + 1));
        CallbackContextGuard
    }

    /// Reports whether the current thread is inside a host callback.
    #[must_use]
    pub fn is_active_on_current_thread() -> bool {
        CALLBACK_DEPTH.with(|depth| depth.get() != 0)
    }
}

/// Leaves the callback context on drop.
#[must_use]
pub struct CallbackContextGuard;

impl Drop for CallbackContextGuard {
    fn drop(&mut self) {
        CALLBACK_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

/// Serializes callback dispatch to a single owner thread with reentrancy.
///
/// Only one thread may dispatch at a time; the same thread may re-enter
/// (nested dispatch). Other threads block until the owner releases the gate.
/// This is the generic form of the packet/RPC dispatch gate.
pub struct DispatchGate {
    state: Mutex<DispatchGateState>,
    ready: Condvar,
}

#[derive(Default)]
struct DispatchGateState {
    owner: Option<ThreadId>,
    depth: usize,
}

impl DispatchGate {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(DispatchGateState::default()),
            ready: Condvar::new(),
        }
    }

    /// Acquires the gate for the current thread, blocking until it is free.
    pub fn enter(&self) -> DispatchGuard<'_> {
        let current = thread::current().id();
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        loop {
            match state.owner.as_ref() {
                None => {
                    state.owner = Some(current);
                    state.depth = 1;
                    break;
                }
                Some(owner) if owner == &current => {
                    state.depth += 1;
                    break;
                }
                Some(_) => {
                    state = self
                        .ready
                        .wait(state)
                        .unwrap_or_else(|error| error.into_inner());
                }
            }
        }
        DispatchGuard { gate: self }
    }

    /// Reports whether the current thread owns the gate.
    #[must_use]
    pub fn is_owned_by_current_thread(&self) -> bool {
        let current = thread::current().id();
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .owner
            .as_ref()
            == Some(&current)
    }
}

impl Default for DispatchGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Releases the dispatch gate on drop.
#[must_use]
pub struct DispatchGuard<'gate> {
    gate: &'gate DispatchGate,
}

impl Drop for DispatchGuard<'_> {
    fn drop(&mut self) {
        let current = thread::current().id();
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        debug_assert_eq!(state.owner.as_ref(), Some(&current));
        if state.depth > 1 {
            state.depth -= 1;
        } else {
            state.depth = 0;
            state.owner = None;
            self.gate.ready.notify_one();
        }
    }
}

/// Tracks in-flight callbacks and whether new callbacks may start.
///
/// This is the generic form of the per-registration callback gate: it counts
/// active dispatches, can be disabled so no new dispatch starts, and waits
/// until every in-flight dispatch has drained.
pub struct CallbackGate {
    state: Mutex<CallbackGateState>,
    drained: Condvar,
}

#[derive(Default)]
struct CallbackGateState {
    allowed: bool,
    in_flight: usize,
}

impl CallbackGate {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(CallbackGateState {
                allowed: true,
                in_flight: 0,
            }),
            drained: Condvar::new(),
        }
    }

    /// Reports whether new callbacks may currently start.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .allowed
    }

    /// Enables or disables new callback starts. In-flight callbacks are
    /// unaffected; use [`Self::wait_until_drained`] to wait for them.
    pub fn set_allowed(&self, allowed: bool) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .allowed = allowed;
    }

    /// Starts a callback if one may start, returning a guard that counts it
    /// as in flight until dropped. Returns `None` when callbacks are disabled.
    pub fn enter(&self) -> Option<CallbackGateGuard<'_>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if !state.allowed {
            return None;
        }
        state.in_flight += 1;
        Some(CallbackGateGuard { gate: self })
    }

    /// Blocks until every in-flight callback has drained.
    pub fn wait_until_drained(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        while state.in_flight != 0 {
            state = self
                .drained
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
    }
}

impl Default for CallbackGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Counts one in-flight callback until dropped.
#[must_use]
pub struct CallbackGateGuard<'gate> {
    gate: &'gate CallbackGate,
}

impl Drop for CallbackGateGuard<'_> {
    fn drop(&mut self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        debug_assert!(state.in_flight != 0);
        state.in_flight -= 1;
        if state.in_flight == 0 {
            self.gate.drained.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CallbackContext, CallbackGate, DispatchGate};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn callback_context_tracks_nested_depth() {
        assert!(!CallbackContext::is_active_on_current_thread());
        let outer = CallbackContext::enter();
        assert!(CallbackContext::is_active_on_current_thread());
        let inner = CallbackContext::enter();
        assert!(CallbackContext::is_active_on_current_thread());
        drop(inner);
        assert!(CallbackContext::is_active_on_current_thread());
        drop(outer);
        assert!(!CallbackContext::is_active_on_current_thread());
    }

    #[test]
    fn dispatch_gate_permits_reentrant_entry_on_the_owner_thread() {
        let gate = DispatchGate::new();
        let _outer = gate.enter();
        assert!(gate.is_owned_by_current_thread());
        let _inner = gate.enter();
        assert!(gate.is_owned_by_current_thread());
    }

    #[test]
    fn dispatch_gate_blocks_until_the_owner_releases() {
        let gate = Arc::new(DispatchGate::new());
        let owner = gate.enter();
        let gate_clone = Arc::clone(&gate);
        let handle = thread::spawn(move || {
            let _guard = gate_clone.enter();
            gate_clone.is_owned_by_current_thread()
        });
        drop(owner);
        assert!(handle.join().unwrap());
    }

    #[test]
    fn callback_gate_counts_in_flight_and_drains() {
        let gate = CallbackGate::new();
        assert!(gate.is_allowed());
        let first = gate.enter().unwrap();
        let second = gate.enter().unwrap();
        gate.set_allowed(false);
        assert!(!gate.is_allowed());
        assert!(gate.enter().is_none());
        drop(first);
        drop(second);
        gate.wait_until_drained();
    }

    #[test]
    fn callback_gate_rejects_entry_when_disabled() {
        let gate = CallbackGate::new();
        gate.set_allowed(false);
        assert!(gate.enter().is_none());
    }
}
