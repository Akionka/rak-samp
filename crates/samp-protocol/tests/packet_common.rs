use samp_protocol::packet::common::{
    AIM_SYNC, AUTHENTICATION_REQUEST, AimSync, BULLET_SYNC, BulletSync, CONNECTION_ACCEPTED,
    CONNECTION_ATTEMPT_FAILED, CONNECTION_BANNED, CONNECTION_CLOSED, CONNECTION_LOST,
    CONNECTION_NO_FREE_SLOT, CONNECTION_PASSWORD_INVALID, ConnectionAccepted, PASSENGER_SYNC,
    PassengerSync, PlayerSync, RemoteSync, SEND_AIM_SYNC, SEND_AUTHENTICATION_RESPONSE,
    SEND_BULLET_SYNC, SEND_PASSENGER_SYNC, SEND_PLAYER_SYNC, SEND_RCON_COMMAND,
    SEND_SPECTATOR_SYNC, SEND_STATS_UPDATE, SEND_TRAILER_SYNC, SEND_UNOCCUPIED_SYNC,
    SEND_VEHICLE_SYNC, SEND_WEAPONS_UPDATE, SendRconCommand, SendStatsUpdate, SendWeaponsUpdate,
    SpectatorSync, StatsUpdate, TRAILER_SYNC, TrailerSync, UNOCCUPIED_SYNC, UnoccupiedSync,
    VehicleSync, WeaponSlot, WeaponsUpdate,
};
use samp_protocol::{DecodeError, EncodeError, EncodedBits, WireDescriptor, types::Vector3};

fn id<D: WireDescriptor>(_: D) -> u8 {
    D::ID
}

fn assert_vector<D>(descriptor: D, value: D::Value, expected: &[u8])
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let _ = descriptor;
    let bits = D::encode_bits(&value).expect("the packet value must encode");

    assert_eq!(bits.as_bytes(), expected);
    assert_eq!(bits.len_bits(), expected.len() * 8);
    assert_eq!(D::decode_bits(&bits), Ok(value));
}

#[test]
fn common_packet_inventory_has_25_descriptors() {
    assert_eq!(
        [
            id(SEND_RCON_COMMAND),
            id(SEND_AUTHENTICATION_RESPONSE),
            id(SEND_STATS_UPDATE),
            id(SEND_WEAPONS_UPDATE),
            id(SEND_PLAYER_SYNC),
            id(SEND_VEHICLE_SYNC),
            id(SEND_PASSENGER_SYNC),
            id(SEND_AIM_SYNC),
            id(SEND_UNOCCUPIED_SYNC),
            id(SEND_TRAILER_SYNC),
            id(SEND_BULLET_SYNC),
            id(SEND_SPECTATOR_SYNC),
            id(AUTHENTICATION_REQUEST),
            id(CONNECTION_ACCEPTED),
            id(CONNECTION_LOST),
            id(CONNECTION_BANNED),
            id(CONNECTION_ATTEMPT_FAILED),
            id(CONNECTION_NO_FREE_SLOT),
            id(CONNECTION_PASSWORD_INVALID),
            id(CONNECTION_CLOSED),
            id(AIM_SYNC),
            id(BULLET_SYNC),
            id(UNOCCUPIED_SYNC),
            id(TRAILER_SYNC),
            id(PASSENGER_SYNC),
        ],
        [
            201, 12, 205, 204, 207, 200, 211, 203, 209, 210, 206, 212, 12, 34, 33, 36, 29, 31, 37,
            32, 203, 206, 209, 210, 211,
        ]
    );
}

