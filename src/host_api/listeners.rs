//! Plugin listener registration, removal, and callback dispatch.

use super::*;
use crate::host_api::reclamation::PluginRelease;
use modkit_abi::{
    MOD_BUSY, MOD_INVALID_ARGUMENT, MOD_NOT_READY, MOD_OK, MOD_SHUTTING_DOWN, ModResult,
    SAMP_NET_ACTION_BLOCK, SAMP_NET_DIRECTION_INCOMING, SAMP_NET_DIRECTION_OUTGOING,
    SampNetEventCallbackV1, SampNetEventV1, SampReleaseCallbackV1, SubscriptionId,
};

pub(super) struct SubscriptionEntry {
    listener: ListenerHandle,
    release: Option<PluginRelease>,
}

pub(super) unsafe extern "system" fn register_packet(
    direction: SampClientSdkDirection,
    callback: Option<SampClientSdkEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
) -> SampClientSdkResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Packet,
    )
}

pub(super) unsafe extern "system" fn register_rpc(
    direction: SampClientSdkDirection,
    callback: Option<SampClientSdkEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
) -> SampClientSdkResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Rpc,
    )
}

pub(super) unsafe extern "system" fn unregister(
    subscription: SampClientSdkSubscription,
) -> SampClientSdkResult {
    if subscription.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let removed = host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&subscription.id);
    if let Some(entry) = removed {
        entry.remove_deferred();
        debug!("unregistered plugin subscription {}", subscription.id);
        SampClientSdkResult::Ok
    } else {
        chat_commands::unregister(subscription).unwrap_or(SampClientSdkResult::SubscriptionNotFound)
    }
}

pub(super) unsafe extern "system" fn unregister_and_wait(
    subscription: SampClientSdkSubscription,
) -> SampClientSdkResult {
    unregister_and_wait_with_timeout(subscription, None)
}

pub(super) fn unregister_and_wait_with_timeout(
    subscription: SampClientSdkSubscription,
    timeout: Option<Duration>,
) -> SampClientSdkResult {
    if subscription.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let listener = {
        let mut subscriptions = host()
            .subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(listener) = subscriptions.get(&subscription.id) else {
            drop(subscriptions);
            return chat_commands::unregister_and_wait(subscription, timeout)
                .unwrap_or(SampClientSdkResult::SubscriptionNotFound);
        };
        if !listener.listener.can_remove_and_wait() {
            return SampClientSdkResult::CallbackInProgress;
        }
        let Some(listener) = subscriptions.remove(&subscription.id) else {
            return SampClientSdkResult::SubscriptionNotFound;
        };
        listener
    };
    let SubscriptionEntry { listener, release } = listener;
    if let Some(timeout) = timeout {
        if let Err(listener) = listener.remove_and_wait_timeout(timeout) {
            host()
                .subscriptions
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .insert(subscription.id, SubscriptionEntry { listener, release });
            return SampClientSdkResult::TimedOut;
        }
    } else {
        listener.remove_and_wait();
    }
    if let Some(release) = release {
        release.release();
    }
    debug!(
        "unregistered plugin subscription {} and synchronized callbacks",
        subscription.id
    );
    SampClientSdkResult::Ok
}

