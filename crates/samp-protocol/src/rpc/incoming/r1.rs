//! R1 player and session incoming RPC codecs.

use super::fixed::Vector3;
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, IncomingRpc, TrailingPolicy, WireCodec,
    WireReadExt, WireWriteExt,
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

pub struct InitGameCodec;
pub struct RequestClassResponseCodec;
pub struct PlayerStreamInCodec;
pub struct SpawnInfoCodec;
pub struct PlayerAnimationCodec;
pub struct EnableStuntBonusCodec;
pub struct CrimeReportCodec;
pub struct PlayerAttachedObjectCodec;
pub struct TogglePlayerSpectatingCodec;
pub struct ScoresAndPingsCodec;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty) => {
        pub type $name = IncomingRpc<$id, $codec>;
        pub const $constant: $name = IncomingRpc::new();
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
            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;

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
        global_chat_radius: read_f32(reader)?,
        stunt_bonus: read_bit_bool(reader)?,
        nametag_draw_distance: read_f32(reader)?,
        disable_enter_exits: read_bit_bool(reader)?,
        nametag_los: read_bit_bool(reader)?,
        tire_popping: read_bit_bool(reader)?,
        classes_available: read_i32(reader)?,
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
    let player_id = read_u16(reader)?;
    settings.show_player_tags = read_bit_bool(reader)?;
    settings.player_markers_mode = read_i32(reader)?;
    settings.world_time = read_u8(reader)?;
    settings.world_weather = read_u8(reader)?;
    settings.gravity = read_f32(reader)?;
    settings.lan_mode = read_bit_bool(reader)?;
    settings.death_money_drop = read_i32(reader)?;
    settings.instagib = read_bit_bool(reader)?;
    settings.normal_onfoot_send_rate = read_i32(reader)?;
    settings.normal_incar_send_rate = read_i32(reader)?;
    settings.normal_firing_send_rate = read_i32(reader)?;
    settings.send_multiplier = read_i32(reader)?;
    settings.lag_compensation_mode = read_i32(reader)?;
    let host_name = read_string8(reader)?;
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
    write_f32(writer, settings.global_chat_radius)?;
    write_bit_bool(writer, settings.stunt_bonus)?;
    write_f32(writer, settings.nametag_draw_distance)?;
    write_bit_bool(writer, settings.disable_enter_exits)?;
    write_bit_bool(writer, settings.nametag_los)?;
    write_bit_bool(writer, settings.tire_popping)?;
    write_i32(writer, settings.classes_available)?;
    write_u16(writer, value.player_id)?;
    write_bit_bool(writer, settings.show_player_tags)?;
    write_i32(writer, settings.player_markers_mode)?;
    write_u8(writer, settings.world_time)?;
    write_u8(writer, settings.world_weather)?;
    write_f32(writer, settings.gravity)?;
    write_bit_bool(writer, settings.lan_mode)?;
    write_i32(writer, settings.death_money_drop)?;
    write_bit_bool(writer, settings.instagib)?;
    write_i32(writer, settings.normal_onfoot_send_rate)?;
    write_i32(writer, settings.normal_incar_send_rate)?;
    write_i32(writer, settings.normal_firing_send_rate)?;
    write_i32(writer, settings.send_multiplier)?;
    write_i32(writer, settings.lag_compensation_mode)?;
    write_string8(writer, &value.host_name)?;
    write_bytes(writer, &value.vehicle_models)?;
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
        player_id: read_u16(reader)?,
        team: read_u8(reader)?,
        model: read_i32(reader)?,
        position: read_vector3(reader)?,
        rotation: read_f32(reader)?,
        color: read_i32(reader)?,
        fighting_style: read_u8(reader)?,
        weapon_skill_levels: [0; 11],
    };
    for skill_level in &mut weapon_skill_levels {
        *skill_level = read_u16(reader)?;
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
    write_u16(writer, value.player_id)?;
    write_u8(writer, value.team)?;
    write_i32(writer, value.model)?;
    write_vector3(writer, &value.position)?;
    write_f32(writer, value.rotation)?;
    write_i32(writer, value.color)?;
    write_u8(writer, value.fighting_style)?;
    for skill_level in value.weapon_skill_levels {
        write_u16(writer, skill_level)?;
    }
    Ok(())
}

