use crate::{CommandReceipt, Subscription};
use modkit_abi::{
    ModResult, SAMP_NET_ACTION_BLOCK, SAMP_NET_ACTION_CONTINUE, SAMP_NET_DIRECTION_INCOMING,
    SAMP_NET_DIRECTION_OUTGOING, SAMP_NET_PRIORITY_HIGH, SAMP_NET_RELIABILITY_RELIABLE_ORDERED,
    SampNetEventCallbackV1, SampNetEventV1, SampNetSendOptionsV1,
};
use modkit_sdk::{Core, SampNetService};
use std::{
    ffi::c_void,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Incoming,
    Outgoing,
}

impl Direction {
    const fn raw(self) -> u32 {
        match self {
            Self::Incoming => SAMP_NET_DIRECTION_INCOMING,
            Self::Outgoing => SAMP_NET_DIRECTION_OUTGOING,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Continue,
    Block,
}

impl Action {
    const fn raw(self) -> u32 {
        match self {
            Self::Continue => SAMP_NET_ACTION_CONTINUE,
            Self::Block => SAMP_NET_ACTION_BLOCK,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendOptions {
    pub priority: u32,
    pub reliability: u32,
    pub ordering_channel: u8,
    pub timestamp: bool,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            priority: SAMP_NET_PRIORITY_HIGH,
            reliability: SAMP_NET_RELIABILITY_RELIABLE_ORDERED,
            ordering_channel: 0,
            timestamp: false,
        }
    }
}

impl From<SendOptions> for SampNetSendOptionsV1 {
    fn from(value: SendOptions) -> Self {
        Self {
            priority: value.priority,
            reliability: value.reliability,
            ordering_channel: value.ordering_channel,
            timestamp: u8::from(value.timestamp),
            reserved: [0; 2],
        }
    }
}

pub struct Event<'callback> {
    service: SampNetService,
    raw: *mut SampNetEventV1,
    callback: PhantomData<&'callback mut SampNetEventV1>,
}

impl Event<'_> {
    #[must_use]
    pub fn id(&self) -> u8 {
        unsafe { self.service.event_id(self.raw) }.unwrap_or(0)
    }

    pub fn reset_read(&mut self) -> Result<(), ModResult> {
        unsafe { self.service.event_reset(self.raw) }
    }

    #[must_use]
    pub fn remaining_bits(&self) -> usize {
        unsafe { self.service.event_remaining_bits(self.raw) }.unwrap_or(0) as usize
    }

    pub fn read_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, ModResult> {
        let mut out = vec![0; bit_len.div_ceil(u8::BITS as usize)];
        self.read_bits_into(&mut out, bit_len)?;
        Ok(out)
    }

    pub(crate) fn read_bits_into(
        &mut self,
        out: &mut [u8],
        bit_len: usize,
    ) -> Result<(), ModResult> {
        if out.len() != bit_len.div_ceil(u8::BITS as usize) {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        }
        let bit_len = u32::try_from(bit_len).map_err(|_| modkit_abi::MOD_INVALID_ARGUMENT)?;
        unsafe { self.service.event_read_bits(self.raw, out, bit_len) }
    }

    pub fn replace_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), ModResult> {
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        }
        let bit_len = u32::try_from(bit_len).map_err(|_| modkit_abi::MOD_INVALID_ARGUMENT)?;
        unsafe { self.service.event_replace_bits(self.raw, bytes, bit_len) }
    }

    pub fn read_encoded_string(&mut self, out: &mut [u8]) -> Result<usize, ModResult> {
        unsafe { self.service.event_read_encoded_string(self.raw, out) }.map(|len| len as usize)
    }

    pub(crate) fn encode_string(
        &self,
        value: &[u8],
        out: &mut [u8],
    ) -> Result<(usize, usize), ModResult> {
        self.service
            .encode_string(value, out)
            .map(|(bytes, bits)| (bytes as usize, bits as usize))
    }
}

type EventHandler = dyn for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static;

struct EventState {
    service: SampNetService,
    handler: Box<EventHandler>,
}

#[derive(Clone, Copy)]
pub struct Net {
    core: Core,
    service: SampNetService,
}

impl Net {
    pub(crate) const fn new(core: Core, service: SampNetService) -> Self {
        Self { core, service }
    }

    pub fn on_packet(
        self,
        direction: Direction,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
    ) -> Result<Subscription, ModResult> {
        self.register(direction, handler, true)
    }

    pub fn on_rpc(
        self,
        direction: Direction,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
    ) -> Result<Subscription, ModResult> {
        self.register(direction, handler, false)
    }

