//! R1 exact-bit incoming Packet codecs.
//!
//! These codecs describe remote player, remote vehicle, and marker synchronization.
//! Local outgoing synchronization with the same packet IDs has different layouts and
//! remains in [`super::common`].

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, IncomingPacket, TrailingPolicy, WireCodec,
};

pub use super::common::Vector3;

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
    ($name:ident, $constant:ident, $id:literal, $codec:ty) => {
        pub type $name = IncomingPacket<$id, $codec>;
        pub const $constant: $name = IncomingPacket::new();
    };
}

descriptor!(
    RemotePlayerSyncPacket,
    PLAYER_SYNC,
    207,
    RemotePlayerSyncCodec
);
descriptor!(
    RemoteVehicleSyncPacket,
    VEHICLE_SYNC,
    200,
    RemoteVehicleSyncCodec
);
descriptor!(MarkersSyncPacket, MARKERS_SYNC, 208, MarkersSyncCodec);

impl WireCodec for RemotePlayerSyncCodec {
    type Value = RemotePlayerSync;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let player_id = read_u16(reader)?;
        let left_right_keys = read_bit_bool(reader)?
            .then(|| read_u16(reader))
            .transpose()?;
        let up_down_keys = read_bit_bool(reader)?
            .then(|| read_u16(reader))
            .transpose()?;
        let key_data = read_u16(reader)?;
        let position = read_vector3(reader)?;
        let quaternion = read_normalized_quaternion(reader)?;
        let (health, armour) = decode_health_armour(read_u8(reader)?);
        let weapon = read_u8(reader)?;
        let special_action = read_u8(reader)?;
        let move_speed = read_compressed_vector(reader)?;
        let surfing = read_bit_bool(reader)?
            .then(|| {
                Ok(RemotePlayerSurfing {
                    vehicle_id: read_u16(reader)?,
                    offsets: read_vector3(reader)?,
                })
            })
            .transpose()?;
        let animation = read_bit_bool(reader)?
            .then(|| {
                Ok(RemotePlayerAnimation {
                    id: read_u16(reader)?,
                    flags: read_u16(reader)?,
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
        write_u16(writer, value.player_id)?;
        write_option_u16(writer, value.left_right_keys)?;
        write_option_u16(writer, value.up_down_keys)?;
        write_u16(writer, value.key_data)?;
        write_vector3(writer, value.position)?;
        write_normalized_quaternion(writer, value.quaternion)?;
        write_u8(writer, encode_health_armour(value.health, value.armour))?;
        write_u8(writer, value.weapon)?;
        write_u8(writer, value.special_action)?;
        write_compressed_vector(writer, value.move_speed)?;
        write_bit_bool(writer, value.surfing.is_some())?;
        if let Some(surfing) = value.surfing {
            write_u16(writer, surfing.vehicle_id)?;
            write_vector3(writer, surfing.offsets)?;
        }
        write_bit_bool(writer, value.animation.is_some())?;
        if let Some(animation) = value.animation {
            write_u16(writer, animation.id)?;
            write_u16(writer, animation.flags)?;
        }
        Ok(())
    }
}

impl WireCodec for RemoteVehicleSyncCodec {
    type Value = RemoteVehicleSync;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let player_id = read_u16(reader)?;
        let vehicle_id = read_u16(reader)?;
        let left_right_keys = read_u16(reader)?;
        let up_down_keys = read_u16(reader)?;
        let key_data = read_u16(reader)?;
        let quaternion = read_normalized_quaternion(reader)?;
        let position = read_vector3(reader)?;
        let move_speed = read_compressed_vector(reader)?;
        let vehicle_health = read_u16(reader)?;
        let (player_health, armour) = decode_health_armour(read_u8(reader)?);
        let current_weapon = read_u8(reader)?;
        let siren = read_bit_bool(reader)?;
        let landing_gear = read_bit_bool(reader)?;
        let train_speed = read_bit_bool(reader)?
            .then(|| read_i32(reader))
            .transpose()?;
        let trailer_id = read_bit_bool(reader)?
            .then(|| read_u16(reader))
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
        write_u16(writer, value.player_id)?;
        write_u16(writer, value.vehicle_id)?;
        write_u16(writer, value.left_right_keys)?;
        write_u16(writer, value.up_down_keys)?;
        write_u16(writer, value.key_data)?;
        write_normalized_quaternion(writer, value.quaternion)?;
        write_vector3(writer, value.position)?;
        write_compressed_vector(writer, value.move_speed)?;
        write_u16(writer, value.vehicle_health)?;
        write_u8(
            writer,
            encode_health_armour(value.player_health, value.armour),
        )?;
        write_u8(writer, value.current_weapon)?;
        write_bit_bool(writer, value.siren)?;
        write_bit_bool(writer, value.landing_gear)?;
        write_bit_bool(writer, value.train_speed.is_some())?;
        if let Some(train_speed) = value.train_speed {
            write_i32(writer, train_speed)?;
        }
        write_bit_bool(writer, value.trailer_id.is_some())?;
        if let Some(trailer_id) = value.trailer_id {
            write_u16(writer, trailer_id)?;
        }
        Ok(())
    }
}

impl WireCodec for MarkersSyncCodec {
    type Value = MarkersSync;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::TerminalAlignmentPadding;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let count =
            usize::try_from(read_i32(reader)?).map_err(|_| DecodeError::LengthExceedsLimit {
                length: usize::MAX,
                limit: MAX_MARKERS,
            })?;
        if count > MAX_MARKERS {
            return Err(DecodeError::LengthExceedsLimit {
                length: count,
                limit: MAX_MARKERS,
            });
        }
        let mut markers = Vec::with_capacity(count);
        for _ in 0..count {
            let player_id = read_u16(reader)?;
            let coordinates = read_bit_bool(reader)?
                .then(|| {
                    Ok(MarkerCoordinates {
                        x: read_i16(reader)?,
                        y: read_i16(reader)?,
                        z: read_i16(reader)?,
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
        write_i32(writer, value.markers.len() as i32)?;
        for marker in &value.markers {
            write_u16(writer, marker.player_id)?;
            write_bit_bool(writer, marker.coordinates.is_some())?;
            if let Some(coordinates) = marker.coordinates {
                write_i16(writer, coordinates.x)?;
                write_i16(writer, coordinates.y)?;
                write_i16(writer, coordinates.z)?;
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
        write_u16(writer, value)?;
    }
    Ok(())
}

fn read_compressed_vector<R: BitRead>(reader: &mut R) -> Result<Vector3, DecodeError<R::Error>> {
    let magnitude = read_f32(reader)?;
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
    write_f32(writer, magnitude)?;
    if magnitude != 0.0 {
        write_compressed_float(writer, value.x / magnitude)?;
        write_compressed_float(writer, value.y / magnitude)?;
        write_compressed_float(writer, value.z / magnitude)?;
    }
    Ok(())
}

fn read_compressed_float<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    Ok(f32::from(read_u16(reader)?) / 32_767.5 - 1.0)
}

fn write_compressed_float<W: BitWrite>(
    writer: &mut W,
    value: f32,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(
        writer,
        ((value.clamp(-1.0, 1.0) + 1.0) * 32_767.5).round() as u16,
    )
}

fn read_normalized_quaternion<R: BitRead>(
    reader: &mut R,
) -> Result<[f32; 4], DecodeError<R::Error>> {
    let w_negative = read_bit_bool(reader)?;
    let x_negative = read_bit_bool(reader)?;
    let y_negative = read_bit_bool(reader)?;
    let z_negative = read_bit_bool(reader)?;
    let mut x = f32::from(read_u16(reader)?) / 65_535.0;
    let mut y = f32::from(read_u16(reader)?) / 65_535.0;
    let mut z = f32::from(read_u16(reader)?) / 65_535.0;
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
        write_u16(
            writer,
            (component.abs().clamp(0.0, 1.0) * 65_535.0).floor() as u16,
        )?;
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

fn read_u8<R: BitRead>(reader: &mut R) -> Result<u8, DecodeError<R::Error>> {
    Ok(read_fixed::<R, 1>(reader)?[0])
}

fn write_u8<W: BitWrite>(writer: &mut W, value: u8) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &[value])
}

fn read_i16<R: BitRead>(reader: &mut R) -> Result<i16, DecodeError<R::Error>> {
    Ok(i16::from_le_bytes(read_fixed(reader)?))
}

fn write_i16<W: BitWrite>(writer: &mut W, value: i16) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    Ok(u16::from_le_bytes(read_fixed(reader)?))
}

fn write_u16<W: BitWrite>(writer: &mut W, value: u16) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_i32<R: BitRead>(reader: &mut R) -> Result<i32, DecodeError<R::Error>> {
    Ok(i32::from_le_bytes(read_fixed(reader)?))
}

fn write_i32<W: BitWrite>(writer: &mut W, value: i32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_f32<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    Ok(f32::from_le_bytes(read_fixed(reader)?))
}

fn write_f32<W: BitWrite>(writer: &mut W, value: f32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_vector3<R: BitRead>(reader: &mut R) -> Result<Vector3, DecodeError<R::Error>> {
    Ok(Vector3 {
        x: read_f32(reader)?,
        y: read_f32(reader)?,
        z: read_f32(reader)?,
    })
}

fn write_vector3<W: BitWrite>(writer: &mut W, value: Vector3) -> Result<(), EncodeError<W::Error>> {
    write_f32(writer, value.x)?;
    write_f32(writer, value.y)?;
    write_f32(writer, value.z)
}

fn read_fixed<R: BitRead, const LENGTH: usize>(
    reader: &mut R,
) -> Result<[u8; LENGTH], DecodeError<R::Error>> {
    let bytes = read_bytes(reader, LENGTH)?;
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

fn read_bytes<R: BitRead>(reader: &mut R, length: usize) -> Result<Vec<u8>, DecodeError<R::Error>> {
    read_bits(reader, length * u8::BITS as usize)
}

fn write_bytes<W: BitWrite>(writer: &mut W, bytes: &[u8]) -> Result<(), EncodeError<W::Error>> {
    writer
        .write_left_aligned_bits(bytes, bytes.len() * u8::BITS as usize)
        .map_err(EncodeError::Source)
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
