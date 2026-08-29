mod abi;
mod callbacks;
mod host_api;
mod protocol;
mod subscriptions;

use super::*;
use crate::events::{ProtocolAction, test_support};
use samp_protocol::EncodedBits;
use samp_protocol::rpc::incoming::{common as protocol_common, r1 as protocol_r1};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

static REGISTRATION_TEST_LOCK: Mutex<()> = Mutex::new(());

struct FailingReplacementCodec;

impl samp_protocol::WireCodec for FailingReplacementCodec {
    type Value = bool;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        reader
            .read_left_aligned_bits(8)
            .map(|bits| bits[0] != 0)
            .map_err(samp_protocol::DecodeError::Source)
    }

    fn encode<W: samp_protocol::BitWrite>(
        _writer: &mut W,
        _value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        Err(samp_protocol::EncodeError::LengthExceedsLimit {
            length: 2,
            limit: 1,
        })
    }
}

type FailingIncomingRpc =
    samp_protocol::IncomingRpc<201, FailingReplacementCodec, samp_protocol::ExactBitsPolicy>;

struct ThreeBitCodec;

impl samp_protocol::WireCodec for ThreeBitCodec {
    type Value = u8;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        let bytes = reader
            .read_left_aligned_bits(3)
            .map_err(samp_protocol::DecodeError::Source)?;
        Ok(bytes[0] >> 5)
    }

    fn encode<W: samp_protocol::BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value << 5], 3)
            .map_err(samp_protocol::EncodeError::Source)
    }
}

type NonByteAlignedOutgoingRpc =
    samp_protocol::OutgoingRpc<201, ThreeBitCodec, samp_protocol::ExactBytesPolicy>;

struct ByteCodec;

impl samp_protocol::WireCodec for ByteCodec {
    type Value = u8;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        reader
            .read_left_aligned_bits(8)
            .map(|bits| bits[0])
            .map_err(samp_protocol::DecodeError::Source)
    }

    fn encode<W: samp_protocol::BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value], 8)
            .map_err(samp_protocol::EncodeError::Source)
    }
}

type CustomOutgoingPacket =
    samp_protocol::OutgoingPacket<204, ByteCodec, samp_protocol::ExactBitsPolicy>;

struct DropCounter(Arc<AtomicUsize>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}

fn assert_default_is_zeroed<T: Default>() {
    let value = T::default();
    let bytes = unsafe {
        core::slice::from_raw_parts((&value as *const T).cast::<u8>(), core::mem::size_of::<T>())
    };
    assert!(bytes.iter().all(|byte| *byte == 0));
}

#[test]
fn safe_rpc_registration_dispatches_and_synchronizes() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let subscription = test_support::test_api()
        .on_rpc(SampClientSdkDirection::Incoming, move |event| {
            assert_eq!(event.id(), 42);
            observed.fetch_add(1, Ordering::AcqRel);
            SampClientSdkHookAction::Block
        })
        .expect("test registration must succeed");

    assert_eq!(subscription.id(), 1);
    assert_eq!(
        test_support::invoke_registered_callback(42),
        Some(SampClientSdkHookAction::Block)
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
    assert_eq!(test_support::invoke_registered_callback(42), None);
    assert_eq!(
        test_support::registration_stats().unregister_and_wait_calls,
        1
    );
}

