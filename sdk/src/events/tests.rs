use super::*;
use super::{
    core::{PayloadWriter, RpcEncoder},
    rpc::{incoming, outgoing},
    test_support::{TestEvent, assert_replacement_round_trip, test_api},
};
use crate::{SampClientSdkEventV1, SampClientSdkHookAction};

fn encode_bytes<T>(descriptor: Rpc<T>, value: T) -> Vec<u8> {
    let RpcEncoder::Bytes(encode) = descriptor.encode else {
        panic!("test descriptor must use a byte-aligned encoder");
    };
    encode(value).expect("test payload must be valid")
}

#[test]
fn payload_writer_preserves_partial_bit_lengths() {
    let mut writer = PayloadWriter::new();
    writer.u8(0xA5);
    writer.bits(&[0b1100_0000], 3);
    let payload = writer.finish_bits();

    assert_eq!(payload.len_bits(), 11);
    assert_eq!(payload.as_bytes(), &[0xA5, 0b1100_0000]);
}

#[test]
fn encoded_payload_rejects_bits_outside_its_buffer() {
    assert!(matches!(
        EncodedPayload::from_bits(vec![0], 9),
        Err(EventError::InvalidBitLength {
            bit_len: 9,
            byte_len: 1
        })
    ));
}

fn test_vector3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
}

fn test_spawn_info() -> incoming::SpawnInfo {
    incoming::SpawnInfo {
        team: 7,
        skin: 411,
        unused: 0xA5,
        position: test_vector3(1.0, 2.0, 3.0),
        rotation: 4.0,
        weapons: [22, 24, 31],
        ammo: [100, 200, 300],
    }
}

fn test_animation() -> incoming::Animation {
    incoming::Animation {
        animation_library: b"PED".to_vec(),
        animation_name: b"WALK".to_vec(),
        frame_delta: 4.0,
        looped: true,
        lock_x: false,
        lock_y: true,
        freeze: false,
        time: -1,
    }
}

#[test]
fn r1_player_stream_in_includes_all_eleven_weapon_skill_levels() {
    let value = incoming::PlayerStreamIn {
        player_id: 42,
        team: 3,
        model: 411,
        position: test_vector3(1.0, 2.0, 3.0),
        rotation: 90.0,
        color: -1,
        fighting_style: 4,
        weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };
    let encoded = incoming::PLAYER_STREAM_IN
        .encode(test_api(), value)
        .expect("R1 player stream-in payload must encode");

    assert_eq!(encoded.len_bits(), 400);
    assert_eq!(
        encoded.as_bytes(),
        &[
            0x2A, 0x00, 0x03, 0x9B, 0x01, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0xB4, 0x42, 0xFF, 0xFF, 0xFF, 0xFF, 0x04,
            0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00,
            0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0A, 0x00,
        ]
    );
    assert_replacement_round_trip(incoming::PLAYER_STREAM_IN, value);
}

