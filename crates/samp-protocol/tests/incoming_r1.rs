use samp_protocol::rpc::incoming::{Vector3, r1::*};
use samp_protocol::{
    BitRead, BitStream, BitWrite, DecodeError, EncodeError, EncodedBits, WireCodec, WireDescriptor,
};

fn vector3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
}

fn spawn_info() -> SpawnInfo {
    SpawnInfo {
        team: 7,
        skin: 411,
        unused: 0xA5,
        position: vector3(1.0, 2.0, 3.0),
        rotation: 4.0,
        weapons: [22, 24, 31],
        ammo: [100, 200, 300],
    }
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

fn init_game() -> InitGame {
    InitGame {
        player_id: 42,
        host_name: b"R1 host".to_vec(),
        settings: GameSettings {
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
        },
        vehicle_models: [1; 212],
    }
}

fn assert_round_trip<D>(_descriptor: D, value: D::Value, expected_bits: usize)
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let encoded = D::encode_bits(&value).expect("the R1 RPC value must encode");
    assert_eq!(encoded.len_bits(), expected_bits);
    assert_eq!(D::decode_bits(&encoded), Ok(value));
}

fn id<D: WireDescriptor>(_: D) -> u8 {
    D::ID
}

#[test]
fn r1_stunt_bonus_preserves_its_exact_one_bit_payload() {
    let encoded =
        EnableStuntBonusRpc::encode_bits(&true).expect("the R1 stunt-bonus payload must encode");

    assert_eq!(encoded, EncodedBits::from_bits([0x80], 1).unwrap());
    assert_eq!(EnableStuntBonusRpc::decode_bits(&encoded), Ok(true));
}

