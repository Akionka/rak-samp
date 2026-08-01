use crate::BitStream;
use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Condvar, Mutex, Weak},
    thread::{self, ThreadId},
};

/// The direction in which SA-MP traffic crosses the client boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Incoming,
    Outgoing,
}

/// The decision returned by a packet or RPC listener.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HookAction {
    /// Continue through the remaining listeners and then invoke the original client code.
    #[default]
    Continue,
    /// Stop dispatch and suppress the original packet or RPC operation.
    Block,
}

/// A packet passed to a registered listener.
pub struct PacketEvent<'a> {
    id: u8,
    payload: &'a mut BitStream,
}

impl<'a> PacketEvent<'a> {
    pub(crate) fn new(id: u8, payload: &'a mut BitStream) -> Self {
        Self { id, payload }
    }

    #[must_use]
    pub fn id(&self) -> u8 {
        self.id
    }

    #[must_use]
    pub fn payload(&self) -> &BitStream {
        self.payload
    }

    pub fn payload_mut(&mut self) -> &mut BitStream {
        self.payload
    }
}

/// An RPC passed to a registered listener.
pub struct RpcEvent<'a> {
    id: u8,
    payload: &'a mut BitStream,
}

impl<'a> RpcEvent<'a> {
    pub(crate) fn new(id: u8, payload: &'a mut BitStream) -> Self {
        Self { id, payload }
    }

    #[must_use]
    pub fn id(&self) -> u8 {
        self.id
    }

    #[must_use]
    pub fn payload(&self) -> &BitStream {
        self.payload
    }

    pub fn payload_mut(&mut self) -> &mut BitStream {
        self.payload
    }
}

/// A synchronous packet callback. Callbacks execute in registration order.
pub type PacketHandler =
    dyn for<'event> FnMut(&mut PacketEvent<'event>) -> HookAction + Send + 'static;
/// A synchronous RPC callback. Callbacks execute in registration order.
pub type RpcHandler = dyn for<'event> FnMut(&mut RpcEvent<'event>) -> HookAction + Send + 'static;

/// Identifier for a registered callback.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ListenerId(u64);

impl ListenerId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Removes its listener when dropped.
#[derive(Debug)]
pub struct ListenerHandle {
    id: ListenerId,
    registry: Weak<Registry>,
}

impl ListenerHandle {
    #[must_use]
    pub const fn id(&self) -> ListenerId {
        self.id
    }

    /// Removes this listener now. Dropping the handle has the same effect.
    pub fn remove(self) {}

    pub(crate) fn can_remove_and_wait(&self) -> bool {
        self.registry
            .upgrade()
            .is_none_or(|registry| !registry.dispatch_gate.is_owned_by_current_thread())
    }

    pub(crate) fn remove_and_wait(self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.remove(self.id);
            registry.synchronize_dispatch();
        }
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.remove(self.id);
        }
    }
}

pub(crate) struct Registry {
    dispatch_gate: DispatchGate,
    state: Mutex<RegistryState>,
}

struct DispatchGate {
    state: Mutex<DispatchGateState>,
    ready: Condvar,
}

#[derive(Default)]
struct DispatchGateState {
    owner: Option<ThreadId>,
    depth: usize,
}

struct DispatchGuard<'gate> {
    gate: &'gate DispatchGate,
}

impl DispatchGate {
    fn new() -> Self {
        Self {
            state: Mutex::new(DispatchGateState::default()),
            ready: Condvar::new(),
        }
    }

    fn enter(&self) -> DispatchGuard<'_> {
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