#[test]
fn common_packets_preserve_exact_vectors() {
    let zero = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let aim = AimSync {
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
    let bullet = BulletSync {
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

    assert_vector(
        SEND_RCON_COMMAND,
        b"rcon".to_vec(),
        &[4, 0, 0, 0, b'r', b'c', b'o', b'n'],
    );
    assert_vector(
        SEND_AUTHENTICATION_RESPONSE,
        b"ok".to_vec(),
        &[2, b'o', b'k'],
    );
    assert_vector(
        SEND_STATS_UPDATE,
        StatsUpdate {
            money: -1,
            drunk_level: 42,
        },
        &[0xFF, 0xFF, 0xFF, 0xFF, 42, 0, 0, 0],
    );
    assert_vector(
        SEND_WEAPONS_UPDATE,
        WeaponsUpdate {
            player_target: 1,
            actor_target: 2,
            weapons: vec![WeaponSlot {
                slot: 3,
                weapon: 24,
                ammo: 50,
            }],
        },
        &[1, 0, 2, 0, 3, 24, 50, 0],
    );
    let mut player_expected = vec![0; 68];
    player_expected[..6].copy_from_slice(&[1, 0, 2, 0, 3, 0]);
    player_expected[34..38].copy_from_slice(&[4, 5, 6, 7]);
    player_expected[62..].copy_from_slice(&[8, 0, 9, 0, 10, 0]);
    assert_vector(
        SEND_PLAYER_SYNC,
        PlayerSync {
            left_right_keys: 1,
            up_down_keys: 2,
            key_data: 3,
            position: zero,
            quaternion: [0.0; 4],
            health: 4,
            armour: 5,
            weapon_and_special_key: 6,
            special_action: 7,
            move_speed: zero,
            surfing_offsets: zero,
            surfing_vehicle_id: 8,
            animation_id: 9,
            animation_flags: 10,
        },
        &player_expected,
    );
    assert_vector(
        SEND_VEHICLE_SYNC,
        VehicleSync {
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
        },
        &[0; 63],
    );
    assert_vector(
        SEND_PASSENGER_SYNC,
        PassengerSync {
            vehicle_id: 0,
            seat_driveby_cuffed: 0,
            weapon_and_special_key: 0,
            health: 0,
            armour: 0,
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
        },
        &[0; 24],
    );
    assert_vector(
        SEND_AIM_SYNC,
        aim,
        &[
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
        ],
    );
    assert_vector(
        SEND_UNOCCUPIED_SYNC,
        UnoccupiedSync {
            vehicle_id: 0,
            seat_id: 0,
            roll: zero,
            direction: zero,
            position: zero,
            move_speed: zero,
            turn_speed: zero,
            vehicle_health: 0.0,
        },
        &[0; 67],
    );
    assert_vector(
        SEND_TRAILER_SYNC,
        TrailerSync {
            trailer_id: 0,
            position: zero,
            quaternion: [0.0; 4],
            move_speed: zero,
            turn_speed: zero,
        },
        &[0; 54],
    );
    assert_vector(
        SEND_BULLET_SYNC,
        bullet,
        &[
            1, 0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0x40, 0,
            0, 0xA0, 0x40, 0, 0, 0xC0, 0x40, 0, 0, 0xE0, 0x40, 0, 0, 0, 0x41, 0, 0, 0x10, 0x41, 24,
        ],
    );
    assert_vector(
        SEND_SPECTATOR_SYNC,
        SpectatorSync {
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
        },
        &[0; 18],
    );

    assert_vector(
        AUTHENTICATION_REQUEST,
        b"auth".to_vec(),
        &[4, b'a', b'u', b't', b'h'],
    );
    assert_vector(
        CONNECTION_ACCEPTED,
        ConnectionAccepted {
            ip: -1,
            port: 0x1234,
            player_id: 0x5678,
            challenge: 42,
        },
        &[0xFF, 0xFF, 0xFF, 0xFF, 0x34, 0x12, 0x78, 0x56, 42, 0, 0, 0],
    );
    assert_vector(CONNECTION_LOST, (), &[]);
    assert_vector(CONNECTION_BANNED, (), &[]);
    assert_vector(CONNECTION_ATTEMPT_FAILED, (), &[]);
    assert_vector(CONNECTION_NO_FREE_SLOT, (), &[]);
    assert_vector(CONNECTION_PASSWORD_INVALID, (), &[]);
    assert_vector(CONNECTION_CLOSED, (), &[]);
    assert_vector(
        AIM_SYNC,
        RemoteSync {
            player_id: 0x1234,
            data: aim,
        },
        &[
            0x34,
            0x12,
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
        ],
    );
    assert_vector(
        BULLET_SYNC,
        RemoteSync {
            player_id: 0,
            data: BulletSync {
                target_type: 0,
                target_id: 0,
                origin: zero,
                target: zero,
                center: zero,
                weapon_id: 0,
            },
        },
        &[0; 42],
    );
    assert_vector(
        UNOCCUPIED_SYNC,
        RemoteSync {
            player_id: 0,
            data: UnoccupiedSync {
                vehicle_id: 0,
                seat_id: 0,
                roll: zero,
                direction: zero,
                position: zero,
                move_speed: zero,
                turn_speed: zero,
                vehicle_health: 0.0,
            },
        },
        &[0; 69],
    );
    assert_vector(
        TRAILER_SYNC,
        RemoteSync {
            player_id: 0,
            data: TrailerSync {
                trailer_id: 0,
                position: zero,
                quaternion: [0.0; 4],
                move_speed: zero,
                turn_speed: zero,
            },
        },
        &[0; 56],
    );
    assert_vector(
        PASSENGER_SYNC,
        RemoteSync {
            player_id: 0,
            data: PassengerSync {
                vehicle_id: 0,
                seat_driveby_cuffed: 0,
                weapon_and_special_key: 0,
                health: 0,
                armour: 0,
                left_right_keys: 0,
                up_down_keys: 0,
                key_data: 0,
                position: zero,
            },
        },
        &[0; 26],
    );
}

#[test]
fn common_packets_preserve_length_limits_and_exact_byte_trailing_policy() {
    assert_eq!(
        SendWeaponsUpdate::encode_bits(&WeaponsUpdate {
            player_target: 0,
            actor_target: 0,
            weapons: vec![
                WeaponSlot {
                    slot: 0,
                    weapon: 0,
                    ammo: 0,
                };
                14
            ],
        }),
        Err(EncodeError::LengthExceedsLimit {
            length: 14,
            limit: 13,
        })
    );
    assert_eq!(
        SendRconCommand::encode_bits(&vec![b'x'; 4097]),
        Err(EncodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );

    let too_many_weapons = EncodedBits::from_bits([0; 60], 480).expect("the storage is valid");
    assert_eq!(
        SendWeaponsUpdate::decode_bits(&too_many_weapons),
        Err(DecodeError::LengthExceedsLimit {
            length: 14,
            limit: 13,
        })
    );
    let oversized_rcon = EncodedBits::from_bits([1, 16, 0, 0], 32).expect("the storage is valid");
    assert_eq!(
        SendRconCommand::decode_bits(&oversized_rcon),
        Err(DecodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );

    let non_byte_aligned = EncodedBits::from_bits([0], 7).expect("the storage is valid");
    assert_eq!(
        SendStatsUpdate::decode_bits(&non_byte_aligned),
        Err(DecodeError::OutOfBounds {
            requested_bits: 32,
            available_bits: 7,
        })
    );
    let trailing = EncodedBits::from_bits([0; 9], 72).expect("the storage is valid");
    assert_eq!(
        SendStatsUpdate::decode_bits(&trailing),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 8,
            allowed_bits: 0,
        })
    );
}
