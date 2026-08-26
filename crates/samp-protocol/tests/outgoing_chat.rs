use samp_protocol::rpc::outgoing::chat::{SendChat, SendCommand};
use samp_protocol::{DecodeError, EncodeError, EncodedBits, WireDescriptor};

#[test]
fn send_chat_encodes_and_decodes_the_exact_vector() {
    let bits = SendChat::encode_bits(&b"hi".to_vec()).unwrap();

    assert_eq!(bits.as_bytes(), &[2, b'h', b'i']);
    assert_eq!(bits.len_bits(), 24);
    assert_eq!(SendChat::decode_bits(&bits).unwrap(), b"hi");
}

#[test]
fn send_command_encodes_and_decodes_the_exact_vector() {
    let bits = SendCommand::encode_bits(&b"/hi".to_vec()).unwrap();

    assert_eq!(bits.as_bytes(), &[3, 0, 0, 0, b'/', b'h', b'i']);
    assert_eq!(bits.len_bits(), 56);
    assert_eq!(SendCommand::decode_bits(&bits).unwrap(), b"/hi");
}

#[test]
fn chat_and_command_reject_values_past_their_wire_limits() {
    assert_eq!(
        SendChat::encode_bits(&vec![b'x'; 256]),
        Err(EncodeError::LengthExceedsLimit {
            length: 256,
            limit: 255,
        })
    );
    assert_eq!(
        SendCommand::encode_bits(&vec![b'x'; 4097]),
        Err(EncodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );
}

#[test]
fn chat_and_command_accept_their_maximum_wire_lengths() {
    let chat = vec![b'c'; 255];
    let command = vec![b'/'; 4096];

    let chat_bits = SendChat::encode_bits(&chat).unwrap();
    assert_eq!(chat_bits.len_bits(), 256 * 8);
    assert_eq!(SendChat::decode_bits(&chat_bits).unwrap(), chat);

    let command_bits = SendCommand::encode_bits(&command).unwrap();
    assert_eq!(command_bits.len_bits(), 4100 * 8);
    assert_eq!(SendCommand::decode_bits(&command_bits).unwrap(), command);
}

#[test]
fn command_rejects_a_declared_length_past_its_limit() {
    let bits = EncodedBits::from_bits([0x01, 0x10, 0, 0], 32).unwrap();

    assert_eq!(
        SendCommand::decode_bits(&bits),
        Err(DecodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );
}

#[test]
fn chat_rejects_a_truncated_length_prefix() {
    let bits = EncodedBits::from_bits([0], 7).unwrap();

    assert_eq!(
        SendChat::decode_bits(&bits),
        Err(DecodeError::OutOfBounds {
            requested_bits: 8,
            available_bits: 7,
        })
    );
}
