//! R1 player and session incoming RPC codecs.

use crate::types::Vector3;
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBitsPolicy, WireCodec, WireReadExt,
    WireWriteExt,
};

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

/// R1's `onPlayerStreamIn` payload (RPC 32).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStreamIn {
    pub player_id: u16,
    pub team: u8,
    pub model: i32,
    pub position: Vector3,
    pub rotation: f32,
    pub color: i32,
    pub fighting_style: u8,
    /// R1 sends all eleven weapon-skill categories after the fixed player data.
    pub weapon_skill_levels: [u16; 11],
}

/// R1's player animation payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub animation_library: Vec<u8>,
    pub animation_name: Vec<u8>,
    pub frame_delta: f32,
    pub looped: bool,
    pub lock_x: bool,
    pub lock_y: bool,
    pub freeze: bool,
    pub time: i32,
}

/// R1's `onApplyPlayerAnimation` payload (RPC 86).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerAnimation {
    pub player_id: u16,
    pub animation: Animation,
}

/// R1's `onPlayCrimeReport` payload (RPC 112).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrimeReport {
    pub suspect_id: u16,
    pub in_vehicle: bool,
    pub vehicle_model: i32,
    pub vehicle_color: i32,
    pub crime: i32,
    pub coordinates: Vector3,
}

/// An attached player object, present only when the create bit is set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachedObject {
    pub model_id: i32,
    pub bone: i32,
    pub offset: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
    pub color1: i32,
    pub color2: i32,
}

/// R1's `onSetPlayerAttachedObject` payload (RPC 113).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerAttachedObject {
    pub player_id: u16,
    pub index: i32,
    pub object: Option<AttachedObject>,
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
struct PlayerStreamInCodec;
struct SpawnInfoCodec;
struct PlayerAnimationCodec;
struct EnableStuntBonusCodec;
struct CrimeReportCodec;
struct PlayerAttachedObjectCodec;
struct TogglePlayerSpectatingCodec;
struct ScoresAndPingsCodec;

macro_rules! descriptor_value {
    (InitGameCodec) => {
        InitGame
    };
    (RequestClassResponseCodec) => {
        RequestClassResponse
    };
    (PlayerStreamInCodec) => {
        PlayerStreamIn
    };
    (SpawnInfoCodec) => {
        SpawnInfo
    };
    (PlayerAnimationCodec) => {
        PlayerAnimation
    };
    (EnableStuntBonusCodec) => {
        bool
    };
    (CrimeReportCodec) => {
        CrimeReport
    };
    (PlayerAttachedObjectCodec) => {
        PlayerAttachedObject
    };
    (TogglePlayerSpectatingCodec) => {
        bool
    };
    (ScoresAndPingsCodec) => {
        ScoresAndPings
    };
}

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ident) => {
        crate::wire::nominal_descriptor!(
            incoming rpc,
            $name,
            $constant,
            $id,
            $codec,
            descriptor_value!($codec),
            ExactBitsPolicy
        );
    };
}

descriptor!(InitGameRpc, INIT_GAME, 139, InitGameCodec);
descriptor!(
    RequestClassResponseRpc,
    REQUEST_CLASS_RESPONSE,
    128,
    RequestClassResponseCodec
);
descriptor!(PlayerStreamInRpc, PLAYER_STREAM_IN, 32, PlayerStreamInCodec);
descriptor!(SpawnInfoRpc, SET_SPAWN_INFO, 68, SpawnInfoCodec);
descriptor!(
    PlayerAnimationRpc,
    APPLY_PLAYER_ANIMATION,
    86,
    PlayerAnimationCodec
);
descriptor!(
    EnableStuntBonusRpc,
    ENABLE_STUNT_BONUS,
    104,
    EnableStuntBonusCodec
);
descriptor!(CrimeReportRpc, PLAY_CRIME_REPORT, 112, CrimeReportCodec);
descriptor!(
    PlayerAttachedObjectRpc,
    SET_PLAYER_ATTACHED_OBJECT,
    113,
    PlayerAttachedObjectCodec
);
descriptor!(
    TogglePlayerSpectatingRpc,
    TOGGLE_PLAYER_SPECTATING,
    124,
    TogglePlayerSpectatingCodec
);
descriptor!(
    ScoresAndPingsRpc,
    UPDATE_SCORES_AND_PINGS,
    155,
    ScoresAndPingsCodec
);

macro_rules! r1_codec {
    ($codec:ident, $value:ty, $decode:ident, $encode:ident) => {
        impl WireCodec for $codec {
            type Value = $value;
            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                $decode(reader)
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                $encode(writer, value)
            }
        }
    };
}

r1_codec!(InitGameCodec, InitGame, decode_init_game, encode_init_game);
r1_codec!(
    RequestClassResponseCodec,
    RequestClassResponse,
    decode_request_class_response,
    encode_request_class_response
);
r1_codec!(
    PlayerStreamInCodec,
    PlayerStreamIn,
    decode_player_stream_in,
    encode_player_stream_in
);
r1_codec!(
    SpawnInfoCodec,
    SpawnInfo,
    decode_spawn_info,
    encode_spawn_info
);
r1_codec!(
    PlayerAnimationCodec,
    PlayerAnimation,
    decode_player_animation,
    encode_player_animation
);
r1_codec!(
    EnableStuntBonusCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);
