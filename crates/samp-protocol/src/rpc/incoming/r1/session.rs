use super::*;

/// The server can send at most one score/ping entry for each R1 player slot.
pub const MAX_SCORE_PING_ENTRIES: usize = 1_000;

/// Settings supplied by `onInitGame` (RPC 139).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameSettings {
    pub zone_names: bool,
    pub use_cj_walk: bool,
    pub allow_weapons: bool,
    pub limit_global_chat_radius: bool,
    pub global_chat_radius: f32,
    pub stunt_bonus: bool,
    pub nametag_draw_distance: f32,
    pub disable_enter_exits: bool,
    pub nametag_los: bool,
    pub tire_popping: bool,
    pub classes_available: i32,
    pub show_player_tags: bool,
    pub player_markers_mode: i32,
    pub world_time: u8,
    pub world_weather: u8,
    pub gravity: f32,
    pub lan_mode: bool,
    pub death_money_drop: i32,
    pub instagib: bool,
    pub normal_onfoot_send_rate: i32,
    pub normal_incar_send_rate: i32,
    pub normal_firing_send_rate: i32,
    pub send_multiplier: i32,
    pub lag_compensation_mode: i32,
    pub vehicle_friendly_fire: bool,
}

/// R1's `onInitGame` payload (RPC 139).
#[derive(Clone, Debug, PartialEq)]
pub struct InitGame {
    pub player_id: u16,
    pub host_name: Vec<u8>,
    pub settings: GameSettings,
    /// R1's 212 vehicle-model capability flags, retained byte-for-byte.
    pub vehicle_models: [u8; 212],
}

/// A class preview or spawn definition shared by the class and spawn RPCs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnInfo {
    pub team: u8,
    pub skin: i32,
    /// R1 serializes this byte between the skin and position. Its purpose is unknown.
    pub unused: u8,
    pub position: Vector3,
    pub rotation: f32,
    pub weapons: [i32; 3],
    pub ammo: [i32; 3],
}

/// R1's `onRequestClassResponse` payload (RPC 128).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestClassResponse {
    pub can_spawn: bool,
    pub spawn: SpawnInfo,
}

/// One score and ping record sent by RPC 155.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScorePing {
    pub player_id: u16,
    pub score: i32,
    pub ping: i32,
}

/// R1's `onUpdateScoresAndPings` payload (RPC 155), retained in wire order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoresAndPings {
    pub entries: Vec<ScorePing>,
}

struct InitGameCodec;

struct RequestClassResponseCodec;

struct SpawnInfoCodec;

struct EnableStuntBonusCodec;

struct ScoresAndPingsCodec;

descriptor!(
    InitGameRpc,
    INIT_GAME,
    139,
    InitGameCodec,
    InitGame,
    ExactBitsPolicy
);

descriptor!(
    RequestClassResponseRpc,
    REQUEST_CLASS_RESPONSE,
    128,
    RequestClassResponseCodec,
    RequestClassResponse,
    ExactBitsPolicy
);

descriptor!(
    SpawnInfoRpc,
    SET_SPAWN_INFO,
    68,
    SpawnInfoCodec,
    SpawnInfo,
    ExactBitsPolicy
);

descriptor!(
    EnableStuntBonusRpc,
    ENABLE_STUNT_BONUS,
    104,
    EnableStuntBonusCodec,
    bool,
    ExactBitsPolicy
);

descriptor!(
    ScoresAndPingsRpc,
    UPDATE_SCORES_AND_PINGS,
    155,
    ScoresAndPingsCodec,
    ScoresAndPings,
    ExactBitsPolicy
);

r1_codec!(InitGameCodec, InitGame, decode_init_game, encode_init_game);

r1_codec!(
    RequestClassResponseCodec,
    RequestClassResponse,
    decode_request_class_response,
    encode_request_class_response
);

r1_codec!(
    SpawnInfoCodec,
    SpawnInfo,
    decode_spawn_info,
    encode_spawn_info
);

r1_codec!(
    EnableStuntBonusCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);

r1_codec!(
    ScoresAndPingsCodec,
    ScoresAndPings,
    decode_scores_and_pings,
    encode_scores_and_pings
);

fn decode_init_game<R: BitRead>(reader: &mut R) -> Result<InitGame, DecodeError<R::Error>> {
    let mut settings = GameSettings {
        zone_names: reader.read_bit_bool()?,
        use_cj_walk: reader.read_bit_bool()?,
        allow_weapons: reader.read_bit_bool()?,
        limit_global_chat_radius: reader.read_bit_bool()?,
        global_chat_radius: reader.read_f32_le()?,
        stunt_bonus: reader.read_bit_bool()?,
        nametag_draw_distance: reader.read_f32_le()?,
        disable_enter_exits: reader.read_bit_bool()?,
        nametag_los: reader.read_bit_bool()?,
        tire_popping: reader.read_bit_bool()?,
        classes_available: reader.read_i32_le()?,
        show_player_tags: false,
        player_markers_mode: 0,
        world_time: 0,
        world_weather: 0,
        gravity: 0.0,
        lan_mode: false,
        death_money_drop: 0,
        instagib: false,
        normal_onfoot_send_rate: 0,
        normal_incar_send_rate: 0,
        normal_firing_send_rate: 0,
        send_multiplier: 0,
        lag_compensation_mode: 0,
        vehicle_friendly_fire: false,
    };
    let player_id = reader.read_u16_le()?;
    settings.show_player_tags = reader.read_bit_bool()?;
    settings.player_markers_mode = reader.read_i32_le()?;
    settings.world_time = reader.read_u8()?;
    settings.world_weather = reader.read_u8()?;
    settings.gravity = reader.read_f32_le()?;
    settings.lan_mode = reader.read_bit_bool()?;
    settings.death_money_drop = reader.read_i32_le()?;
    settings.instagib = reader.read_bit_bool()?;
    settings.normal_onfoot_send_rate = reader.read_i32_le()?;
    settings.normal_incar_send_rate = reader.read_i32_le()?;
    settings.normal_firing_send_rate = reader.read_i32_le()?;
    settings.send_multiplier = reader.read_i32_le()?;
    settings.lag_compensation_mode = reader.read_i32_le()?;
    let host_name = reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?;
    let vehicle_models = read_fixed(reader)?;
    settings.vehicle_friendly_fire = read_bool32(reader)?;
    Ok(InitGame {
        player_id,
        host_name,
        settings,
        vehicle_models,
    })
}

