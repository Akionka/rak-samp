use super::core::{PayloadWriter, handle};
/// Deferred R1 exact-bit RakNet Packet helpers.
///
/// Common byte-aligned Packet layouts live in `samp-protocol`. This module retains the R1 remote
/// player, remote vehicle, and marker-sync layouts, whose exact-bit encoding is deferred.
use super::{EncodedPayload, Event, EventError, IncomingPacket, RpcAction, Vector3};
use crate::{HostApi, SampClientSdkEventV1, SampClientSdkHookAction};

/// R1 marker packets cannot contain more players than the protocol player-slot limit.
pub const MAX_MARKERS: usize = 1_000;

pub const VEHICLE_SYNC_ID: u8 = 200;
pub const PLAYER_SYNC_ID: u8 = 207;
pub const MARKERS_SYNC_ID: u8 = 208;

/// The optional surfing data carried by an incoming remote player-sync packet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemotePlayerSurfing {
    pub vehicle_id: u16,
    pub offsets: Vector3,
}

/// The optional animation data carried by an incoming remote player-sync packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemotePlayerAnimation {
    pub id: u16,
    pub flags: u16,
}

/// The compressed R1 `ID_PLAYER_SYNC` layout received from a remote player (packet 207).
///
/// Unlike local outgoing [`PlayerSync`], its optional key, surf, and animation fields are
/// serialized as individual bits and the velocity and quaternion are compressed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemotePlayerSync {
    pub player_id: u16,
    pub left_right_keys: Option<u16>,
    pub up_down_keys: Option<u16>,
    pub key_data: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub health: u8,
    pub armour: u8,
    pub weapon: u8,
    pub special_action: u8,
    pub move_speed: Vector3,
    pub surfing: Option<RemotePlayerSurfing>,
    pub animation: Option<RemotePlayerAnimation>,
}

/// The compressed R1 `ID_VEHICLE_SYNC` layout received from a remote player (packet 200).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteVehicleSync {
    pub player_id: u16,
    pub vehicle_id: u16,
    pub left_right_keys: u16,
    pub up_down_keys: u16,
    pub key_data: u16,
    pub quaternion: [f32; 4],
    pub position: Vector3,
    pub move_speed: Vector3,
    pub vehicle_health: u16,
    pub player_health: u8,
    pub armour: u8,
    pub current_weapon: u8,
    pub siren: bool,
    pub landing_gear: bool,
    pub train_speed: Option<i32>,
    pub trailer_id: Option<u16>,
}

/// Signed R1 marker coordinates. They are not floating-point world coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkerCoordinates {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// One R1 player marker, active or inactive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marker {
    pub player_id: u16,
    pub coordinates: Option<MarkerCoordinates>,
}

/// The R1 `ID_MARKERS_SYNC` payload (packet 208).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkersSync {
    pub markers: Vec<Marker>,
}

pub(super) fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    Ok(Vector3 {
        x: event.read_f32()?,
        y: event.read_f32()?,
        z: event.read_f32()?,
    })
}

pub(super) fn write_vector3(writer: &mut PayloadWriter, value: Vector3) {
    writer.vector3(value);
}

pub(super) fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

pub(super) fn read_bit_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_bits(1)?[0] & 0x80 != 0)
}

pub(super) fn read_compressed_float(event: &mut Event<'_>) -> Result<f32, EventError> {
    Ok(f32::from(event.read_u16()?) / 32_767.5 - 1.0)
}

pub(super) fn compressed_float_code(value: f32) -> u16 {
    ((value.clamp(-1.0, 1.0) + 1.0) * 32_767.5).round() as u16
}

pub(super) fn write_compressed_float(writer: &mut PayloadWriter, value: f32) {
    writer.u16(compressed_float_code(value));
}

