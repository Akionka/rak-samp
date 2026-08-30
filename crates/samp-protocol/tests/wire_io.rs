use samp_protocol::{
    BitRead, BitStream, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt,
    limits::MAX_STRING32_BYTES,
    types::{Vector2, Vector3},
};

#[test]
fn wire_bit_bool_reads_exactly_one_bit_at_the_current_cursor() {
    let mut reader = BitStream::from_bits([0b1010_1000], 5).unwrap();

    assert_eq!(
        BitRead::read_left_aligned_bits(&mut reader, 3),
        Ok(vec![0b1010_0000])
    );
    assert_eq!(WireReadExt::read_bit_bool(&mut reader), Ok(false));
    assert_eq!(WireReadExt::read_bit_bool(&mut reader), Ok(true));
    assert_eq!(reader.remaining_bits(), 0);
}

#[test]
fn wire_bit_bool_reports_missing_input_as_protocol_bounds_failure() {
    assert_eq!(
        WireReadExt::read_bit_bool(&mut BitStream::new()),
        Err(DecodeError::OutOfBounds {
            requested_bits: 1,
            available_bits: 0,
        })
    );
}

#[test]
fn wire_bit_bool_writes_exactly_one_bit_at_the_current_cursor() {
    let mut writer = BitStream::new();

    BitWrite::write_left_aligned_bits(&mut writer, &[0b1010_0000], 3).unwrap();
    WireWriteExt::write_bit_bool(&mut writer, false).unwrap();
    WireWriteExt::write_bit_bool(&mut writer, true).unwrap();

    assert_eq!(writer.len_bits(), 5);
    assert_eq!(writer.as_bytes(), &[0b1010_1000]);
}

#[test]
fn wire_primitives_preserve_values_from_a_non_byte_aligned_cursor() {
    let vector2 = Vector2 { x: 1.5, y: -2.5 };
    let vector3 = Vector3 {
        x: 1.0,
        y: -2.0,
        z: 0.5,
    };
    let mut writer = BitStream::new();

    BitWrite::write_left_aligned_bits(&mut writer, &[0b1010_0000], 3).unwrap();
    WireWriteExt::write_u8(&mut writer, 0x7f).unwrap();
    WireWriteExt::write_i16_le(&mut writer, -123).unwrap();
    WireWriteExt::write_u16_le(&mut writer, 456).unwrap();
    WireWriteExt::write_i32_le(&mut writer, -78_901).unwrap();
    WireWriteExt::write_u32_le(&mut writer, 123_456).unwrap();
    WireWriteExt::write_f32_le(&mut writer, -0.25).unwrap();
    WireWriteExt::write_vector2_le(&mut writer, &vector2).unwrap();
    WireWriteExt::write_vector3_le(&mut writer, &vector3).unwrap();

    let mut reader = BitStream::from_bits(writer.as_bytes(), writer.len_bits()).unwrap();

    assert_eq!(
        BitRead::read_left_aligned_bits(&mut reader, 3),
        Ok(vec![0b1010_0000])
    );
    assert_eq!(WireReadExt::read_u8(&mut reader), Ok(0x7f));
    assert_eq!(WireReadExt::read_i16_le(&mut reader), Ok(-123));
    assert_eq!(WireReadExt::read_u16_le(&mut reader), Ok(456));
    assert_eq!(WireReadExt::read_i32_le(&mut reader), Ok(-78_901));
    assert_eq!(WireReadExt::read_u32_le(&mut reader), Ok(123_456));
    assert_eq!(WireReadExt::read_f32_le(&mut reader), Ok(-0.25));
    assert_eq!(WireReadExt::read_vector2_le(&mut reader), Ok(vector2));
    assert_eq!(WireReadExt::read_vector3_le(&mut reader), Ok(vector3));
    assert_eq!(reader.remaining_bits(), 0);
}

