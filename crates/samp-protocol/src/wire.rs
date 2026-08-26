use core::marker::PhantomData;

use crate::{
    BitRead, BitStream, BitStreamError, BitWrite, DecodeError, EncodeError, EncodedBits,
    EncodedBitsError,
};

/// Distinguishes the independent SA-MP Packet and RPC ID namespaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireKind {
    /// A RakNet packet descriptor.
    Packet,
    /// An SA-MP RPC descriptor.
    Rpc,
}

/// Defines how a decoded wire message must finish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrailingPolicy {
    /// The codec may finish on any bit boundary, but no bits may remain.
    ExactBits,
    /// The complete payload must be byte-aligned and fully consumed.
    ExactBytes,
    /// The codec may leave fewer than eight terminal alignment bits unread.
    TerminalAlignmentPadding,
}

impl TrailingPolicy {
    fn validate<E>(self, payload_bits: usize, remaining_bits: usize) -> Result<(), DecodeError<E>> {
        match self {
            Self::ExactBits if remaining_bits == 0 => Ok(()),
            Self::ExactBits => Err(DecodeError::UnexpectedTrailingBits {
                remaining_bits,
                allowed_bits: 0,
            }),
            Self::ExactBytes if !payload_bits.is_multiple_of(u8::BITS as usize) => {
                Err(DecodeError::NonByteAligned {
                    bit_len: payload_bits,
                })
            }
            Self::ExactBytes if remaining_bits == 0 => Ok(()),
            Self::ExactBytes => Err(DecodeError::UnexpectedTrailingBits {
                remaining_bits,
                allowed_bits: 0,
            }),
            Self::TerminalAlignmentPadding if remaining_bits < u8::BITS as usize => Ok(()),
            Self::TerminalAlignmentPadding => Err(DecodeError::UnexpectedTrailingBits {
                remaining_bits,
                allowed_bits: u8::BITS as usize - 1,
            }),
        }
    }
}

/// Encodes and decodes one Protocol value through a transport-neutral bit I/O contract.
pub trait WireCodec {
    /// The Rust value carried by this codec.
    type Value;

    /// The required payload termination after decoding this value.
    const TRAILING_POLICY: TrailingPolicy;

    /// Decodes one value from a raw left-aligned bit reader.
    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>>;

    /// Encodes one value to a raw left-aligned bit writer.
    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>>;
}

/// Describes one typed Protocol Packet or RPC without callback lifecycle behavior.
pub trait WireDescriptor {
    /// The Rust value carried by this descriptor.
    type Value;
    /// The statically dispatched codec for this descriptor.
    type Codec: WireCodec<Value = Self::Value>;

    /// The raw Packet or RPC ID.
    const ID: u8;
    /// The independent ID namespace for [`Self::ID`].
    const KIND: WireKind;
    /// The required trailing-bit validation for this descriptor.
    const TRAILING_POLICY: TrailingPolicy;

    /// Decodes a value from any compatible reader and validates its trailing bits.
    fn decode_from<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let payload_bits = reader.remaining_bits();
        let value = Self::Codec::decode(reader)?;
        Self::TRAILING_POLICY.validate(payload_bits, reader.remaining_bits())?;
        Ok(value)
    }

    /// Encodes a value to any compatible writer without erasing its source error type.
    fn encode_to<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        Self::Codec::encode(writer, value)
    }

    /// Encodes a value as a cursor-free canonical exact-bit payload.
    fn encode_bits(value: &Self::Value) -> Result<EncodedBits, EncodeError<BitStreamError>> {
        let mut stream = BitStream::new();
        Self::encode_to(&mut stream, value)?;
        EncodedBits::from_bits(stream.as_bytes().to_vec(), stream.len_bits())
            .map_err(encode_bits_error)
    }

    /// Decodes a cursor-free exact-bit payload through the owned Protocol bitstream.
    fn decode_bits(bits: &EncodedBits) -> Result<Self::Value, DecodeError<BitStreamError>> {
        let mut stream = BitStream::from_bits(bits.as_bytes().to_vec(), bits.len_bits())
            .map_err(DecodeError::Source)?;
        Self::decode_from(&mut stream)
    }
}

mod sealed {
    pub trait IncomingPacketDescriptor {}
    pub trait OutgoingPacketDescriptor {}
    pub trait IncomingRpcDescriptor {}
    pub trait OutgoingRpcDescriptor {}
}

/// Marks a descriptor that may decode an incoming RakNet Packet callback.
///
/// ```compile_fail
/// use samp_protocol::{IncomingPacketDescriptor, packet::common::SEND_AIM_SYNC};
///
/// fn register<D: IncomingPacketDescriptor>(_: D) {}
///
/// register(SEND_AIM_SYNC);
/// ```
pub trait IncomingPacketDescriptor: WireDescriptor + sealed::IncomingPacketDescriptor {}

