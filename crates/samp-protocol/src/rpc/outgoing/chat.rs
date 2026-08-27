//! Outgoing chat and slash-command RPC descriptors.
//!
//! Built-ins are named without exposing their private codec implementation:
//!
//! ```
//! use samp_protocol::{WireDescriptor, rpc::outgoing::chat::SendChat};
//!
//! assert_eq!(SendChat::ID, 101);
//! ```

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, WireCodec, WireReadExt,
    WireWriteExt,
};

crate::wire::nominal_descriptor!(
    outgoing rpc,
    SendChat,
    SEND_CHAT,
    101,
    String8,
    Vec<u8>,
    ExactBytesPolicy
);
crate::wire::nominal_descriptor!(
    outgoing rpc,
    SendCommand,
    SEND_COMMAND,
    50,
    String32<4096>,
    Vec<u8>,
    ExactBytesPolicy
);

/// A byte string with an unsigned 8-bit byte-length prefix.
struct String8;

impl WireCodec for String8 {
    type Value = Vec<u8>;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        WireReadExt::read_len_prefixed_bytes_u8(reader, usize::from(u8::MAX))
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        WireWriteExt::write_len_prefixed_bytes_u8(writer, value, usize::from(u8::MAX))
    }
}

/// A byte string with an unsigned little-endian 32-bit byte-length prefix.
struct String32<const LIMIT: usize>;

impl<const LIMIT: usize> WireCodec for String32<LIMIT> {
    type Value = Vec<u8>;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let limit = LIMIT.min(u32::MAX as usize);
        WireReadExt::read_len_prefixed_bytes_u32_le(reader, limit)
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        let limit = LIMIT.min(u32::MAX as usize);
        WireWriteExt::write_len_prefixed_bytes_u32_le(writer, value, limit)
    }
}
