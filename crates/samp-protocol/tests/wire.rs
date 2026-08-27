use core::{any::type_name, mem::size_of};

use samp_protocol::{
    BitRead, BitStream, BitWrite, DecodeError, EncodeError, EncodedBits, ExactBitsPolicy,
    ExactBytesPolicy, IncomingPacket, OutgoingRpc, Packet, Rpc, TerminalAlignmentPaddingPolicy,
    TrailingPolicy, WireCodec, WireDescriptor, WireKind,
    packet::{common::SendAimSync, r1::RemotePlayerSyncPacket},
    rpc::{
        incoming::{ServerMessageRpc, fixed::AttachCameraToObject, r1::InitGameRpc},
        outgoing::{chat::SendChat, common::SendDeathNotification},
    },
};

struct ThreeBitValue;

impl WireCodec for ThreeBitValue {
    type Value = u8;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let bytes = reader
            .read_left_aligned_bits(3)
            .map_err(DecodeError::Source)?;
        Ok(bytes[0] >> 5)
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value << 5], 3)
            .map_err(EncodeError::Source)
    }
}

struct OneByteValue;

impl WireCodec for OneByteValue {
    type Value = u8;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let bytes = reader
            .read_left_aligned_bits(u8::BITS as usize)
            .map_err(DecodeError::Source)?;
        Ok(bytes[0])
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value], u8::BITS as usize)
            .map_err(EncodeError::Source)
    }
}

type ThreeBitPacket = Packet<207, ThreeBitValue, ExactBitsPolicy>;
type ThreeBitRpc = Rpc<61, ThreeBitValue, ExactBitsPolicy>;
type ExactBytePacket = Packet<1, ThreeBitValue, ExactBytesPolicy>;
type MarkerPacket = Packet<208, ThreeBitValue, TerminalAlignmentPaddingPolicy>;
type OneBytePacket = Packet<2, OneByteValue, ExactBytesPolicy>;

#[test]
fn built_in_descriptors_have_nominal_public_identities() {
    assert_eq!(
        type_name::<SendAimSync>(),
        "samp_protocol::packet::common::SendAimSync"
    );
    assert_eq!(
        type_name::<RemotePlayerSyncPacket>(),
        "samp_protocol::packet::r1::RemotePlayerSyncPacket"
    );
    assert_eq!(
        type_name::<ServerMessageRpc>(),
        "samp_protocol::rpc::incoming::fixed::ServerMessageRpc"
    );
    assert_eq!(
        type_name::<AttachCameraToObject>(),
        "samp_protocol::rpc::incoming::fixed::phase15::AttachCameraToObject"
    );
    assert_eq!(
        type_name::<InitGameRpc>(),
        "samp_protocol::rpc::incoming::r1::InitGameRpc"
    );
    assert_eq!(
        type_name::<SendDeathNotification>(),
        "samp_protocol::rpc::outgoing::common::SendDeathNotification"
    );
    assert_eq!(
        type_name::<SendChat>(),
        "samp_protocol::rpc::outgoing::chat::SendChat"
    );
}

#[test]
fn one_codec_composes_with_distinct_descriptor_id_direction_kind_and_policy() {
    type Incoming = IncomingPacket<7, ThreeBitValue, ExactBitsPolicy>;
    type Outgoing = OutgoingRpc<8, ThreeBitValue, ExactBytesPolicy>;

    assert_eq!(Incoming::ID, 7);
    assert_eq!(Incoming::KIND, WireKind::Packet);
    assert_eq!(Outgoing::ID, 8);
    assert_eq!(Outgoing::KIND, WireKind::Rpc);
    assert_eq!(Incoming::TRAILING_POLICY, TrailingPolicy::ExactBits);
    assert_eq!(Outgoing::TRAILING_POLICY, TrailingPolicy::ExactBytes);
    assert_eq!(
        Incoming::decode_bits(&Incoming::encode_bits(&5).unwrap()),
        Ok(5)
    );
    assert_eq!(
        Outgoing::encode_bits(&5),
        Err(EncodeError::NonByteAlignedPayload { bit_len: 3 })
    );
}

#[test]
fn packet_and_rpc_descriptors_are_distinct_zero_sized_wire_types() {
    assert_eq!(size_of::<ThreeBitPacket>(), 0);
    assert_eq!(size_of::<ThreeBitRpc>(), 0);
    assert_eq!(ThreeBitPacket::ID, 207);
    assert_eq!(ThreeBitRpc::ID, 61);
    assert_eq!(ThreeBitPacket::KIND, WireKind::Packet);
    assert_eq!(ThreeBitRpc::KIND, WireKind::Rpc);
    assert_eq!(ThreeBitPacket::TRAILING_POLICY, TrailingPolicy::ExactBits);
}

