//! Platform-independent SA-MP protocol values and wire primitives.
//!
//! This crate owns the bounded Protocol bitstream. It has no connection to a
//! loaded SA-MP client, SDK callback, or native RakNet transport stream.

mod bitstream;
mod catalog;
mod encoded_bits;
mod error;
pub mod limits;
pub mod packet;
pub mod rpc;
pub mod types;
mod wire;
mod wire_io;

pub use bitstream::{BitRead, BitStream, BitStreamError, BitWrite, MAX_BIT_STREAM_BITS};
pub use catalog::{packet_name, rpc_name};
pub use encoded_bits::{EncodedBits, EncodedBitsError};
pub use error::{DecodeError, EncodeError};
pub use wire::{
    ExactBitsPolicy, ExactBytesPolicy, IncomingPacket, IncomingPacketDescriptor, IncomingRpc,
    IncomingRpcDescriptor, OutgoingPacket, OutgoingPacketDescriptor, OutgoingRpc,
    OutgoingRpcDescriptor, TerminalAlignmentPaddingPolicy, TrailingPolicy, TrailingPolicyMarker,
    WireCodec, WireDescriptor, WireKind,
};
pub use wire_io::{WireReadExt, WireWriteExt};