#[test]
fn r1_complex_incoming_rpc_helpers_decode_and_atomically_replace() {
    let settings = incoming::GameSettings {
        zone_names: true,
        use_cj_walk: false,
        allow_weapons: true,
        limit_global_chat_radius: false,
        global_chat_radius: 100.0,
        stunt_bonus: true,
        nametag_draw_distance: 70.0,
        disable_enter_exits: false,
        nametag_los: true,
        tire_popping: false,
        classes_available: 5,
        show_player_tags: true,
        player_markers_mode: 1,
        world_time: 12,
        world_weather: 7,
        gravity: 0.008,
        lan_mode: false,
        death_money_drop: 500,
        instagib: false,
        normal_onfoot_send_rate: 30,
        normal_incar_send_rate: 30,
        normal_firing_send_rate: 30,
        send_multiplier: 2,
        lag_compensation_mode: 1,
        vehicle_friendly_fire: true,
    };
    assert_replacement_round_trip(
        incoming::INIT_GAME,
        incoming::InitGame {
            player_id: 42,
            host_name: b"R1 host".to_vec(),
            settings,
            vehicle_models: [1; 212],
        },
    );
    assert_replacement_round_trip(
        incoming::REQUEST_CLASS_RESPONSE,
        incoming::RequestClassResponse {
            can_spawn: true,
            spawn: test_spawn_info(),
        },
    );
    assert_replacement_round_trip(
        incoming::PLAYER_STREAM_IN,
        incoming::PlayerStreamIn {
            player_id: 42,
            team: 3,
            model: 411,
            position: test_vector3(1.0, 2.0, 3.0),
            rotation: 90.0,
            color: -1,
            fighting_style: 4,
            weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        },
    );
    assert_replacement_round_trip(
        incoming::CREATE_3D_TEXT,
        incoming::TextLabel3D {
            id: 4,
            color: -1,
            position: test_vector3(1.0, 2.0, 3.0),
            distance: 50.0,
            test_los: true,
            attached_player_id: u16::MAX,
            attached_vehicle_id: u16::MAX,
            text: b"encoded 3D text".to_vec(),
        },
    );
    assert_replacement_round_trip(
        incoming::CREATE_OBJECT,
        incoming::Object {
            object_id: 9,
            model_id: 1337,
            position: test_vector3(1.0, 2.0, 3.0),
            rotation: test_vector3(4.0, 5.0, 6.0),
            draw_distance: 300.0,
            no_camera_collision: true,
            attach_to_vehicle_id: u16::MAX,
            attach_to_object_id: u16::MAX,
            attachment: None,
            textures_count: 2,
            materials: vec![
                incoming::ObjectMaterial::Texture(incoming::TextureMaterial {
                    material_id: 0,
                    model_id: 18646,
                    library_name: b"matcolours".to_vec(),
                    texture_name: b"grey-10-percent".to_vec(),
                    color: -1,
                }),
                incoming::ObjectMaterial::Text(incoming::TextMaterial {
                    material_id: 1,
                    material_size: 90,
                    font_name: b"Arial".to_vec(),
                    font_size: 20,
                    bold: 1,
                    font_color: -1,
                    background_color: 0,
                    align: 2,
                    text: b"material text".to_vec(),
                }),
            ],
        },
    );
    assert_replacement_round_trip(incoming::SET_SPAWN_INFO, test_spawn_info());
    assert_replacement_round_trip(
        incoming::INIT_MENU,
        incoming::InitMenu {
            menu_id: 1,
            two_columns: true,
            title: *b"R1 menu                         ",
            position: Vector2 { x: 10.0, y: 20.0 },
            columns: vec![
                incoming::MenuColumn {
                    width: 100.0,
                    title: *b"first                           ",
                    rows: vec![*b"one                             "],
                },
                incoming::MenuColumn {
                    width: 200.0,
                    title: *b"second                          ",
                    rows: vec![*b"two                             "],
                },
            ],
            rows: [-1; incoming::MAX_MENU_ROWS],
            menu: false,
        },
    );
    assert_replacement_round_trip(
        incoming::INTERPOLATE_CAMERA,
        incoming::InterpolateCamera {
            set_position: true,
            from_position: test_vector3(1.0, 2.0, 3.0),
            destination: test_vector3(4.0, 5.0, 6.0),
            time_ms: 500,
            mode: 2,
        },
    );
    assert_replacement_round_trip(
        incoming::TOGGLE_SELECT_TEXT_DRAW,
        incoming::ToggleSelectTextDraw {
            enabled: true,
            hover_color: -1,
        },
    );
    assert_replacement_round_trip(
        incoming::SET_OBJECT_MATERIAL,
        incoming::ObjectMaterialUpdate {
            object_id: 9,
            material: incoming::ObjectMaterial::Texture(incoming::TextureMaterial {
                material_id: 1,
                model_id: 123,
                library_name: b"lib".to_vec(),
                texture_name: b"texture".to_vec(),
                color: 0x1122_3344,
            }),
        },
    );
    assert_replacement_round_trip(
        incoming::SET_OBJECT_MATERIAL,
        incoming::ObjectMaterialUpdate {
            object_id: 9,
            material: incoming::ObjectMaterial::Text(incoming::TextMaterial {
                material_id: 2,
                material_size: 90,
                font_name: b"Arial".to_vec(),
                font_size: 20,
                bold: 0,
                font_color: -1,
                background_color: 0,
                align: 1,
                text: b"encoded material update".to_vec(),
            }),
        },
    );
    assert_replacement_round_trip(
        incoming::APPLY_PLAYER_ANIMATION,
        incoming::PlayerAnimation {
            player_id: 7,
            animation: test_animation(),
        },
    );
    assert_replacement_round_trip(incoming::ENABLE_STUNT_BONUS, true);
    assert_replacement_round_trip(
        incoming::PLAY_CRIME_REPORT,
        incoming::CrimeReport {
            suspect_id: 7,
            in_vehicle: true,
            vehicle_model: 411,
            vehicle_color: 4,
            crime: 9,
            coordinates: test_vector3(1.0, 2.0, 3.0),
        },
    );
    assert_replacement_round_trip(
        incoming::SET_PLAYER_ATTACHED_OBJECT,
        incoming::PlayerAttachedObject {
            player_id: 7,
            index: 3,
            object: Some(incoming::AttachedObject {
                model_id: 19327,
                bone: 1,
                offset: test_vector3(1.0, 2.0, 3.0),
                rotation: test_vector3(4.0, 5.0, 6.0),
                scale: test_vector3(1.0, 1.0, 1.0),
                color1: -1,
                color2: 0,
            }),
        },
    );
    assert_replacement_round_trip(
        incoming::ENTER_EDIT_OBJECT,
        incoming::EnterEditObject {
            player_object: true,
            object_id: 5,
        },
    );
    assert_replacement_round_trip(incoming::TOGGLE_PLAYER_SPECTATING, false);
    assert_replacement_round_trip(
        incoming::SHOW_TEXT_DRAW,
        incoming::ShowTextDraw {
            textdraw_id: 99,
            textdraw: incoming::TextDraw {
                flags: 1,
                letter_width: 0.5,
                letter_height: 1.0,
                letter_color: -1,
                line_width: 2.0,
                line_height: 3.0,
                box_color: 0,
                shadow: 1,
                outline: 2,
                background_color: 0,
                style: 4,
                selectable: 1,
                position: Vector2 { x: 100.0, y: 200.0 },
                model_id: 1234,
                rotation: test_vector3(0.0, 0.0, 1.0),
                zoom: 1.5,
                color1: -1,
                color2: 2,
                text: b"textdraw".to_vec(),
            },
        },
    );
    assert_replacement_round_trip(incoming::TEXT_DRAW_HIDE, 99);
    assert_replacement_round_trip(
        incoming::UPDATE_SCORES_AND_PINGS,
        incoming::ScoresAndPings {
            entries: vec![incoming::ScorePing {
                player_id: 7,
                score: -100,
                ping: 42,
            }],
        },
    );
    assert_replacement_round_trip(
        incoming::VEHICLE_STREAM_IN,
        incoming::VehicleStreamIn {
            vehicle_id: 9,
            vehicle: incoming::StreamedVehicle {
                model: 411,
                position: test_vector3(1.0, 2.0, 3.0),
                rotation: 45.0,
                body_color1: 1,
                body_color2: 2,
                health: 900.0,
                interior_id: 3,
                door_damage_status: 4,
                panel_damage_status: 5,
                light_damage_status: 6,
                tire_damage_status: 7,
                add_siren: 8,
                mod_slots: [9; 14],
                paint_job: 10,
                interior_color1: 11,
                interior_color2: 12,
            },
        },
    );
    assert_replacement_round_trip(incoming::DISABLE_VEHICLE_COLLISIONS, true);
    assert_replacement_round_trip(incoming::TOGGLE_CAMERA_TARGET_NOTIFYING, false);
    assert_replacement_round_trip(
        incoming::APPLY_ACTOR_ANIMATION,
        incoming::ActorAnimation {
            actor_id: 8,
            animation: test_animation(),
        },
    );
}