/// Marks a descriptor that may decode an outgoing RakNet Packet callback or encode a send.
///
/// ```compile_fail
/// use samp_protocol::{OutgoingPacketDescriptor, packet::common::AUTHENTICATION_REQUEST};
///
/// fn register<D: OutgoingPacketDescriptor>(_: D) {}
///
/// register(AUTHENTICATION_REQUEST);
/// ```
pub trait OutgoingPacketDescriptor: WireDescriptor + sealed::OutgoingPacketDescriptor {}

/// Marks a descriptor that may decode an incoming SA-MP RPC callback.
///
/// ```compile_fail
/// use samp_protocol::{IncomingRpcDescriptor, rpc::outgoing::chat::SEND_CHAT};
///
/// fn register<D: IncomingRpcDescriptor>(_: D) {}
///
/// register(SEND_CHAT);
/// ```
pub trait IncomingRpcDescriptor: WireDescriptor + sealed::IncomingRpcDescriptor {}

/// Marks a descriptor that may decode an outgoing SA-MP RPC callback or encode a send.
///
/// ```compile_fail
/// use samp_protocol::{OutgoingRpcDescriptor, rpc::incoming::SERVER_MESSAGE};
///
/// fn register<D: OutgoingRpcDescriptor>(_: D) {}
///
/// register(SERVER_MESSAGE);
/// ```
pub trait OutgoingRpcDescriptor: WireDescriptor + sealed::OutgoingRpcDescriptor {}

/// A concrete zero-sized typed RakNet Packet descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Packet<const ID: u8, C>(PhantomData<C>);

impl<const ID: u8, C> Packet<ID, C> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec> WireDescriptor for Packet<ID, C> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Packet;
    const TRAILING_POLICY: TrailingPolicy = C::TRAILING_POLICY;
}

/// A concrete zero-sized typed incoming RakNet Packet descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IncomingPacket<const ID: u8, C>(PhantomData<C>);

impl<const ID: u8, C> IncomingPacket<ID, C> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec> WireDescriptor for IncomingPacket<ID, C> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Packet;
    const TRAILING_POLICY: TrailingPolicy = C::TRAILING_POLICY;
}

impl<const ID: u8, C: WireCodec> sealed::IncomingPacketDescriptor for IncomingPacket<ID, C> {}

impl<const ID: u8, C: WireCodec> IncomingPacketDescriptor for IncomingPacket<ID, C> {}

/// A concrete zero-sized typed outgoing RakNet Packet descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutgoingPacket<const ID: u8, C>(PhantomData<C>);

impl<const ID: u8, C> OutgoingPacket<ID, C> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec> WireDescriptor for OutgoingPacket<ID, C> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Packet;
    const TRAILING_POLICY: TrailingPolicy = C::TRAILING_POLICY;
}

impl<const ID: u8, C: WireCodec> sealed::OutgoingPacketDescriptor for OutgoingPacket<ID, C> {}

impl<const ID: u8, C: WireCodec> OutgoingPacketDescriptor for OutgoingPacket<ID, C> {}

/// A concrete zero-sized typed SA-MP RPC descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rpc<const ID: u8, C>(PhantomData<C>);

impl<const ID: u8, C> Rpc<ID, C> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec> WireDescriptor for Rpc<ID, C> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = C::TRAILING_POLICY;
}

/// A concrete zero-sized typed incoming SA-MP RPC descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IncomingRpc<const ID: u8, C>(PhantomData<C>);

impl<const ID: u8, C> IncomingRpc<ID, C> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec> WireDescriptor for IncomingRpc<ID, C> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = C::TRAILING_POLICY;
}

impl<const ID: u8, C: WireCodec> sealed::IncomingRpcDescriptor for IncomingRpc<ID, C> {}

impl<const ID: u8, C: WireCodec> IncomingRpcDescriptor for IncomingRpc<ID, C> {}

/// A concrete zero-sized typed outgoing SA-MP RPC descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutgoingRpc<const ID: u8, C>(PhantomData<C>);

impl<const ID: u8, C> OutgoingRpc<ID, C> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec> WireDescriptor for OutgoingRpc<ID, C> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = C::TRAILING_POLICY;
}

impl<const ID: u8, C: WireCodec> sealed::OutgoingRpcDescriptor for OutgoingRpc<ID, C> {}

impl<const ID: u8, C: WireCodec> OutgoingRpcDescriptor for OutgoingRpc<ID, C> {}

fn encode_bits_error(error: EncodedBitsError) -> EncodeError<BitStreamError> {
    match error {
        EncodedBitsError::InvalidBitLength { bit_len, byte_len } => {
            EncodeError::InvalidBitLength { bit_len, byte_len }
        }
        EncodedBitsError::NonMinimalStorage { bit_len, byte_len } => {
            EncodeError::NonMinimalStorage { bit_len, byte_len }
        }
        EncodedBitsError::PayloadTooLarge { requested_bits } => EncodeError::PayloadTooLarge {
            requested_bits,
            limit_bits: crate::MAX_BIT_STREAM_BITS,
        },
    }
}
