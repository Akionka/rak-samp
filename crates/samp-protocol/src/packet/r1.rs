//! R1 exact-bit incoming Packet codecs.
//!
//! These codecs describe remote player, remote vehicle, and marker synchronization.
//! Local outgoing synchronization with the same packet IDs has different layouts and
//! remains in [`super::common`].

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBitsPolicy, IncomingPacket,
    TerminalAlignmentPaddingPolicy, WireCodec, WireReadExt, WireWriteExt,
};

use crate::types::Vector3;

/// R1 marker packets cannot contain more players than the Protocol player-slot limit.
pub const MAX_MARKERS: usize = 1_000;

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

pub struct RemotePlayerSyncCodec;
pub struct RemoteVehicleSyncCodec;
pub struct MarkersSyncCodec;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty, $policy:ty) => {
        pub type $name = IncomingPacket<$id, $codec, $policy>;
        pub const $constant: $name = IncomingPacket::new();
    };
}

descriptor!(
    RemotePlayerSyncPacket,
    PLAYER_SYNC,
    207,
    RemotePlayerSyncCodec,
    ExactBitsPolicy
);
descriptor!(
    RemoteVehicleSyncPacket,
    VEHICLE_SYNC,
    200,
    RemoteVehicleSyncCodec,
    ExactBitsPolicy
);
descriptor!(
    MarkersSyncPacket,
    MARKERS_SYNC,
    208,
    MarkersSyncCodec,
    TerminalAlignmentPaddingPolicy
);

impl WireCodec for RemotePlayerSyncCodec {
    type Value = RemotePlayerSync;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let player_id = reader.read_u16_le()?;
        let left_right_keys = read_bit_bool(reader)?
            .then(|| reader.read_u16_le())
            .transpose()?;
        let up_down_keys = read_bit_bool(reader)?
            .then(|| reader.read_u16_le())
            .transpose()?;
        let key_data = reader.read_u16_le()?;
        let position = reader.read_vector3_le()?;
        let quaternion = read_normalized_quaternion(reader)?;
        let (health, armour) = decode_health_armour(reader.read_u8()?);
        let weapon = reader.read_u8()?;
        let special_action = reader.read_u8()?;
        let move_speed = read_compressed_vector(reader)?;
        let surfing = read_bit_bool(reader)?
            .then(|| {
                Ok(RemotePlayerSurfing {
                    vehicle_id: reader.read_u16_le()?,
                    offsets: reader.read_vector3_le()?,
                })
            })
            .transpose()?;
        let animation = read_bit_bool(reader)?
            .then(|| {
                Ok(RemotePlayerAnimation {
                    id: reader.read_u16_le()?,
                    flags: reader.read_u16_le()?,
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

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.player_id)?;
        write_option_u16(writer, value.left_right_keys)?;
        write_option_u16(writer, value.up_down_keys)?;
        writer.write_u16_le(value.key_data)?;
        writer.write_vector3_le(&value.position)?;
        write_normalized_quaternion(writer, value.quaternion)?;
        writer.write_u8(encode_health_armour(value.health, value.armour))?;
        writer.write_u8(value.weapon)?;
        writer.write_u8(value.special_action)?;
        write_compressed_vector(writer, value.move_speed)?;
        write_bit_bool(writer, value.surfing.is_some())?;
        if let Some(surfing) = value.surfing {
            writer.write_u16_le(surfing.vehicle_id)?;
            writer.write_vector3_le(&surfing.offsets)?;
        }
        write_bit_bool(writer, value.animation.is_some())?;
        if let Some(animation) = value.animation {
            writer.write_u16_le(animation.id)?;
            writer.write_u16_le(animation.flags)?;
        }
        Ok(())
    }
}

impl WireCodec for RemoteVehicleSyncCodec {
    type Value = RemoteVehicleSync;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let player_id = reader.read_u16_le()?;
        let vehicle_id = reader.read_u16_le()?;
        let left_right_keys = reader.read_u16_le()?;
        let up_down_keys = reader.read_u16_le()?;
        let key_data = reader.read_u16_le()?;
        let quaternion = read_normalized_quaternion(reader)?;
        let position = reader.read_vector3_le()?;
        let move_speed = read_compressed_vector(reader)?;
        let vehicle_health = reader.read_u16_le()?;
        let (player_health, armour) = decode_health_armour(reader.read_u8()?);
        let current_weapon = reader.read_u8()?;
        let siren = read_bit_bool(reader)?;
        let landing_gear = read_bit_bool(reader)?;
        let train_speed = read_bit_bool(reader)?
            .then(|| reader.read_i32_le())
            .transpose()?;
        let trailer_id = read_bit_bool(reader)?
            .then(|| reader.read_u16_le())
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

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.player_id)?;
        writer.write_u16_le(value.vehicle_id)?;
        writer.write_u16_le(value.left_right_keys)?;
        writer.write_u16_le(value.up_down_keys)?;
        writer.write_u16_le(value.key_data)?;
        write_normalized_quaternion(writer, value.quaternion)?;
        writer.write_vector3_le(&value.position)?;
        write_compressed_vector(writer, value.move_speed)?;
        writer.write_u16_le(value.vehicle_health)?;
        writer.write_u8(encode_health_armour(value.player_health, value.armour))?;
        writer.write_u8(value.current_weapon)?;
        write_bit_bool(writer, value.siren)?;
        write_bit_bool(writer, value.landing_gear)?;
        write_bit_bool(writer, value.train_speed.is_some())?;
        if let Some(train_speed) = value.train_speed {
            writer.write_i32_le(train_speed)?;
        }
        write_bit_bool(writer, value.trailer_id.is_some())?;
        if let Some(trailer_id) = value.trailer_id {
            writer.write_u16_le(trailer_id)?;
        }
        Ok(())
    }
}