#[test]
fn wire_length_prefixed_bytes_enforce_limits_before_reading_payloads() {
    let mut writer = BitStream::new();

    BitWrite::write_left_aligned_bits(&mut writer, &[0b1010_0000], 3).unwrap();
    WireWriteExt::write_len_prefixed_bytes_u8(&mut writer, &[1, 2], 2).unwrap();
    WireWriteExt::write_len_prefixed_bytes_u16_le(&mut writer, &[3, 4], 2).unwrap();
    WireWriteExt::write_len_prefixed_bytes_u32_le(&mut writer, &[5, 6], 2).unwrap();

    let mut reader = BitStream::from_bits(writer.as_bytes(), writer.len_bits()).unwrap();
    BitRead::read_left_aligned_bits(&mut reader, 3).unwrap();
    assert_eq!(
        WireReadExt::read_len_prefixed_bytes_u8(&mut reader, 2),
        Ok(vec![1, 2])
    );
    assert_eq!(
        WireReadExt::read_len_prefixed_bytes_u16_le(&mut reader, 2),
        Ok(vec![3, 4])
    );
    assert_eq!(
        WireReadExt::read_len_prefixed_bytes_u32_le(&mut reader, 2),
        Ok(vec![5, 6])
    );

    let mut oversized_prefix = BitStream::new();
    BitWrite::write_left_aligned_bits(&mut oversized_prefix, &[0b1010_0000], 3).unwrap();
    WireWriteExt::write_u8(&mut oversized_prefix, 3).unwrap();
    let mut reader =
        BitStream::from_bits(oversized_prefix.as_bytes(), oversized_prefix.len_bits()).unwrap();
    BitRead::read_left_aligned_bits(&mut reader, 3).unwrap();

    assert_eq!(
        WireReadExt::read_len_prefixed_bytes_u8(&mut reader, 2),
        Err(DecodeError::LengthExceedsLimit {
            length: 3,
            limit: 2,
        })
    );
    assert_eq!(
        WireWriteExt::write_len_prefixed_bytes_u8(&mut BitStream::new(), &[1, 2, 3], 2),
        Err(EncodeError::LengthExceedsLimit {
            length: 3,
            limit: 2,
        })
    );
    let mut oversized_u16_prefix = BitStream::new();
    WireWriteExt::write_u16_le(&mut oversized_u16_prefix, 3).unwrap();
    let mut oversized_u16_prefix_reader = BitStream::from_bits(
        oversized_u16_prefix.as_bytes(),
        oversized_u16_prefix.len_bits(),
    )
    .unwrap();
    assert_eq!(
        WireReadExt::read_len_prefixed_bytes_u16_le(&mut oversized_u16_prefix_reader, 2),
        Err(DecodeError::LengthExceedsLimit {
            length: 3,
            limit: 2,
        })
    );
    assert_eq!(
        WireWriteExt::write_len_prefixed_bytes_u16_le(&mut BitStream::new(), &[1, 2, 3], 2,),
        Err(EncodeError::LengthExceedsLimit {
            length: 3,
            limit: 2,
        })
    );
    assert_eq!(
        WireWriteExt::write_len_prefixed_bytes_u8(&mut BitStream::new(), &vec![0; 256], 256),
        Err(EncodeError::LengthExceedsLimit {
            length: 256,
            limit: u8::MAX as usize,
        })
    );
    assert_eq!(
        WireWriteExt::write_len_prefixed_bytes_u16_le(
            &mut BitStream::new(),
            &vec![0; usize::from(u16::MAX) + 1],
            usize::from(u16::MAX) + 1,
        ),
        Err(EncodeError::LengthExceedsLimit {
            length: usize::from(u16::MAX) + 1,
            limit: usize::from(u16::MAX),
        })
    );
}

#[test]
fn wire_primitives_keep_source_failures_separate_from_validation() {
    struct RejectingReader;

    impl BitRead for RejectingReader {
        type Error = &'static str;

        fn remaining_bits(&self) -> usize {
            u16::BITS as usize
        }

        fn read_left_aligned_bits_into(
            &mut self,
            _: &mut [u8],
            _: usize,
        ) -> Result<(), Self::Error> {
            Err("read failed")
        }
    }

    struct RejectingWriter;

    impl BitWrite for RejectingWriter {
        type Error = &'static str;

        fn write_left_aligned_bits(&mut self, _: &[u8], _: usize) -> Result<(), Self::Error> {
            Err("write failed")
        }
    }

    assert_eq!(
        WireReadExt::read_u8(&mut RejectingReader),
        Err(DecodeError::Source("read failed"))
    );
    assert_eq!(
        WireReadExt::read_bit_bool(&mut RejectingReader),
        Err(DecodeError::Source("read failed"))
    );
    assert_eq!(
        WireWriteExt::write_u8(&mut RejectingWriter, 1),
        Err(EncodeError::Source("write failed"))
    );
    assert_eq!(
        WireWriteExt::write_bit_bool(&mut RejectingWriter, true),
        Err(EncodeError::Source("write failed"))
    );
    assert_eq!(
        WireReadExt::read_len_prefixed_bytes_u16_le(&mut RejectingReader, 1),
        Err(DecodeError::Source("read failed"))
    );
    assert_eq!(
        WireWriteExt::write_len_prefixed_bytes_u16_le(&mut RejectingWriter, &[1], 1),
        Err(EncodeError::Source("write failed"))
    );
    assert_eq!(MAX_STRING32_BYTES, 4096);
}