#[test]
fn r1_remote_sync_and_markers_decode_and_atomically_replace() {
    assert_replacement_round_trip(
        packet::incoming::PLAYER_SYNC,
        packet::RemotePlayerSync {
            player_id: 1,
            left_right_keys: Some(2),
            up_down_keys: None,
            key_data: 3,
            position: test_vector3(1.0, 2.0, 3.0),
            quaternion: [-1.0, 0.0, 0.0, 0.0],
            health: 100,
            armour: 98,
            weapon: 24,
            special_action: 0,
            move_speed: test_vector3(0.0, 0.0, 0.0),
            surfing: Some(packet::RemotePlayerSurfing {
                vehicle_id: 4,
                offsets: test_vector3(4.0, 5.0, 6.0),
            }),
            animation: Some(packet::RemotePlayerAnimation { id: 7, flags: 8 }),
        },
    );
    assert_replacement_round_trip(
        packet::incoming::VEHICLE_SYNC,
        packet::RemoteVehicleSync {
            player_id: 1,
            vehicle_id: 2,
            left_right_keys: 3,
            up_down_keys: 4,
            key_data: 5,
            quaternion: [1.0, 0.0, 0.0, 0.0],
            position: test_vector3(1.0, 2.0, 3.0),
            // R1's compressed-vector zero components decode to -1 / 65536 after the
            // writer's integer conversion; use the exact representable values here.
            move_speed: test_vector3(1.0, -1.0 / 65_536.0, -1.0 / 65_536.0),
            vehicle_health: 900,
            player_health: 98,
            armour: 0,
            current_weapon: 24,
            siren: true,
            landing_gear: false,
            train_speed: Some(-7),
            trailer_id: Some(6),
        },
    );
    assert_replacement_round_trip(
        packet::incoming::MARKERS_SYNC,
        packet::MarkersSync {
            markers: vec![
                packet::Marker {
                    player_id: 1,
                    coordinates: None,
                },
                packet::Marker {
                    player_id: 2,
                    coordinates: Some(packet::MarkerCoordinates { x: -1, y: -2, z: 3 }),
                },
            ],
        },
    );
}

