use super::wire::{decode_bool32, encode_bool32, read_bool32, write_bool32};
use crate::types::Vector3;
use crate::{BitRead, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt};

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

struct PlayerStreamInCodec;

struct PlayerAnimationCodec;

struct CrimeReportCodec;

struct PlayerAttachedObjectCodec;

struct TogglePlayerSpectatingCodec;

descriptor!(
    PlayerStreamInRpc,
    PLAYER_STREAM_IN,
    32,
    PlayerStreamInCodec,
    PlayerStreamIn,
    ExactBitsPolicy
);

descriptor!(
    PlayerAnimationRpc,
    APPLY_PLAYER_ANIMATION,
    86,
    PlayerAnimationCodec,
    PlayerAnimation,
    ExactBitsPolicy
);

descriptor!(
    CrimeReportRpc,
    PLAY_CRIME_REPORT,
    112,
    CrimeReportCodec,
    CrimeReport,
    ExactBitsPolicy
);

descriptor!(
    PlayerAttachedObjectRpc,
    SET_PLAYER_ATTACHED_OBJECT,
    113,
    PlayerAttachedObjectCodec,
    PlayerAttachedObject,
    ExactBitsPolicy
);

descriptor!(
    TogglePlayerSpectatingRpc,
    TOGGLE_PLAYER_SPECTATING,
    124,
    TogglePlayerSpectatingCodec,
    bool,
    ExactBitsPolicy
);

r1_codec!(
    PlayerStreamInCodec,
    PlayerStreamIn,
    decode_player_stream_in,
    encode_player_stream_in
);

r1_codec!(
    PlayerAnimationCodec,
    PlayerAnimation,
    decode_player_animation,
    encode_player_animation
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

pub(super) fn decode_animation<R: BitRead>(
    reader: &mut R,
) -> Result<Animation, DecodeError<R::Error>> {
    Ok(Animation {
        animation_library: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        animation_name: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        frame_delta: reader.read_f32_le()?,
        looped: reader.read_bit_bool()?,
        lock_x: reader.read_bit_bool()?,
        lock_y: reader.read_bit_bool()?,
        freeze: reader.read_bit_bool()?,
        time: reader.read_i32_le()?,
    })
}

pub(super) fn encode_animation<W: BitWrite>(
    writer: &mut W,
    value: &Animation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_len_prefixed_bytes_u8(&value.animation_library, usize::from(u8::MAX))?;
    writer.write_len_prefixed_bytes_u8(&value.animation_name, usize::from(u8::MAX))?;
    writer.write_f32_le(value.frame_delta)?;
    writer.write_bit_bool(value.looped)?;
    writer.write_bit_bool(value.lock_x)?;
    writer.write_bit_bool(value.lock_y)?;
    writer.write_bit_bool(value.freeze)?;
    writer.write_i32_le(value.time)
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
    let object = reader
        .read_bit_bool()?
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
    writer.write_bit_bool(value.object.is_some())?;
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
