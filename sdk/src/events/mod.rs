//! Typed SA-MP RPC and packet helpers modeled after MoonLoader's `samp.events`.
//!
//! Register every descriptor through the same directional typed methods on [`crate::Net`]. Typed
//! RPC catalogs live under [`rpc::incoming`] and [`rpc::outgoing`]; Packet catalogs live in
//! [`samp_protocol::packet`].

mod callback;
mod core;
pub mod rpc;

#[cfg(test)]
pub(crate) use callback::{
    CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
};
pub use callback::{Incoming, Outgoing, PacketKind, RpcKind, TypedCallbackDescriptor};
pub(crate) use callback::{handle as handle_typed_callback, registration as callback_registration};
#[cfg(test)]
pub(crate) use core::TypedDescriptor;
pub(crate) use core::handle_protocol;
pub use core::{
    EncodedPayload, Event, EventError, IncomingPacket, IncomingRpc, MAX_ENCODED_STRING_BYTES,
    MAX_STRING32_BYTES, OutgoingPacket, OutgoingRpc, Packet, ProtocolAction, Rpc, Vector2, Vector3,
};

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