#[test]
fn typed_helpers_reject_trailing_bits_before_invoking_the_callback() {
    let api = test_api();
    let mut raw = TestEvent::new(
        incoming::ENABLE_STUNT_BONUS.id(),
        EncodedPayload::from_bits(vec![0b1000_0000], 2).unwrap(),
    );
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .unwrap();
    assert!(matches!(
        incoming::ENABLE_STUNT_BONUS.handle(&mut event, |_| panic!("must not dispatch")),
        Err(EventError::UnexpectedBitLength {
            bit_len: 1,
            expected: 0
        })
    ));
}

#[test]
fn marker_sync_keeps_negative_r1_coordinates_as_signed_i16_values() {
    let payload = packet::incoming::MARKERS_SYNC
        .encode(
            test_api(),
            packet::MarkersSync {
                markers: vec![
                    packet::Marker {
                        player_id: 1,
                        coordinates: None,
                    },
                    packet::Marker {
                        player_id: 2,
                        coordinates: Some(packet::MarkerCoordinates { x: -1, y: -2, z: 3 }),
                    },
                ],
            },
        )
        .unwrap();
    assert_eq!(payload.len_bits(), 114);
    assert_eq!(
        payload.as_bytes(),
        &[
            2, 0, 0, 0, 1, 0, 1, 0, 0x7F, 0xFF, 0xFF, 0xBF, 0xC0, 0xC0, 0
        ]
    );
}