#[test]
fn safe_callback_panic_fails_open() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Outgoing, |_| {
            panic!("test callback panic")
        })
        .expect("test registration must succeed");

    assert_eq!(
        test_support::invoke_registered_callback(10),
        Some(SampClientSdkHookAction::Continue)
    );
    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn id_filtered_callback_ignores_unrelated_events() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let subscription = test_support::test_api()
        .on_rpc_id(SampClientSdkDirection::Incoming, 42, move |_| {
            observed.fetch_add(1, Ordering::AcqRel);
            SampClientSdkHookAction::Block
        })
        .expect("test registration must succeed");

    assert_eq!(
        test_support::invoke_registered_callback(41),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(
        test_support::invoke_registered_callback(42),
        Some(SampClientSdkHookAction::Block)
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_callback_decodes_matching_descriptor_and_fails_open() {
    use samp_protocol::WireDescriptor;

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_rpc(protocol_r1::ENABLE_STUNT_BONUS, move |enabled| {
            assert!(enabled);
            observed.fetch_add(1, Ordering::AcqRel);
            ProtocolAction::Block
        })
        .expect("test registration must succeed");
    assert_eq!(test_support::registration_stats().registered_callbacks, 1);

    assert_eq!(
        test_support::invoke_registered_callback(99),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_payload(
            protocol_r1::EnableStuntBonusRpc::ID,
            EncodedBits::from_bits(
                protocol_r1::EnableStuntBonusRpc::encode_bits(&true)
                    .expect("the Protocol test payload must encode")
                    .as_bytes()
                    .to_vec(),
                1,
            )
            .expect("the Protocol test payload must preserve its bit length"),
        ),
        Some(SampClientSdkHookAction::Block)
    );
    assert_eq!(
        test_support::invoke_registered_callback(protocol_r1::EnableStuntBonusRpc::ID),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_chat_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::WireDescriptor;

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_outgoing_typed_rpc(
            samp_protocol::rpc::outgoing::chat::SEND_CHAT,
            |text| match text.as_slice() {
                b"continue" => ProtocolAction::Continue,
                b"block" => ProtocolAction::Block,
                b"replace" => ProtocolAction::Replace(b"changed".to_vec()),
                _ => unreachable!("test payload must select a callback action"),
            },
        )
        .expect("test registration must succeed");

    for (value, expected) in [
        (b"continue".as_slice(), SampClientSdkHookAction::Continue),
        (b"block".as_slice(), SampClientSdkHookAction::Block),
    ] {
        let bits = samp_protocol::rpc::outgoing::chat::SendChat::encode_bits(&value.to_vec())
            .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload =
            EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(101, payload),
            Some(expected)
        );
    }

    let bits = samp_protocol::rpc::outgoing::chat::SendChat::encode_bits(&b"replace".to_vec())
        .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload =
        EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(101, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![7, b'c', b'h', b'a', b'n', b'g', b'e', b'd'],
            64,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_common_outgoing_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::{
        WireDescriptor,
        rpc::outgoing::common::{DialogResponse, SEND_DIALOG_RESPONSE, SendDialogResponse},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let subscription = api
        .on_outgoing_typed_rpc(SEND_DIALOG_RESPONSE, |response| {
            match response.input.as_slice() {
                b"continue" => ProtocolAction::Continue,
                b"block" => ProtocolAction::Block,
                b"replace" => ProtocolAction::Replace(DialogResponse {
                    input: b"changed".to_vec(),
                    ..response
                }),
                _ => unreachable!("test payload must select a callback action"),
            }
        })
        .expect("test registration must succeed");

    for (input, expected) in [
        (b"continue".as_slice(), SampClientSdkHookAction::Continue),
        (b"block".as_slice(), SampClientSdkHookAction::Block),
    ] {
        let bits = SendDialogResponse::encode_bits(&DialogResponse {
            dialog_id: 0x1234,
            button: 1,
            list_item: 0x5678,
            input: input.to_vec(),
        })
        .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload =
            EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(62, payload),
            Some(expected)
        );
    }

    let bits = SendDialogResponse::encode_bits(&DialogResponse {
        dialog_id: 0x1234,
        button: 1,
        list_item: 0x5678,
        input: b"replace".to_vec(),
    })
    .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload =
        EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(62, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![
                0x34, 0x12, 1, 0x78, 0x56, 7, b'c', b'h', b'a', b'n', b'g', b'e', b'd'
            ],
            104,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_common_packet_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::{
        WireDescriptor,
        packet::common::{CONNECTION_ACCEPTED, ConnectionAccepted, ConnectionAcceptedPacket},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_packet(CONNECTION_ACCEPTED, |connection| {
            match connection.challenge {
                1 => ProtocolAction::Continue,
                2 => ProtocolAction::Block,
                3 => ProtocolAction::Replace(ConnectionAccepted {
                    challenge: 42,
                    ..connection
                }),
                _ => unreachable!("test payload must select a callback action"),
            }
        })
        .expect("test registration must succeed");

    for (challenge, expected) in [
        (1, SampClientSdkHookAction::Continue),
        (2, SampClientSdkHookAction::Block),
    ] {
        let bits = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
            ip: -1,
            port: 1,
            player_id: 2,
            challenge,
        })
        .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload =
            EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(34, payload),
            Some(expected)
        );
    }

    let bits = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
        ip: -1,
        port: 1,
        player_id: 2,
        challenge: 3,
    })
    .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload =
        EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(34, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![0xFF, 0xFF, 0xFF, 0xFF, 1, 0, 2, 0, 42, 0, 0, 0,],
            96,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn malformed_typed_packet_is_diagnosed_before_fail_open() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };
    use samp_protocol::{
        WireDescriptor,
        packet::common::{CONNECTION_ACCEPTED, ConnectionAccepted, ConnectionAcceptedPacket},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let handler_calls = Arc::new(AtomicUsize::new(0));
    let captured_calls = Arc::clone(&handler_calls);
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_packet(CONNECTION_ACCEPTED, move |_| {
            captured_calls.fetch_add(1, Ordering::Relaxed);
            ProtocolAction::Continue
        })
        .expect("test registration must succeed");

    let bits = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
        ip: -1,
        port: 1,
        player_id: 2,
        challenge: 3,
    })
    .unwrap();
    let mut bytes = bits.as_bytes().to_vec();
    bytes.push(0);
    let payload = EncodedBits::from_bits(bytes, bits.len_bits() + 8).unwrap();

    assert_eq!(
        test_support::invoke_registered_callback_with_payload(34, payload),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(handler_calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Debug,
            direction: "incoming",
            kind: "packet",
            id: 34,
            phase: CallbackFailurePhase::DecodeMalformed,
        }]
    );

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn typed_source_failure_is_warned_before_fail_open() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let handler_calls = Arc::new(AtomicUsize::new(0));
    let captured_calls = Arc::clone(&handler_calls);
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_rpc(protocol_common::SET_PLAYER_DRUNK, move |_| {
            captured_calls.fetch_add(1, Ordering::Relaxed);
            ProtocolAction::Continue
        })
        .unwrap();
    let payload = EncodedBits::from_bits(vec![7, 0, 0, 0], 32).unwrap();

    let outcome = test_support::invoke_registered_callback_with_source_failure(
        35,
        payload,
        SampClientSdkResult::NativeCallFailed,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(handler_calls.load(Ordering::Relaxed), 0);
    assert_eq!(outcome.bytes, [7, 0, 0, 0]);
    assert_eq!(outcome.bit_len, 32);
    assert_eq!(outcome.replacement_calls, 0);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "rpc",
            id: 35,
            phase: CallbackFailurePhase::DecodeSource,
        }]
    );

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn replacement_encode_failure_preserves_payload_without_host_mutation() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_rpc(FailingIncomingRpc::new(), ProtocolAction::Replace)
        .unwrap();
    let original = EncodedBits::from_bits(vec![0x80], 8).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        201,
        original,
        SampClientSdkResult::Ok,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, [0x80]);
    assert_eq!(outcome.bit_len, 8);
    assert_eq!(outcome.replacement_calls, 0);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "rpc",
            id: 201,
            phase: CallbackFailurePhase::ReplacementEncode,
        }]
    );

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn host_rejection_preserves_incoming_rpc_and_packet_payloads() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };
    use samp_protocol::{
        WireDescriptor,
        packet::common::{CONNECTION_ACCEPTED, ConnectionAccepted, ConnectionAcceptedPacket},
        rpc::incoming::common::{SET_PLAYER_DRUNK, SetPlayerDrunk},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let net = Samp::from_api(test_support::test_api()).net();
    let rpc_subscription = net
        .on_incoming_typed_rpc(SET_PLAYER_DRUNK, |_| ProtocolAction::Replace(8))
        .unwrap();
    let original = SetPlayerDrunk::encode_bits(&7).unwrap();
    let original_bytes = original.as_bytes().to_vec();
    let original_bit_len = original.len_bits();
    let payload = EncodedBits::from_bits(original_bytes.clone(), original_bit_len).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        35,
        payload,
        SampClientSdkResult::NativeCallFailed,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, original_bytes);
    assert_eq!(outcome.bit_len, original_bit_len);
    assert_eq!(outcome.replacement_calls, 1);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "rpc",
            id: 35,
            phase: CallbackFailurePhase::ReplacementHost,
        }]
    );
    rpc_subscription.unregister_and_wait().unwrap();

    test_support::reset_registration();
    let packet_subscription = net
        .on_incoming_typed_packet(CONNECTION_ACCEPTED, |connection| {
            ProtocolAction::Replace(ConnectionAccepted {
                challenge: 42,
                ..connection
            })
        })
        .unwrap();
    let original = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
        ip: -1,
        port: 1,
        player_id: 2,
        challenge: 3,
    })
    .unwrap();
    let original_bytes = original.as_bytes().to_vec();
    let original_bit_len = original.len_bits();
    let payload = EncodedBits::from_bits(original_bytes.clone(), original_bit_len).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        34,
        payload,
        SampClientSdkResult::NativeCallFailed,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, original_bytes);
    assert_eq!(outcome.bit_len, original_bit_len);
    assert_eq!(outcome.replacement_calls, 1);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "packet",
            id: 34,
            phase: CallbackFailurePhase::ReplacementHost,
        }]
    );
    packet_subscription.unregister_and_wait().unwrap();
}

