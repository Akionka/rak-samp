use samp_protocol::rpc::incoming::{
    CHAT_BUBBLE, CHAT_MESSAGE, ChatBubble, ChatMessage, Checkpoint, DISPLAY_GAME_TEXT,
    GIVE_PLAYER_MONEY, GIVE_PLAYER_WEAPON, GameText, PLAY_SOUND, PLAYER_JOIN, PLAYER_QUIT,
    PLAYER_STREAM_OUT, PUT_PLAYER_IN_VEHICLE, PlaySound, PlayerJoin, PlayerName, PlayerQuit,
    PlayerSkin, PlayerTeam, PlayerTime, PlayerWeapon, PutPlayerInVehicle, SERVER_MESSAGE,
    SET_CHECKPOINT, SET_INTERIOR, SET_PLAYER_ARMED_WEAPON, SET_PLAYER_ARMOUR,
    SET_PLAYER_FACING_ANGLE, SET_PLAYER_HEALTH, SET_PLAYER_NAME, SET_PLAYER_POS,
    SET_PLAYER_POS_FIND_Z, SET_PLAYER_SKIN, SET_PLAYER_TEAM, SET_PLAYER_TIME,
    SET_PLAYER_WANTED_LEVEL, SET_WEATHER, SET_WORLD_BOUNDS, SET_WORLD_TIME, ServerMessage,
    TOGGLE_PLAYER_CONTROLLABLE, VEHICLE_STREAM_OUT, Vector3, WorldBounds,
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
