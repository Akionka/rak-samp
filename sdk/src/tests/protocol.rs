//! Protocol send and convenience tests.

use super::*;

#[test]
fn owned_bit_stream_send_helpers_preserve_exact_partial_bit_lengths() {
    let mut stream = samp_protocol::BitStream::new();
    stream.write_bits(&[0b0000_0101], 3).unwrap();

    let api = test_support::test_api();
    assert_eq!(
        api.send_packet_stream(200, &stream, SampClientSdkSendOptions::default()),
        SampClientSdkResult::NativeCallFailed
    );
    assert_eq!(
        api.send_rpc_stream(62, &stream, SampClientSdkSendOptions::default()),
        SampClientSdkResult::NativeCallFailed
    );
}

#[test]
fn send_chat_uses_the_protocol_bounded_rpc_101_payload() {
    let api = test_support::test_api();
    assert_eq!(api.send_chat(b"hi"), Ok(()));
    assert_eq!(api.send_chat(b"/hi"), Ok(()));
    assert_eq!(
        api.send_chat(&[b'x'; 256]),
        Err(ProtocolSendError::Encode(
            samp_protocol::EncodeError::LengthExceedsLimit {
                length: 256,
                limit: 255,
            }
        ))
    );
    assert_eq!(api.send_request_spawn(), Ok(()));
}

#[test]
fn immediate_protocol_send_preserves_the_host_error_domain() {
    let api = test_support::test_api();
    test_support::set_next_send_result(SampClientSdkResult::NotReady);

    assert_eq!(
        api.send_request_spawn(),
        Err(ProtocolSendError::Host(SampClientSdkResult::NotReady))
    );
}

#[test]
fn queued_protocol_send_preserves_the_descriptor_framing_error() {
    let api = test_support::test_api();

    assert_eq!(
        api.submit_protocol_rpc(NonByteAlignedOutgoingRpc::new(), 5)
            .err(),
        Some(ProtocolSendError::Encode(
            samp_protocol::EncodeError::NonByteAlignedPayload { bit_len: 3 }
        ))
    );
}

#[test]
fn local_player_protocol_actions_preserve_their_wire_vectors() {
    let api = test_support::test_api();
    assert_eq!(api.send_request_class(9), Ok(()));
    assert_eq!(api.send_interior_change(7), Ok(()));
    assert_eq!(api.send_spawn(), Ok(()));
    assert_eq!(api.send_enter_vehicle(0x1234, true), Ok(()));
    assert_eq!(api.send_exit_vehicle(0x1234), Ok(()));
}

#[test]
fn typed_protocol_action_conveniences_preserve_their_wire_vectors() {
    let api = test_support::test_api();
    assert_eq!(api.send_dialog_response(0x1234, 1, 0x3456, b"ok"), Ok(()));
    assert_eq!(api.send_click_player(0x1234, 2), Ok(()));
    assert_eq!(api.send_click_textdraw(0x1234), Ok(()));
    assert_eq!(api.send_death_by_player(0x1234, 9), Ok(()));
    assert_eq!(api.send_menu_quit(), Ok(()));
    assert_eq!(api.send_menu_select_row(7), Ok(()));
    assert_eq!(api.send_picked_up_pickup(9), Ok(()));
    assert_eq!(api.send_vehicle_destroyed(0x1234), Ok(()));
    assert_eq!(
        api.send_dialog_response(0, 0, 0, &[b'x'; 256]),
        Err(ProtocolSendError::Encode(
            samp_protocol::EncodeError::LengthExceedsLimit {
                length: 256,
                limit: 255,
            }
        ))
    );
}

