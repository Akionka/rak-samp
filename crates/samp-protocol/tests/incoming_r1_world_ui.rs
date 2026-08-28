use samp_protocol::{
    DecodeError, EncodeError, EncodedBits, TrailingPolicy, WireDescriptor,
    limits::MAX_STRING32_BYTES,
    rpc::incoming::r1::*,
    types::{Vector2, Vector3},
};

fn vector2(x: f32, y: f32) -> Vector2 {
    Vector2 { x, y }
}

fn vector3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
}

fn animation() -> Animation {
    Animation {
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

fn menu() -> InitMenu {
    InitMenu {
        menu_id: 1,
        two_columns: true,
        title: *b"R1 menu                         ",
        position: vector2(10.0, 20.0),
        columns: vec![
            MenuColumn {
                width: 100.0,
                title: *b"first                           ",
                rows: vec![*b"one                             "],
            },
            MenuColumn {
                width: 200.0,
                title: *b"second                          ",
                rows: vec![*b"two                             "],
            },
        ],
        rows: [-1; MAX_MENU_ROWS],
        menu: false,
    }
}

fn textdraw() -> ShowTextDraw {
    ShowTextDraw {
        textdraw_id: 99,
        textdraw: TextDraw {
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
            position: vector2(100.0, 200.0),
            model_id: 1234,
            rotation: vector3(0.0, 0.0, 1.0),
            zoom: 1.5,
            color1: -1,
            color2: 2,
            text: b"textdraw".to_vec(),
        },
    }
}

fn vehicle() -> VehicleStreamIn {
    VehicleStreamIn {
        vehicle_id: 9,
        vehicle: StreamedVehicle {
            model: 411,
            position: vector3(1.0, 2.0, 3.0),
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
    }
}

fn assert_round_trip<D>(descriptor: D, value: D::Value, expected_bits: usize)
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let _ = descriptor;
    let encoded = D::encode_bits(&value).expect("the R1 RPC value must encode");
    assert_eq!(encoded.len_bits(), expected_bits);
    assert_eq!(D::decode_bits(&encoded), Ok(value));
}

fn id<D: WireDescriptor>(_: D) -> u8 {
    D::ID
}

#[test]
fn r1_world_ui_vehicle_and_actor_descriptors_have_expected_ids_and_policies() {
    assert_eq!(
        [
            id(INIT_MENU),
            id(INTERPOLATE_CAMERA),
            id(TOGGLE_SELECT_TEXT_DRAW),
            id(ENTER_EDIT_OBJECT),
            id(SHOW_TEXT_DRAW),
            id(TEXT_DRAW_HIDE),
            id(VEHICLE_STREAM_IN),
            id(DISABLE_VEHICLE_COLLISIONS),
            id(TOGGLE_CAMERA_TARGET_NOTIFYING),
            id(APPLY_ACTOR_ANIMATION),
        ],
        [76, 82, 83, 117, 134, 135, 164, 167, 170, 173]
    );
    assert_eq!(InitMenuRpc::TRAILING_POLICY, TrailingPolicy::ExactBytes);
    assert_eq!(ShowTextDrawRpc::TRAILING_POLICY, TrailingPolicy::ExactBytes);
    assert_eq!(
        VehicleStreamInRpc::TRAILING_POLICY,
        TrailingPolicy::ExactBytes
    );
    assert_eq!(
        InterpolateCameraRpc::TRAILING_POLICY,
        TrailingPolicy::ExactBits
    );
    assert_eq!(
        ApplyActorAnimationRpc::TRAILING_POLICY,
        TrailingPolicy::ExactBits
    );
}

#[test]
fn r1_world_ui_vehicle_and_actor_values_keep_exact_lengths() {
    assert_round_trip(INIT_MENU, menu(), 1_880);
    assert_round_trip(
        INTERPOLATE_CAMERA,
        InterpolateCamera {
            set_position: true,
            from_position: vector3(1.0, 2.0, 3.0),
            destination: vector3(4.0, 5.0, 6.0),
            time_ms: 500,
            mode: 2,
        },
        233,
    );
    assert_round_trip(
        TOGGLE_SELECT_TEXT_DRAW,
        ToggleSelectTextDraw {
            enabled: true,
            hover_color: -1,
        },
        33,
    );
    assert_round_trip(
        ENTER_EDIT_OBJECT,
        EnterEditObject {
            player_object: true,
            object_id: 5,
        },
        17,
    );
    assert_round_trip(SHOW_TEXT_DRAW, textdraw(), 600);
    assert_round_trip(TEXT_DRAW_HIDE, 99, 16);
    assert_round_trip(VEHICLE_STREAM_IN, vehicle(), 504);
    assert_round_trip(DISABLE_VEHICLE_COLLISIONS, true, 1);
    assert_round_trip(TOGGLE_CAMERA_TARGET_NOTIFYING, false, 1);
    assert_round_trip(
        APPLY_ACTOR_ANIMATION,
        ActorAnimation {
            actor_id: 8,
            animation: animation(),
        },
        156,
    );
}

#[test]
fn r1_world_ui_vehicle_and_actor_exact_vectors_match_the_sdk_layout() {
    assert_eq!(
        InterpolateCameraRpc::encode_bits(&InterpolateCamera {
            set_position: true,
            from_position: vector3(0.0, 0.0, 0.0),
            destination: vector3(0.0, 0.0, 0.0),
            time_ms: 0,
            mode: 0,
        }),
        Ok(EncodedBits::from_bits(
            [
                0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ],
            233,
        )
        .unwrap())
    );
    assert_eq!(
        ToggleSelectTextDrawRpc::encode_bits(&ToggleSelectTextDraw {
            enabled: true,
            hover_color: -1,
        }),
        Ok(EncodedBits::from_bits([0xFF, 0xFF, 0xFF, 0xFF, 0x80], 33).unwrap())
    );
    assert_eq!(
        EnterEditObjectRpc::encode_bits(&EnterEditObject {
            player_object: true,
            object_id: 5,
        }),
        Ok(EncodedBits::from_bits([0x82, 0x80, 0x00], 17).unwrap())
    );
    assert_eq!(
        TextDrawHideRpc::encode_bits(&99),
        Ok(EncodedBits::from_bits([0x63, 0x00], 16).unwrap())
    );
    assert_eq!(
        DisableVehicleCollisionsRpc::encode_bits(&true),
        Ok(EncodedBits::from_bits([0x80], 1).unwrap())
    );
    assert_eq!(
        ToggleCameraTargetNotifyingRpc::encode_bits(&false),
        Ok(EncodedBits::from_bits([0x00], 1).unwrap())
    );
    assert_eq!(
        ApplyActorAnimationRpc::encode_bits(&ActorAnimation {
            actor_id: 8,
            animation: animation(),
        }),
        Ok(EncodedBits::from_bits(
            [
                0x08, 0x00, 0x03, b'P', b'E', b'D', 0x04, b'W', b'A', b'L', b'K', 0x00, 0x00, 0x80,
                0x40, 0xAF, 0xFF, 0xFF, 0xFF, 0xF0,
            ],
            156,
        )
        .unwrap())
    );
    assert_eq!(
        ShowTextDrawRpc::encode_bits(&textdraw())
            .unwrap()
            .as_bytes(),
        &[
            0x63, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x80, 0x3F, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, 0x00, 0x00, 0xC8, 0x42, 0x00, 0x00, 0x48,
            0x43, 0xD2, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            0x3F, 0x00, 0x00, 0xC0, 0x3F, 0xFF, 0xFF, 0x02, 0x00, 0x08, 0x00, b't', b'e', b'x',
            b't', b'd', b'r', b'a', b'w',
        ]
    );
    assert_eq!(
        VehicleStreamInRpc::encode_bits(&vehicle())
            .unwrap()
            .as_bytes(),
        &[
            0x09, 0x00, 0x9B, 0x01, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x40,
            0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x34, 0x42, 0x01, 0x02, 0x00, 0x00, 0x61, 0x44,
            0x03, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x06, 0x07, 0x08, 0x09, 0x09,
            0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09, 0x0A, 0x0B,
            0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00,
        ]
    );
}

#[test]
fn r1_world_ui_and_actor_limits_are_enforced() {
    let mut missing_menu_column = menu();
    missing_menu_column.columns.pop();
    assert_eq!(
        InitMenuRpc::encode_bits(&missing_menu_column),
        Err(EncodeError::InvalidCollectionLength {
            length: 1,
            expected: 2,
        })
    );

    let mut oversized_menu = menu();
    oversized_menu.columns[0].rows = vec![[0; 32]; MAX_MENU_ROWS + 1];
    assert_eq!(
        InitMenuRpc::encode_bits(&oversized_menu),
        Err(EncodeError::LengthExceedsLimit {
            length: MAX_MENU_ROWS + 1,
            limit: MAX_MENU_ROWS,
        })
    );

    let mut oversized_textdraw = textdraw();
    oversized_textdraw.textdraw.text = vec![0; MAX_STRING32_BYTES + 1];
    assert_eq!(
        ShowTextDrawRpc::encode_bits(&oversized_textdraw),
        Err(EncodeError::LengthExceedsLimit {
            length: MAX_STRING32_BYTES + 1,
            limit: MAX_STRING32_BYTES,
        })
    );

    let oversized = vec![b'x'; usize::from(u8::MAX) + 1];
    let actor = ActorAnimation {
        actor_id: 8,
        animation: Animation {
            animation_library: oversized,
            ..animation()
        },
    };
    assert_eq!(
        ApplyActorAnimationRpc::encode_bits(&actor),
        Err(EncodeError::LengthExceedsLimit {
            length: usize::from(u8::MAX) + 1,
            limit: usize::from(u8::MAX),
        })
    );
}

#[test]
fn r1_world_ui_descriptors_reject_semantic_trailing_bits() {
    let encoded = EnterEditObjectRpc::encode_bits(&EnterEditObject {
        player_object: true,
        object_id: 5,
    })
    .unwrap();
    let trailing = EncodedBits::from_bits(encoded.as_bytes(), 18).unwrap();
    assert_eq!(
        EnterEditObjectRpc::decode_bits(&trailing),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 1,
            allowed_bits: 0,
        })
    );

    let encoded = TextDrawHideRpc::encode_bits(&99).unwrap();
    let trailing = EncodedBits::from_bits([encoded.as_bytes(), &[0x00]].concat(), 24).unwrap();
    assert_eq!(
        TextDrawHideRpc::decode_bits(&trailing),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 8,
            allowed_bits: 0,
        })
    );
}
