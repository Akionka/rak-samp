use samp_protocol::rpc::incoming::{
    ATTACH_OBJECT_TO_PLAYER, Actor, ActorAngle, ActorHealth, ActorPosition, AttachObjectToPlayer,
    AudioStream, CANCEL_EDIT, CHAT_BUBBLE, CHAT_MESSAGE, CLEAR_ACTOR_ANIMATION, CLIENT_CHECK,
    CREATE_ACTOR, CREATE_EXPLOSION, ChatBubble, ChatMessage, Checkpoint, ClientCheck,
    DESTROY_ACTOR, DESTROY_OBJECT, DESTROY_PICKUP, DESTROY_WEAPON_PICKUP, DISABLE_CHECKPOINT,
    DISABLE_RACE_CHECKPOINT, DISPLAY_GAME_TEXT, EDIT_ATTACHED_OBJECT, ENTER_SELECT_OBJECT,
    Explosion, FORCE_CLASS_SELECTION, GAMEMODE_RESTART, GIVE_PLAYER_MONEY, GIVE_PLAYER_WEAPON,
    GameText, HIDE_MENU, LINK_VEHICLE_TO_INTERIOR, MapIcon, ObjectPosition, ObjectRotation,
    PLAY_AUDIO_STREAM, PLAY_SOUND, PLAYER_DEATH_NOTIFICATION, PLAYER_JOIN, PLAYER_QUIT,
    PLAYER_STREAM_OUT, PUT_PLAYER_IN_VEHICLE, PlaySound, PlayerColor, PlayerDeathNotification,
    PlayerJoin, PlayerName, PlayerNameTag, PlayerQuit, PlayerSkill, PlayerSkin, PlayerTeam,
    PlayerTime, PlayerWeapon, PutPlayerInVehicle, REMOVE_3D_TEXT_LABEL, REMOVE_BUILDING,
    REMOVE_PLAYER_FROM_VEHICLE, REMOVE_VEHICLE_COMPONENT, REQUEST_SPAWN_RESPONSE,
    RESET_PLAYER_MONEY, RESET_PLAYER_WEAPONS, RaceCheckpoint, RemoveBuilding, SERVER_MESSAGE,
    SERVER_STATISTICS_RESPONSE, SET_ACTOR_FACING_ANGLE, SET_ACTOR_HEALTH, SET_ACTOR_POSITION,
    SET_CAMERA_BEHIND, SET_CHECKPOINT, SET_INTERIOR, SET_MAP_ICON, SET_OBJECT_POSITION,
    SET_OBJECT_ROTATION, SET_PLAYER_ARMED_WEAPON, SET_PLAYER_ARMOUR, SET_PLAYER_COLOR,
    SET_PLAYER_DRUNK, SET_PLAYER_DRUNK_HANDLING, SET_PLAYER_DRUNK_VISUALS, SET_PLAYER_FACING_ANGLE,
    SET_PLAYER_HEALTH, SET_PLAYER_NAME, SET_PLAYER_OBJECT_NO_CAMERA_COL, SET_PLAYER_POS,
    SET_PLAYER_POS_FIND_Z, SET_PLAYER_SKILL_LEVEL, SET_PLAYER_SKIN, SET_PLAYER_TEAM,
    SET_PLAYER_TIME, SET_PLAYER_WANTED_LEVEL, SET_RACE_CHECKPOINT, SET_SHOP_NAME, SET_TOGGLE_CLOCK,
    SET_VEHICLE_ANGLE, SET_VEHICLE_HEALTH, SET_VEHICLE_PARAMS_EX, SET_VEHICLE_POSITION,
    SET_VEHICLE_TIRES, SET_WEATHER, SET_WORLD_BOUNDS, SET_WORLD_TIME, SHOW_MENU,
    SHOW_PLAYER_NAME_TAG, STOP_AUDIO_STREAM, ServerMessage, TOGGLE_PLAYER_CONTROLLABLE,
    TOGGLE_WIDESCREEN, UPDATE_GLOBAL_TIMER, VEHICLE_DAMAGE_STATUS_UPDATE, VEHICLE_STREAM_OUT,
    VEHICLE_TUNING_NOTIFICATION, Vector3, VehicleAngle, VehicleComponent, VehicleDamageStatus,
    VehicleHealth, VehicleInterior, VehicleParamsEx, VehiclePosition, VehicleTuningNotification,
    WorldBounds,
};
use samp_protocol::{DecodeError, EncodeError, EncodedBits, WireDescriptor};