fn encode_init_game<W: BitWrite>(
    writer: &mut W,
    value: &InitGame,
) -> Result<(), EncodeError<W::Error>> {
    let settings = value.settings;
    writer.write_bit_bool(settings.zone_names)?;
    writer.write_bit_bool(settings.use_cj_walk)?;
    writer.write_bit_bool(settings.allow_weapons)?;
    writer.write_bit_bool(settings.limit_global_chat_radius)?;
    writer.write_f32_le(settings.global_chat_radius)?;
    writer.write_bit_bool(settings.stunt_bonus)?;
    writer.write_f32_le(settings.nametag_draw_distance)?;
    writer.write_bit_bool(settings.disable_enter_exits)?;
    writer.write_bit_bool(settings.nametag_los)?;
    writer.write_bit_bool(settings.tire_popping)?;
    writer.write_i32_le(settings.classes_available)?;
    writer.write_u16_le(value.player_id)?;
    writer.write_bit_bool(settings.show_player_tags)?;
    writer.write_i32_le(settings.player_markers_mode)?;
    writer.write_u8(settings.world_time)?;
    writer.write_u8(settings.world_weather)?;
    writer.write_f32_le(settings.gravity)?;
    writer.write_bit_bool(settings.lan_mode)?;
    writer.write_i32_le(settings.death_money_drop)?;
    writer.write_bit_bool(settings.instagib)?;
    writer.write_i32_le(settings.normal_onfoot_send_rate)?;
    writer.write_i32_le(settings.normal_incar_send_rate)?;
    writer.write_i32_le(settings.normal_firing_send_rate)?;
    writer.write_i32_le(settings.send_multiplier)?;
    writer.write_i32_le(settings.lag_compensation_mode)?;
    writer.write_len_prefixed_bytes_u8(&value.host_name, usize::from(u8::MAX))?;
    writer.write_bytes(&value.vehicle_models)?;
    write_bool32(writer, settings.vehicle_friendly_fire)
}

fn decode_request_class_response<R: BitRead>(
    reader: &mut R,
) -> Result<RequestClassResponse, DecodeError<R::Error>> {
    Ok(RequestClassResponse {
        can_spawn: read_bool8(reader)?,
        spawn: decode_spawn_info(reader)?,
    })
}

fn encode_request_class_response<W: BitWrite>(
    writer: &mut W,
    value: &RequestClassResponse,
) -> Result<(), EncodeError<W::Error>> {
    write_bool8(writer, value.can_spawn)?;
    encode_spawn_info(writer, &value.spawn)
}

fn decode_spawn_info<R: BitRead>(reader: &mut R) -> Result<SpawnInfo, DecodeError<R::Error>> {
    Ok(SpawnInfo {
        team: reader.read_u8()?,
        skin: reader.read_i32_le()?,
        unused: reader.read_u8()?,
        position: reader.read_vector3_le()?,
        rotation: reader.read_f32_le()?,
        weapons: [
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
        ],
        ammo: [
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
        ],
    })
}

fn encode_spawn_info<W: BitWrite>(
    writer: &mut W,
    value: &SpawnInfo,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.team)?;
    writer.write_i32_le(value.skin)?;
    writer.write_u8(value.unused)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.rotation)?;
    for weapon in value.weapons {
        writer.write_i32_le(weapon)?;
    }
    for ammo in value.ammo {
        writer.write_i32_le(ammo)?;
    }
    Ok(())
}

fn decode_scores_and_pings<R: BitRead>(
    reader: &mut R,
) -> Result<ScoresAndPings, DecodeError<R::Error>> {
    let remaining_bits = reader.remaining_bits();
    if !remaining_bits.is_multiple_of(80) {
        return Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits,
            allowed_bits: 0,
        });
    }
    let count = remaining_bits / 80;
    if count > MAX_SCORE_PING_ENTRIES {
        return Err(DecodeError::LengthExceedsLimit {
            length: count,
            limit: MAX_SCORE_PING_ENTRIES,
        });
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(ScorePing {
            player_id: reader.read_u16_le()?,
            score: reader.read_i32_le()?,
            ping: reader.read_i32_le()?,
        });
    }
    Ok(ScoresAndPings { entries })
}

fn encode_scores_and_pings<W: BitWrite>(
    writer: &mut W,
    value: &ScoresAndPings,
) -> Result<(), EncodeError<W::Error>> {
    if value.entries.len() > MAX_SCORE_PING_ENTRIES {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.entries.len(),
            limit: MAX_SCORE_PING_ENTRIES,
        });
    }
    for entry in &value.entries {
        writer.write_u16_le(entry.player_id)?;
        writer.write_i32_le(entry.score)?;
        writer.write_i32_le(entry.ping)?;
    }
    Ok(())
}
