use super::{ProtocolAction, handle_encoded_string_protocol, handle_protocol};
use crate::{Action, Event};

pub(crate) use super::core::CallbackFailurePhase;

pub struct Incoming;
pub struct Outgoing;
pub struct PacketKind;
pub struct RpcKind;

mod private {
    use super::{Action, CallbackFailurePhase, Event, ProtocolAction};

    pub trait CallbackAdapter<Direction, Kind> {
        type Value;
        type State: Send + Sync + 'static;

        const DIRECTION: &'static str;
        const KIND: &'static str;

        fn id(&self) -> u8;
        fn into_state(self) -> Self::State;
        fn handle<F>(
            state: &Self::State,
            event: &mut Event<'_>,
            handler: F,
        ) -> Result<Action, CallbackFailurePhase>
        where
            F: FnOnce(Self::Value) -> ProtocolAction<Self::Value>;
    }
}

pub trait TypedCallbackDescriptor<Direction, Kind>:
    private::CallbackAdapter<Direction, Kind>
{
    fn id(&self) -> u8;
}

macro_rules! protocol_adapter {
    ($direction:ty, $kind:ty, $descriptor:path, $direction_name:literal, $kind_name:literal) => {
        impl<D> private::CallbackAdapter<$direction, $kind> for D
        where
            D: $descriptor,
        {
            type Value = D::Value;
            type State = ();

            const DIRECTION: &'static str = $direction_name;
            const KIND: &'static str = $kind_name;

            fn id(&self) -> u8 {
                D::ID
            }

            fn into_state(self) -> Self::State {}

            fn handle<F>(
                _state: &Self::State,
                event: &mut Event<'_>,
                handler: F,
            ) -> Result<Action, CallbackFailurePhase>
            where
                F: FnOnce(Self::Value) -> ProtocolAction<Self::Value>,
            {
                handle_protocol::<D>(event, handler)
            }
        }
    };
}

protocol_adapter!(
    Incoming,
    PacketKind,
    samp_protocol::IncomingPacketDescriptor,
    "incoming",
    "packet"
);
protocol_adapter!(
    Outgoing,
    PacketKind,
    samp_protocol::OutgoingPacketDescriptor,
    "outgoing",
    "packet"
);
protocol_adapter!(
    Outgoing,
    RpcKind,
    samp_protocol::OutgoingRpcDescriptor,
    "outgoing",
    "rpc"
);