#[test]
fn r1_player_and_session_descriptors_have_unique_expected_ids() {
    let ids = [
        id(INIT_GAME),
        id(REQUEST_CLASS_RESPONSE),
        id(PLAYER_STREAM_IN),
        id(SET_SPAWN_INFO),
        id(APPLY_PLAYER_ANIMATION),
        id(ENABLE_STUNT_BONUS),
        id(PLAY_CRIME_REPORT),
        id(SET_PLAYER_ATTACHED_OBJECT),
        id(TOGGLE_PLAYER_SPECTATING),
        id(UPDATE_SCORES_AND_PINGS),
    ];

    assert_eq!(ids, [139, 128, 32, 68, 86, 104, 112, 113, 124, 155]);
    let mut unique = ids;
    unique.sort_unstable();
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn r1_player_stream_in_matches_its_exact_vector() {
    let value = PlayerStreamIn {
        player_id: 42,
        team: 3,
        model: 411,
        position: vector3(1.0, 2.0, 3.0),
        rotation: 90.0,
        color: -1,
        fighting_style: 4,
        weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };
    let encoded = PlayerStreamInRpc::encode_bits(&value).expect("the R1 stream-in must encode");

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
    assert_eq!(PlayerStreamInRpc::decode_bits(&encoded), Ok(value));
}

#[test]
fn r1_player_and_session_values_keep_their_exact_bit_lengths() {
    assert_round_trip(INIT_GAME, init_game(), 2_187);
    assert_round_trip(
        REQUEST_CLASS_RESPONSE,
        RequestClassResponse {
            can_spawn: true,
            spawn: spawn_info(),
        },
        376,
    );
    assert_round_trip(
        PLAYER_STREAM_IN,
        PlayerStreamIn {
            player_id: 42,
            team: 3,
            model: 411,
            position: vector3(1.0, 2.0, 3.0),
            rotation: 90.0,
            color: -1,
            fighting_style: 4,
            weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        },
        400,
    );
    assert_round_trip(SET_SPAWN_INFO, spawn_info(), 368);
    assert_round_trip(
        APPLY_PLAYER_ANIMATION,
        PlayerAnimation {
            player_id: 7,
            animation: animation(),
        },
        156,
    );
    assert_round_trip(ENABLE_STUNT_BONUS, true, 1);
    assert_round_trip(
        PLAY_CRIME_REPORT,
        CrimeReport {
            suspect_id: 7,
            in_vehicle: true,
            vehicle_model: 411,
            vehicle_color: 4,
            crime: 9,
            coordinates: vector3(1.0, 2.0, 3.0),
        },
        240,
    );
    assert_round_trip(
        SET_PLAYER_ATTACHED_OBJECT,
        PlayerAttachedObject {
            player_id: 7,
            index: 3,
            object: None,
        },
        49,
    );
    assert_round_trip(
        SET_PLAYER_ATTACHED_OBJECT,
        PlayerAttachedObject {
            player_id: 7,
            index: 3,
            object: Some(AttachedObject {
                model_id: 19327,
                bone: 1,
                offset: vector3(1.0, 2.0, 3.0),
                rotation: vector3(4.0, 5.0, 6.0),
                scale: vector3(1.0, 1.0, 1.0),
                color1: -1,
                color2: 0,
            }),
        },
        465,
    );
    assert_round_trip(TOGGLE_PLAYER_SPECTATING, false, 32);
    assert_round_trip(
        UPDATE_SCORES_AND_PINGS,
        ScoresAndPings {
            entries: vec![
                ScorePing {
                    player_id: 7,
                    score: -100,
                    ping: 42,
                },
                ScorePing {
                    player_id: 8,
                    score: 100,
                    ping: 24,
                },
            ],
        },
        160,
    );
}

#[test]
fn r1_rpc_codec_preserves_values_from_an_unaligned_cursor() {
    let value = init_game();
    let mut writer = BitStream::new();
    BitWrite::write_left_aligned_bits(&mut writer, &[0b1010_0000], 3).unwrap();
    InitGameCodec::encode(&mut writer, &value).unwrap();

    let mut reader = BitStream::from_bits(writer.as_bytes(), writer.len_bits()).unwrap();
    assert_eq!(
        BitRead::read_left_aligned_bits(&mut reader, 3),
        Ok(vec![0b1010_0000])
    );
    assert_eq!(InitGameCodec::decode(&mut reader), Ok(value));
    assert_eq!(reader.remaining_bits(), 0);
}

#[test]
fn r1_codecs_reject_malformed_lengths_and_bounds() {
    let stream_in = PlayerStreamIn {
        player_id: 42,
        team: 3,
        model: 411,
        position: vector3(1.0, 2.0, 3.0),
        rotation: 90.0,
        color: -1,
        fighting_style: 4,
        weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };
    let encoded = PlayerStreamInRpc::encode_bits(&stream_in).expect("the stream-in must encode");
    let truncated = EncodedBits::from_bits(encoded.as_bytes()[..49].to_vec(), 392)
        .expect("the truncated stream-in payload must be valid bits");
    assert!(PlayerStreamInRpc::decode_bits(&truncated).is_err());
    let mut trailing_bytes = encoded.as_bytes().to_vec();
    trailing_bytes.push(0);
    let trailing = EncodedBits::from_bits(trailing_bytes, 408)
        .expect("the stream-in payload with trailing bits must be valid");
    assert_eq!(
        PlayerStreamInRpc::decode_bits(&trailing),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 8,
            allowed_bits: 0,
        })
    );

    assert_eq!(
        ScoresAndPingsRpc::decode_bits(
            &EncodedBits::from_bits([0x80], 1).expect("the malformed payload must be valid bits"),
        ),
        Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits: 1,
            allowed_bits: 0,
        })
    );

    let oversized_entries = ScoresAndPings {
        entries: vec![
            ScorePing {
                player_id: 0,
                score: 0,
                ping: 0,
            };
            MAX_SCORE_PING_ENTRIES + 1
        ],
    };
    assert_eq!(
        ScoresAndPingsRpc::encode_bits(&oversized_entries),
        Err(EncodeError::LengthExceedsLimit {
            length: MAX_SCORE_PING_ENTRIES + 1,
            limit: MAX_SCORE_PING_ENTRIES,
        })
    );
    let oversized_bits = EncodedBits::from_bits(
        vec![0; (MAX_SCORE_PING_ENTRIES + 1) * 10],
        (MAX_SCORE_PING_ENTRIES + 1) * 80,
    )
    .expect("the oversized score payload must be valid bits");
    assert_eq!(
        ScoresAndPingsRpc::decode_bits(&oversized_bits),
        Err(DecodeError::LengthExceedsLimit {
            length: MAX_SCORE_PING_ENTRIES + 1,
            limit: MAX_SCORE_PING_ENTRIES,
        })
    );

    let mut max_string = animation();
    max_string.animation_library = vec![b'x'; u8::MAX as usize];
    assert!(
        PlayerAnimationRpc::decode_bits(
            &PlayerAnimationRpc::encode_bits(&PlayerAnimation {
                player_id: 0,
                animation: max_string,
            })
            .expect("a maximum String8 value must encode"),
        )
        .is_ok()
    );
    let mut oversized_string = animation();
    oversized_string.animation_library = vec![b'x'; u8::MAX as usize + 1];
    assert_eq!(
        PlayerAnimationRpc::encode_bits(&PlayerAnimation {
            player_id: 0,
            animation: oversized_string,
        }),
        Err(EncodeError::LengthExceedsLimit {
            length: u8::MAX as usize + 1,
            limit: u8::MAX as usize,
        })
    );
}
