use samp_protocol::rpc::incoming::{
    ATTACH_CAMERA_TO_OBJECT, ATTACH_TRAILER_TO_VEHICLE, CLEAR_PLAYER_ANIMATION,
    CONNECTION_REJECTED, CREATE_GANG_ZONE, CREATE_PICKUP, CameraLookAt,
    DETACH_TRAILER_FROM_VEHICLE, GANG_ZONE_DESTROY, GANG_ZONE_FLASH, GANG_ZONE_STOP_FLASH,
    GangZone, MOVE_OBJECT, MoveObject, PLAYER_DEATH, PLAYER_ENTER_VEHICLE, PLAYER_EXIT_VEHICLE,
    Pickup, PlayerEnterVehicle, PlayerExitVehicle, PlayerFightingStyle, REMOVE_MAP_ICON,
    SET_CAMERA_LOOK_AT, SET_CAMERA_POSITION, SET_GRAVITY, SET_PLAYER_FIGHTING_STYLE,
    SET_PLAYER_SPECIAL_ACTION, SET_PLAYER_VELOCITY, SET_VEHICLE_NUMBER_PLATE, SET_VEHICLE_PARAMS,
    SET_VEHICLE_VELOCITY, SET_WEAPON_AMMO, SPECTATE_PLAYER, SPECTATE_VEHICLE, STOP_OBJECT,
    Spectate, TEXT_DRAW_SET_STRING, TextDrawSetString, TextDrawString, TrailerAttachment, Vector2,
    Vector3, VehicleNumberPlate, VehicleParams, VehicleVelocity, WeaponAmmo,
};
use samp_protocol::{DecodeError, EncodeError, EncodedBits, WireDescriptor};

fn id<D: WireDescriptor>(_: D) -> u8 {
    D::ID
}

fn assert_vector<D>(_descriptor: D, value: &D::Value, expected: &[u8])
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let bits = D::encode_bits(value).expect("the fixed RPC value must encode");

    assert_eq!(bits.as_bytes(), expected);
    assert_eq!(bits.len_bits(), expected.len() * 8);
    assert_eq!(D::decode_bits(&bits), Ok(value.clone()));
}

