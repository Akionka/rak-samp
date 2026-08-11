//! Plugin listener registration, removal, and callback dispatch.

use super::*;

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
        .remove(&subscription.id)
        .is_some();
    if removed {
        debug!("unregistered plugin subscription {}", subscription.id);
        SampClientSdkResult::Ok
    } else {
        chat_commands::unregister(subscription).unwrap_or(SampClientSdkResult::SubscriptionNotFound)
    }
}

pub(super) unsafe extern "system" fn unregister_and_wait(
    subscription: SampClientSdkSubscription,
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
            return chat_commands::unregister_and_wait(subscription)
                .unwrap_or(SampClientSdkResult::SubscriptionNotFound);
        };
        if !listener.can_remove_and_wait() {
            return SampClientSdkResult::CallbackInProgress;
        }
        let Some(listener) = subscriptions.remove(&subscription.id) else {
            return SampClientSdkResult::SubscriptionNotFound;
        };
        listener
    };
    listener.remove_and_wait();
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

    let id = host().next_subscription.fetch_add(1, Ordering::AcqRel);
    host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(id, listener);
    unsafe { subscription.write(SampClientSdkSubscription { id }) };
    debug!("registered {kind:?} subscription {id}");
    SampClientSdkResult::Ok
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
