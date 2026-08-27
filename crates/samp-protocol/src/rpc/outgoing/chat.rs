//! Outgoing chat and slash-command RPC codecs.

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, OutgoingRpc, WireCodec,
    WireReadExt, WireWriteExt,
};

/// The `onSendChat` RPC ID.
pub type SendChat = OutgoingRpc<101, String8, ExactBytesPolicy>;
/// The `onSendChat` descriptor.
pub const SEND_CHAT: SendChat = OutgoingRpc::new();
/// The `onSendCommand` RPC ID.
pub type SendCommand = OutgoingRpc<50, String32<4096>, ExactBytesPolicy>;
/// The `onSendCommand` descriptor.
pub const SEND_COMMAND: SendCommand = OutgoingRpc::new();

/// A byte string with an unsigned 8-bit byte-length prefix.
pub struct String8;

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
pub struct String32<const LIMIT: usize>;

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