    fn is_owned_by_current_thread(&self) -> bool {
        let current = thread::current().id();
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .owner
            .as_ref()
            == Some(&current)
    }
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

struct RegistryState {
    next_id: u64,
    listeners: BTreeMap<ListenerId, Listener>,
}

enum Listener {
    Packet {
        direction: Direction,
        callback: Option<Box<PacketHandler>>,
    },
    Rpc {
        direction: Direction,
        callback: Option<Box<RpcHandler>>,
    },
}

impl Registry {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            dispatch_gate: DispatchGate::new(),
            state: Mutex::new(RegistryState {
                next_id: 1,
                listeners: BTreeMap::new(),
            }),
        })
    }

    pub(crate) fn register_packet(
        self: &Arc<Self>,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut PacketEvent<'event>) -> HookAction + Send + 'static,
    ) -> ListenerHandle {
        self.register(Listener::Packet {
            direction,
            callback: Some(Box::new(callback)),
        })
    }

    pub(crate) fn register_rpc(
        self: &Arc<Self>,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut RpcEvent<'event>) -> HookAction + Send + 'static,
    ) -> ListenerHandle {
        self.register(Listener::Rpc {
            direction,
            callback: Some(Box::new(callback)),
        })
    }

    pub(crate) fn dispatch_packet(
        &self,
        direction: Direction,
        id: u8,
        payload: &mut BitStream,
    ) -> HookAction {
        let _dispatch = self.dispatch_gate.enter();
        let listener_ids = self.matching_ids(direction, ListenerKind::Packet);
        for listener_id in listener_ids {
            let Some(mut callback) = self.take_packet_callback(listener_id) else {
                continue;
            };

            payload.reset_read();
            let action = catch_unwind(AssertUnwindSafe(|| {
                callback(&mut PacketEvent::new(id, payload))
            }));
            match action {
                Ok(_) => self.restore_packet_callback(listener_id, callback),
                Err(_) => self.remove(listener_id),
            }
            if matches!(action, Ok(HookAction::Block)) {
                return HookAction::Block;
            }
        }
        HookAction::Continue
    }

    pub(crate) fn has_packet_listener(&self, direction: Direction) -> bool {
        self.has_listener(direction, ListenerKind::Packet)
    }

    pub(crate) fn dispatch_rpc(
        &self,
        direction: Direction,
        id: u8,
        payload: &mut BitStream,
    ) -> HookAction {
        let _dispatch = self.dispatch_gate.enter();
        let listener_ids = self.matching_ids(direction, ListenerKind::Rpc);
        for listener_id in listener_ids {
            let Some(mut callback) = self.take_rpc_callback(listener_id) else {
                continue;
            };

            payload.reset_read();
            let action = catch_unwind(AssertUnwindSafe(|| {
                callback(&mut RpcEvent::new(id, payload))
            }));
            match action {
                Ok(_) => self.restore_rpc_callback(listener_id, callback),
                Err(_) => self.remove(listener_id),
            }
            if matches!(action, Ok(HookAction::Block)) {
                return HookAction::Block;
            }
        }
        HookAction::Continue
    }

    pub(crate) fn has_rpc_listener(&self, direction: Direction) -> bool {
        self.has_listener(direction, ListenerKind::Rpc)
    }

    fn register(self: &Arc<Self>, listener: Listener) -> ListenerHandle {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let id = ListenerId(state.next_id);
        state.next_id = state.next_id.saturating_add(1);
        state.listeners.insert(id, listener);
        ListenerHandle {
            id,
            registry: Arc::downgrade(self),
        }
    }

    fn remove(&self, id: ListenerId) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .listeners
            .remove(&id);
    }

    fn synchronize_dispatch(&self) {
        let _dispatch = self.dispatch_gate.enter();
    }

    fn matching_ids(&self, direction: Direction, kind: ListenerKind) -> Vec<ListenerId> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .listeners
            .iter()
            .filter_map(|(id, listener)| match (kind, listener) {
                (
                    ListenerKind::Packet,
                    Listener::Packet {
                        direction: candidate,
                        ..
                    },
                )
                | (
                    ListenerKind::Rpc,
                    Listener::Rpc {
                        direction: candidate,
                        ..
                    },
                ) if *candidate == direction => Some(*id),
                _ => None,
            })
            .collect()
    }

    fn has_listener(&self, direction: Direction, kind: ListenerKind) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .listeners
            .values()
            .any(|listener| {
                matches!(
                    (kind, listener),
                    (
                        ListenerKind::Packet,
                        Listener::Packet {
                            direction: candidate,
                            ..
                        },
                    )
                        | (
                            ListenerKind::Rpc,
                            Listener::Rpc {
                                direction: candidate,
                                ..
                            },
                        ) if *candidate == direction
                )
            })
    }

    fn take_packet_callback(&self, id: ListenerId) -> Option<Box<PacketHandler>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match state.listeners.get_mut(&id) {
            Some(Listener::Packet { callback, .. }) => callback.take(),
            _ => None,
        }
    }

    fn restore_packet_callback(&self, id: ListenerId, callback: Box<PacketHandler>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(Listener::Packet { callback: slot, .. }) = state.listeners.get_mut(&id) {
            *slot = Some(callback);
        }
    }

    fn take_rpc_callback(&self, id: ListenerId) -> Option<Box<RpcHandler>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match state.listeners.get_mut(&id) {
            Some(Listener::Rpc { callback, .. }) => callback.take(),
            _ => None,
        }
    }

    fn restore_rpc_callback(&self, id: ListenerId, callback: Box<RpcHandler>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(Listener::Rpc { callback: slot, .. }) = state.listeners.get_mut(&id) {
            *slot = Some(callback);
        }
    }
}