fn id<D: WireDescriptor>(_: D) -> u8 {
    D::ID
}

fn encode<D: WireDescriptor>(
    _: D,
    value: &D::Value,
) -> Result<EncodedBits, EncodeError<samp_protocol::BitStreamError>> {
    D::encode_bits(value)
}

fn decode<D: WireDescriptor>(
    _: D,
    bits: &EncodedBits,
) -> Result<D::Value, DecodeError<samp_protocol::BitStreamError>> {
    D::decode_bits(bits)
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
fn fixed_incoming_rpc_inventory_has_29_unique_entries() {
    let ids = [
        id(SERVER_MESSAGE),
        id(DISPLAY_GAME_TEXT),
        id(SET_PLAYER_POS),
        id(SET_PLAYER_POS_FIND_Z),
        id(SET_PLAYER_HEALTH),
        id(SET_PLAYER_ARMOUR),
        id(SET_PLAYER_FACING_ANGLE),
        id(TOGGLE_PLAYER_CONTROLLABLE),
        id(PLAY_SOUND),
        id(SET_CHECKPOINT),
        id(CHAT_MESSAGE),
        id(CHAT_BUBBLE),
        id(PLAYER_JOIN),
        id(PLAYER_QUIT),
        id(SET_PLAYER_NAME),
        id(SET_PLAYER_TIME),
        id(SET_WORLD_BOUNDS),
        id(GIVE_PLAYER_MONEY),
        id(GIVE_PLAYER_WEAPON),
        id(SET_WORLD_TIME),
        id(SET_WEATHER),
        id(SET_PLAYER_SKIN),
        id(SET_INTERIOR),
        id(SET_PLAYER_ARMED_WEAPON),
        id(SET_PLAYER_WANTED_LEVEL),
        id(SET_PLAYER_TEAM),
        id(PUT_PLAYER_IN_VEHICLE),
        id(PLAYER_STREAM_OUT),
        id(VEHICLE_STREAM_OUT),
    ];

    assert_eq!(
        ids,
        [
            93, 73, 12, 13, 14, 66, 19, 15, 16, 107, 101, 59, 137, 138, 11, 29, 17, 18, 22, 94,
            152, 153, 156, 67, 133, 69, 70, 163, 165,
        ]
    );

    let mut unique = ids;
    unique.sort_unstable();
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn second_fixed_incoming_rpc_inventory_has_30_unique_entries() {
    let ids = [
        id(SET_VEHICLE_POSITION),
        id(SET_VEHICLE_ANGLE),
        id(SET_VEHICLE_HEALTH),
        id(RESET_PLAYER_MONEY),
        id(RESET_PLAYER_WEAPONS),
        id(CANCEL_EDIT),
        id(SET_TOGGLE_CLOCK),
        id(SET_PLAYER_DRUNK),
        id(SET_RACE_CHECKPOINT),
        id(PLAY_AUDIO_STREAM),
        id(SET_OBJECT_POSITION),
        id(SET_OBJECT_ROTATION),
        id(DESTROY_OBJECT),
        id(PLAYER_DEATH_NOTIFICATION),
        id(SET_MAP_ICON),
        id(REMOVE_VEHICLE_COMPONENT),
        id(REMOVE_3D_TEXT_LABEL),
        id(UPDATE_GLOBAL_TIMER),
        id(DESTROY_PICKUP),
        id(LINK_VEHICLE_TO_INTERIOR),
        id(SET_PLAYER_COLOR),
        id(REQUEST_SPAWN_RESPONSE),
        id(SET_SHOP_NAME),
        id(SET_PLAYER_SKILL_LEVEL),
        id(REMOVE_BUILDING),
        id(ATTACH_OBJECT_TO_PLAYER),
        id(SHOW_MENU),
        id(HIDE_MENU),
        id(CREATE_EXPLOSION),
        id(SHOW_PLAYER_NAME_TAG),
    ];

    assert_eq!(
        ids,
        [
            159, 160, 147, 20, 21, 28, 30, 35, 38, 41, 45, 46, 47, 55, 56, 57, 58, 60, 63, 65, 72,
            129, 33, 34, 43, 75, 77, 78, 79, 80,
        ]
    );

    let mut unique = ids;
    unique.sort_unstable();
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn fixed_incoming_rpcs_preserve_exact_vectors() {
    assert_vector(
        SERVER_MESSAGE,
        &ServerMessage {
            color: 0xAABB_CCDD,
            text: b"Hi".to_vec(),
        },
        &[0xDD, 0xCC, 0xBB, 0xAA, 2, 0, 0, 0, b'H', b'i'],
    );
    assert_vector(
        DISPLAY_GAME_TEXT,
        &GameText {
            style: -2,
            time_ms: 1500,
            text: b"ok".to_vec(),
        },
        &[
            0xFE, 0xFF, 0xFF, 0xFF, 0xDC, 5, 0, 0, 2, 0, 0, 0, b'o', b'k',
        ],
    );
    assert_vector(
        SET_PLAYER_POS,
        &Vector3 {
            x: 1.0,
            y: -2.0,
            z: 0.5,
        },
        &[0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0, 0x3F],
    );
    assert_vector(
        SET_PLAYER_POS_FIND_Z,
        &Vector3 {
            x: -1.0,
            y: 2.0,
            z: -0.5,
        },
        &[0, 0, 0x80, 0xBF, 0, 0, 0, 0x40, 0, 0, 0, 0xBF],
    );
    assert_vector(SET_PLAYER_HEALTH, &1.5, &[0, 0, 0xC0, 0x3F]);
    assert_vector(SET_PLAYER_ARMOUR, &2.5, &[0, 0, 0x20, 0x40]);
    assert_vector(SET_PLAYER_FACING_ANGLE, &180.0, &[0, 0, 0x34, 0x43]);
    assert_vector(TOGGLE_PLAYER_CONTROLLABLE, &true, &[1]);
    assert_vector(
        PLAY_SOUND,
        &PlaySound {
            sound_id: -2,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
        &[
            0xFE, 0xFF, 0xFF, 0xFF, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40,
        ],
    );
    assert_vector(
        SET_CHECKPOINT,
        &Checkpoint {
            position: Vector3 {
                x: -1.0,
                y: 0.0,
                z: 1.0,
            },
            radius: 5.0,
        },
        &[
            0, 0, 0x80, 0xBF, 0, 0, 0, 0, 0, 0, 0x80, 0x3F, 0, 0, 0xA0, 0x40,
        ],
    );
    assert_vector(
        CHAT_MESSAGE,
        &ChatMessage {
            player_id: 42,
            text: b"yo".to_vec(),
        },
        &[42, 0, 2, b'y', b'o'],
    );
    assert_vector(
        CHAT_BUBBLE,
        &ChatBubble {
            player_id: 42,
            color: 0xFF80_4020,
            draw_distance: 25.0,
            duration_ms: -50,
            text: b"hi".to_vec(),
        },
        &[
            42, 0, 0x20, 0x40, 0x80, 0xFF, 0, 0, 0xC8, 0x41, 0xCE, 0xFF, 0xFF, 0xFF, 2, b'h', b'i',
        ],
    );
    assert_vector(
        PLAYER_JOIN,
        &PlayerJoin {
            player_id: 15,
            color: 0xFF80_4020,
            is_npc: true,
            nickname: b"NPC".to_vec(),
        },
        &[15, 0, 0x20, 0x40, 0x80, 0xFF, 1, 3, b'N', b'P', b'C'],
    );
    assert_vector(
        PLAYER_QUIT,
        &PlayerQuit {
            player_id: 15,
            reason: 2,
        },
        &[15, 0, 2],
    );
    assert_vector(
        SET_PLAYER_NAME,
        &PlayerName {
            player_id: 15,
            name: b"New".to_vec(),
            success: false,
        },
        &[15, 0, 3, b'N', b'e', b'w', 0],
    );
    assert_vector(
        SET_PLAYER_TIME,
        &PlayerTime {
            hour: 23,
            minute: 59,
        },
        &[23, 59],
    );
    assert_vector(
        SET_WORLD_BOUNDS,
        &WorldBounds {
            max_x: 1.0,
            min_x: -2.0,
            max_y: 3.0,
            min_y: -4.0,
        },
        &[
            0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0xC0,
        ],
    );
    assert_vector(GIVE_PLAYER_MONEY, &-500, &[0x0C, 0xFE, 0xFF, 0xFF]);
    assert_vector(
        GIVE_PLAYER_WEAPON,
        &PlayerWeapon {
            weapon_id: -1,
            ammo: 30,
        },
        &[0xFF, 0xFF, 0xFF, 0xFF, 30, 0, 0, 0],
    );
    assert_vector(SET_WORLD_TIME, &12, &[12]);
    assert_vector(SET_WEATHER, &7, &[7]);
    assert_vector(
        SET_PLAYER_SKIN,
        &PlayerSkin {
            player_id: -1,
            skin_id: 299,
        },
        &[0xFF, 0xFF, 0xFF, 0xFF, 0x2B, 1, 0, 0],
    );
    assert_vector(SET_INTERIOR, &3, &[3]);
    assert_vector(SET_PLAYER_ARMED_WEAPON, &-5, &[0xFB, 0xFF, 0xFF, 0xFF]);
    assert_vector(SET_PLAYER_WANTED_LEVEL, &6, &[6]);
    assert_vector(
        SET_PLAYER_TEAM,
        &PlayerTeam {
            player_id: 42,
            team_id: 7,
        },
        &[42, 0, 7],
    );
    assert_vector(
        PUT_PLAYER_IN_VEHICLE,
        &PutPlayerInVehicle {
            vehicle_id: 1234,
            seat_id: 3,
        },
        &[0xD2, 4, 3],
    );
    assert_vector(PLAYER_STREAM_OUT, &0xBEEF, &[0xEF, 0xBE]);
    assert_vector(VEHICLE_STREAM_OUT, &0x1234, &[0x34, 0x12]);
}

#[test]
fn second_fixed_incoming_rpcs_preserve_exact_vectors() {
    assert_vector(
        SET_VEHICLE_POSITION,
        &VehiclePosition {
            vehicle_id: 0x1234,
            position: Vector3 {
                x: 1.0,
                y: -2.0,
                z: 0.5,
            },
        },
        &[0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0, 0x3F],
    );
    assert_vector(
        SET_VEHICLE_ANGLE,
        &VehicleAngle {
            vehicle_id: 0xABCD,
            angle: 180.0,
        },
        &[0xCD, 0xAB, 0, 0, 0x34, 0x43],
    );
    assert_vector(
        SET_VEHICLE_HEALTH,
        &VehicleHealth {
            vehicle_id: 0x1234,
            health: 1.5,
        },
        &[0x34, 0x12, 0, 0, 0xC0, 0x3F],
    );
    assert_vector(RESET_PLAYER_MONEY, &(), &[]);
    assert_vector(RESET_PLAYER_WEAPONS, &(), &[]);
    assert_vector(CANCEL_EDIT, &(), &[]);
    assert_vector(SET_TOGGLE_CLOCK, &true, &[1]);
    assert_vector(SET_PLAYER_DRUNK, &-2, &[0xFE, 0xFF, 0xFF, 0xFF]);
    assert_vector(
        SET_RACE_CHECKPOINT,
        &RaceCheckpoint {
            checkpoint_type: 2,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            next_position: Vector3 {
                x: -1.0,
                y: -2.0,
                z: -3.0,
            },
            size: 4.5,
        },
        &[
            2, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0xBF, 0, 0, 0, 0xC0,
            0, 0, 0x40, 0xC0, 0, 0, 0x90, 0x40,
        ],
    );
    assert_vector(
        PLAY_AUDIO_STREAM,
        &AudioStream {
            url: b"url".to_vec(),
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            radius: 50.0,
            use_position: true,
        },
        &[
            3, b'u', b'r', b'l', 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x48,
            0x42, 1,
        ],
    );
    assert_vector(
        SET_OBJECT_POSITION,
        &ObjectPosition {
            object_id: 0x1234,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
        &[
            0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40,
        ],
    );
    assert_vector(
        SET_OBJECT_ROTATION,
        &ObjectRotation {
            object_id: 0x1234,
            rotation: Vector3 {
                x: -1.0,
                y: -2.0,
                z: -3.0,
            },
        },
        &[
            0x34, 0x12, 0, 0, 0x80, 0xBF, 0, 0, 0, 0xC0, 0, 0, 0x40, 0xC0,
        ],
    );
    assert_vector(DESTROY_OBJECT, &0xBEEF, &[0xEF, 0xBE]);
    assert_vector(
        PLAYER_DEATH_NOTIFICATION,
        &PlayerDeathNotification {
            killer_id: 2,
            killed_id: 7,
            reason: 53,
        },
        &[2, 0, 7, 0, 53],
    );
    assert_vector(
        SET_MAP_ICON,
        &MapIcon {
            icon_id: 5,
            position: Vector3 {
                x: 1.0,
                y: -2.0,
                z: 0.5,
            },
            icon_type: 3,
            color: -1,
            style: 1,
        },
        &[
            5, 0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0, 0x3F, 3, 0xFF, 0xFF, 0xFF, 0xFF, 1,
        ],
    );
    assert_vector(
        REMOVE_VEHICLE_COMPONENT,
        &VehicleComponent {
            vehicle_id: 0x1234,
            component_id: 0x5678,
        },
        &[0x34, 0x12, 0x78, 0x56],
    );
    assert_vector(REMOVE_3D_TEXT_LABEL, &0x1234, &[0x34, 0x12]);
    assert_vector(UPDATE_GLOBAL_TIMER, &-1, &[0xFF, 0xFF, 0xFF, 0xFF]);
    assert_vector(DESTROY_PICKUP, &-3, &[0xFD, 0xFF, 0xFF, 0xFF]);
    assert_vector(
        LINK_VEHICLE_TO_INTERIOR,
        &VehicleInterior {
            vehicle_id: 0x1234,
            interior_id: 7,
        },
        &[0x34, 0x12, 7],
    );
    assert_vector(
        SET_PLAYER_COLOR,
        &PlayerColor {
            player_id: 0x1234,
            color: 0x1122_3344,
        },
        &[0x34, 0x12, 0x44, 0x33, 0x22, 0x11],
    );
    assert_vector(REQUEST_SPAWN_RESPONSE, &false, &[0]);
    assert_vector(SET_SHOP_NAME, &[0xA5; 32], &[0xA5; 32]);
    assert_vector(
        SET_PLAYER_SKILL_LEVEL,
        &PlayerSkill {
            player_id: 0x1234,
            skill: -2,
            level: 0x5678,
        },
        &[0x34, 0x12, 0xFE, 0xFF, 0xFF, 0xFF, 0x78, 0x56],
    );
    assert_vector(
        REMOVE_BUILDING,
        &RemoveBuilding {
            model_id: -100,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            radius: 4.5,
        },
        &[
            0x9C, 0xFF, 0xFF, 0xFF, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x90,
            0x40,
        ],
    );
    assert_vector(
        ATTACH_OBJECT_TO_PLAYER,
        &AttachObjectToPlayer {
            object_id: 1,
            player_id: 2,
            offsets: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: Vector3 {
                x: -1.0,
                y: -2.0,
                z: -3.0,
            },
        },
        &[
            1, 0, 2, 0, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0xBF, 0, 0,
            0, 0xC0, 0, 0, 0x40, 0xC0,
        ],
    );
    assert_vector(SHOW_MENU, &5, &[5]);
    assert_vector(HIDE_MENU, &6, &[6]);
    assert_vector(
        CREATE_EXPLOSION,
        &Explosion {
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            style: -2,
            radius: 4.5,
        },
        &[
            0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0xFE, 0xFF, 0xFF, 0xFF, 0, 0, 0x90,
            0x40,
        ],
    );
    assert_vector(
        SHOW_PLAYER_NAME_TAG,
        &PlayerNameTag {
            player_id: 0x1234,
            show: true,
        },
        &[0x34, 0x12, 1],
    );
}

#[test]
fn second_fixed_incoming_rpcs_reject_invalid_lengths_and_trailing_bits() {
    assert_eq!(
        encode(
            PLAY_AUDIO_STREAM,
            &AudioStream {
                url: vec![b'x'; 256],
                position: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                radius: 0.0,
                use_position: false,
            },
        ),
        Err(EncodeError::LengthExceedsLimit {
            length: 256,
            limit: 255,
        })
    );

    let truncated_shop_name = EncodedBits::from_bits([0; 31], 248).unwrap();
    assert_eq!(
        decode(SET_SHOP_NAME, &truncated_shop_name),
        Err(DecodeError::OutOfBounds {
            requested_bits: 256,
            available_bits: 248,
        })
    );

    let empty_with_trailing_byte = EncodedBits::from_bits([0], 8).unwrap();
    assert_eq!(
        decode(RESET_PLAYER_MONEY, &empty_with_trailing_byte),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 8,
            allowed_bits: 0,
        })
    );

    let non_byte_aligned = EncodedBits::from_bits([0; 33], 257).unwrap();
    assert_eq!(
        decode(SET_SHOP_NAME, &non_byte_aligned),
        Err(DecodeError::NonByteAligned { bit_len: 257 })
    );

    let nonzero_bool = EncodedBits::from_bits([2], 8).unwrap();
    assert_eq!(decode(REQUEST_SPAWN_RESPONSE, &nonzero_bool), Ok(true));
}

#[test]
fn final_fixed_incoming_rpc_inventory_has_26_unique_entries() {
    let ids = [
        id(CLIENT_CHECK),
        id(SET_VEHICLE_PARAMS_EX),
        id(VEHICLE_TUNING_NOTIFICATION),
        id(SET_VEHICLE_TIRES),
        id(VEHICLE_DAMAGE_STATUS_UPDATE),
        id(TOGGLE_WIDESCREEN),
        id(DESTROY_ACTOR),
        id(DESTROY_WEAPON_PICKUP),
        id(EDIT_ATTACHED_OBJECT),
        id(ENTER_SELECT_OBJECT),
        id(SERVER_STATISTICS_RESPONSE),
        id(SET_PLAYER_DRUNK_VISUALS),
        id(SET_PLAYER_DRUNK_HANDLING),
        id(CREATE_ACTOR),
        id(CLEAR_ACTOR_ANIMATION),
        id(SET_ACTOR_FACING_ANGLE),
        id(SET_ACTOR_POSITION),
        id(SET_ACTOR_HEALTH),
        id(SET_PLAYER_OBJECT_NO_CAMERA_COL),
        id(DISABLE_CHECKPOINT),
        id(DISABLE_RACE_CHECKPOINT),
        id(GAMEMODE_RESTART),
        id(STOP_AUDIO_STREAM),
        id(REMOVE_PLAYER_FROM_VEHICLE),
        id(FORCE_CLASS_SELECTION),
        id(SET_CAMERA_BEHIND),
    ];

    assert_eq!(
        ids,
        [
            103, 24, 96, 98, 106, 111, 172, 151, 116, 27, 102, 92, 150, 171, 174, 175, 176, 178,
            169, 37, 39, 40, 42, 71, 74, 162,
        ]
    );

    let mut unique = ids;
    unique.sort_unstable();
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn final_fixed_incoming_rpcs_preserve_exact_vectors() {
    assert_vector(
        CLIENT_CHECK,
        &ClientCheck {
            request_type: 7,
            subject: -2,
            offset: 0x1234,
            length: 0x5678,
        },
        &[7, 0xFE, 0xFF, 0xFF, 0xFF, 0x34, 0x12, 0x78, 0x56],
    );
    assert_vector(
        SET_VEHICLE_PARAMS_EX,
        &VehicleParamsEx {
            vehicle_id: 0x1234,
            params: [0xA1; 8],
            doors: [0xB2; 4],
            windows: [0xC3; 4],
        },
        &[
            0x34, 0x12, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xB2, 0xB2, 0xB2, 0xB2,
            0xC3, 0xC3, 0xC3, 0xC3,
        ],
    );
    assert_vector(
        VEHICLE_TUNING_NOTIFICATION,
        &VehicleTuningNotification {
            player_id: 0x1234,
            event: -1,
            vehicle_id: 2,
            param1: -3,
            param2: 4,
        },
        &[
            0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF, 2, 0, 0, 0, 0xFD, 0xFF, 0xFF, 0xFF, 4, 0, 0, 0,
        ],
    );
    assert_vector(SET_VEHICLE_TIRES, &(0x1234, 5), &[0x34, 0x12, 5]);
    assert_vector(
        VEHICLE_DAMAGE_STATUS_UPDATE,
        &VehicleDamageStatus {
            vehicle_id: 0x1234,
            panel_damage: -1,
            door_damage: 2,
            lights: 3,
            tires: 4,
        },
        &[0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF, 2, 0, 0, 0, 3, 4],
    );
    assert_vector(TOGGLE_WIDESCREEN, &true, &[1]);
    assert_vector(DESTROY_ACTOR, &0x1234, &[0x34, 0x12]);
    assert_vector(DESTROY_WEAPON_PICKUP, &0x56, &[0x56]);
    assert_vector(EDIT_ATTACHED_OBJECT, &-2, &[0xFE, 0xFF, 0xFF, 0xFF]);
    assert_vector(ENTER_SELECT_OBJECT, &(), &[]);
    assert_vector(SERVER_STATISTICS_RESPONSE, &(), &[]);
    assert_vector(SET_PLAYER_DRUNK_VISUALS, &-3, &[0xFD, 0xFF, 0xFF, 0xFF]);
    assert_vector(SET_PLAYER_DRUNK_HANDLING, &4, &[4, 0, 0, 0]);
    assert_vector(
        CREATE_ACTOR,
        &Actor {
            actor_id: 0x1234,
            skin_id: -2,
            position: Vector3 {
                x: 1.0,
                y: -2.0,
                z: 0.5,
            },
            rotation: 90.0,
            health: 100.0,
        },
        &[
            0x34, 0x12, 0xFE, 0xFF, 0xFF, 0xFF, 0, 0, 0x80, 0x3F, 0, 0, 0, 0xC0, 0, 0, 0, 0x3F, 0,
            0, 0xB4, 0x42, 0, 0, 0xC8, 0x42,
        ],
    );
    assert_vector(CLEAR_ACTOR_ANIMATION, &0x1234, &[0x34, 0x12]);
    assert_vector(
        SET_ACTOR_FACING_ANGLE,
        &ActorAngle {
            actor_id: 0x1234,
            angle: 180.0,
        },
        &[0x34, 0x12, 0, 0, 0x34, 0x43],
    );
    assert_vector(
        SET_ACTOR_POSITION,
        &ActorPosition {
            actor_id: 0x1234,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
        &[
            0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40,
        ],
    );
    assert_vector(
        SET_ACTOR_HEALTH,
        &ActorHealth {
            actor_id: 0x1234,
            health: 1.5,
        },
        &[0x34, 0x12, 0, 0, 0xC0, 0x3F],
    );
    assert_vector(SET_PLAYER_OBJECT_NO_CAMERA_COL, &0x1234, &[0x34, 0x12]);
    assert_vector(DISABLE_CHECKPOINT, &(), &[]);
    assert_vector(DISABLE_RACE_CHECKPOINT, &(), &[]);
    assert_vector(GAMEMODE_RESTART, &(), &[]);
    assert_vector(STOP_AUDIO_STREAM, &(), &[]);
    assert_vector(REMOVE_PLAYER_FROM_VEHICLE, &(), &[]);
    assert_vector(FORCE_CLASS_SELECTION, &(), &[]);
    assert_vector(SET_CAMERA_BEHIND, &(), &[]);

    let nonzero_bool = EncodedBits::from_bits([2], 8).unwrap();
    assert_eq!(decode(TOGGLE_WIDESCREEN, &nonzero_bool), Ok(true));
}

#[test]
fn final_fixed_incoming_rpcs_reject_truncated_and_trailing_values() {
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

    assert_rejects_malformed_value(
        CLIENT_CHECK,
        &ClientCheck {
            request_type: 0,
            subject: 0,
            offset: 0,
            length: 0,
        },
    );
    assert_rejects_malformed_value(
        SET_VEHICLE_PARAMS_EX,
        &VehicleParamsEx {
            vehicle_id: 0,
            params: [0; 8],
            doors: [0; 4],
            windows: [0; 4],
        },
    );
    assert_rejects_malformed_value(
        VEHICLE_TUNING_NOTIFICATION,
        &VehicleTuningNotification {
            player_id: 0,
            event: 0,
            vehicle_id: 0,
            param1: 0,
            param2: 0,
        },
    );
    assert_rejects_malformed_value(SET_VEHICLE_TIRES, &(0, 0));
    assert_rejects_malformed_value(
        VEHICLE_DAMAGE_STATUS_UPDATE,
        &VehicleDamageStatus {
            vehicle_id: 0,
            panel_damage: 0,
            door_damage: 0,
            lights: 0,
            tires: 0,
        },
    );
    assert_rejects_malformed_value(TOGGLE_WIDESCREEN, &false);
    assert_rejects_malformed_value(DESTROY_ACTOR, &0);
    assert_rejects_malformed_value(DESTROY_WEAPON_PICKUP, &0);
    assert_rejects_malformed_value(EDIT_ATTACHED_OBJECT, &0);
    assert_rejects_malformed_value(ENTER_SELECT_OBJECT, &());
    assert_rejects_malformed_value(SERVER_STATISTICS_RESPONSE, &());
    assert_rejects_malformed_value(SET_PLAYER_DRUNK_VISUALS, &0);
    assert_rejects_malformed_value(SET_PLAYER_DRUNK_HANDLING, &0);
    assert_rejects_malformed_value(
        CREATE_ACTOR,
        &Actor {
            actor_id: 0,
            skin_id: 0,
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: 0.0,
            health: 0.0,
        },
    );
    assert_rejects_malformed_value(CLEAR_ACTOR_ANIMATION, &0);
    assert_rejects_malformed_value(
        SET_ACTOR_FACING_ANGLE,
        &ActorAngle {
            actor_id: 0,
            angle: 0.0,
        },
    );
    assert_rejects_malformed_value(
        SET_ACTOR_POSITION,
        &ActorPosition {
            actor_id: 0,
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
    );
    assert_rejects_malformed_value(
        SET_ACTOR_HEALTH,
        &ActorHealth {
            actor_id: 0,
            health: 0.0,
        },
    );
    assert_rejects_malformed_value(SET_PLAYER_OBJECT_NO_CAMERA_COL, &0);
    assert_rejects_malformed_value(DISABLE_CHECKPOINT, &());
    assert_rejects_malformed_value(DISABLE_RACE_CHECKPOINT, &());
    assert_rejects_malformed_value(GAMEMODE_RESTART, &());
    assert_rejects_malformed_value(STOP_AUDIO_STREAM, &());
    assert_rejects_malformed_value(REMOVE_PLAYER_FROM_VEHICLE, &());
    assert_rejects_malformed_value(FORCE_CLASS_SELECTION, &());
    assert_rejects_malformed_value(SET_CAMERA_BEHIND, &());
}

#[test]
fn fixed_incoming_rpcs_reject_invalid_lengths_and_trailing_bits() {
    assert_eq!(
        encode(
            SERVER_MESSAGE,
            &ServerMessage {
                color: 0,
                text: vec![b'x'; 4097],
            },
        ),
        Err(EncodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );
    assert_eq!(
        encode(
            CHAT_MESSAGE,
            &ChatMessage {
                player_id: 0,
                text: vec![b'x'; 256],
            },
        ),
        Err(EncodeError::LengthExceedsLimit {
            length: 256,
            limit: 255,
        })
    );

    let declared_length_past_limit =
        EncodedBits::from_bits([0, 0, 0, 0, 1, 0x10, 0, 0], 64).unwrap();
    assert_eq!(
        decode(SERVER_MESSAGE, &declared_length_past_limit),
        Err(DecodeError::LengthExceedsLimit {
            length: 4097,
            limit: 4096,
        })
    );

    let non_byte_aligned = EncodedBits::from_bits([12, 0], 9).unwrap();
    assert_eq!(
        decode(SET_WORLD_TIME, &non_byte_aligned),
        Err(DecodeError::NonByteAligned { bit_len: 9 })
    );
}
