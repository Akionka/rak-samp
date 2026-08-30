//! Typed SA-MP Packet and RPC callback adapters.

mod callback;
mod core;

pub use callback::{Incoming, Outgoing, PacketKind, RpcKind, TypedCallbackDescriptor};
pub(crate) use callback::{handle as handle_typed_callback, registration as callback_registration};
pub use core::{EventError, ProtocolAction};
pub(crate) use core::{handle_encoded_string_protocol, handle_protocol};
