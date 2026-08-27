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
    /// The decoder accepts no tail or the exact structural bits needed for byte alignment.
    TerminalAlignmentPadding,
}

impl TrailingPolicy {
    fn validate<R: BitRead>(
        self,
        payload_bits: usize,
        meaningful_bits: usize,
        reader: &mut R,
    ) -> Result<(), DecodeError<R::Error>> {
        let remaining_bits = reader.remaining_bits();
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
            Self::TerminalAlignmentPadding if remaining_bits == 0 => Ok(()),
            Self::TerminalAlignmentPadding => {
                let required_bits =
                    (u8::BITS as usize - meaningful_bits % u8::BITS as usize) % u8::BITS as usize;
                if remaining_bits != required_bits {
                    return Err(DecodeError::InvalidTerminalPaddingLength {
                        remaining_bits,
                        required_bits,
                    });
                }
                reader
                    .read_left_aligned_bits(required_bits)
                    .map_err(DecodeError::Source)?;
                Ok(())
            }
        }
    }
}

/// Encodes and decodes one Protocol value through a transport-neutral bit I/O contract.
pub trait WireCodec {
    /// The Rust value carried by this codec.
    type Value;

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
        let meaningful_bits = payload_bits - reader.remaining_bits();
        Self::TRAILING_POLICY.validate(payload_bits, meaningful_bits, reader)?;
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
        if Self::TRAILING_POLICY == TrailingPolicy::ExactBytes
            && !stream.len_bits().is_multiple_of(u8::BITS as usize)
        {
            return Err(EncodeError::NonByteAlignedPayload {
                bit_len: stream.len_bits(),
            });
        }
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
    use super::TrailingPolicy;

    pub trait TrailingPolicyMarker {
        const POLICY: TrailingPolicy;
    }
    pub trait IncomingPacketDescriptor {}
    pub trait OutgoingPacketDescriptor {}
    pub trait IncomingRpcDescriptor {}
    pub trait OutgoingRpcDescriptor {}
}

/// Selects one finite trailing-bit policy for a generic descriptor.
///
/// This trait is sealed. Custom messages select one of the three provided marker types.
pub trait TrailingPolicyMarker: sealed::TrailingPolicyMarker {}

/// Selects exact meaningful-bit framing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExactBitsPolicy;

impl sealed::TrailingPolicyMarker for ExactBitsPolicy {
    const POLICY: TrailingPolicy = TrailingPolicy::ExactBits;
}

impl TrailingPolicyMarker for ExactBitsPolicy {}

/// Selects exact byte-aligned framing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExactBytesPolicy;

impl sealed::TrailingPolicyMarker for ExactBytesPolicy {
    const POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;
}

impl TrailingPolicyMarker for ExactBytesPolicy {}

/// Selects structural terminal byte-alignment padding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerminalAlignmentPaddingPolicy;

impl sealed::TrailingPolicyMarker for TerminalAlignmentPaddingPolicy {
    const POLICY: TrailingPolicy = TrailingPolicy::TerminalAlignmentPadding;
}

impl TrailingPolicyMarker for TerminalAlignmentPaddingPolicy {}

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

/// A zero-sized typed RakNet Packet descriptor with an explicit trailing policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Packet<const ID: u8, C, P>(PhantomData<(C, P)>);

impl<const ID: u8, C, P> Packet<ID, C, P> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> WireDescriptor for Packet<ID, C, P> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Packet;
    const TRAILING_POLICY: TrailingPolicy = P::POLICY;
}

/// A zero-sized typed incoming RakNet Packet descriptor with an explicit trailing policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IncomingPacket<const ID: u8, C, P>(PhantomData<(C, P)>);

impl<const ID: u8, C, P> IncomingPacket<ID, C, P> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> WireDescriptor
    for IncomingPacket<ID, C, P>
{
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Packet;
    const TRAILING_POLICY: TrailingPolicy = P::POLICY;
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> sealed::IncomingPacketDescriptor
    for IncomingPacket<ID, C, P>
{
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> IncomingPacketDescriptor
    for IncomingPacket<ID, C, P>
{
}

/// A zero-sized typed outgoing RakNet Packet descriptor with an explicit trailing policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutgoingPacket<const ID: u8, C, P>(PhantomData<(C, P)>);

impl<const ID: u8, C, P> OutgoingPacket<ID, C, P> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> WireDescriptor
    for OutgoingPacket<ID, C, P>
{
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Packet;
    const TRAILING_POLICY: TrailingPolicy = P::POLICY;
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> sealed::OutgoingPacketDescriptor
    for OutgoingPacket<ID, C, P>
{
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> OutgoingPacketDescriptor
    for OutgoingPacket<ID, C, P>
{
}

/// A zero-sized typed SA-MP RPC descriptor with an explicit trailing policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rpc<const ID: u8, C, P>(PhantomData<(C, P)>);

impl<const ID: u8, C, P> Rpc<ID, C, P> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> WireDescriptor for Rpc<ID, C, P> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = P::POLICY;
}

/// A zero-sized typed incoming SA-MP RPC descriptor with an explicit trailing policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IncomingRpc<const ID: u8, C, P>(PhantomData<(C, P)>);

impl<const ID: u8, C, P> IncomingRpc<ID, C, P> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> WireDescriptor for IncomingRpc<ID, C, P> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = P::POLICY;
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> sealed::IncomingRpcDescriptor
    for IncomingRpc<ID, C, P>
{
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> IncomingRpcDescriptor
    for IncomingRpc<ID, C, P>
{
}

/// A zero-sized typed outgoing SA-MP RPC descriptor with an explicit trailing policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutgoingRpc<const ID: u8, C, P>(PhantomData<(C, P)>);

impl<const ID: u8, C, P> OutgoingRpc<ID, C, P> {
    /// Creates this zero-sized descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> WireDescriptor for OutgoingRpc<ID, C, P> {
    type Value = C::Value;
    type Codec = C;

    const ID: u8 = ID;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = P::POLICY;
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> sealed::OutgoingRpcDescriptor
    for OutgoingRpc<ID, C, P>
{
}

impl<const ID: u8, C: WireCodec, P: TrailingPolicyMarker> OutgoingRpcDescriptor
    for OutgoingRpc<ID, C, P>
{
}

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