#[test]
fn successful_non_byte_aligned_replacement_uses_one_host_call() {
    use crate::events::take_test_callback_diagnostics;
    use samp_protocol::{WireDescriptor, packet::r1};

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let net = Samp::from_api(test_support::test_api()).net();
    let value = r1::MarkersSync {
        markers: vec![r1::Marker {
            player_id: 1,
            coordinates: None,
        }],
    };
    let replacement = value.clone();
    let subscription = net
        .on_incoming_typed_packet(r1::MARKERS_SYNC, move |_| {
            ProtocolAction::Replace(replacement.clone())
        })
        .unwrap();
    let original = r1::MarkersSyncPacket::encode_bits(&value).unwrap();
    assert_eq!(original.len_bits(), 49);
    let payload =
        EncodedBits::from_bits(original.as_bytes().to_vec(), original.len_bits()).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        r1::MarkersSyncPacket::ID,
        payload,
        SampClientSdkResult::Ok,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, original.as_bytes());
    assert_eq!(outcome.bit_len, 49);
    assert_eq!(outcome.replacement_calls, 1);
    assert!(take_test_callback_diagnostics().is_empty());

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn normal_typed_methods_accept_all_descriptor_sources() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let net = Samp::from_api(test_support::test_api()).net();

    let subscriptions = [
        net.on_outgoing_typed_packet(samp_protocol::packet::common::SEND_PLAYER_SYNC, |_| {
            ProtocolAction::Continue
        })
        .expect("Protocol Packet registration must succeed"),
        net.on_incoming_typed_rpc(samp_protocol::rpc::incoming::common::SHOW_DIALOG, |_| {
            ProtocolAction::Continue
        })
        .expect("encoded-string Protocol RPC registration must succeed"),
        net.on_outgoing_typed_rpc(samp_protocol::rpc::outgoing::common::SEND_DAMAGE, |_| {
            ProtocolAction::Continue
        })
        .expect("Protocol outgoing RPC registration must succeed"),
    ];

    assert_eq!(test_support::registration_stats().registered_callbacks, 3);
    for subscription in subscriptions {
        subscription
            .unregister_and_wait()
            .expect("test shutdown must synchronize");
    }
}