#[test]
fn marker_sync_accepts_terminal_byte_alignment_padding() {
    let api = test_api();
    let value = packet::MarkersSync {
        markers: vec![packet::Marker {
            player_id: 1,
            coordinates: None,
        }],
    };
    let canonical = packet::incoming::MARKERS_SYNC
        .encode(api, value.clone())
        .expect("marker payload must encode");
    assert_eq!(canonical.len_bits(), 49);

    let mut bytes = canonical.as_bytes().to_vec();
    // The packet transport can leave its terminal byte's unused bits unspecified.
    *bytes.last_mut().expect("marker payload has a final byte") |= 0x40;
    let padded = EncodedPayload::from_bits(bytes, 56)
        .expect("the rounded marker payload remains in its buffer");
    let mut raw = TestEvent::new(packet::incoming::MARKERS_SYNC.id(), padded);
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .expect("test event is not null");
    assert_eq!(
        packet::incoming::MARKERS_SYNC
            .handle(&mut event, |decoded| {
                assert_eq!(decoded, value);
                RpcAction::Replace(decoded)
            })
            .expect("terminal alignment padding must be accepted"),
        SampClientSdkHookAction::Continue
    );
    assert_eq!(raw.bit_len, canonical.len_bits());
    assert_eq!(raw.bytes, canonical.as_bytes());

    let mut bytes = canonical.as_bytes().to_vec();
    bytes.push(0);
    let mut raw = TestEvent::new(
        packet::incoming::MARKERS_SYNC.id(),
        EncodedPayload::from_bits(bytes, 57).expect("the malformed suffix fits"),
    );
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .expect("test event is not null");
    assert!(matches!(
        packet::incoming::MARKERS_SYNC.handle(&mut event, |_| panic!(
            "a full trailing byte must not dispatch"
        )),
        Err(EventError::UnexpectedBitLength {
            bit_len: 8,
            expected: 0
        })
    ));
}

#[test]
fn set_player_skin_uses_rpc_153_and_two_i32_values() {
    assert_eq!(incoming::SET_PLAYER_SKIN.id(), 153);
    let RpcEncoder::Bytes(encode) = incoming::SET_PLAYER_SKIN.encode else {
        panic!("SetPlayerSkin must use a byte-aligned encoder");
    };
    let bytes = encode(incoming::PlayerSkin {
        player_id: 0,
        skin_id: 411,
    })
    .expect("valid i32 skin payload");

    assert_eq!(bytes, [0, 0, 0, 0, 0x9B, 0x01, 0, 0]);
    assert_eq!(EncodedPayload::from_bytes(bytes).unwrap().len_bits(), 64);
}

#[test]
fn fixed_layout_incoming_rpc_helpers_use_their_protocol_ids() {
    let descriptors = [
        (incoming::CANCEL_EDIT.id(), 28),
        (incoming::SET_TOGGLE_CLOCK.id(), 30),
        (incoming::SET_PLAYER_DRUNK.id(), 35),
        (incoming::SET_RACE_CHECKPOINT.id(), 38),
        (incoming::PLAY_AUDIO_STREAM.id(), 41),
        (incoming::SET_OBJECT_POSITION.id(), 45),
        (incoming::SET_OBJECT_ROTATION.id(), 46),
        (incoming::DESTROY_OBJECT.id(), 47),
        (incoming::PLAYER_DEATH_NOTIFICATION.id(), 55),
        (incoming::SET_MAP_ICON.id(), 56),
        (incoming::REMOVE_VEHICLE_COMPONENT.id(), 57),
        (incoming::REMOVE_3D_TEXT_LABEL.id(), 58),
        (incoming::UPDATE_GLOBAL_TIMER.id(), 60),
        (incoming::DESTROY_PICKUP.id(), 63),
        (incoming::LINK_VEHICLE_TO_INTERIOR.id(), 65),
        (incoming::SET_PLAYER_COLOR.id(), 72),
    ];

    for (actual, expected) in descriptors {
        assert_eq!(actual, expected);
    }
}