#[test]
fn additional_typed_protocol_actions_preserve_their_wire_vectors() {
    let api = test_support::test_api();
    assert_eq!(api.send_vehicle_damage(0x1234, 1, 2, 3, 4), Ok(()));
    assert_eq!(api.send_scm_event(4, 1, 2, 3), Ok(()));
    assert_eq!(api.send_give_damage(0x1234, 1.0, 24, 9), Ok(()));
    assert_eq!(api.send_take_damage(0x1234, 1.0, 24, 9), Ok(()));

    let protocol_zero = samp_protocol::types::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let attached = samp_protocol::rpc::outgoing::common::EditAttachedObject {
        response: 0,
        index: 0,
        model_id: 0,
        bone: 0,
        position: protocol_zero,
        rotation: protocol_zero,
        scale: protocol_zero,
        color1: 0,
        color2: 0,
    };
    assert_eq!(api.send_edit_attached_object(attached), Ok(()));
    assert_eq!(
        api.send_edit_object(samp_protocol::rpc::outgoing::common::EditObject {
            player_object: false,
            object_id: 0,
            response: 0,
            position: protocol_zero,
            rotation: protocol_zero,
        }),
        Ok(())
    );
    assert_eq!(api.send_rcon_command(b"rcon"), Ok(()));
    assert_eq!(
        api.send_rcon_command(&[b'x'; samp_protocol::limits::MAX_STRING32_BYTES + 1]),
        Err(ProtocolSendError::Encode(
            samp_protocol::EncodeError::LengthExceedsLimit {
                length: samp_protocol::limits::MAX_STRING32_BYTES + 1,
                limit: samp_protocol::limits::MAX_STRING32_BYTES,
            }
        ))
    );
}

#[test]
fn typed_sync_send_conveniences_preserve_their_fixed_wire_vectors() {
    let api = test_support::test_api();
    let zero = samp_protocol::types::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    assert_eq!(
        api.send_aim_sync(samp_protocol::packet::common::AimSync {
            camera_mode: 0,
            camera_front: zero,
            camera_position: zero,
            aim_z: 0.0,
            zoom_and_weapon_state: 0,
            aspect_ratio: 0,
        }),
        Ok(())
    );
    assert_eq!(
        api.send_bullet_sync(samp_protocol::packet::common::BulletSync {
            target_type: 0,
            target_id: 0,
            origin: zero,
            target: zero,
            center: zero,
            weapon_id: 0,
        }),
        Ok(())
    );
    assert_eq!(
        api.send_vehicle_sync(samp_protocol::packet::common::VehicleSync {
            vehicle_id: 0,
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            quaternion: [0.0; 4],
            position: zero,
            move_speed: zero,
            vehicle_health: 0.0,
            player_health: 0,
            armour: 0,
            weapon_and_special_key: 0,
            siren: 0,
            landing_gear_state: 0,
            trailer_id: 0,
            vehicle_specific: [0; 4],
        }),
        Ok(())
    );
    assert_eq!(
        api.send_player_sync(samp_protocol::packet::common::PlayerSync {
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
            quaternion: [0.0; 4],
            health: 0,
            armour: 0,
            weapon_and_special_key: 0,
            special_action: 0,
            move_speed: zero,
            surfing_offsets: zero,
            surfing_vehicle_id: 0,
            animation_id: 0,
            animation_flags: 0,
        }),
        Ok(())
    );
    assert_eq!(
        api.send_spectator_sync(samp_protocol::packet::common::SpectatorSync {
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
        }),
        Ok(())
    );
    assert_eq!(
        api.send_trailer_sync(samp_protocol::packet::common::TrailerSync {
            trailer_id: 0,
            position: zero,
            quaternion: [0.0; 4],
            move_speed: zero,
            turn_speed: zero,
        }),
        Ok(())
    );
    assert_eq!(
        api.send_passenger_sync(samp_protocol::packet::common::PassengerSync {
            vehicle_id: 0,
            seat_driveby_cuffed: 0,
            weapon_and_special_key: 0,
            health: 0,
            armour: 0,
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
        }),
        Ok(())
    );
    assert_eq!(
        api.send_unoccupied_sync(samp_protocol::packet::common::UnoccupiedSync {
            vehicle_id: 0,
            seat_id: 0,
            roll: zero,
            direction: zero,
            position: zero,
            move_speed: zero,
            turn_speed: zero,
            vehicle_health: 0.0,
        }),
        Ok(())
    );
}