r1_codec!(
    CrimeReportCodec,
    CrimeReport,
    decode_crime_report,
    encode_crime_report
);
r1_codec!(
    PlayerAttachedObjectCodec,
    PlayerAttachedObject,
    decode_player_attached_object,
    encode_player_attached_object
);
r1_codec!(
    TogglePlayerSpectatingCodec,
    bool,
    decode_bool32,
    encode_bool32
);
r1_codec!(
    ScoresAndPingsCodec,
    ScoresAndPings,
    decode_scores_and_pings,
    encode_scores_and_pings
);

fn decode_init_game<R: BitRead>(reader: &mut R) -> Result<InitGame, DecodeError<R::Error>> {
    let mut settings = GameSettings {
        zone_names: read_bit_bool(reader)?,
        use_cj_walk: read_bit_bool(reader)?,
        allow_weapons: read_bit_bool(reader)?,
        limit_global_chat_radius: read_bit_bool(reader)?,
        global_chat_radius: reader.read_f32_le()?,
        stunt_bonus: read_bit_bool(reader)?,
        nametag_draw_distance: reader.read_f32_le()?,
        disable_enter_exits: read_bit_bool(reader)?,
        nametag_los: read_bit_bool(reader)?,
        tire_popping: read_bit_bool(reader)?,
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
    settings.show_player_tags = read_bit_bool(reader)?;
    settings.player_markers_mode = reader.read_i32_le()?;
    settings.world_time = reader.read_u8()?;
    settings.world_weather = reader.read_u8()?;
    settings.gravity = reader.read_f32_le()?;
    settings.lan_mode = read_bit_bool(reader)?;
    settings.death_money_drop = reader.read_i32_le()?;
    settings.instagib = read_bit_bool(reader)?;
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
    write_bit_bool(writer, settings.zone_names)?;
    write_bit_bool(writer, settings.use_cj_walk)?;
    write_bit_bool(writer, settings.allow_weapons)?;
    write_bit_bool(writer, settings.limit_global_chat_radius)?;
    writer.write_f32_le(settings.global_chat_radius)?;
    write_bit_bool(writer, settings.stunt_bonus)?;
    writer.write_f32_le(settings.nametag_draw_distance)?;
    write_bit_bool(writer, settings.disable_enter_exits)?;
    write_bit_bool(writer, settings.nametag_los)?;
    write_bit_bool(writer, settings.tire_popping)?;
    writer.write_i32_le(settings.classes_available)?;
    writer.write_u16_le(value.player_id)?;
    write_bit_bool(writer, settings.show_player_tags)?;
    writer.write_i32_le(settings.player_markers_mode)?;
    writer.write_u8(settings.world_time)?;
    writer.write_u8(settings.world_weather)?;
    writer.write_f32_le(settings.gravity)?;
    write_bit_bool(writer, settings.lan_mode)?;
    writer.write_i32_le(settings.death_money_drop)?;
    write_bit_bool(writer, settings.instagib)?;
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

fn decode_player_stream_in<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerStreamIn, DecodeError<R::Error>> {
    let mut weapon_skill_levels = [0; 11];
    let value = PlayerStreamIn {
        player_id: reader.read_u16_le()?,
        team: reader.read_u8()?,
        model: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
        rotation: reader.read_f32_le()?,
        color: reader.read_i32_le()?,
        fighting_style: reader.read_u8()?,
        weapon_skill_levels: [0; 11],
    };
    for skill_level in &mut weapon_skill_levels {
        *skill_level = reader.read_u16_le()?;
    }
    Ok(PlayerStreamIn {
        weapon_skill_levels,
        ..value
    })
}

fn encode_player_stream_in<W: BitWrite>(
    writer: &mut W,
    value: &PlayerStreamIn,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.team)?;
    writer.write_i32_le(value.model)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.rotation)?;
    writer.write_i32_le(value.color)?;
    writer.write_u8(value.fighting_style)?;
    for skill_level in value.weapon_skill_levels {
        writer.write_u16_le(skill_level)?;
    }
    Ok(())
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

fn decode_player_animation<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerAnimation, DecodeError<R::Error>> {
    Ok(PlayerAnimation {
        player_id: reader.read_u16_le()?,
        animation: decode_animation(reader)?,
    })
}

fn encode_player_animation<W: BitWrite>(
    writer: &mut W,
    value: &PlayerAnimation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    encode_animation(writer, &value.animation)
}

fn decode_animation<R: BitRead>(reader: &mut R) -> Result<Animation, DecodeError<R::Error>> {
    Ok(Animation {
        animation_library: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        animation_name: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        frame_delta: reader.read_f32_le()?,
        looped: read_bit_bool(reader)?,
        lock_x: read_bit_bool(reader)?,
        lock_y: read_bit_bool(reader)?,
        freeze: read_bit_bool(reader)?,
        time: reader.read_i32_le()?,
    })
}

fn encode_animation<W: BitWrite>(
    writer: &mut W,
    value: &Animation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_len_prefixed_bytes_u8(&value.animation_library, usize::from(u8::MAX))?;
    writer.write_len_prefixed_bytes_u8(&value.animation_name, usize::from(u8::MAX))?;
    writer.write_f32_le(value.frame_delta)?;
    write_bit_bool(writer, value.looped)?;
    write_bit_bool(writer, value.lock_x)?;
    write_bit_bool(writer, value.lock_y)?;
    write_bit_bool(writer, value.freeze)?;
    writer.write_i32_le(value.time)
}

fn decode_bit_bool<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    read_bit_bool(reader)
}

