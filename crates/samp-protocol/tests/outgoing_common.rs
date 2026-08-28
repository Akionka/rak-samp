use samp_protocol::rpc::outgoing::common::{
    ActorDamage, CameraTargetUpdate, ClickPlayer, ClientCheckResponse, ClientJoin, Damage,
    DeathNotification, DialogResponse, EditAttachedObject, EditObject, EnterEditObject,
    EnterVehicle, MoneyIncrease, NpcJoin, SEND_CAMERA_TARGET_UPDATE, SEND_CLICK_PLAYER,
    SEND_CLICK_TEXT_DRAW, SEND_CLIENT_CHECK_RESPONSE, SEND_CLIENT_JOIN, SEND_DAMAGE,
    SEND_DEATH_NOTIFICATION, SEND_DIALOG_RESPONSE, SEND_EDIT_ATTACHED_OBJECT, SEND_EDIT_OBJECT,
    SEND_ENTER_EDIT_OBJECT, SEND_ENTER_VEHICLE, SEND_EXIT_VEHICLE, SEND_GIVE_ACTOR_DAMAGE,
    SEND_INTERIOR_CHANGE, SEND_MAP_MARKER, SEND_MENU_SELECT, SEND_MONEY_INCREASE, SEND_NPC_JOIN,
    SEND_PICKED_UP_PICKUP, SEND_PICKED_UP_WEAPON, SEND_QUIT_MENU, SEND_REQUEST_CLASS,
    SEND_REQUEST_SPAWN, SEND_SERVER_STATISTICS_REQUEST, SEND_SPAWN, SEND_UPDATE_SCORES_AND_PINGS,
    SEND_VEHICLE_DAMAGED, SEND_VEHICLE_DESTROYED, SEND_VEHICLE_TUNING, SendDialogResponse,
    SendInteriorChange, VehicleDamage, VehicleTuning,
};
use samp_protocol::types::Vector3;
use samp_protocol::{DecodeError, EncodeError, EncodedBits, WireDescriptor};

fn id<D: WireDescriptor>(_: D) -> u8 {
    D::ID
}

fn assert_vector<D>(descriptor: D, value: D::Value, expected: &[u8])
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let _ = descriptor;
    let bits = D::encode_bits(&value).expect("the common outgoing RPC value must encode");

    assert_eq!(bits.as_bytes(), expected);
    assert_eq!(bits.len_bits(), expected.len() * 8);
    assert_eq!(D::decode_bits(&bits), Ok(value));
}

fn assert_exact_vector<D>(descriptor: D, value: D::Value, expected: &[u8], expected_bits: usize)
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let _ = descriptor;
    let bits = D::encode_bits(&value).expect("the common exact-bit RPC value must encode");

    assert_eq!(bits.as_bytes(), expected);
    assert_eq!(bits.len_bits(), expected_bits);
    assert_eq!(D::decode_bits(&bits), Ok(value));
}

fn assert_rejects_trailing_bit<D>(descriptor: D, value: &D::Value, expected_bits: usize)
where
    D: WireDescriptor,
    D::Value: core::fmt::Debug + PartialEq,
{
    let _ = descriptor;
    let bits = D::encode_bits(value).expect("the common exact-bit RPC value must encode");
    let trailing = EncodedBits::from_bits(bits.as_bytes(), expected_bits + 1)
        .expect("one trailing bit still uses minimal storage");

    assert_eq!(
        D::decode_bits(&trailing),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 1,
            allowed_bits: 0,
        })
    );
}