#[test]
fn r1_complex_incoming_rpc_helpers_use_their_protocol_ids() {
    let descriptors = [
        (incoming::INIT_GAME.id(), 139),
        (incoming::REQUEST_CLASS_RESPONSE.id(), 128),
        (incoming::PLAYER_STREAM_IN.id(), 32),
        (incoming::CREATE_3D_TEXT.id(), 36),
        (incoming::CREATE_OBJECT.id(), 44),
        (incoming::SET_SPAWN_INFO.id(), 68),
        (incoming::INIT_MENU.id(), 76),
        (incoming::INTERPOLATE_CAMERA.id(), 82),
        (incoming::TOGGLE_SELECT_TEXT_DRAW.id(), 83),
        (incoming::SET_OBJECT_MATERIAL.id(), 84),
        (incoming::APPLY_PLAYER_ANIMATION.id(), 86),
        (incoming::ENABLE_STUNT_BONUS.id(), 104),
        (incoming::PLAY_CRIME_REPORT.id(), 112),
        (incoming::SET_PLAYER_ATTACHED_OBJECT.id(), 113),
        (incoming::ENTER_EDIT_OBJECT.id(), 117),
        (incoming::TOGGLE_PLAYER_SPECTATING.id(), 124),
        (incoming::SHOW_TEXT_DRAW.id(), 134),
        (incoming::TEXT_DRAW_HIDE.id(), 135),
        (incoming::UPDATE_SCORES_AND_PINGS.id(), 155),
        (incoming::VEHICLE_STREAM_IN.id(), 164),
        (incoming::DISABLE_VEHICLE_COLLISIONS.id(), 167),
        (incoming::TOGGLE_CAMERA_TARGET_NOTIFYING.id(), 170),
        (incoming::APPLY_ACTOR_ANIMATION.id(), 173),
    ];
    for (actual, expected) in descriptors {
        assert_eq!(actual, expected);
    }
}

#[test]
fn fixed_layout_incoming_rpc_helpers_encode_exact_vectors() {
    let race_checkpoint = encode_bytes(
        incoming::SET_RACE_CHECKPOINT,
        incoming::RaceCheckpoint {
            checkpoint_type: 2,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            next_position: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            size: 7.0,
        },
    );
    assert_eq!(
        race_checkpoint,
        [
            2, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0x40, 0, 0, 0xA0,
            0x40, 0, 0, 0xC0, 0x40, 0, 0, 0xE0, 0x40,
        ]
    );

    let audio_stream = encode_bytes(
        incoming::PLAY_AUDIO_STREAM,
        incoming::AudioStream {
            url: b"x.y".to_vec(),
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            radius: 4.0,
            use_position: true,
        },
    );
    assert_eq!(
        audio_stream,
        [
            3, b'x', b'.', b'y', 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80,
            0x40, 1,
        ]
    );

    assert_eq!(
        encode_bytes(
            incoming::SET_MAP_ICON,
            incoming::MapIcon {
                icon_id: 7,
                position: Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                icon_type: 4,
                color: -1,
                style: 2,
            },
        ),
        [
            7, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 4, 0xFF, 0xFF, 0xFF, 0xFF, 2,
        ]
    );
    assert_eq!(
        encode_bytes(
            incoming::PLAYER_DEATH_NOTIFICATION,
            incoming::PlayerDeathNotification {
                killer_id: 0x1234,
                killed_id: 0x5678,
                reason: 9,
            },
        ),
        [0x34, 0x12, 0x78, 0x56, 9]
    );
    assert_eq!(
        encode_bytes(
            incoming::SET_PLAYER_COLOR,
            incoming::PlayerColor {
                player_id: 0x1234,
                color: -1,
            },
        ),
        [0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF]
    );
}

#[test]
fn remaining_outgoing_rpc_helpers_use_their_protocol_ids() {
    let descriptors = [
        (outgoing::connection::SEND_CLIENT_JOIN.id(), 25),
        (outgoing::SEND_ENTER_EDIT_OBJECT.id(), 27),
        (outgoing::SEND_MONEY_INCREASE.id(), 31),
        (outgoing::connection::SEND_NPC_JOIN.id(), 54),
        (outgoing::SEND_VEHICLE_TUNING.id(), 96),
        (outgoing::SEND_PICKED_UP_WEAPON.id(), 97),
        (outgoing::SEND_SERVER_STATISTICS_REQUEST.id(), 102),
        (outgoing::SEND_CLIENT_CHECK_RESPONSE.id(), 103),
        (outgoing::SEND_VEHICLE_DAMAGED.id(), 106),
        (outgoing::SEND_DAMAGE.id(), 115),
        (outgoing::SEND_EDIT_ATTACHED_OBJECT.id(), 116),
        (outgoing::SEND_EDIT_OBJECT.id(), 117),
        (outgoing::SEND_PICKED_UP_PICKUP.id(), 131),
        (outgoing::SEND_QUIT_MENU.id(), 140),
        (outgoing::SEND_CAMERA_TARGET_UPDATE.id(), 168),
        (outgoing::SEND_GIVE_ACTOR_DAMAGE.id(), 177),
    ];

    for (actual, expected) in descriptors {
        assert_eq!(actual, expected);
    }
}