#[test]
fn normal_typed_protocol_callback_preserves_all_actions() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let subscription = Samp::from_api(test_support::test_api())
        .net()
        .on_outgoing_typed_rpc(
            samp_protocol::rpc::outgoing::common::SEND_DAMAGE,
            move |damage| match observed.fetch_add(1, Ordering::AcqRel) {
                0 => ProtocolAction::Continue,
                1 => ProtocolAction::Block,
                2 => ProtocolAction::Replace(damage),
                _ => unreachable!("test invokes exactly three actions"),
            },
        )
        .expect("Protocol typed registration must succeed");
    let payload = || {
        EncodedBits::from_bits(
            vec![
                0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00, 0x00,
                0x00,
            ],
            113,
        )
        .expect("the exact damage vector is valid")
    };

    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(115, payload()),
        Some((
            SampClientSdkHookAction::Continue,
            payload().as_bytes().to_vec(),
            113,
        ))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(115, payload()),
        Some((
            SampClientSdkHookAction::Block,
            payload().as_bytes().to_vec(),
            113,
        ))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(115, payload()),
        Some((
            SampClientSdkHookAction::Continue,
            payload().as_bytes().to_vec(),
            113,
        ))
    );
    assert_eq!(calls.load(Ordering::Acquire), 3);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn custom_protocol_packet_callback_preserves_all_actions() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let descriptor = CustomOutgoingPacket::new();
    let subscription = Samp::from_api(test_support::test_api())
        .net()
        .on_outgoing_typed_packet(descriptor, move |value| {
            match observed.fetch_add(1, Ordering::AcqRel) {
                0 => ProtocolAction::Continue,
                1 => ProtocolAction::Block,
                2 => ProtocolAction::Replace(value + 1),
                _ => unreachable!("test invokes exactly three actions"),
            }
        })
        .expect("custom typed Packet registration must succeed");
    let payload = || EncodedBits::from_bits(vec![7], 8).expect("test payload must be valid");

    assert_eq!(test_support::registration_stats().registered_callbacks, 1);
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(204, payload()),
        Some((SampClientSdkHookAction::Continue, vec![7], 8))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(204, payload()),
        Some((SampClientSdkHookAction::Block, vec![7], 8))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(204, payload()),
        Some((SampClientSdkHookAction::Continue, vec![8], 8))
    );
    assert_eq!(calls.load(Ordering::Acquire), 3);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_server_message_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::{
        WireDescriptor,
        rpc::incoming::common::{SERVER_MESSAGE, ServerMessage, ServerMessageRpc},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let subscription = api
        .on_incoming_typed_rpc(SERVER_MESSAGE, |message| match message.text.as_slice() {
            b"continue" => ProtocolAction::Continue,
            b"block" => ProtocolAction::Block,
            b"replace" => ProtocolAction::Replace(ServerMessage {
                color: message.color,
                text: b"changed".to_vec(),
            }),
            _ => unreachable!("test payload must select a callback action"),
        })
        .expect("test registration must succeed");

    for (text, expected) in [
        (b"continue".as_slice(), SampClientSdkHookAction::Continue),
        (b"block".as_slice(), SampClientSdkHookAction::Block),
    ] {
        let bits = ServerMessageRpc::encode_bits(&ServerMessage {
            color: 0x1122_3344,
            text: text.to_vec(),
        })
        .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload =
            EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(93, payload),
            Some(expected)
        );
    }

    let bits = ServerMessageRpc::encode_bits(&ServerMessage {
        color: 0x1122_3344,
        text: b"replace".to_vec(),
    })
    .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload =
        EncodedBits::from_bits(bytes, bit_len).expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(93, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![
                0x44, 0x33, 0x22, 0x11, 7, 0, 0, 0, b'c', b'h', b'a', b'n', b'g', b'e', b'd'
            ],
            120,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn register_handlers_collects_every_supported_handler_form() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();

    let subscriptions = register_handlers!(test_support::test_api();
        packet(SampClientSdkDirection::Incoming, |_| SampClientSdkHookAction::Continue),
        rpc(SampClientSdkDirection::Outgoing, |_| SampClientSdkHookAction::Continue),
        packet_id(SampClientSdkDirection::Incoming, 1, |_| SampClientSdkHookAction::Continue),
        rpc_id(SampClientSdkDirection::Outgoing, 2, |_| SampClientSdkHookAction::Continue),
        incoming_typed_packet(
            samp_protocol::packet::r1::PLAYER_SYNC,
            |_| ProtocolAction::Continue
        ),
        incoming_typed_rpc(
            protocol_r1::ENABLE_STUNT_BONUS,
            |_| ProtocolAction::Continue
        ),
    )
    .expect("all test registrations must succeed");

    assert_eq!(subscriptions.len(), 6);
    assert_eq!(
        test_support::registration_stats().registered_callbacks,
        subscriptions.len()
    );
    subscriptions
        .unregister_and_wait()
        .expect("test shutdown must synchronize every callback");
    assert_eq!(test_support::registration_stats().registered_callbacks, 0);
}