impl WireCodec for MarkersSyncCodec {
    type Value = MarkersSync;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let count = usize::try_from(reader.read_i32_le()?).map_err(|_| {
            DecodeError::LengthExceedsLimit {
                length: usize::MAX,
                limit: MAX_MARKERS,
            }
        })?;
        if count > MAX_MARKERS {
            return Err(DecodeError::LengthExceedsLimit {
                length: count,
                limit: MAX_MARKERS,
            });
        }
        let mut markers = Vec::with_capacity(count);
        for _ in 0..count {
            let player_id = reader.read_u16_le()?;
            let coordinates = read_bit_bool(reader)?
                .then(|| {
                    Ok(MarkerCoordinates {
                        x: WireReadExt::read_i16_le(reader)?,
                        y: WireReadExt::read_i16_le(reader)?,
                        z: WireReadExt::read_i16_le(reader)?,
                    })
                })
                .transpose()?;
            markers.push(Marker {
                player_id,
                coordinates,
            });
        }
        Ok(MarkersSync { markers })
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        if value.markers.len() > MAX_MARKERS {
            return Err(EncodeError::LengthExceedsLimit {
                length: value.markers.len(),
                limit: MAX_MARKERS,
            });
        }
        writer.write_i32_le(value.markers.len() as i32)?;
        for marker in &value.markers {
            writer.write_u16_le(marker.player_id)?;
            write_bit_bool(writer, marker.coordinates.is_some())?;
            if let Some(coordinates) = marker.coordinates {
                WireWriteExt::write_i16_le(writer, coordinates.x)?;
                WireWriteExt::write_i16_le(writer, coordinates.y)?;
                WireWriteExt::write_i16_le(writer, coordinates.z)?;
            }
        }
        Ok(())
    }
}

fn write_option_u16<W: BitWrite>(
    writer: &mut W,
    value: Option<u16>,
) -> Result<(), EncodeError<W::Error>> {
    write_bit_bool(writer, value.is_some())?;
    if let Some(value) = value {
        writer.write_u16_le(value)?;
    }
    Ok(())
}