#[test]
fn further_fixed_layout_incoming_rpc_helpers_encode_exact_vectors() {
    assert_eq!(incoming::SET_SHOP_NAME.id(), 33);
    assert_eq!(incoming::CREATE_GANG_ZONE.id(), 108);
    assert_eq!(incoming::SET_VEHICLE_PARAMS_EX.id(), 24);
    assert_eq!(incoming::CREATE_ACTOR.id(), 171);

    assert_eq!(
        encode_bytes(
            incoming::CREATE_GANG_ZONE,
            incoming::GangZone {
                zone_id: 0x1234,
                square_start: Vector2 { x: 1.0, y: 2.0 },
                square_end: Vector2 { x: 3.0, y: 4.0 },
                color: -1,
            },
        ),
        [
            0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0x40, 0xFF,
            0xFF, 0xFF, 0xFF,
        ]
    );
    assert_eq!(
        encode_bytes(
            incoming::SET_VEHICLE_PARAMS_EX,
            incoming::VehicleParamsEx {
                vehicle_id: 1,
                params: [2; 8],
                doors: [3; 4],
                windows: [4; 4],
            },
        ),
        [1, 0, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4]
    );
    assert_eq!(
        encode_bytes(
            incoming::CREATE_ACTOR,
            incoming::Actor {
                actor_id: 7,
                skin_id: 411,
                position: Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                rotation: 4.0,
                health: 5.0,
            },
        ),
        [
            7, 0, 0x9B, 1, 0, 0, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80,
            0x40, 0, 0, 0xA0, 0x40,
        ]
    );
}

#[test]
fn outgoing_damage_keeps_its_one_bit_boolean_and_exact_payload_length() {
    let payload = outgoing::SEND_DAMAGE
        .encode(
            test_api(),
            outgoing::Damage {
                player_id: 0x1234,
                damage: 1.0,
                weapon: 24,
                body_part: 9,
                take: true,
            },
        )
        .expect("damage payload must encode");

    assert_eq!(payload.len_bits(), 113);
    assert_eq!(
        payload.as_bytes(),
        [
            0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00, 0x00,
            0x00,
        ]
    );
}

#[test]
fn packet_helpers_filter_the_documented_packet_ids() {
    assert_eq!(packet::outgoing::SEND_AUTHENTICATION_RESPONSE.id(), 12);
    assert_eq!(packet::outgoing::SEND_WEAPONS_UPDATE.id(), 204);
    assert_eq!(packet::outgoing::SEND_RCON_COMMAND.id(), 201);
    assert_eq!(packet::outgoing::SEND_STATS_UPDATE.id(), 205);
    assert_eq!(packet::outgoing::SEND_PLAYER_SYNC.id(), 207);
    assert_eq!(packet::outgoing::SEND_VEHICLE_SYNC.id(), 200);
    assert_eq!(packet::outgoing::SEND_PASSENGER_SYNC.id(), 211);
    assert_eq!(packet::outgoing::SEND_AIM_SYNC.id(), 203);
    assert_eq!(packet::outgoing::SEND_UNOCCUPIED_SYNC.id(), 209);
    assert_eq!(packet::outgoing::SEND_TRAILER_SYNC.id(), 210);
    assert_eq!(packet::outgoing::SEND_BULLET_SYNC.id(), 206);
    assert_eq!(packet::outgoing::SEND_SPECTATOR_SYNC.id(), 212);
    assert_eq!(packet::incoming::AIM_SYNC.id(), 203);
    assert_eq!(packet::incoming::VEHICLE_SYNC.id(), 200);
    assert_eq!(packet::incoming::BULLET_SYNC.id(), 206);
    assert_eq!(packet::incoming::PLAYER_SYNC.id(), 207);
    assert_eq!(packet::incoming::MARKERS_SYNC.id(), 208);
    assert_eq!(packet::incoming::UNOCCUPIED_SYNC.id(), 209);
    assert_eq!(packet::incoming::TRAILER_SYNC.id(), 210);
    assert_eq!(packet::incoming::PASSENGER_SYNC.id(), 211);
    assert_eq!(packet::incoming::AUTHENTICATION_REQUEST.id(), 12);
    assert_eq!(packet::incoming::CONNECTION_ACCEPTED.id(), 34);
    assert_eq!(packet::incoming::CONNECTION_LOST.id(), 33);
    assert_eq!(packet::incoming::CONNECTION_BANNED.id(), 36);
    assert_eq!(packet::incoming::CONNECTION_ATTEMPT_FAILED.id(), 29);
    assert_eq!(packet::incoming::CONNECTION_NO_FREE_SLOT.id(), 31);
    assert_eq!(packet::incoming::CONNECTION_PASSWORD_INVALID.id(), 37);
    assert_eq!(packet::incoming::CONNECTION_CLOSED.id(), 32);
}