#[test]
fn phase15_fixed_incoming_rpc_inventory_has_29_unique_entries() {
    let ids = [
        id(ATTACH_CAMERA_TO_OBJECT),
        id(GANG_ZONE_STOP_FLASH),
        id(CLEAR_PLAYER_ANIMATION),
        id(SET_PLAYER_SPECIAL_ACTION),
        id(SET_PLAYER_FIGHTING_STYLE),
        id(SET_PLAYER_VELOCITY),
        id(SET_VEHICLE_VELOCITY),
        id(CREATE_PICKUP),
        id(MOVE_OBJECT),
        id(TEXT_DRAW_SET_STRING),
        id(CREATE_GANG_ZONE),
        id(GANG_ZONE_DESTROY),
        id(GANG_ZONE_FLASH),
        id(STOP_OBJECT),
        id(SET_VEHICLE_NUMBER_PLATE),
        id(SPECTATE_PLAYER),
        id(SPECTATE_VEHICLE),
        id(CONNECTION_REJECTED),
        id(REMOVE_MAP_ICON),
        id(SET_WEAPON_AMMO),
        id(SET_GRAVITY),
        id(ATTACH_TRAILER_TO_VEHICLE),
        id(DETACH_TRAILER_FROM_VEHICLE),
        id(SET_CAMERA_POSITION),
        id(SET_CAMERA_LOOK_AT),
        id(SET_VEHICLE_PARAMS),
        id(PLAYER_DEATH),
        id(PLAYER_ENTER_VEHICLE),
        id(PLAYER_EXIT_VEHICLE),
    ];

    assert_eq!(
        ids,
        [
            81, 85, 87, 88, 89, 90, 91, 95, 99, 105, 108, 120, 121, 122, 123, 126, 127, 130, 144,
            145, 146, 148, 149, 157, 158, 161, 166, 26, 154,
        ]
    );

    let mut unique = ids;
    unique.sort_unstable();
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn phase15_fixed_incoming_rpcs_preserve_exact_vectors() {
    assert_vector(ATTACH_CAMERA_TO_OBJECT, &0x1234, &[0x34, 0x12]);
    assert_vector(GANG_ZONE_STOP_FLASH, &0x5678, &[0x78, 0x56]);
    assert_vector(CLEAR_PLAYER_ANIMATION, &0x9ABC, &[0xBC, 0x9A]);
    assert_vector(SET_PLAYER_SPECIAL_ACTION, &0xDE, &[0xDE]);
    assert_vector(
        SET_PLAYER_FIGHTING_STYLE,
        &PlayerFightingStyle {
            player_id: 0x1234,
            style_id: 0x56,
        },
        &[0x34, 0x12, 0x56],
    );
    assert_vector(
        SET_PLAYER_VELOCITY,
        &Vector3 {
            x: 1.0,
            y: -2.0,
            z: 0.5,
        },
        &[0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0, 0x3F],
    );
    assert_vector(
        SET_VEHICLE_VELOCITY,
        &VehicleVelocity {
            turn: true,
            velocity: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
        &[1, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40],
    );
    assert_vector(
        CREATE_PICKUP,
        &Pickup {
            id: -1,
            model: 411,
            pickup_type: 2,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
        &[
            0xFF, 0xFF, 0xFF, 0xFF, 0x9B, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0,
            0, 0x40, 0x40,
        ],
    );
    assert_vector(
        MOVE_OBJECT,
        &MoveObject {
            object_id: 0x1234,
            from_position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            destination: Vector3 {
                x: -1.0,
                y: -2.0,
                z: -3.0,
            },
            speed: 4.0,
            rotation: Vector3 {
                x: 5.0,
                y: 6.0,
                z: 7.0,
            },
        },
        &[
            0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0xBF, 0, 0,
            0, 0xC0, 0, 0, 0x40, 0xC0, 0, 0, 0x80, 0x40, 0, 0, 0xA0, 0x40, 0, 0, 0xC0, 0x40, 0, 0,
            0xE0, 0x40,
        ],
    );
    assert_vector(
        TEXT_DRAW_SET_STRING,
        &TextDrawString {
            textdraw_id: 0x1234,
            text: b"abc".to_vec(),
        },
        &[0x34, 0x12, 3, 0, b'a', b'b', b'c'],
    );
    assert_vector(
        CREATE_GANG_ZONE,
        &GangZone {
            zone_id: 0x1234,
            square_start: Vector2 { x: 1.0, y: 2.0 },
            square_end: Vector2 { x: 3.0, y: 4.0 },
            color: -1,
        },
        &[
            0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0x40, 0xFF,
            0xFF, 0xFF, 0xFF,
        ],
    );
    assert_vector(GANG_ZONE_DESTROY, &0x5678, &[0x78, 0x56]);
    assert_vector(
        GANG_ZONE_FLASH,
        &(0x1234, -1),
        &[0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    assert_vector(STOP_OBJECT, &0xBC9A, &[0x9A, 0xBC]);
    assert_vector(
        SET_VEHICLE_NUMBER_PLATE,
        &VehicleNumberPlate {
            vehicle_id: 0x1234,
            text: b"VPL".to_vec(),
        },
        &[0x34, 0x12, 3, b'V', b'P', b'L'],
    );
    assert_vector(
        SPECTATE_PLAYER,
        &Spectate {
            target_id: 0x1234,
            camera_type: 0x56,
        },
        &[0x34, 0x12, 0x56],
    );
    assert_vector(
        SPECTATE_VEHICLE,
        &Spectate {
            target_id: 0x5678,
            camera_type: 0x9A,
        },
        &[0x78, 0x56, 0x9A],
    );
    assert_vector(CONNECTION_REJECTED, &0xBC, &[0xBC]);
    assert_vector(REMOVE_MAP_ICON, &0xDE, &[0xDE]);
    assert_vector(
        SET_WEAPON_AMMO,
        &WeaponAmmo {
            weapon_id: 0x56,
            ammo: 0x1234,
        },
        &[0x56, 0x34, 0x12],
    );
    assert_vector(SET_GRAVITY, &1.0, &[0, 0, 0x80, 0x3F]);
    assert_vector(
        ATTACH_TRAILER_TO_VEHICLE,
        &TrailerAttachment {
            trailer_id: 0x1234,
            vehicle_id: 0x5678,
        },
        &[0x34, 0x12, 0x78, 0x56],
    );
    assert_vector(DETACH_TRAILER_FROM_VEHICLE, &0x9ABC, &[0xBC, 0x9A]);
    assert_vector(
        SET_CAMERA_POSITION,
        &Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        &[0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40],
    );
    assert_vector(
        SET_CAMERA_LOOK_AT,
        &CameraLookAt {
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            cut_type: 4,
        },
        &[0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 4],
    );
    assert_vector(
        SET_VEHICLE_PARAMS,
        &VehicleParams {
            vehicle_id: 0x1234,
            objective: true,
            doors_locked: false,
        },
        &[0x34, 0x12, 1, 0],
    );
    assert_vector(PLAYER_DEATH, &0x5678, &[0x78, 0x56]);
    assert_vector(
        PLAYER_ENTER_VEHICLE,
        &PlayerEnterVehicle {
            player_id: 0x1234,
            vehicle_id: 0x5678,
            passenger: true,
        },
        &[0x34, 0x12, 0x78, 0x56, 1],
    );
    assert_vector(
        PLAYER_EXIT_VEHICLE,
        &PlayerExitVehicle {
            player_id: 0x9ABC,
            vehicle_id: 0xDEF0,
        },
        &[0xBC, 0x9A, 0xF0, 0xDE],
    );

    let nonzero_bool = EncodedBits::from_bits([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 104)
        .expect("the nonzero Boolean vector must be valid");
    assert_eq!(
        <samp_protocol::rpc::incoming::SetVehicleVelocity as WireDescriptor>::decode_bits(
            &nonzero_bool,
        ),
        Ok(VehicleVelocity {
            turn: true,
            velocity: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        })
    );
}

#[test]
fn phase15_fixed_incoming_rpcs_reject_malformed_values() {
    fn assert_rejects_malformed_value<D>(_descriptor: D, value: &D::Value)
    where
        D: WireDescriptor,
        D::Value: Clone + core::fmt::Debug + PartialEq,
    {
        let encoded = D::encode_bits(value).expect("the fixed RPC value must encode");
        if encoded.len_bits() > 0 {
            let truncated_bytes = encoded.as_bytes()[..encoded.as_bytes().len() - 1].to_vec();
            let truncated = EncodedBits::from_bits(truncated_bytes, encoded.len_bits() - 8)
                .expect("the truncated fixed RPC payload must be valid bits");
            assert!(D::decode_bits(&truncated).is_err());
        }

        let mut bytes = encoded.as_bytes().to_vec();
        bytes.push(0);
        let trailing = EncodedBits::from_bits(bytes, encoded.len_bits() + 8)
            .expect("the trailing fixed RPC payload must be valid bits");
        assert_eq!(
            D::decode_bits(&trailing),
            Err(DecodeError::UnexpectedTrailingBits {
                remaining_bits: 8,
                allowed_bits: 0,
            })
        );
    }

    assert_rejects_malformed_value(ATTACH_CAMERA_TO_OBJECT, &0);
    assert_rejects_malformed_value(GANG_ZONE_STOP_FLASH, &0);
    assert_rejects_malformed_value(CLEAR_PLAYER_ANIMATION, &0);
    assert_rejects_malformed_value(SET_PLAYER_SPECIAL_ACTION, &0);
    assert_rejects_malformed_value(
        SET_PLAYER_FIGHTING_STYLE,
        &PlayerFightingStyle {
            player_id: 0,
            style_id: 0,
        },
    );
    assert_rejects_malformed_value(
        SET_PLAYER_VELOCITY,
        &Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    );
    assert_rejects_malformed_value(
        SET_VEHICLE_VELOCITY,
        &VehicleVelocity {
            turn: false,
            velocity: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
    );
    assert_rejects_malformed_value(
        CREATE_PICKUP,
        &Pickup {
            id: 0,
            model: 0,
            pickup_type: 0,
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
    );
    assert_rejects_malformed_value(
        MOVE_OBJECT,
        &MoveObject {
            object_id: 0,
            from_position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            destination: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            speed: 0.0,
            rotation: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
    );
    assert_rejects_malformed_value(
        TEXT_DRAW_SET_STRING,
        &TextDrawString {
            textdraw_id: 0,
            text: Vec::new(),
        },
    );
    assert_rejects_malformed_value(
        CREATE_GANG_ZONE,
        &GangZone {
            zone_id: 0,
            square_start: Vector2 { x: 0.0, y: 0.0 },
            square_end: Vector2 { x: 0.0, y: 0.0 },
            color: 0,
        },
    );
    assert_rejects_malformed_value(GANG_ZONE_DESTROY, &0);
    assert_rejects_malformed_value(GANG_ZONE_FLASH, &(0, 0));
    assert_rejects_malformed_value(STOP_OBJECT, &0);
    assert_rejects_malformed_value(
        SET_VEHICLE_NUMBER_PLATE,
        &VehicleNumberPlate {
            vehicle_id: 0,
            text: Vec::new(),
        },
    );
    assert_rejects_malformed_value(
        SPECTATE_PLAYER,
        &Spectate {
            target_id: 0,
            camera_type: 0,
        },
    );
    assert_rejects_malformed_value(
        SPECTATE_VEHICLE,
        &Spectate {
            target_id: 0,
            camera_type: 0,
        },
    );
    assert_rejects_malformed_value(CONNECTION_REJECTED, &0);
    assert_rejects_malformed_value(REMOVE_MAP_ICON, &0);
    assert_rejects_malformed_value(
        SET_WEAPON_AMMO,
        &WeaponAmmo {
            weapon_id: 0,
            ammo: 0,
        },
    );
    assert_rejects_malformed_value(SET_GRAVITY, &0.0);
    assert_rejects_malformed_value(
        ATTACH_TRAILER_TO_VEHICLE,
        &TrailerAttachment {
            trailer_id: 0,
            vehicle_id: 0,
        },
    );
    assert_rejects_malformed_value(DETACH_TRAILER_FROM_VEHICLE, &0);
    assert_rejects_malformed_value(
        SET_CAMERA_POSITION,
        &Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    );
    assert_rejects_malformed_value(
        SET_CAMERA_LOOK_AT,
        &CameraLookAt {
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cut_type: 0,
        },
    );
    assert_rejects_malformed_value(
        SET_VEHICLE_PARAMS,
        &VehicleParams {
            vehicle_id: 0,
            objective: false,
            doors_locked: false,
        },
    );
    assert_rejects_malformed_value(PLAYER_DEATH, &0);
    assert_rejects_malformed_value(
        PLAYER_ENTER_VEHICLE,
        &PlayerEnterVehicle {
            player_id: 0,
            vehicle_id: 0,
            passenger: false,
        },
    );
    assert_rejects_malformed_value(
        PLAYER_EXIT_VEHICLE,
        &PlayerExitVehicle {
            player_id: 0,
            vehicle_id: 0,
        },
    );

    let oversized = TextDrawString {
        textdraw_id: 0,
        text: vec![0; 4097],
    };
    assert_eq!(
        TextDrawSetString::encode_bits(&oversized),
        Err(EncodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );

    let oversized_length = EncodedBits::from_bits([0, 0, 1, 0x10], 32)
        .expect("the oversized text length must be valid bits");
    assert!(matches!(
        TextDrawSetString::decode_bits(&oversized_length),
        Err(DecodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    ));
}
