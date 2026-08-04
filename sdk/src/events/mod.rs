//! Typed SA-MP RPC and packet helpers modeled after MoonLoader's `samp.events`.
//!
//! Register one raw callback through [`crate::HostApi`] and invoke a matching descriptor helper
//! from it. Typed RPC catalogs live under [`rpc::incoming`] and [`rpc::outgoing`]; packet
//! catalogs live under [`packet::incoming`] and [`packet::outgoing`].

mod core;
pub mod packet;
pub mod rpc;

pub use core::{
    EncodedPayload, Event, EventError, MAX_ENCODED_STRING_BYTES, MAX_STRING32_BYTES, Packet, Rpc,
    RpcAction, Vector2, Vector3,
};

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