fn encode_bit_bool<W: BitWrite>(writer: &mut W, value: &bool) -> Result<(), EncodeError<W::Error>> {
    write_bit_bool(writer, *value)
}

fn decode_crime_report<R: BitRead>(reader: &mut R) -> Result<CrimeReport, DecodeError<R::Error>> {
    Ok(CrimeReport {
        suspect_id: reader.read_u16_le()?,
        in_vehicle: read_bool32(reader)?,
        vehicle_model: reader.read_i32_le()?,
        vehicle_color: reader.read_i32_le()?,
        crime: reader.read_i32_le()?,
        coordinates: reader.read_vector3_le()?,
    })
}

fn encode_crime_report<W: BitWrite>(
    writer: &mut W,
    value: &CrimeReport,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.suspect_id)?;
    write_bool32(writer, value.in_vehicle)?;
    writer.write_i32_le(value.vehicle_model)?;
    writer.write_i32_le(value.vehicle_color)?;
    writer.write_i32_le(value.crime)?;
    writer.write_vector3_le(&value.coordinates)
}

fn decode_player_attached_object<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerAttachedObject, DecodeError<R::Error>> {
    let player_id = reader.read_u16_le()?;
    let index = reader.read_i32_le()?;
    let object = read_bit_bool(reader)?
        .then(|| decode_attached_object(reader))
        .transpose()?;
    Ok(PlayerAttachedObject {
        player_id,
        index,
        object,
    })
}

fn encode_player_attached_object<W: BitWrite>(
    writer: &mut W,
    value: &PlayerAttachedObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.index)?;
    write_bit_bool(writer, value.object.is_some())?;
    if let Some(object) = value.object {
        encode_attached_object(writer, &object)?;
    }
    Ok(())
}

fn decode_attached_object<R: BitRead>(
    reader: &mut R,
) -> Result<AttachedObject, DecodeError<R::Error>> {
    Ok(AttachedObject {
        model_id: reader.read_i32_le()?,
        bone: reader.read_i32_le()?,
        offset: reader.read_vector3_le()?,
        rotation: reader.read_vector3_le()?,
        scale: reader.read_vector3_le()?,
        color1: reader.read_i32_le()?,
        color2: reader.read_i32_le()?,
    })
}

fn encode_attached_object<W: BitWrite>(
    writer: &mut W,
    value: &AttachedObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.model_id)?;
    writer.write_i32_le(value.bone)?;
    writer.write_vector3_le(&value.offset)?;
    writer.write_vector3_le(&value.rotation)?;
    writer.write_vector3_le(&value.scale)?;
    writer.write_i32_le(value.color1)?;
    writer.write_i32_le(value.color2)
}

fn decode_bool32<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u32_le()? != 0)
}

fn encode_bool32<W: BitWrite>(writer: &mut W, value: &bool) -> Result<(), EncodeError<W::Error>> {
    write_bool32(writer, *value)
}

fn read_bool32<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u32_le()? != 0)
}

fn write_bool32<W: BitWrite>(writer: &mut W, value: bool) -> Result<(), EncodeError<W::Error>> {
    writer.write_u32_le(u32::from(value))
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

fn read_bit_bool<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(read_bits(reader, 1)?[0] & 0x80 != 0)
}

fn write_bit_bool<W: BitWrite>(writer: &mut W, value: bool) -> Result<(), EncodeError<W::Error>> {
    writer
        .write_left_aligned_bits(&[u8::from(value) << 7], 1)
        .map_err(EncodeError::Source)
}

fn read_bool8<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u8()? != 0)
}

fn write_bool8<W: BitWrite>(writer: &mut W, value: bool) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(u8::from(value))
}

fn read_fixed<R: BitRead, const LENGTH: usize>(
    reader: &mut R,
) -> Result<[u8; LENGTH], DecodeError<R::Error>> {
    let bytes = reader.read_bytes(LENGTH)?;
    match bytes.try_into() {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(DecodeError::OutOfBounds {
            requested_bits: LENGTH * u8::BITS as usize,
            available_bits: 0,
        }),
    }
}

fn read_bits<R: BitRead>(reader: &mut R, bit_len: usize) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let available_bits = reader.remaining_bits();
    if bit_len > available_bits {
        return Err(DecodeError::OutOfBounds {
            requested_bits: bit_len,
            available_bits,
        });
    }
    reader
        .read_left_aligned_bits(bit_len)
        .map_err(DecodeError::Source)
}