#[test]
fn common_outgoing_rpc_inventory_has_30_unique_entries() {
    let ids = [
        id(SEND_DEATH_NOTIFICATION),
        id(SEND_MAP_MARKER),
        id(SEND_INTERIOR_CHANGE),
        id(SEND_UPDATE_SCORES_AND_PINGS),
        id(SEND_MONEY_INCREASE),
        id(SEND_PICKED_UP_WEAPON),
        id(SEND_PICKED_UP_PICKUP),
        id(SEND_CAMERA_TARGET_UPDATE),
        id(SEND_CLIENT_JOIN),
        id(SEND_NPC_JOIN),
        id(SEND_VEHICLE_DAMAGED),
        id(SEND_ENTER_EDIT_OBJECT),
        id(SEND_EDIT_ATTACHED_OBJECT),
        id(SEND_SPAWN),
        id(SEND_REQUEST_CLASS),
        id(SEND_REQUEST_SPAWN),
        id(SEND_SERVER_STATISTICS_REQUEST),
        id(SEND_CLIENT_CHECK_RESPONSE),
        id(SEND_DIALOG_RESPONSE),
        id(SEND_CLICK_PLAYER),
        id(SEND_CLICK_TEXT_DRAW),
        id(SEND_MENU_SELECT),
        id(SEND_QUIT_MENU),
        id(SEND_ENTER_VEHICLE),
        id(SEND_EXIT_VEHICLE),
        id(SEND_VEHICLE_DESTROYED),
        id(SEND_VEHICLE_TUNING),
        id(SEND_DAMAGE),
        id(SEND_GIVE_ACTOR_DAMAGE),
        id(SEND_EDIT_OBJECT),
    ];

    assert_eq!(
        ids,
        [
            53, 119, 118, 155, 31, 97, 131, 168, 25, 54, 106, 27, 116, 52, 128, 129, 102, 103, 62,
            23, 83, 132, 140, 26, 154, 136, 96, 115, 177, 117,
        ]
    );

    let mut unique = ids;
    unique.sort_unstable();
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn common_exact_bit_outgoing_rpcs_preserve_vectors_and_reject_trailing_bits() {
    let damage = Damage {
        player_id: 0x1234,
        damage: 1.0,
        weapon: 24,
        body_part: 9,
        take: true,
    };
    assert_exact_vector(
        SEND_DAMAGE,
        damage,
        &[
            0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00, 0x00,
            0x00,
        ],
        113,
    );

    let actor_damage = ActorDamage {
        unused: false,
        actor_id: 0x5678,
        damage: 2.5,
        weapon: -1,
        body_part: 3,
    };
    assert_exact_vector(
        SEND_GIVE_ACTOR_DAMAGE,
        actor_damage,
        &[
            0x3C, 0x2B, 0x00, 0x00, 0x10, 0x20, 0x7F, 0xFF, 0xFF, 0xFF, 0x81, 0x80, 0x00, 0x00,
            0x00,
        ],
        113,
    );

    let edit_object = EditObject {
        player_object: true,
        object_id: 0x1234,
        response: -2,
        position: Vector3 {
            x: 1.0,
            y: -2.0,
            z: 0.5,
        },
        rotation: Vector3 {
            x: 90.0,
            y: -45.0,
            z: 180.0,
        },
    };
    assert_exact_vector(
        SEND_EDIT_OBJECT,
        edit_object,
        &[
            0x9A, 0x09, 0x7F, 0x7F, 0xFF, 0xFF, 0x80, 0x00, 0x40, 0x1F, 0x80, 0x00, 0x00, 0x60,
            0x00, 0x00, 0x00, 0x1F, 0x80, 0x00, 0x5A, 0x21, 0x00, 0x00, 0x1A, 0x61, 0x00, 0x00,
            0x1A, 0x21, 0x80,
        ],
        241,
    );

    assert_rejects_trailing_bit(SEND_DAMAGE, &damage, 113);
    assert_rejects_trailing_bit(SEND_GIVE_ACTOR_DAMAGE, &actor_damage, 113);
    assert_rejects_trailing_bit(SEND_EDIT_OBJECT, &edit_object, 241);
}

#[test]
fn common_outgoing_rpcs_preserve_exact_vectors() {
    let vector = Vector3 {
        x: 1.0,
        y: -2.0,
        z: 0.5,
    };
    assert_vector(
        SEND_DEATH_NOTIFICATION,
        DeathNotification {
            reason: 9,
            killer_id: 0x1234,
        },
        &[9, 0x34, 0x12],
    );
    assert_vector(
        SEND_MAP_MARKER,
        vector,
        &[0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0, 0x3F],
    );
    assert_vector(SEND_INTERIOR_CHANGE, 7, &[7]);
    assert_vector(SEND_UPDATE_SCORES_AND_PINGS, (), &[]);
    assert_vector(
        SEND_MONEY_INCREASE,
        MoneyIncrease {
            amount: -2,
            increase_type: 1500,
        },
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xDC, 5, 0, 0],
    );
    assert_vector(SEND_PICKED_UP_WEAPON, 0x1234, &[0x34, 0x12]);
    assert_vector(SEND_PICKED_UP_PICKUP, -2, &[0xFE, 0xFF, 0xFF, 0xFF]);
    assert_vector(
        SEND_CAMERA_TARGET_UPDATE,
        CameraTargetUpdate {
            object_id: 1,
            vehicle_id: 2,
            player_id: 3,
            actor_id: 4,
        },
        &[1, 0, 2, 0, 3, 0, 4, 0],
    );
    assert_vector(
        SEND_CLIENT_JOIN,
        ClientJoin {
            version: -1,
            mod_id: 3,
            nickname: b"me".to_vec(),
            challenge_response: -2,
            join_auth_key: b"k".to_vec(),
            client_version: b"v1".to_vec(),
            challenge_response2: 7,
        },
        &[
            0xFF, 0xFF, 0xFF, 0xFF, 3, 2, b'm', b'e', 0xFE, 0xFF, 0xFF, 0xFF, 1, b'k', 2, b'v',
            b'1', 7, 0, 0, 0,
        ],
    );
    assert_vector(
        SEND_NPC_JOIN,
        NpcJoin {
            version: -1,
            mod_id: 3,
            nickname: b"me".to_vec(),
            challenge_response: -2,
        },
        &[
            0xFF, 0xFF, 0xFF, 0xFF, 3, 2, b'm', b'e', 0xFE, 0xFF, 0xFF, 0xFF,
        ],
    );
    assert_vector(
        SEND_VEHICLE_DAMAGED,
        VehicleDamage {
            vehicle_id: 0x1234,
            panel_damage: -1,
            door_damage: 2,
            lights: 3,
            tires: 4,
        },
        &[0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF, 2, 0, 0, 0, 3, 4],
    );
    assert_vector(
        SEND_ENTER_EDIT_OBJECT,
        EnterEditObject {
            object_type: -1,
            object_id: 0x1234,
            model_id: 411,
            position: vector,
        },
        &[
            0xFF, 0xFF, 0xFF, 0xFF, 0x34, 0x12, 0x9B, 1, 0, 0, 0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0,
            0, 0, 0x3F,
        ],
    );
    assert_vector(
        SEND_EDIT_ATTACHED_OBJECT,
        EditAttachedObject {
            response: 0,
            index: 0,
            model_id: 0,
            bone: 0,
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            scale: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            color1: 0,
            color2: 0,
        },
        &[0; 60],
    );
    assert_vector(SEND_SPAWN, (), &[]);
    assert_vector(SEND_REQUEST_CLASS, -2, &[0xFE, 0xFF, 0xFF, 0xFF]);
    assert_vector(SEND_REQUEST_SPAWN, (), &[]);
    assert_vector(SEND_SERVER_STATISTICS_REQUEST, (), &[]);
    assert_vector(
        SEND_CLIENT_CHECK_RESPONSE,
        ClientCheckResponse {
            request_type: 1,
            result1: -2,
            result2: 3,
        },
        &[1, 0xFE, 0xFF, 0xFF, 0xFF, 3],
    );
    assert_vector(
        SEND_DIALOG_RESPONSE,
        DialogResponse {
            dialog_id: 0x1234,
            button: 1,
            list_item: 0x5678,
            input: b"ok".to_vec(),
        },
        &[0x34, 0x12, 1, 0x78, 0x56, 2, b'o', b'k'],
    );
    assert_vector(
        SEND_CLICK_PLAYER,
        ClickPlayer {
            player_id: 0x1234,
            source: 2,
        },
        &[0x34, 0x12, 2],
    );
    assert_vector(SEND_CLICK_TEXT_DRAW, 0x1234, &[0x34, 0x12]);
    assert_vector(SEND_MENU_SELECT, 7, &[7]);
    assert_vector(SEND_QUIT_MENU, (), &[]);
    assert_vector(
        SEND_ENTER_VEHICLE,
        EnterVehicle {
            vehicle_id: 0x1234,
            passenger: true,
        },
        &[0x34, 0x12, 1],
    );
    assert_vector(SEND_EXIT_VEHICLE, 0x1234, &[0x34, 0x12]);
    assert_vector(SEND_VEHICLE_DESTROYED, 0x1234, &[0x34, 0x12]);
    assert_vector(
        SEND_VEHICLE_TUNING,
        VehicleTuning {
            vehicle_id: 1,
            param1: -2,
            param2: 3,
            event: 4,
        },
        &[1, 0, 0, 0, 0xFE, 0xFF, 0xFF, 0xFF, 3, 0, 0, 0, 4, 0, 0, 0],
    );
}

#[test]
fn common_outgoing_rpcs_reject_non_byte_aligned_or_trailing_payloads() {
    let non_byte_aligned = EncodedBits::from_bits([7], 7).expect("the test storage is valid");
    assert_eq!(
        SendInteriorChange::decode_bits(&non_byte_aligned),
        Err(DecodeError::OutOfBounds {
            requested_bits: 8,
            available_bits: 7,
        })
    );

    let trailing = EncodedBits::from_bits([7, 0], 16).expect("the test storage is valid");
    assert_eq!(
        SendInteriorChange::decode_bits(&trailing),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 8,
            allowed_bits: 0,
        })
    );
}

#[test]
fn common_outgoing_string_fields_keep_their_byte_limit() {
    let oversized = vec![b'x'; 256];
    assert_eq!(
        SendDialogResponse::encode_bits(&DialogResponse {
            dialog_id: 0,
            button: 0,
            list_item: 0,
            input: oversized,
        }),
        Err(EncodeError::LengthExceedsLimit {
            length: 256,
            limit: 255,
        })
    );
}