fn decode_spawn_info<R: BitRead>(reader: &mut R) -> Result<SpawnInfo, DecodeError<R::Error>> {
    Ok(SpawnInfo {
        team: read_u8(reader)?,
        skin: read_i32(reader)?,
        unused: read_u8(reader)?,
        position: read_vector3(reader)?,
        rotation: read_f32(reader)?,
        weapons: [read_i32(reader)?, read_i32(reader)?, read_i32(reader)?],
        ammo: [read_i32(reader)?, read_i32(reader)?, read_i32(reader)?],
    })
}

fn encode_spawn_info<W: BitWrite>(
    writer: &mut W,
    value: &SpawnInfo,
) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, value.team)?;
    write_i32(writer, value.skin)?;
    write_u8(writer, value.unused)?;
    write_vector3(writer, &value.position)?;
    write_f32(writer, value.rotation)?;
    for weapon in value.weapons {
        write_i32(writer, weapon)?;
    }
    for ammo in value.ammo {
        write_i32(writer, ammo)?;
    }
    Ok(())
}

fn decode_player_animation<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerAnimation, DecodeError<R::Error>> {
    Ok(PlayerAnimation {
        player_id: read_u16(reader)?,
        animation: decode_animation(reader)?,
    })
}

fn encode_player_animation<W: BitWrite>(
    writer: &mut W,
    value: &PlayerAnimation,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, value.player_id)?;
    encode_animation(writer, &value.animation)
}

fn decode_animation<R: BitRead>(reader: &mut R) -> Result<Animation, DecodeError<R::Error>> {
    Ok(Animation {
        animation_library: read_string8(reader)?,
        animation_name: read_string8(reader)?,
        frame_delta: read_f32(reader)?,
        looped: read_bit_bool(reader)?,
        lock_x: read_bit_bool(reader)?,
        lock_y: read_bit_bool(reader)?,
        freeze: read_bit_bool(reader)?,
        time: read_i32(reader)?,
    })
}

fn encode_animation<W: BitWrite>(
    writer: &mut W,
    value: &Animation,
) -> Result<(), EncodeError<W::Error>> {
    write_string8(writer, &value.animation_library)?;
    write_string8(writer, &value.animation_name)?;
    write_f32(writer, value.frame_delta)?;
    write_bit_bool(writer, value.looped)?;
    write_bit_bool(writer, value.lock_x)?;
    write_bit_bool(writer, value.lock_y)?;
    write_bit_bool(writer, value.freeze)?;
    write_i32(writer, value.time)
}

fn decode_bit_bool<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    read_bit_bool(reader)
}

fn encode_bit_bool<W: BitWrite>(writer: &mut W, value: &bool) -> Result<(), EncodeError<W::Error>> {
    write_bit_bool(writer, *value)
}

fn decode_crime_report<R: BitRead>(reader: &mut R) -> Result<CrimeReport, DecodeError<R::Error>> {
    Ok(CrimeReport {
        suspect_id: read_u16(reader)?,
        in_vehicle: read_bool32(reader)?,
        vehicle_model: read_i32(reader)?,
        vehicle_color: read_i32(reader)?,
        crime: read_i32(reader)?,
        coordinates: read_vector3(reader)?,
    })
}

fn encode_crime_report<W: BitWrite>(
    writer: &mut W,
    value: &CrimeReport,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, value.suspect_id)?;
    write_bool32(writer, value.in_vehicle)?;
    write_i32(writer, value.vehicle_model)?;
    write_i32(writer, value.vehicle_color)?;
    write_i32(writer, value.crime)?;
    write_vector3(writer, &value.coordinates)
}