pub(super) fn decode_compressed_vector(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    let magnitude = event.read_f32()?;
    if magnitude == 0.0 {
        return Ok(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
    }
    Ok(Vector3 {
        x: read_compressed_float(event)? * magnitude,
        y: read_compressed_float(event)? * magnitude,
        z: read_compressed_float(event)? * magnitude,
    })
}

pub(super) fn encode_compressed_vector(writer: &mut PayloadWriter, value: Vector3) {
    let magnitude = (value.x.mul_add(value.x, value.y * value.y) + value.z * value.z).sqrt();
    writer.f32(magnitude);
    if magnitude != 0.0 {
        write_compressed_float(writer, value.x / magnitude);
        write_compressed_float(writer, value.y / magnitude);
        write_compressed_float(writer, value.z / magnitude);
    }
}

pub(super) fn decode_normalized_quaternion(event: &mut Event<'_>) -> Result<[f32; 4], EventError> {
    let w_negative = read_bit_bool(event)?;
    let x_negative = read_bit_bool(event)?;
    let y_negative = read_bit_bool(event)?;
    let z_negative = read_bit_bool(event)?;
    let mut x = f32::from(event.read_u16()?) / 65_535.0;
    let mut y = f32::from(event.read_u16()?) / 65_535.0;
    let mut z = f32::from(event.read_u16()?) / 65_535.0;
    if x_negative {
        x = -x;
    }
    if y_negative {
        y = -y;
    }
    if z_negative {
        z = -z;
    }
    let mut w = (1.0 - x * x - y * y - z * z).max(0.0).sqrt();
    if w_negative {
        w = -w;
    }
    Ok([w, x, y, z])
}

pub(super) fn encode_normalized_quaternion(writer: &mut PayloadWriter, value: [f32; 4]) {
    let [w, x, y, z] = value;
    writer.bool(w < 0.0);
    writer.bool(x < 0.0);
    writer.bool(y < 0.0);
    writer.bool(z < 0.0);
    for component in [x, y, z] {
        writer.u16((component.abs().clamp(0.0, 1.0) * 65_535.0).floor() as u16);
    }
}

pub(super) fn decode_health_armour(value: u8) -> (u8, u8) {
    (((value >> 4) * 7).min(100), ((value & 0x0F) * 7).min(100))
}

pub(super) fn encode_health_armour(health: u8, armour: u8) -> u8 {
    let health = if health >= 100 {
        0xF0
    } else {
        (health / 7) << 4
    };
    let armour = if armour >= 100 { 0x0F } else { armour / 7 };
    health | armour
}

pub(super) fn decode_remote_player_sync(
    event: &mut Event<'_>,
) -> Result<RemotePlayerSync, EventError> {
    let player_id = event.read_u16()?;
    let left_right_keys = read_bit_bool(event)?
        .then(|| event.read_u16())
        .transpose()?;
    let up_down_keys = read_bit_bool(event)?
        .then(|| event.read_u16())
        .transpose()?;
    let key_data = event.read_u16()?;
    let position = decode_vector3(event)?;
    let quaternion = decode_normalized_quaternion(event)?;
    let (health, armour) = decode_health_armour(event.read_u8()?);
    let weapon = event.read_u8()?;
    let special_action = event.read_u8()?;
    let move_speed = decode_compressed_vector(event)?;
    let surfing = read_bit_bool(event)?
        .then(|| {
            Ok(RemotePlayerSurfing {
                vehicle_id: event.read_u16()?,
                offsets: decode_vector3(event)?,
            })
        })
        .transpose()?;
    let animation = read_bit_bool(event)?
        .then(|| {
            Ok(RemotePlayerAnimation {
                id: event.read_u16()?,
                flags: event.read_u16()?,
            })
        })
        .transpose()?;
    Ok(RemotePlayerSync {
        player_id,
        left_right_keys,
        up_down_keys,
        key_data,
        position,
        quaternion,
        health,
        armour,
        weapon,
        special_action,
        move_speed,
        surfing,
        animation,
    })
}

pub(super) fn encode_remote_player_sync(
    _api: HostApi,
    value: RemotePlayerSync,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.bool(value.left_right_keys.is_some());
    if let Some(keys) = value.left_right_keys {
        writer.u16(keys);
    }
    writer.bool(value.up_down_keys.is_some());
    if let Some(keys) = value.up_down_keys {
        writer.u16(keys);
    }
    writer.u16(value.key_data);
    write_vector3(&mut writer, value.position);
    encode_normalized_quaternion(&mut writer, value.quaternion);
    writer.u8(encode_health_armour(value.health, value.armour));
    writer.u8(value.weapon);
    writer.u8(value.special_action);
    encode_compressed_vector(&mut writer, value.move_speed);
    writer.bool(value.surfing.is_some());
    if let Some(surfing) = value.surfing {
        writer.u16(surfing.vehicle_id);
        write_vector3(&mut writer, surfing.offsets);
    }
    writer.bool(value.animation.is_some());
    if let Some(animation) = value.animation {
        writer.u16(animation.id);
        writer.u16(animation.flags);
    }
    Ok(writer.finish_bits())
}

pub(super) fn decode_remote_vehicle_sync(
    event: &mut Event<'_>,
) -> Result<RemoteVehicleSync, EventError> {
    let player_id = event.read_u16()?;
    let vehicle_id = event.read_u16()?;
    let left_right_keys = event.read_u16()?;
    let up_down_keys = event.read_u16()?;
    let key_data = event.read_u16()?;
    let quaternion = decode_normalized_quaternion(event)?;
    let position = decode_vector3(event)?;
    let move_speed = decode_compressed_vector(event)?;
    let vehicle_health = event.read_u16()?;
    let (player_health, armour) = decode_health_armour(event.read_u8()?);
    let current_weapon = event.read_u8()?;
    let siren = read_bit_bool(event)?;
    let landing_gear = read_bit_bool(event)?;
    let train_speed = read_bit_bool(event)?
        .then(|| decode_i32(event))
        .transpose()?;
    let trailer_id = read_bit_bool(event)?
        .then(|| event.read_u16())
        .transpose()?;
    Ok(RemoteVehicleSync {
        player_id,
        vehicle_id,
        left_right_keys,
        up_down_keys,
        key_data,
        quaternion,
        position,
        move_speed,
        vehicle_health,
        player_health,
        armour,
        current_weapon,
        siren,
        landing_gear,
        train_speed,
        trailer_id,
    })
}

pub(super) fn encode_remote_vehicle_sync(
    _api: HostApi,
    value: RemoteVehicleSync,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u16(value.vehicle_id);
    writer.u16(value.left_right_keys);
    writer.u16(value.up_down_keys);
    writer.u16(value.key_data);
    encode_normalized_quaternion(&mut writer, value.quaternion);
    write_vector3(&mut writer, value.position);
    encode_compressed_vector(&mut writer, value.move_speed);
    writer.u16(value.vehicle_health);
    writer.u8(encode_health_armour(value.player_health, value.armour));
    writer.u8(value.current_weapon);
    writer.bool(value.siren);
    writer.bool(value.landing_gear);
    writer.bool(value.train_speed.is_some());
    if let Some(train_speed) = value.train_speed {
        writer.u32(train_speed as u32);
    }
    writer.bool(value.trailer_id.is_some());
    if let Some(trailer_id) = value.trailer_id {
        writer.u16(trailer_id);
    }
    Ok(writer.finish_bits())
}

pub(super) fn decode_markers_sync(event: &mut Event<'_>) -> Result<MarkersSync, EventError> {
    let count = decode_i32(event)?;
    let count = usize::try_from(count).map_err(|_| EventError::ValueOutOfRange {
        value: 0,
        maximum: MAX_MARKERS,
    })?;
    if count > MAX_MARKERS {
        return Err(EventError::LengthExceedsLimit {
            length: count,
            limit: MAX_MARKERS,
        });
    }
    let mut markers = Vec::with_capacity(count);
    for _ in 0..count {
        let player_id = event.read_u16()?;
        let coordinates = read_bit_bool(event)?
            .then(|| {
                Ok(MarkerCoordinates {
                    x: event.read_u16()? as i16,
                    y: event.read_u16()? as i16,
                    z: event.read_u16()? as i16,
                })
            })
            .transpose()?;
        markers.push(Marker {
            player_id,
            coordinates,
        });
    }
    consume_terminal_alignment_padding(event)?;
    Ok(MarkersSync { markers })
}

// R1 supplies `ID_MARKERS_SYNC` through a byte-backed `Packet`, even though each
// marker has a one-bit active flag. Its advertised packet bit size consequently
// includes up to seven terminal transport-padding bits after the final marker.
// The packet buffer does not promise their value, so consume that sub-byte suffix;
// a complete extra byte is semantic trailing data and remains malformed.
pub(super) fn consume_terminal_alignment_padding(event: &mut Event<'_>) -> Result<(), EventError> {
    let padding_bits = event.remaining_bits();
    if padding_bits == 0 {
        return Ok(());
    }
    if padding_bits >= u8::BITS as usize {
        return Err(EventError::UnexpectedBitLength {
            bit_len: padding_bits,
            expected: 0,
        });
    }
    let _ = event.read_bits(padding_bits)?;
    Ok(())
}

pub(super) fn encode_markers_sync(
    _api: HostApi,
    value: MarkersSync,
) -> Result<EncodedPayload, EventError> {
    if value.markers.len() > MAX_MARKERS {
        return Err(EventError::LengthExceedsLimit {
            length: value.markers.len(),
            limit: MAX_MARKERS,
        });
    }
    let mut writer = PayloadWriter::new();
    writer.u32(value.markers.len() as u32);
    for marker in value.markers {
        writer.u16(marker.player_id);
        writer.bool(marker.coordinates.is_some());
        if let Some(coordinates) = marker.coordinates {
            writer.u16(coordinates.x as u16);
            writer.u16(coordinates.y as u16);
            writer.u16(coordinates.z as u16);
        }
    }
    Ok(writer.finish_bits())
}

pub mod incoming;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compressed_float_codes_survive_decode_and_replacement_quantization() {
        for code in u16::MIN..=u16::MAX {
            let decoded = f32::from(code) / 32_767.5 - 1.0;
            assert_eq!(compressed_float_code(decoded), code, "code {code}");
        }
    }
}