fn read_compressed_vector<R: BitRead>(reader: &mut R) -> Result<Vector3, DecodeError<R::Error>> {
    let magnitude = reader.read_f32_le()?;
    if magnitude == 0.0 {
        return Ok(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
    }
    Ok(Vector3 {
        x: read_compressed_float(reader)? * magnitude,
        y: read_compressed_float(reader)? * magnitude,
        z: read_compressed_float(reader)? * magnitude,
    })
}

fn write_compressed_vector<W: BitWrite>(
    writer: &mut W,
    value: Vector3,
) -> Result<(), EncodeError<W::Error>> {
    let magnitude = (value.x.mul_add(value.x, value.y * value.y) + value.z * value.z).sqrt();
    writer.write_f32_le(magnitude)?;
    if magnitude != 0.0 {
        write_compressed_float(writer, value.x / magnitude)?;
        write_compressed_float(writer, value.y / magnitude)?;
        write_compressed_float(writer, value.z / magnitude)?;
    }
    Ok(())
}

fn read_compressed_float<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    Ok(f32::from(reader.read_u16_le()?) / 32_767.5 - 1.0)
}

fn write_compressed_float<W: BitWrite>(
    writer: &mut W,
    value: f32,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(((value.clamp(-1.0, 1.0) + 1.0) * 32_767.5).round() as u16)
}

fn read_normalized_quaternion<R: BitRead>(
    reader: &mut R,
) -> Result<[f32; 4], DecodeError<R::Error>> {
    let w_negative = read_bit_bool(reader)?;
    let x_negative = read_bit_bool(reader)?;
    let y_negative = read_bit_bool(reader)?;
    let z_negative = read_bit_bool(reader)?;
    let mut x = f32::from(reader.read_u16_le()?) / 65_535.0;
    let mut y = f32::from(reader.read_u16_le()?) / 65_535.0;
    let mut z = f32::from(reader.read_u16_le()?) / 65_535.0;
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

fn write_normalized_quaternion<W: BitWrite>(
    writer: &mut W,
    [w, x, y, z]: [f32; 4],
) -> Result<(), EncodeError<W::Error>> {
    write_bit_bool(writer, w < 0.0)?;
    write_bit_bool(writer, x < 0.0)?;
    write_bit_bool(writer, y < 0.0)?;
    write_bit_bool(writer, z < 0.0)?;
    for component in [x, y, z] {
        writer.write_u16_le((component.abs().clamp(0.0, 1.0) * 65_535.0).floor() as u16)?;
    }
    Ok(())
}

fn decode_health_armour(value: u8) -> (u8, u8) {
    (((value >> 4) * 7).min(100), ((value & 0x0F) * 7).min(100))
}

fn encode_health_armour(health: u8, armour: u8) -> u8 {
    let health = if health >= 100 {
        0xF0
    } else {
        (health / 7) << 4
    };
    let armour = if armour >= 100 { 0x0F } else { armour / 7 };
    health | armour
}

fn read_bit_bool<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(read_bits(reader, 1)?[0] & 0x80 != 0)
}

fn write_bit_bool<W: BitWrite>(writer: &mut W, value: bool) -> Result<(), EncodeError<W::Error>> {
    writer
        .write_left_aligned_bits(&[u8::from(value) << 7], 1)
        .map_err(EncodeError::Source)
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

#[cfg(test)]
mod tests {
    #[test]
    fn compressed_float_codes_survive_decode_and_replacement_quantization() {
        for code in u16::MIN..=u16::MAX {
            let decoded = f32::from(code) / 32_767.5 - 1.0;
            assert_eq!(
                ((decoded.clamp(-1.0, 1.0) + 1.0) * 32_767.5).round() as u16,
                code,
                "code {code}"
            );
        }
    }
}
