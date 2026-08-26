use core::mem::size_of;

use samp_protocol::{
    BitRead, BitWrite, DecodeError, EncodeError, EncodedBits, Packet, Rpc, TrailingPolicy,
    WireCodec, WireDescriptor, WireKind,
};

struct ThreeBitValue;

impl WireCodec for ThreeBitValue {
    type Value = u8;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;

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

struct ThreeBitExactByteValue;

impl WireCodec for ThreeBitExactByteValue {
    type Value = u8;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        ThreeBitValue::decode(reader)
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        ThreeBitValue::encode(writer, value)
    }
}

struct ThreeBitMarkerValue;

impl WireCodec for ThreeBitMarkerValue {
    type Value = u8;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::TerminalAlignmentPadding;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        ThreeBitValue::decode(reader)
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        ThreeBitValue::encode(writer, value)
    }
}

struct OneByteValue;

impl WireCodec for OneByteValue {
    type Value = u8;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

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

type ThreeBitPacket = Packet<207, ThreeBitValue>;
type ThreeBitRpc = Rpc<61, ThreeBitValue>;
type ExactBytePacket = Packet<1, ThreeBitExactByteValue>;
type MarkerPacket = Packet<208, ThreeBitMarkerValue>;
type OneBytePacket = Packet<2, OneByteValue>;

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

    let byte_aligned = EncodedBits::from_bits([0xA5], 8).unwrap();
    assert_eq!(OneBytePacket::decode_bits(&byte_aligned), Ok(0xA5));
}

#[test]
fn terminal_alignment_padding_accepts_fewer_than_eight_bits_and_rejects_a_full_byte() {
    let padded = EncodedBits::from_bits([0b1011_1111], 8).unwrap();
    assert_eq!(MarkerPacket::decode_bits(&padded), Ok(5));

    let full_byte_suffix = EncodedBits::from_bits([0b1010_0000, 0], 11).unwrap();
    assert_eq!(
        MarkerPacket::decode_bits(&full_byte_suffix),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 8,
            allowed_bits: 7,
        })
    );
}
