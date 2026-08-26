use samp_protocol::{
    BitRead, BitStream, BitStreamError, BitWrite, DecodeError, EncodeError, EncodedBits,
    EncodedBitsError, packet_name, rpc_name,
};

#[test]
fn bit_stream_preserves_the_sdk_right_aligned_partial_read() {
    let mut stream = BitStream::new();
    stream.write_bits(&[0b0000_0101], 3).unwrap();

    assert_eq!(stream.as_bytes(), &[0b1010_0000]);
    assert_eq!(stream.read_bits(3), Ok(vec![0b0000_0101]));
}

#[test]
fn raw_bit_read_is_left_aligned_and_msb_first() {
    let mut stream = BitStream::from_bits([0b1010_0000], 3).unwrap();

    assert_eq!(
        BitRead::read_left_aligned_bits(&mut stream, 3),
        Ok(vec![0b1010_0000])
    );
}

#[test]
fn raw_bit_write_accepts_left_aligned_partial_bits() {
    let mut stream = BitStream::new();

    BitWrite::write_left_aligned_bits(&mut stream, &[0b1010_0000], 3).unwrap();

    assert_eq!(stream.as_bytes(), &[0b1010_0000]);
    assert_eq!(stream.read_bits(3), Ok(vec![0b0000_0101]));
}

#[test]
fn bit_stream_retains_exact_cursor_and_bounds_errors() {
    let mut stream = BitStream::from_bytes([0xAB, 0xCD]).unwrap();
    stream.set_read_offset(4).unwrap();

    assert_eq!(stream.read_bits(4), Ok(vec![0x0B]));
    stream.ignore_bits(8).unwrap();
    assert_eq!(
        stream.read_bits(1),
        Err(BitStreamError::OutOfBounds {
            requested_bits: 1,
            available_bits: 0,
        })
    );
}

#[test]
fn encoded_bits_canonicalize_unused_bits_and_reject_non_minimal_storage() {
    let bits = EncodedBits::from_bits([0b1011_1111], 3).unwrap();

    assert_eq!(bits.as_bytes(), &[0b1010_0000]);
    assert_eq!(bits.len_bits(), 3);
    assert_eq!(
        EncodedBits::from_bits([0b1010_0000, 0], 3),
        Err(EncodedBitsError::NonMinimalStorage {
            bit_len: 3,
            byte_len: 2,
        })
    );
}

#[test]
fn encoded_bits_support_empty_and_reject_invalid_lengths() {
    assert_eq!(EncodedBits::from_bits([], 0).unwrap().as_bytes(), &[]);
    assert_eq!(
        EncodedBits::from_bits([], 1),
        Err(EncodedBitsError::InvalidBitLength {
            bit_len: 1,
            byte_len: 0,
        })
    );
}

#[test]
fn protocol_errors_keep_source_and_wire_details_separate() {
    let source = DecodeError::Source("host status");
    assert_eq!(source, DecodeError::Source("host status"));
    assert_eq!(
        DecodeError::<&str>::OutOfBounds {
            requested_bits: 8,
            available_bits: 3,
        },
        DecodeError::OutOfBounds {
            requested_bits: 8,
            available_bits: 3,
        }
    );
    assert_eq!(
        EncodeError::<&str>::PayloadTooLarge {
            requested_bits: 9,
            limit_bits: 8,
        },
        EncodeError::PayloadTooLarge {
            requested_bits: 9,
            limit_bits: 8,
        }
    );
}

#[test]
fn packet_and_rpc_catalogs_remain_separate_and_non_exhaustive() {
    assert_eq!(rpc_name(61), Some("ShowDialog"));
    assert_eq!(packet_name(207), Some("PLAYER_SYNC"));
    assert_eq!(rpc_name(0), None);
    assert_eq!(packet_name(255), None);
}
