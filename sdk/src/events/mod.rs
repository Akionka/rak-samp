//! Typed SA-MP RPC and packet helpers modeled after MoonLoader's `samp.events`.
//!
//! Register one raw callback through [`crate::HostApi`] and invoke a matching descriptor helper
//! from it. Typed RPC catalogs live under [`rpc::incoming`] and [`rpc::outgoing`]; deferred
//! R1 exact-bit Packet catalogs live under [`packet::incoming`].

mod core;
pub mod packet;
pub mod rpc;

#[cfg(test)]
pub(crate) use core::TypedDescriptor;
pub(crate) use core::handle_protocol;
pub use core::{
    EncodedPayload, Event, EventError, IncomingPacket, IncomingRpc, MAX_ENCODED_STRING_BYTES,
    MAX_STRING32_BYTES, OutgoingPacket, OutgoingRpc, Packet, Rpc, RpcAction, Vector2, Vector3,
};

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