#[test]
fn packet_helpers_encode_exact_fixed_layout_vectors() {
    assert_eq!(
        encode_bytes(
            packet::outgoing::SEND_STATS_UPDATE,
            packet::StatsUpdate {
                money: -1,
                drunk_level: 42,
            },
        ),
        [0xFF, 0xFF, 0xFF, 0xFF, 42, 0, 0, 0]
    );
    assert_eq!(
        encode_bytes(
            packet::outgoing::SEND_WEAPONS_UPDATE,
            packet::WeaponsUpdate {
                player_target: 1,
                actor_target: 2,
                weapons: vec![packet::WeaponSlot {
                    slot: 3,
                    weapon: 24,
                    ammo: 50,
                }],
            },
        ),
        [1, 0, 2, 0, 3, 24, 50, 0]
    );

    let aim = packet::AimSync {
        camera_mode: 7,
        camera_front: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        camera_position: Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        aim_z: 7.0,
        zoom_and_weapon_state: 0b1010_0101,
        aspect_ratio: 9,
    };
    assert_eq!(
        encode_bytes(packet::outgoing::SEND_AIM_SYNC, aim),
        [
            7,
            0,
            0,
            0x80,
            0x3F,
            0,
            0,
            0,
            0x40,
            0,
            0,
            0x40,
            0x40,
            0,
            0,
            0x80,
            0x40,
            0,
            0,
            0xA0,
            0x40,
            0,
            0,
            0xC0,
            0x40,
            0,
            0,
            0xE0,
            0x40,
            0b1010_0101,
            9,
        ]
    );

    let bullet = packet::BulletSync {
        target_type: 1,
        target_id: 0x1234,
        origin: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        target: Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        center: Vector3 {
            x: 7.0,
            y: 8.0,
            z: 9.0,
        },
        weapon_id: 24,
    };
    let bytes = encode_bytes(packet::outgoing::SEND_BULLET_SYNC, bullet);
    assert_eq!(bytes.len(), 40);
    assert_eq!(
        &bytes[..15],
        &[
            1, 0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40
        ]
    );
    assert_eq!(
        &bytes[27..],
        &[0, 0, 0xE0, 0x40, 0, 0, 0, 0x41, 0, 0, 0x10, 0x41, 24]
    );

    let player = packet::PlayerSync {
        left_right_keys: 1,
        up_down_keys: 2,
        key_data: 3,
        position: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        quaternion: [0.0; 4],
        health: 4,
        armour: 5,
        weapon_and_special_key: 6,
        special_action: 7,
        move_speed: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        surfing_offsets: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        surfing_vehicle_id: 8,
        animation_id: 9,
        animation_flags: 10,
    };
    let bytes = encode_bytes(packet::outgoing::SEND_PLAYER_SYNC, player);
    assert_eq!(bytes.len(), 68);
    assert_eq!(&bytes[..6], &[1, 0, 2, 0, 3, 0]);
    assert_eq!(&bytes[34..38], &[4, 5, 6, 7]);
    assert_eq!(&bytes[62..], &[8, 0, 9, 0, 10, 0]);
}