#[test]
fn subscription_set_retains_each_failed_shutdown_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let mut subscriptions = SubscriptionSet::new();
    subscriptions.push(
        api.on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    subscriptions.push(
        api.on_rpc(SampClientSdkDirection::Outgoing, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscriptions
        .unregister_and_wait()
        .expect_err("failed callbacks must remain available for retry");
    assert_eq!(error.failures().len(), 2);
    assert!(
        error
            .failures()
            .iter()
            .all(|failure| failure.result() == SampClientSdkResult::CallbackInProgress)
    );
    assert_eq!(test_support::registration_stats().registered_callbacks, 2);

    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    error
        .into_subscriptions()
        .unregister_and_wait()
        .expect("retry must synchronize every callback");
    let stats = test_support::registration_stats();
    assert_eq!(stats.unregister_and_wait_calls, 4);
    assert_eq!(stats.registered_callbacks, 0);
}

#[test]
fn subscription_set_preserves_earlier_registrations_after_a_registration_failure() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    let error = SubscriptionSet::new()
        .try_add(Ok(subscription))
        .and_then(|subscriptions| subscriptions.try_add(Err(SampClientSdkResult::NotReady)))
        .expect_err("the synthetic second registration must fail");
    assert_eq!(error.result(), SampClientSdkResult::NotReady);
    let subscriptions = error.into_subscriptions();
    assert_eq!(subscriptions.len(), 1);
    subscriptions
        .unregister_and_wait()
        .expect("retained subscription must remain cleanly removable");
}

#[test]
fn failed_synchronized_shutdown_keeps_the_subscription_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_rpc(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscription
        .unregister_and_wait()
        .expect_err("callback-thread shutdown must remain retryable");
    assert_eq!(error.result(), SampClientSdkResult::CallbackInProgress);
    let subscription = error.into_subscription();
    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    subscription
        .unregister_and_wait()
        .expect("retry must synchronize");
}

#[test]
fn failed_registration_releases_the_handler() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    test_support::set_register_result(SampClientSdkResult::NotReady);
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));

    let result = test_support::test_api().on_packet(SampClientSdkDirection::Incoming, move |_| {
        let _ = &counter;
        SampClientSdkHookAction::Continue
    });
    assert_eq!(result.unwrap_err(), SampClientSdkResult::NotReady);
    assert_eq!(drops.load(Ordering::Acquire), 1);
}

#[test]
fn dropping_a_subscription_detaches_without_freeing_callback_state() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, move |_| {
            let _ = &counter;
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    drop(subscription);
    assert_eq!(drops.load(Ordering::Acquire), 0);
    assert_eq!(test_support::invoke_registered_callback(1), None);
    assert_eq!(test_support::registration_stats().unregister_calls, 1);
}
