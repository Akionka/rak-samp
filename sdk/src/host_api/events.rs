//! Packet/RPC listener-registration `HostApi` wrappers.

use crate::{
    CallbackState, HostApi, MAX_RAKNET_DECODED_STRING_BYTES, RegisterListener,
    SampClientSdkDirection, SampClientSdkEncodedString, SampClientSdkHookAction,
    SampClientSdkResult, SampClientSdkSubscription, Subscription, dispatch_callback, events,
};

impl HostApi {
    /// Encodes a NUL-free byte string with the current SA-MP client's RakNet compressor.
    pub fn encode_string(
        self,
        value: &[u8],
    ) -> Result<SampClientSdkEncodedString, SampClientSdkResult> {
        let capacity_bits = value
            .len()
            .checked_mul(16)
            .and_then(|bits| bits.checked_add(16))
            .ok_or(SampClientSdkResult::PayloadTooLarge)?;
        let mut bytes = vec![0_u8; capacity_bits.div_ceil(u8::BITS as usize)];
        let mut bit_len = 0;
        let result = unsafe {
            (self.raw.encode_string)(
                value.as_ptr(),
                value.len(),
                bytes.as_mut_ptr(),
                bytes.len(),
                &raw mut bit_len,
            )
        };
        if result != SampClientSdkResult::Ok {
            return Err(result);
        }
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(SampClientSdkResult::NativeCallFailed);
        }
        bytes.truncate(bit_len.div_ceil(u8::BITS as usize));
        Ok(SampClientSdkEncodedString { bytes, bit_len })
    }

    /// Decodes one native RakNet-compressed string from an owned bit stream.
    ///
    /// On success, advances `stream`'s read cursor by exactly the bits the
    /// client decoder consumed. The returned byte string has no terminating
    /// NUL and is bounded to [`MAX_RAKNET_DECODED_STRING_BYTES`]. On failure,
    /// the stream cursor is unchanged.
    pub fn decode_string(
        self,
        stream: &mut samp_protocol::BitStream,
    ) -> Result<Vec<u8>, SampClientSdkResult> {
        let mut output = vec![0_u8; MAX_RAKNET_DECODED_STRING_BYTES + 1];
        let mut output_len = 0_usize;
        let mut output_read_offset = 0_usize;
        let result = unsafe {
            (self.raw.decode_string)(
                stream.as_bytes().as_ptr(),
                stream.len_bytes(),
                stream.len_bits(),
                stream.read_offset_bits(),
                output.as_mut_ptr(),
                output.len(),
                &raw mut output_len,
                &raw mut output_read_offset,
            )
        };
        if result != SampClientSdkResult::Ok {
            return Err(result);
        }
        if output_len > MAX_RAKNET_DECODED_STRING_BYTES || output_read_offset > stream.len_bits() {
            return Err(SampClientSdkResult::NativeCallFailed);
        }
        stream
            .set_read_offset(output_read_offset)
            .map_err(|_| SampClientSdkResult::NativeCallFailed)?;
        output.truncate(output_len);
        Ok(output)
    }

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

    /// Registers an incoming packet callback that decodes one typed descriptor.
    ///
    /// Nonmatching packet IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_packet`] when decode failures need plugin-specific reporting.
    pub fn on_incoming_typed_packet<D, F>(
        self,
        packet: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: events::TypedCallbackDescriptor<events::Incoming, events::PacketKind> + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (id, state) =
            events::callback_registration::<D, events::Incoming, events::PacketKind>(packet);
        self.on_packet_id(SampClientSdkDirection::Incoming, id, move |event| {
            events::handle_typed_callback::<D, events::Incoming, events::PacketKind, _>(
                &state, event, &handler,
            )
        })
    }

    /// Registers an outgoing packet callback that decodes one typed descriptor.
    ///
    /// Nonmatching packet IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_packet`] when decode failures need plugin-specific reporting.
    pub fn on_outgoing_typed_packet<D, F>(
        self,
        packet: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: events::TypedCallbackDescriptor<events::Outgoing, events::PacketKind> + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (id, state) =
            events::callback_registration::<D, events::Outgoing, events::PacketKind>(packet);
        self.on_packet_id(SampClientSdkDirection::Outgoing, id, move |event| {
            events::handle_typed_callback::<D, events::Outgoing, events::PacketKind, _>(
                &state, event, &handler,
            )
        })
    }

    /// Registers an incoming RPC callback that decodes one typed descriptor.
    ///
    /// Nonmatching RPC IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_rpc`] when decode failures need plugin-specific reporting.
    pub fn on_incoming_typed_rpc<D, F>(
        self,
        rpc: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: events::TypedCallbackDescriptor<events::Incoming, events::RpcKind> + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (id, state) =
            events::callback_registration::<D, events::Incoming, events::RpcKind>(rpc);
        self.on_rpc_id(SampClientSdkDirection::Incoming, id, move |event| {
            events::handle_typed_callback::<D, events::Incoming, events::RpcKind, _>(
                &state, event, &handler,
            )
        })
    }

    /// Registers an outgoing RPC callback that decodes one typed descriptor.
    ///
    /// Nonmatching RPC IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_rpc`] when decode failures need plugin-specific reporting.
    pub fn on_outgoing_typed_rpc<D, F>(
        self,
        rpc: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: events::TypedCallbackDescriptor<events::Outgoing, events::RpcKind> + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (id, state) =
            events::callback_registration::<D, events::Outgoing, events::RpcKind>(rpc);
        self.on_rpc_id(SampClientSdkDirection::Outgoing, id, move |event| {
            events::handle_typed_callback::<D, events::Outgoing, events::RpcKind, _>(
                &state, event, &handler,
            )
        })
    }

    /// Registers an incoming RPC callback that decodes one Protocol-owned descriptor.
    ///
    /// Nonmatching IDs and decode failures continue without calling `handler`. Source failures
    /// retain their host status, while malformed payloads remain Protocol decode failures.
    pub fn on_incoming_protocol_rpc<D, F>(
        self,
        _descriptor: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: samp_protocol::IncomingRpcDescriptor + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        self.on_rpc_id(SampClientSdkDirection::Incoming, D::ID, move |event| {
            events::handle_protocol::<D>(event, &handler)
                .unwrap_or(SampClientSdkHookAction::Continue)
        })
    }

    /// Registers an outgoing RPC callback that decodes one Protocol-owned descriptor.
    ///
    /// Nonmatching IDs and decode failures continue without calling `handler`. Source failures
    /// retain their host status, while malformed payloads remain Protocol decode failures.
    pub fn on_outgoing_protocol_rpc<D, F>(
        self,
        _descriptor: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: samp_protocol::OutgoingRpcDescriptor + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        self.on_rpc_id(SampClientSdkDirection::Outgoing, D::ID, move |event| {
            events::handle_protocol::<D>(event, &handler)
                .unwrap_or(SampClientSdkHookAction::Continue)
        })
    }

    /// Registers an incoming Packet callback that decodes one Protocol-owned descriptor.
    ///
    /// Nonmatching IDs and decode failures continue without calling `handler`. Source failures
    /// retain their host status, while malformed payloads remain Protocol decode failures.
    pub fn on_incoming_protocol_packet<D, F>(
        self,
        _descriptor: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: samp_protocol::IncomingPacketDescriptor + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        self.on_packet_id(SampClientSdkDirection::Incoming, D::ID, move |event| {
            events::handle_protocol::<D>(event, &handler)
                .unwrap_or(SampClientSdkHookAction::Continue)
        })
    }

    /// Registers an outgoing Packet callback that decodes one Protocol-owned descriptor.
    ///
    /// Nonmatching IDs and decode failures continue without calling `handler`. Source failures
    /// retain their host status, while malformed payloads remain Protocol decode failures.
    pub fn on_outgoing_protocol_packet<D, F>(
        self,
        _descriptor: D,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        D: samp_protocol::OutgoingPacketDescriptor + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        self.on_packet_id(SampClientSdkDirection::Outgoing, D::ID, move |event| {
            events::handle_protocol::<D>(event, &handler)
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