#[test]
fn descriptor_round_trips_a_non_byte_aligned_value_as_encoded_bits() {
    let encoded = ThreeBitPacket::encode_bits(&5).unwrap();

    assert_eq!(encoded.as_bytes(), &[0b1010_0000]);
    assert_eq!(encoded.len_bits(), 3);
    assert_eq!(ThreeBitPacket::decode_bits(&encoded), Ok(5));
}

#[test]
fn descriptors_dispatch_generically_and_keep_writer_source_errors() {
    struct RejectingWriter;

    impl BitWrite for RejectingWriter {
        type Error = &'static str;

        fn write_left_aligned_bits(&mut self, _: &[u8], _: usize) -> Result<(), Self::Error> {
            Err("writer rejected bits")
        }
    }

    let mut writer = RejectingWriter;
    assert_eq!(
        ThreeBitPacket::encode_to(&mut writer, &5),
        Err(EncodeError::Source("writer rejected bits"))
    );
}

#[test]
fn descriptors_keep_reader_source_errors() {
    struct RejectingReader;

    impl BitRead for RejectingReader {
        type Error = &'static str;

        fn remaining_bits(&self) -> usize {
            3
        }

        fn read_left_aligned_bits(&mut self, _: usize) -> Result<Vec<u8>, Self::Error> {
            Err("reader rejected bits")
        }
    }

    let mut reader = RejectingReader;
    assert_eq!(
        ThreeBitPacket::decode_from(&mut reader),
        Err(DecodeError::Source("reader rejected bits"))
    );
}

#[test]
fn exact_bit_and_exact_byte_policies_reject_their_invalid_inputs() {
    let extra_bit = EncodedBits::from_bits([0b1011_0000], 4).unwrap();
    assert_eq!(
        ThreeBitPacket::decode_bits(&extra_bit),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 1,
            allowed_bits: 0,
        })
    );

    let non_byte_aligned = EncodedBits::from_bits([0b1010_0000], 3).unwrap();
    assert_eq!(
        ExactBytePacket::decode_bits(&non_byte_aligned),
        Err(DecodeError::NonByteAligned { bit_len: 3 })
    );

    let byte_aligned = OneBytePacket::encode_bits(&0xA5).unwrap();
    assert_eq!(byte_aligned.len_bits(), 8);
    assert_eq!(OneBytePacket::decode_bits(&byte_aligned), Ok(0xA5));
}

#[test]
fn exact_byte_canonical_encoding_rejects_non_byte_aligned_codec_output() {
    let mut low_level = BitStream::new();
    ExactBytePacket::encode_to(&mut low_level, &5).unwrap();
    assert_eq!(low_level.len_bits(), 3);

    assert_eq!(
        ExactBytePacket::encode_bits(&5),
        Err(EncodeError::NonByteAlignedPayload { bit_len: 3 })
    );
}

#[test]
fn terminal_alignment_padding_requires_the_exact_structural_length() {
    let canonical = MarkerPacket::encode_bits(&5).unwrap();
    assert_eq!(canonical.len_bits(), 3);
    assert_eq!(MarkerPacket::decode_bits(&canonical), Ok(5));

    let padded = EncodedBits::from_bits([0b1011_1111], 8).unwrap();
    assert_eq!(MarkerPacket::decode_bits(&padded), Ok(5));

    let short = EncodedBits::from_bits([0b1011_1110], 7).unwrap();
    assert_eq!(
        MarkerPacket::decode_bits(&short),
        Err(DecodeError::InvalidTerminalPaddingLength {
            remaining_bits: 4,
            required_bits: 5,
        })
    );

    let long = EncodedBits::from_bits([0b1011_1111, 0], 9).unwrap();
    assert_eq!(
        MarkerPacket::decode_bits(&long),
        Err(DecodeError::InvalidTerminalPaddingLength {
            remaining_bits: 6,
            required_bits: 5,
        })
    );

    let full_byte_suffix = EncodedBits::from_bits([0b1010_0000, 0], 11).unwrap();
    assert_eq!(
        MarkerPacket::decode_bits(&full_byte_suffix),
        Err(DecodeError::InvalidTerminalPaddingLength {
            remaining_bits: 8,
            required_bits: 5,
        })
    );
}

#[test]
fn terminal_alignment_padding_preserves_reader_source_errors() {
    struct PaddingRejectingReader {
        remaining_bits: usize,
    }

    impl BitRead for PaddingRejectingReader {
        type Error = &'static str;

        fn remaining_bits(&self) -> usize {
            self.remaining_bits
        }

        fn read_left_aligned_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, Self::Error> {
            if bit_len == 3 {
                self.remaining_bits = 5;
                Ok(vec![0b1010_0000])
            } else {
                Err("padding read failed")
            }
        }
    }

    let mut reader = PaddingRejectingReader { remaining_bits: 8 };
    assert_eq!(
        MarkerPacket::decode_from(&mut reader),
        Err(DecodeError::Source("padding read failed"))
    );
}
