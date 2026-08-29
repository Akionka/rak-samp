//! Typed SA-MP RPC and packet helpers modeled after MoonLoader's `samp.events`.
//!
//! Register every descriptor through the same directional typed methods on [`crate::Net`].
//! Protocol-owned RPC and Packet catalogs live under [`samp_protocol::rpc`] and
//! [`samp_protocol::packet`].

mod callback;
mod core;

#[cfg(test)]
pub(crate) use callback::{
    CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
};
pub use callback::{Incoming, Outgoing, PacketKind, RpcKind, TypedCallbackDescriptor};
pub(crate) use callback::{handle as handle_typed_callback, registration as callback_registration};
pub use core::{Event, EventError, ProtocolAction};
pub(crate) use core::{handle_encoded_string_protocol, handle_protocol};

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