trait IncomingRpcCapability<D: samp_protocol::IncomingRpcDescriptor> {
    fn handle<F>(event: &mut Event<'_>, handler: F) -> Result<Action, CallbackFailurePhase>
    where
        F: FnOnce(
            <D as samp_protocol::IncomingRpcDescriptor>::Value,
        ) -> ProtocolAction<<D as samp_protocol::IncomingRpcDescriptor>::Value>;
}

impl<D> IncomingRpcCapability<D> for samp_protocol::PlainWire
where
    D: samp_protocol::IncomingRpcDescriptor<Capability = samp_protocol::PlainWire>
        + samp_protocol::WireDescriptor<Value = <D as samp_protocol::IncomingRpcDescriptor>::Value>,
{
    fn handle<F>(event: &mut Event<'_>, handler: F) -> Result<Action, CallbackFailurePhase>
    where
        F: FnOnce(
            <D as samp_protocol::IncomingRpcDescriptor>::Value,
        ) -> ProtocolAction<<D as samp_protocol::IncomingRpcDescriptor>::Value>,
    {
        handle_protocol::<D>(event, handler)
    }
}

impl<D> IncomingRpcCapability<D> for samp_protocol::EncodedStringWire
where
    D: samp_protocol::IncomingRpcDescriptor<Capability = samp_protocol::EncodedStringWire>
        + samp_protocol::EncodedStringWireDescriptor<
            Value = <D as samp_protocol::IncomingRpcDescriptor>::Value,
        >,
{
    fn handle<F>(event: &mut Event<'_>, handler: F) -> Result<Action, CallbackFailurePhase>
    where
        F: FnOnce(
            <D as samp_protocol::IncomingRpcDescriptor>::Value,
        ) -> ProtocolAction<<D as samp_protocol::IncomingRpcDescriptor>::Value>,
    {
        handle_encoded_string_protocol::<D>(event, handler)
    }
}

impl<D> private::CallbackAdapter<Incoming, RpcKind> for D
where
    D: samp_protocol::IncomingRpcDescriptor,
    D::Capability: IncomingRpcCapability<D>,
{
    type Value = <D as samp_protocol::IncomingRpcDescriptor>::Value;
    type State = ();

    const DIRECTION: &'static str = "incoming";
    const KIND: &'static str = "rpc";

    fn id(&self) -> u8 {
        <D as samp_protocol::IncomingRpcDescriptor>::ID
    }

    fn into_state(self) -> Self::State {}

    fn handle<F>(
        _state: &Self::State,
        event: &mut Event<'_>,
        handler: F,
    ) -> Result<Action, CallbackFailurePhase>
    where
        F: FnOnce(Self::Value) -> ProtocolAction<Self::Value>,
    {
        <D::Capability as IncomingRpcCapability<D>>::handle(event, handler)
    }
}

impl<D, Direction, Kind> TypedCallbackDescriptor<Direction, Kind> for D
where
    D: private::CallbackAdapter<Direction, Kind>,
{
    fn id(&self) -> u8 {
        private::CallbackAdapter::id(self)
    }
}

pub(crate) fn registration<D, Direction, Kind>(
    descriptor: D,
) -> (u8, <D as private::CallbackAdapter<Direction, Kind>>::State)
where
    D: TypedCallbackDescriptor<Direction, Kind>,
{
    let id = private::CallbackAdapter::id(&descriptor);
    let state = private::CallbackAdapter::into_state(descriptor);
    (id, state)
}

pub(crate) fn handle<D, Direction, Kind, F>(
    state: &<D as private::CallbackAdapter<Direction, Kind>>::State,
    event: &mut Event<'_>,
    handler: F,
) -> Action
where
    D: TypedCallbackDescriptor<Direction, Kind>,
    F: FnOnce(D::Value) -> ProtocolAction<D::Value>,
{
    <D as private::CallbackAdapter<Direction, Kind>>::handle(state, event, handler).unwrap_or_else(
        |phase| {
            let level = if phase == CallbackFailurePhase::DecodeMalformed {
                log::Level::Debug
            } else {
                log::Level::Warn
            };
            log::log!(
                target: "samp::typed_callback",
                level,
                "typed callback failure: direction={} kind={} id={} phase={phase:?}",
                <D as private::CallbackAdapter<Direction, Kind>>::DIRECTION,
                <D as private::CallbackAdapter<Direction, Kind>>::KIND,
                event.id(),
            );
            Action::Continue
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_preserves_descriptor_ids_for_all_routes() {
        let (incoming_packet, ()) = registration::<_, Incoming, PacketKind>(
            samp_protocol::packet::common::AuthenticationRequest,
        );
        let (outgoing_packet, ()) =
            registration::<_, Outgoing, PacketKind>(samp_protocol::packet::common::SendStatsUpdate);
        let (incoming_rpc, ()) = registration::<_, Incoming, RpcKind>(
            samp_protocol::rpc::incoming::common::ServerMessageRpc,
        );
        let (outgoing_rpc, ()) =
            registration::<_, Outgoing, RpcKind>(samp_protocol::rpc::outgoing::chat::SendChat);

        assert_eq!(incoming_packet, 12);
        assert_eq!(outgoing_packet, 205);
        assert_eq!(incoming_rpc, 93);
        assert_eq!(outgoing_rpc, 101);
    }

    #[test]
    fn outgoing_chat_descriptor_keeps_the_protocol_wire_vector() {
        let payload = <samp_protocol::rpc::outgoing::chat::SendChat as samp_protocol::WireDescriptor>::encode_bits(
            &b"hello".to_vec(),
        )
        .expect("chat payload must encode");

        assert_eq!(payload.as_bytes(), b"\x05hello");
        assert_eq!(payload.len_bits(), 48);
    }
}