#[derive(Clone, Copy)]
enum ListenerKind {
    Packet,
    Rpc,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn dispatches_in_registration_order_and_observes_mutations() {
        let registry = Registry::new();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let first_calls = Arc::clone(&calls);
        let _first = registry.register_packet(Direction::Outgoing, move |event| {
            first_calls
                .lock()
                .unwrap()
                .push((1, event.payload().read_offset_bits()));
            event.payload_mut().write_u8(2).unwrap();
            HookAction::Continue
        });
        let second_calls = Arc::clone(&calls);
        let _second = registry.register_packet(Direction::Outgoing, move |event| {
            second_calls
                .lock()
                .unwrap()
                .push((2, event.payload().len_bytes()));
            HookAction::Continue
        });

        let mut payload = BitStream::from_bytes(vec![1]);
        assert_eq!(
            registry.dispatch_packet(Direction::Outgoing, 42, &mut payload),
            HookAction::Continue
        );
        assert_eq!(*calls.lock().unwrap(), vec![(1, 0), (2, 2)]);
    }

    #[test]
    fn blocks_after_the_first_blocking_listener() {
        let registry = Registry::new();
        let _blocker = registry.register_rpc(Direction::Incoming, |_| HookAction::Block);
        let _unreachable = registry.register_rpc(Direction::Incoming, |_| panic!("must not run"));

        let mut payload = BitStream::new();
        assert_eq!(
            registry.dispatch_rpc(Direction::Incoming, 7, &mut payload),
            HookAction::Block
        );
    }

    #[test]
    fn reports_whether_a_matching_listener_exists() {
        let registry = Registry::new();
        assert!(!registry.has_packet_listener(Direction::Incoming));
        assert!(!registry.has_rpc_listener(Direction::Outgoing));

        let _packet = registry.register_packet(Direction::Incoming, |_| HookAction::Continue);
        let _rpc = registry.register_rpc(Direction::Outgoing, |_| HookAction::Continue);

        assert!(registry.has_packet_listener(Direction::Incoming));
        assert!(!registry.has_packet_listener(Direction::Outgoing));
        assert!(registry.has_rpc_listener(Direction::Outgoing));
        assert!(!registry.has_rpc_listener(Direction::Incoming));
    }

    #[test]
    fn removes_panicking_listener_without_unwinding_ffi_dispatch() {
        let registry = Registry::new();
        let _panic = registry.register_packet(Direction::Incoming, |_| panic!("boom"));
        let successful_calls = Arc::new(Mutex::new(0));
        let successful_calls_clone = Arc::clone(&successful_calls);
        let _success = registry.register_packet(Direction::Incoming, move |_| {
            *successful_calls_clone.lock().unwrap() += 1;
            HookAction::Continue
        });

        let mut payload = BitStream::new();
        registry.dispatch_packet(Direction::Incoming, 1, &mut payload);
        registry.dispatch_packet(Direction::Incoming, 1, &mut payload);
        assert_eq!(*successful_calls.lock().unwrap(), 2);
    }

    #[test]
    fn permits_nested_dispatch_on_the_callback_thread() {
        let registry = Registry::new();
        let nested_registry = Arc::clone(&registry);
        let _emulator = registry.register_packet(Direction::Incoming, move |event| {
            if event.id() == 1 {
                let mut nested_payload = BitStream::new();
                assert_eq!(
                    nested_registry.dispatch_packet(Direction::Incoming, 2, &mut nested_payload),
                    HookAction::Continue
                );
            }
            HookAction::Continue
        });

        let observed = Arc::new(Mutex::new(Vec::new()));
        let callback_observed = Arc::clone(&observed);
        let _observer = registry.register_packet(Direction::Incoming, move |event| {
            callback_observed.lock().unwrap().push(event.id());
            HookAction::Continue
        });

        let mut payload = BitStream::new();
        assert_eq!(
            registry.dispatch_packet(Direction::Incoming, 1, &mut payload),
            HookAction::Continue
        );
        assert_eq!(*observed.lock().unwrap(), vec![2, 1]);
    }
}