    pub fn on_incoming_typed_packet<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<
                crate::events::Incoming,
                crate::events::PacketKind,
            > + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Incoming,
            crate::events::PacketKind,
        >(descriptor);
        self.on_packet(Direction::Incoming, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Incoming,
                crate::events::PacketKind,
                _,
            >(&state, event, &handler)
        })
    }

    pub fn on_outgoing_typed_packet<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<
                crate::events::Outgoing,
                crate::events::PacketKind,
            > + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Outgoing,
            crate::events::PacketKind,
        >(descriptor);
        self.on_packet(Direction::Outgoing, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Outgoing,
                crate::events::PacketKind,
                _,
            >(&state, event, &handler)
        })
    }

    pub fn on_incoming_typed_rpc<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<crate::events::Incoming, crate::events::RpcKind>
            + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Incoming,
            crate::events::RpcKind,
        >(descriptor);
        self.on_rpc(Direction::Incoming, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Incoming,
                crate::events::RpcKind,
                _,
            >(&state, event, &handler)
        })
    }

    pub fn on_outgoing_typed_rpc<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<crate::events::Outgoing, crate::events::RpcKind>
            + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Outgoing,
            crate::events::RpcKind,
        >(descriptor);
        self.on_rpc(Direction::Outgoing, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Outgoing,
                crate::events::RpcKind,
                _,
            >(&state, event, &handler)
        })
    }

    fn register(
        self,
        direction: Direction,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
        packet: bool,
    ) -> Result<Subscription, ModResult> {
        let state = Box::new(EventState {
            service: self.service,
            handler: Box::new(handler),
        });
        let raw = Box::into_raw(state);
        let result = unsafe {
            if packet {
                self.service.register_packet(
                    direction.raw(),
                    dispatch_event as SampNetEventCallbackV1,
                    raw.cast::<c_void>(),
                    release_event,
                )
            } else {
                self.service.register_rpc(
                    direction.raw(),
                    dispatch_event as SampNetEventCallbackV1,
                    raw.cast::<c_void>(),
                    release_event,
                )
            }
        };
        match result {
            Ok(id) => Subscription::new(self.core, id),
            Err(error) => {
                drop(unsafe { Box::from_raw(raw) });
                Err(error)
            }
        }
    }

    pub fn send_packet(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
        options: SendOptions,
    ) -> Result<CommandReceipt, ModResult> {
        let id = self
            .service
            .submit_packet(id, bytes, bit_len, options.into())?;
        CommandReceipt::new(self.core, id)
    }

    pub fn send_rpc(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
        options: SendOptions,
    ) -> Result<CommandReceipt, ModResult> {
        let id = self
            .service
            .submit_rpc(id, bytes, bit_len, options.into())?;
        CommandReceipt::new(self.core, id)
    }

    pub fn emulate_incoming_packet(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<CommandReceipt, ModResult> {
        let id = self
            .service
            .submit_emulate_incoming_packet(id, bytes, bit_len)?;
        CommandReceipt::new(self.core, id)
    }

    pub fn emulate_incoming_rpc(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<CommandReceipt, ModResult> {
        let id = self
            .service
            .submit_emulate_incoming_rpc(id, bytes, bit_len)?;
        CommandReceipt::new(self.core, id)
    }

    pub fn encode_string(self, value: &[u8], out: &mut [u8]) -> Result<(usize, u32), ModResult> {
        self.service
            .encode_string(value, out)
            .map(|(bytes, bits)| (bytes as usize, bits))
    }

    pub fn incoming_emulation_ready(self) -> Result<bool, ModResult> {
        self.service.incoming_emulation_ready()
    }
}

unsafe extern "system" fn dispatch_event(user_data: *mut c_void, raw: *mut SampNetEventV1) -> u32 {
    if user_data.is_null() || raw.is_null() {
        return SAMP_NET_ACTION_CONTINUE;
    }
    let Some(state) = (unsafe { user_data.cast::<EventState>().as_ref() }) else {
        return SAMP_NET_ACTION_CONTINUE;
    };
    let mut event = Event {
        service: state.service,
        raw,
        callback: PhantomData,
    };
    catch_unwind(AssertUnwindSafe(|| (state.handler)(&mut event)))
        .unwrap_or(Action::Continue)
        .raw()
}

unsafe extern "system" fn release_event(user_data: *mut c_void) {
    if !user_data.is_null() {
        drop(unsafe { Box::from_raw(user_data.cast::<EventState>()) });
    }
}
