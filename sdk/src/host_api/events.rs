//! Packet/RPC listener-registration `HostApi` wrappers.

use crate::{
    CallbackState, HostApi, RegisterListener, SampClientSdkDirection, SampClientSdkHookAction,
    SampClientSdkResult, SampClientSdkSubscription, Subscription, dispatch_callback, events,
};

impl HostApi {
    /// Registers a packet callback.
    ///
    /// The callback receives a view that is valid only for that invocation. Use typed packet
    /// descriptors from [`events::packet`] to decode, block, or replace a matching payload.
    pub fn on_packet<F>(
        self,
        direction: SampClientSdkDirection,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.register_listener(direction, handler, self.raw.register_packet)
    }

    /// Registers an RPC callback.
    ///
    /// The callback receives a view that is valid only for that invocation. Use typed RPC
    /// descriptors from [`events::rpc`] to decode, block, or replace a matching payload.
    pub fn on_rpc<F>(
        self,
        direction: SampClientSdkDirection,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.register_listener(direction, handler, self.raw.register_rpc)
    }

    /// Registers a packet callback that runs only for one packet ID.
    pub fn on_packet_id<F>(
        self,
        direction: SampClientSdkDirection,
        packet_id: u8,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.on_packet(direction, move |event| {
            if event.id() == packet_id {
                handler(event)
            } else {
                SampClientSdkHookAction::Continue
            }
        })
    }

    /// Registers an RPC callback that runs only for one RPC ID.
    pub fn on_rpc_id<F>(
        self,
        direction: SampClientSdkDirection,
        rpc_id: u8,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.on_rpc(direction, move |event| {
            if event.id() == rpc_id {
                handler(event)
            } else {
                SampClientSdkHookAction::Continue
            }
        })
    }

    /// Registers a packet callback that decodes one typed packet descriptor.
    ///
    /// Nonmatching packet IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_packet`] when decode failures need plugin-specific reporting.
    pub fn on_typed_packet<T, F>(
        self,
        direction: SampClientSdkDirection,
        packet: events::Packet<T>,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        T: 'static,
        F: Fn(T) -> events::RpcAction<T> + Send + Sync + 'static,
    {
        self.on_packet_id(direction, packet.id(), move |event| {
            packet
                .handle(event, &handler)
                .unwrap_or(SampClientSdkHookAction::Continue)
        })
    }

    /// Registers an RPC callback that decodes one typed RPC descriptor.
    ///
    /// Nonmatching RPC IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_rpc`] when decode failures need plugin-specific reporting.
    pub fn on_typed_rpc<T, F>(
        self,
        direction: SampClientSdkDirection,
        rpc: events::Rpc<T>,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        T: 'static,
        F: Fn(T) -> events::RpcAction<T> + Send + Sync + 'static,
    {
        self.on_rpc_id(direction, rpc.id(), move |event| {
            rpc.handle(event, &handler)
                .unwrap_or(SampClientSdkHookAction::Continue)
        })
    }

    fn register_listener<F>(
        self,
        direction: SampClientSdkDirection,
        handler: F,
        register: RegisterListener,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        let mut callback = Box::new(CallbackState {
            api: self,
            handler: Box::new(handler),
        });
        let mut raw = SampClientSdkSubscription::default();
        let result = unsafe {
            register(
                direction,
                Some(dispatch_callback),
                (&mut *callback as *mut CallbackState).cast(),
                &mut raw,
            )
        };
        if result == SampClientSdkResult::Ok {
            Ok(Subscription {
                api: self,
                raw,
                callback: Some(callback),
            })
        } else {
            Err(result)
        }
    }
}