fn register_listener(
    direction: SampClientSdkDirection,
    callback: Option<SampClientSdkEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
    kind: ListenerKind,
) -> SampClientSdkResult {
    if is_shutting_down() {
        return SampClientSdkResult::ShuttingDown;
    }
    let Some(callback) = callback else {
        return SampClientSdkResult::InvalidArgument;
    };
    if subscription.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let direction = match direction {
        SampClientSdkDirection::Incoming => Direction::Incoming,
        SampClientSdkDirection::Outgoing => Direction::Outgoing,
    };
    let user_data = user_data as usize;
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let listener = match kind {
        ListenerKind::Packet => runtime.on_packet(direction, move |event| {
            call_plugin_callback(callback, user_data, event.id(), event.payload_mut())
        }),
        ListenerKind::Rpc => runtime.on_rpc(direction, move |event| {
            call_plugin_callback(callback, user_data, event.id(), event.payload_mut())
        }),
    };
    let listener = match listener {
        Ok(listener) => listener,
        Err(crate::ListenerRegistrationError::IdExhausted) => return SampClientSdkResult::Busy,
    };

    let Some(id) = next_subscription_id() else {
        listener.remove();
        return SampClientSdkResult::Busy;
    };
    host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(
            id,
            SubscriptionEntry {
                listener,
                release: None,
            },
        );
    unsafe { subscription.write(SampClientSdkSubscription { id }) };
    debug!("registered {kind:?} subscription {id}");
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn register_modkit_packet(
    direction: u32,
    callback: Option<SampNetEventCallbackV1>,
    user_data: *mut c_void,
    release: Option<SampReleaseCallbackV1>,
    subscription: *mut SubscriptionId,
) -> ModResult {
    unsafe {
        register_modkit_listener(
            direction,
            callback,
            user_data,
            release,
            subscription,
            ListenerKind::Packet,
        )
    }
}

pub(super) unsafe extern "system" fn register_modkit_rpc(
    direction: u32,
    callback: Option<SampNetEventCallbackV1>,
    user_data: *mut c_void,
    release: Option<SampReleaseCallbackV1>,
    subscription: *mut SubscriptionId,
) -> ModResult {
    unsafe {
        register_modkit_listener(
            direction,
            callback,
            user_data,
            release,
            subscription,
            ListenerKind::Rpc,
        )
    }
}

unsafe fn register_modkit_listener(
    direction: u32,
    callback: Option<SampNetEventCallbackV1>,
    user_data: *mut c_void,
    release: Option<SampReleaseCallbackV1>,
    subscription: *mut SubscriptionId,
    kind: ListenerKind,
) -> ModResult {
    if is_shutting_down() {
        return MOD_SHUTTING_DOWN;
    }
    let (Some(callback), Some(release)) = (callback, release) else {
        return MOD_INVALID_ARGUMENT;
    };
    if subscription.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let direction = match direction {
        SAMP_NET_DIRECTION_INCOMING => Direction::Incoming,
        SAMP_NET_DIRECTION_OUTGOING => Direction::Outgoing,
        _ => return MOD_INVALID_ARGUMENT,
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    let user_data = user_data as usize;
    let listener = match kind {
        ListenerKind::Packet => runtime.on_packet(direction, move |event| {
            call_modkit_callback(callback, user_data, event.id(), event.payload_mut())
        }),
        ListenerKind::Rpc => runtime.on_rpc(direction, move |event| {
            call_modkit_callback(callback, user_data, event.id(), event.payload_mut())
        }),
    };
    let listener = match listener {
        Ok(listener) => listener,
        Err(crate::ListenerRegistrationError::IdExhausted) => return MOD_BUSY,
    };
    let Some(id) = next_subscription_id() else {
        listener.remove();
        return MOD_BUSY;
    };
    host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(
            id,
            SubscriptionEntry {
                listener,
                release: Some(PluginRelease::new(user_data, release)),
            },
        );
    unsafe { subscription.write(SubscriptionId(id)) };
    debug!("registered modkit {kind:?} subscription {id}");
    MOD_OK
}

fn call_modkit_callback(
    callback: SampNetEventCallbackV1,
    user_data: usize,
    id: u8,
    payload: &mut BitStream,
) -> HookAction {
    let mut event = AbiEvent { id, payload };
    let action = unsafe {
        callback(
            user_data as *mut c_void,
            (&mut event as *mut AbiEvent).cast::<SampNetEventV1>(),
        )
    };
    if action == SAMP_NET_ACTION_BLOCK {
        HookAction::Block
    } else {
        HookAction::Continue
    }
}

impl SubscriptionEntry {
    fn remove_deferred(self) {
        let Self { listener, release } = self;
        if let Some(release) = release {
            super::reclamation::defer(move || {
                listener.remove_and_wait();
                release.release();
            });
        }
    }
}

fn call_plugin_callback(
    callback: SampClientSdkEventCallbackV1,
    user_data: usize,
    id: u8,
    payload: &mut BitStream,
) -> HookAction {
    let mut event = AbiEvent { id, payload };
    let action = unsafe {
        callback(
            user_data as *mut c_void,
            (&mut event as *mut AbiEvent).cast::<SampClientSdkEventV1>(),
        )
    };
    match action {
        SampClientSdkHookAction::Block => HookAction::Block,
        SampClientSdkHookAction::Continue => HookAction::Continue,
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ListenerKind {
    Packet,
    Rpc,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Registry;
    use std::{
        sync::{Arc, mpsc},
        thread,
        time::Duration,
    };

    unsafe extern "system" fn release_sender(user_data: *mut c_void) {
        let sender = unsafe { Box::from_raw(user_data.cast::<mpsc::Sender<()>>()) };
        sender.send(()).unwrap();
    }

    #[test]
    fn deferred_release_waits_for_callback_drain_and_runs_once() {
        let registry = Registry::new();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (continue_tx, continue_rx) = mpsc::channel();
        let listener = registry
            .register_packet(Direction::Incoming, move |_| {
                entered_tx.send(()).unwrap();
                continue_rx.recv().unwrap();
                HookAction::Continue
            })
            .unwrap();
        let dispatch_registry = Arc::clone(&registry);
        let dispatch = thread::spawn(move || {
            let mut payload = BitStream::new();
            dispatch_registry.dispatch_packet(Direction::Incoming, 1, &mut payload);
        });
        entered_rx.recv().unwrap();

        let (released_tx, released_rx) = mpsc::channel::<()>();
        let user_data = Box::into_raw(Box::new(released_tx)).cast::<c_void>();
        SubscriptionEntry {
            listener,
            release: Some(PluginRelease::new(user_data as usize, release_sender)),
        }
        .remove_deferred();

        assert!(released_rx.recv_timeout(Duration::from_millis(20)).is_err());
        continue_tx.send(()).unwrap();
        dispatch.join().unwrap();
        released_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(released_rx.try_recv().is_err());
    }
}