fn decode_player_attached_object<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerAttachedObject, DecodeError<R::Error>> {
    let player_id = read_u16(reader)?;
    let index = read_i32(reader)?;
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
    write_u16(writer, value.player_id)?;
    write_i32(writer, value.index)?;
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
        model_id: read_i32(reader)?,
        bone: read_i32(reader)?,
        offset: read_vector3(reader)?,
        rotation: read_vector3(reader)?,
        scale: read_vector3(reader)?,
        color1: read_i32(reader)?,
        color2: read_i32(reader)?,
    })
}

fn encode_attached_object<W: BitWrite>(
    writer: &mut W,
    value: &AttachedObject,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, value.model_id)?;
    write_i32(writer, value.bone)?;
    write_vector3(writer, &value.offset)?;
    write_vector3(writer, &value.rotation)?;
    write_vector3(writer, &value.scale)?;
    write_i32(writer, value.color1)?;
    write_i32(writer, value.color2)
}

fn decode_bool32<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(read_u32(reader)? != 0)
}

fn encode_bool32<W: BitWrite>(writer: &mut W, value: &bool) -> Result<(), EncodeError<W::Error>> {
    write_bool32(writer, *value)
}

fn read_bool32<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(read_u32(reader)? != 0)
}

fn write_bool32<W: BitWrite>(writer: &mut W, value: bool) -> Result<(), EncodeError<W::Error>> {
    write_u32(writer, u32::from(value))
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
            player_id: read_u16(reader)?,
            score: read_i32(reader)?,
            ping: read_i32(reader)?,
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
        write_u16(writer, entry.player_id)?;
        write_i32(writer, entry.score)?;
        write_i32(writer, entry.ping)?;
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

fn read_u8<R: BitRead>(reader: &mut R) -> Result<u8, DecodeError<R::Error>> {
    WireReadExt::read_u8(reader)
}

fn write_u8<W: BitWrite>(writer: &mut W, value: u8) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_u8(writer, value)
}

fn read_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    WireReadExt::read_u16_le(reader)
}

fn write_u16<W: BitWrite>(writer: &mut W, value: u16) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_u16_le(writer, value)
}

fn read_u32<R: BitRead>(reader: &mut R) -> Result<u32, DecodeError<R::Error>> {
    WireReadExt::read_u32_le(reader)
}

fn write_u32<W: BitWrite>(writer: &mut W, value: u32) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_u32_le(writer, value)
}

fn read_i32<R: BitRead>(reader: &mut R) -> Result<i32, DecodeError<R::Error>> {
    WireReadExt::read_i32_le(reader)
}

fn write_i32<W: BitWrite>(writer: &mut W, value: i32) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_i32_le(writer, value)
}

fn read_f32<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    WireReadExt::read_f32_le(reader)
}

fn write_f32<W: BitWrite>(writer: &mut W, value: f32) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_f32_le(writer, value)
}

fn read_bool8<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(read_u8(reader)? != 0)
}

fn write_bool8<W: BitWrite>(writer: &mut W, value: bool) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, u8::from(value))
}

fn read_vector3<R: BitRead>(reader: &mut R) -> Result<Vector3, DecodeError<R::Error>> {
    WireReadExt::read_vector3_le(reader)
}

fn write_vector3<W: BitWrite>(
    writer: &mut W,
    value: &Vector3,
) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_vector3_le(writer, value)
}

fn read_string8<R: BitRead>(reader: &mut R) -> Result<Vec<u8>, DecodeError<R::Error>> {
    WireReadExt::read_len_prefixed_bytes_u8(reader, usize::from(u8::MAX))
}

fn write_string8<W: BitWrite>(writer: &mut W, value: &[u8]) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_len_prefixed_bytes_u8(writer, value, usize::from(u8::MAX))
}

fn read_fixed<R: BitRead, const LENGTH: usize>(
    reader: &mut R,
) -> Result<[u8; LENGTH], DecodeError<R::Error>> {
    let bytes = WireReadExt::read_bytes(reader, LENGTH)?;
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

fn write_bytes<W: BitWrite>(writer: &mut W, bytes: &[u8]) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_bytes(writer, bytes)
}
