use super::{
    Event, IncomingPacket as LegacyIncomingPacket, IncomingRpc as LegacyIncomingRpc,
    OutgoingPacket as LegacyOutgoingPacket, OutgoingRpc as LegacyOutgoingRpc, ProtocolAction,
    handle_protocol,
};
use crate::SampClientSdkHookAction;

/// Marks callbacks for messages received from the server.
pub struct Incoming;

/// Marks callbacks for messages sent to the server.
pub struct Outgoing;

/// Marks typed Packet callbacks.
pub struct PacketKind;

/// Marks typed RPC callbacks.
pub struct RpcKind;

mod private {
    use super::{Event, ProtocolAction, SampClientSdkHookAction};

    pub trait CallbackAdapter<Direction, Kind> {
        type Value;
        type State: Send + Sync + 'static;

        fn id(&self) -> u8;

        fn into_state(self) -> Self::State;

        fn handle<F>(
            state: &Self::State,
            event: &mut Event<'_>,
            handler: F,
        ) -> SampClientSdkHookAction
        where
            F: FnOnce(Self::Value) -> ProtocolAction<Self::Value>;
    }
}

/// Metadata used by the normal typed callback registration API.
///
/// The direction and kind markers keep invalid registrations from compiling. This trait is sealed;
/// custom messages implement [`samp_protocol::WireCodec`] and use a Protocol generic descriptor.
/// Its inherited `Value` type and [`Self::id`] are the only public callback metadata.
///
/// ```compile_fail
/// use samp_client_sdk::events::{Incoming, RpcKind, TypedCallbackDescriptor};
///
/// struct ExternalDescriptor;
///
/// impl TypedCallbackDescriptor<Incoming, RpcKind> for ExternalDescriptor {
///     fn id(&self) -> u8 { 1 }
/// }
/// ```
///
/// A descriptor also cannot be registered with the wrong direction or kind:
///
/// ```compile_fail
/// use samp_client_sdk::{
///     Net,
///     events::{Incoming, ProtocolAction, RpcKind, TypedCallbackDescriptor},
/// };
///
/// fn register_wrong_direction(net: Net) {
///     let descriptor = samp_protocol::rpc::incoming::SERVER_MESSAGE;
///     let _ = net.on_outgoing_typed_rpc(descriptor, |_| ProtocolAction::Continue);
/// }
/// ```
///
/// External codecs use Protocol's generic descriptors through the same normal method:
///
/// ```
/// use samp_client_sdk::{
///     Net,
///     events::{Incoming, ProtocolAction, RpcKind, TypedCallbackDescriptor},
/// };
/// use samp_protocol::{
///     BitRead, BitWrite, DecodeError, EncodeError, ExactBitsPolicy, IncomingRpc, WireCodec,
/// };
///
/// struct CustomCodec(std::rc::Rc<()>);
///
/// impl WireCodec for CustomCodec {
///     type Value = ();
///
///     fn decode<R: BitRead>(_reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
///         Ok(())
///     }
///
///     fn encode<W: BitWrite>(
///         _writer: &mut W,
///         _value: &Self::Value,
///     ) -> Result<(), EncodeError<W::Error>> {
///         Ok(())
///     }
/// }
///
/// fn register_custom(net: Net) {
///     let descriptor = IncomingRpc::<42, CustomCodec, ExactBitsPolicy>::new();
///     assert_eq!(TypedCallbackDescriptor::<Incoming, RpcKind>::id(&descriptor), 42);
///     let _ = net.on_incoming_typed_rpc(descriptor, |_| ProtocolAction::Continue);
/// }
///
/// fn value_type_is_nameable<D>(_: D::Value)
/// where
///     D: TypedCallbackDescriptor<Incoming, RpcKind>,
/// {
/// }
/// ```
pub trait TypedCallbackDescriptor<Direction, Kind>:
    private::CallbackAdapter<Direction, Kind>
{
    /// Returns the Packet or RPC ID used for Host registration.
    fn id(&self) -> u8;
}

macro_rules! protocol_adapter {
    ($direction:ty, $kind:ty, $descriptor:path) => {
        impl<D> private::CallbackAdapter<$direction, $kind> for D
        where
            D: $descriptor,
        {
            type Value = D::Value;
            type State = ();

            fn id(&self) -> u8 {
                D::ID
            }

            fn into_state(self) -> Self::State {}

            fn handle<F>(
                _state: &Self::State,
                event: &mut Event<'_>,
                handler: F,
            ) -> SampClientSdkHookAction
            where
                F: FnOnce(Self::Value) -> ProtocolAction<Self::Value>,
            {
                handle_protocol::<D>(event, handler).unwrap_or(SampClientSdkHookAction::Continue)
            }
        }
    };
}

protocol_adapter!(
    Incoming,
    PacketKind,
    samp_protocol::IncomingPacketDescriptor
);
protocol_adapter!(
    Outgoing,
    PacketKind,
    samp_protocol::OutgoingPacketDescriptor
);
protocol_adapter!(Incoming, RpcKind, samp_protocol::IncomingRpcDescriptor);
protocol_adapter!(Outgoing, RpcKind, samp_protocol::OutgoingRpcDescriptor);

macro_rules! legacy_adapter {
    ($descriptor:ident, $direction:ty, $kind:ty) => {
        impl<T: 'static> private::CallbackAdapter<$direction, $kind> for $descriptor<T> {
            type Value = T;
            type State = Self;

            fn id(&self) -> u8 {
                (*self).id()
            }

            fn into_state(self) -> Self::State {
                self
            }

            fn handle<F>(
                state: &Self::State,
                event: &mut Event<'_>,
                handler: F,
            ) -> SampClientSdkHookAction
            where
                F: FnOnce(Self::Value) -> ProtocolAction<Self::Value>,
            {
                (*state)
                    .handle(event, handler)
                    .unwrap_or(SampClientSdkHookAction::Continue)
            }
        }
    };
}

legacy_adapter!(LegacyIncomingPacket, Incoming, PacketKind);
legacy_adapter!(LegacyOutgoingPacket, Outgoing, PacketKind);
legacy_adapter!(LegacyIncomingRpc, Incoming, RpcKind);
legacy_adapter!(LegacyOutgoingRpc, Outgoing, RpcKind);

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
) -> SampClientSdkHookAction
where
    D: TypedCallbackDescriptor<Direction, Kind>,
    F: FnOnce(D::Value) -> ProtocolAction<D::Value>,
{
    <D as private::CallbackAdapter<Direction, Kind>>::handle(state, event, handler)
}
