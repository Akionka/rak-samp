//! Common byte-aligned Packet codecs.
//!
//! This module owns the local outgoing packets and non-R1 incoming packets whose
//! layouts are shared across supported client profiles. R1 remote player, remote
//! vehicle, and marker synchronization use the exact-bit layouts in [`super::r1`].

use core::marker::PhantomData;

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, WireCodec, WireReadExt,
    WireWriteExt,
};

use crate::{limits::MAX_STRING32_BYTES, types::Vector3};

/// SA-MP sends at most 13 weapon slots in one weapons-update packet.
pub const MAX_WEAPON_SLOTS: usize = 13;

/// The byte-aligned `ID_STATS_UPDATE` payload (packet 205).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatsUpdate {
    pub money: i32,
    pub drunk_level: i32,
}

/// One four-byte entry in an `ID_WEAPONS_UPDATE` packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponSlot {
    pub slot: u8,
    pub weapon: u8,
    pub ammo: u16,
}

/// MoonLoader's `onSendWeaponsUpdate` payload (packet 204).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponsUpdate {
    pub player_target: u16,
    pub actor_target: u16,
    pub weapons: Vec<WeaponSlot>,
}

/// MoonLoader's `onConnectionRequestAccepted` payload (packet 34).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectionAccepted {
    pub ip: i32,
    pub port: u16,
    pub player_id: u16,
    pub challenge: i32,
}

/// The fixed 68-byte local `ID_PLAYER_SYNC` payload (packet 207).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSync {
    pub left_right_keys: u16,
    pub up_down_keys: u16,
    /// Protocol-defined key bits, preserved as received.
    pub key_data: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub health: u8,
    pub armour: u8,
    /// Protocol-defined weapon and special-key bits, preserved as received.
    pub weapon_and_special_key: u8,
    pub special_action: u8,
    pub move_speed: Vector3,
    pub surfing_offsets: Vector3,
    pub surfing_vehicle_id: u16,
    pub animation_id: u16,
    pub animation_flags: u16,
}

/// The fixed 63-byte local `ID_VEHICLE_SYNC` payload (packet 200).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleSync {
    pub vehicle_id: u16,
    pub left_right_keys: u16,
    pub up_down_keys: u16,
    /// Protocol-defined key bits, preserved as received.
    pub key_data: u16,
    pub quaternion: [f32; 4],
    pub position: Vector3,
    pub move_speed: Vector3,
    pub vehicle_health: f32,
    pub player_health: u8,
    pub armour: u8,
    /// Protocol-defined weapon and special-key bits, preserved as received.
    pub weapon_and_special_key: u8,
    pub siren: u8,
    pub landing_gear_state: u8,
    pub trailer_id: u16,
    /// Four protocol bytes used as bike lean, train speed, or Hydra thrust data.
    pub vehicle_specific: [u8; 4],
}

/// The fixed 24-byte `ID_PASSENGER_SYNC` payload (packet 211), excluding a remote player ID.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PassengerSync {
    pub vehicle_id: u16,
    /// Protocol-defined seat, drive-by, and cuffed bits, preserved as received.
    pub seat_driveby_cuffed: u8,
    /// Protocol-defined weapon and special-key bits, preserved as received.
    pub weapon_and_special_key: u8,
    pub health: u8,
    pub armour: u8,
    pub left_right_keys: u16,
    pub up_down_keys: u16,
    pub key_data: u16,
    pub position: Vector3,
}

/// The fixed 31-byte `ID_AIM_SYNC` payload (packet 203), excluding a remote player ID.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimSync {
    pub camera_mode: u8,
    pub camera_front: Vector3,
    pub camera_position: Vector3,
    pub aim_z: f32,
    /// Protocol-defined camera-zoom and weapon-state bits, preserved as received.
    pub zoom_and_weapon_state: u8,
    pub aspect_ratio: u8,
}

/// The fixed 67-byte `ID_UNOCCUPIED_SYNC` payload (packet 209), excluding a remote player ID.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnoccupiedSync {
    pub vehicle_id: u16,
    pub seat_id: u8,
    pub roll: Vector3,
    pub direction: Vector3,
    pub position: Vector3,
    pub move_speed: Vector3,
    pub turn_speed: Vector3,
    pub vehicle_health: f32,
}

/// The fixed 54-byte `ID_TRAILER_SYNC` payload (packet 210), excluding a remote player ID.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailerSync {
    pub trailer_id: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub move_speed: Vector3,
    pub turn_speed: Vector3,
}

/// The fixed 40-byte `ID_BULLET_SYNC` payload (packet 206), excluding a remote player ID.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BulletSync {
    pub target_type: u8,
    pub target_id: u16,
    pub origin: Vector3,
    pub target: Vector3,
    pub center: Vector3,
    pub weapon_id: u8,
}

/// The fixed 18-byte local `ID_SPECTATOR_SYNC` payload (packet 212).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpectatorSync {
    pub left_right_keys: u16,
    pub up_down_keys: u16,
    pub key_data: u16,
    pub position: Vector3,
}

/// Synchronization data received from a specific remote player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteSync<T> {
    pub player_id: u16,
    pub data: T,
}

struct Empty;
struct String8;
struct String32;
struct StatsUpdateCodec;
struct WeaponsUpdateCodec;
struct ConnectionAcceptedCodec;
struct PlayerSyncCodec;
struct VehicleSyncCodec;
struct PassengerSyncCodec;
struct AimSyncCodec;
struct UnoccupiedSyncCodec;
struct TrailerSyncCodec;
struct BulletSyncCodec;
struct SpectatorSyncCodec;
struct RemoteCodec<C>(PhantomData<C>);

macro_rules! descriptor_value {
    (Empty) => { () };
    (String8) => { Vec<u8> };
    (String32) => { Vec<u8> };
    (StatsUpdateCodec) => { StatsUpdate };
    (WeaponsUpdateCodec) => { WeaponsUpdate };
    (ConnectionAcceptedCodec) => { ConnectionAccepted };
    (PlayerSyncCodec) => { PlayerSync };
    (VehicleSyncCodec) => { VehicleSync };
    (PassengerSyncCodec) => { PassengerSync };
    (AimSyncCodec) => { AimSync };
    (UnoccupiedSyncCodec) => { UnoccupiedSync };
    (TrailerSyncCodec) => { TrailerSync };
    (BulletSyncCodec) => { BulletSync };
    (SpectatorSyncCodec) => { SpectatorSync };
    (RemoteCodec<AimSyncCodec>) => { RemoteSync<AimSync> };
    (RemoteCodec<BulletSyncCodec>) => { RemoteSync<BulletSync> };
    (RemoteCodec<UnoccupiedSyncCodec>) => { RemoteSync<UnoccupiedSync> };
    (RemoteCodec<TrailerSyncCodec>) => { RemoteSync<TrailerSync> };
    (RemoteCodec<PassengerSyncCodec>) => { RemoteSync<PassengerSync> };
}

macro_rules! descriptor {
    (IncomingPacket, $name:ident, $constant:ident, $id:literal, $($codec:tt)+) => {
        crate::wire::nominal_descriptor!(
            incoming packet,
            $name,
            $constant,
            $id,
            $($codec)+,
            descriptor_value!($($codec)+),
            ExactBytesPolicy
        );
    };
    (OutgoingPacket, $name:ident, $constant:ident, $id:literal, $($codec:tt)+) => {
        crate::wire::nominal_descriptor!(
            outgoing packet,
            $name,
            $constant,
            $id,
            $($codec)+,
            descriptor_value!($($codec)+),
            ExactBytesPolicy
        );
    };
}

descriptor!(
    OutgoingPacket,
    SendRconCommand,
    SEND_RCON_COMMAND,
    201,
    String32
);
descriptor!(
    OutgoingPacket,
    SendAuthenticationResponse,
    SEND_AUTHENTICATION_RESPONSE,
    12,
    String8
);
descriptor!(
    OutgoingPacket,
    SendStatsUpdate,
    SEND_STATS_UPDATE,
    205,
    StatsUpdateCodec
);
descriptor!(
    OutgoingPacket,
    SendWeaponsUpdate,
    SEND_WEAPONS_UPDATE,
    204,
    WeaponsUpdateCodec
);
descriptor!(
    OutgoingPacket,
    SendPlayerSync,
    SEND_PLAYER_SYNC,
    207,
    PlayerSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendVehicleSync,
    SEND_VEHICLE_SYNC,
    200,
    VehicleSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendPassengerSync,
    SEND_PASSENGER_SYNC,
    211,
    PassengerSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendAimSync,
    SEND_AIM_SYNC,
    203,
    AimSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendUnoccupiedSync,
    SEND_UNOCCUPIED_SYNC,
    209,
    UnoccupiedSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendTrailerSync,
    SEND_TRAILER_SYNC,
    210,
    TrailerSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendBulletSync,
    SEND_BULLET_SYNC,
    206,
    BulletSyncCodec
);
descriptor!(
    OutgoingPacket,
    SendSpectatorSync,
    SEND_SPECTATOR_SYNC,
    212,
    SpectatorSyncCodec
);

descriptor!(
    IncomingPacket,
    AuthenticationRequest,
    AUTHENTICATION_REQUEST,
    12,
    String8
);
descriptor!(
    IncomingPacket,
    ConnectionAcceptedPacket,
    CONNECTION_ACCEPTED,
    34,
    ConnectionAcceptedCodec
);
descriptor!(IncomingPacket, ConnectionLost, CONNECTION_LOST, 33, Empty);
descriptor!(
    IncomingPacket,
    ConnectionBanned,
    CONNECTION_BANNED,
    36,
    Empty
);
descriptor!(
    IncomingPacket,
    ConnectionAttemptFailed,
    CONNECTION_ATTEMPT_FAILED,
    29,
    Empty
);
descriptor!(
    IncomingPacket,
    ConnectionNoFreeSlot,
    CONNECTION_NO_FREE_SLOT,
    31,
    Empty
);
descriptor!(
    IncomingPacket,
    ConnectionPasswordInvalid,
    CONNECTION_PASSWORD_INVALID,
    37,
    Empty
);
descriptor!(
    IncomingPacket,
    ConnectionClosed,
    CONNECTION_CLOSED,
    32,
    Empty
);
descriptor!(
    IncomingPacket,
    RemoteAimSync,
    AIM_SYNC,
    203,
    RemoteCodec<AimSyncCodec>
);
descriptor!(
    IncomingPacket,
    RemoteBulletSync,
    BULLET_SYNC,
    206,
    RemoteCodec<BulletSyncCodec>
);
descriptor!(
    IncomingPacket,
    RemoteUnoccupiedSync,
    UNOCCUPIED_SYNC,
    209,
    RemoteCodec<UnoccupiedSyncCodec>
);
descriptor!(
    IncomingPacket,
    RemoteTrailerSync,
    TRAILER_SYNC,
    210,
    RemoteCodec<TrailerSyncCodec>
);
descriptor!(
    IncomingPacket,
    RemotePassengerSync,
    PASSENGER_SYNC,
    211,
    RemoteCodec<PassengerSyncCodec>
);

macro_rules! byte_aligned_codec {
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

macro_rules! byte_aligned_bytes_codec {
    ($codec:ident, $read:ident, $write:ident, $max_len:expr) => {
        impl WireCodec for $codec {
            type Value = Vec<u8>;
            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                reader.$read($max_len)
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                writer.$write(value, $max_len)
            }
        }
    };
}

byte_aligned_codec!(Empty, (), read_empty, write_empty);
byte_aligned_bytes_codec!(
    String8,
    read_len_prefixed_bytes_u8,
    write_len_prefixed_bytes_u8,
    usize::from(u8::MAX)
);
byte_aligned_bytes_codec!(
    String32,
    read_len_prefixed_bytes_u32_le,
    write_len_prefixed_bytes_u32_le,
    MAX_STRING32_BYTES
);
byte_aligned_codec!(
    StatsUpdateCodec,
    StatsUpdate,
    read_stats_update,
    write_stats_update
);
byte_aligned_codec!(
    WeaponsUpdateCodec,
    WeaponsUpdate,
    read_weapons_update,
    write_weapons_update
);
byte_aligned_codec!(
    ConnectionAcceptedCodec,
    ConnectionAccepted,
    read_connection_accepted,
    write_connection_accepted
);
byte_aligned_codec!(
    PlayerSyncCodec,
    PlayerSync,
    read_player_sync,
    write_player_sync
);
byte_aligned_codec!(
    VehicleSyncCodec,
    VehicleSync,
    read_vehicle_sync,
    write_vehicle_sync
);
byte_aligned_codec!(
    PassengerSyncCodec,
    PassengerSync,
    read_passenger_sync,
    write_passenger_sync
);
byte_aligned_codec!(AimSyncCodec, AimSync, read_aim_sync, write_aim_sync);
byte_aligned_codec!(
    UnoccupiedSyncCodec,
    UnoccupiedSync,
    read_unoccupied_sync,
    write_unoccupied_sync
);
byte_aligned_codec!(
    TrailerSyncCodec,
    TrailerSync,
    read_trailer_sync,
    write_trailer_sync
);
byte_aligned_codec!(
    BulletSyncCodec,
    BulletSync,
    read_bullet_sync,
    write_bullet_sync
);
byte_aligned_codec!(
    SpectatorSyncCodec,
    SpectatorSync,
    read_spectator_sync,
    write_spectator_sync
);

impl<C> WireCodec for RemoteCodec<C>
where
    C: WireCodec,
{
    type Value = RemoteSync<C::Value>;
    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        Ok(RemoteSync {
            player_id: reader.read_u16_le()?,
            data: C::decode(reader)?,
        })
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.player_id)?;
        C::encode(writer, &value.data)
    }
}

fn read_empty<R: BitRead>(_reader: &mut R) -> Result<(), DecodeError<R::Error>> {
    Ok(())
}

fn write_empty<W: BitWrite>(_writer: &mut W, _value: &()) -> Result<(), EncodeError<W::Error>> {
    Ok(())
}

fn read_stats_update<R: BitRead>(reader: &mut R) -> Result<StatsUpdate, DecodeError<R::Error>> {
    Ok(StatsUpdate {
        money: reader.read_i32_le()?,
        drunk_level: reader.read_i32_le()?,
    })
}

fn write_stats_update<W: BitWrite>(
    writer: &mut W,
    value: &StatsUpdate,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.money)?;
    writer.write_i32_le(value.drunk_level)
}

fn read_weapons_update<R: BitRead>(reader: &mut R) -> Result<WeaponsUpdate, DecodeError<R::Error>> {
    let remaining_bits = reader.remaining_bits();
    if remaining_bits < 32 || !(remaining_bits - 32).is_multiple_of(32) {
        return Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits,
            allowed_bits: 0,
        });
    }
    let weapon_count = (remaining_bits - 32) / 32;
    if weapon_count > MAX_WEAPON_SLOTS {
        return Err(DecodeError::LengthExceedsLimit {
            length: weapon_count,
            limit: MAX_WEAPON_SLOTS,
        });
    }

    let player_target = reader.read_u16_le()?;
    let actor_target = reader.read_u16_le()?;
    let mut weapons = Vec::with_capacity(weapon_count);
    for _ in 0..weapon_count {
        weapons.push(WeaponSlot {
            slot: reader.read_u8()?,
            weapon: reader.read_u8()?,
            ammo: reader.read_u16_le()?,
        });
    }
    Ok(WeaponsUpdate {
        player_target,
        actor_target,
        weapons,
    })
}

fn write_weapons_update<W: BitWrite>(
    writer: &mut W,
    value: &WeaponsUpdate,
) -> Result<(), EncodeError<W::Error>> {
    if value.weapons.len() > MAX_WEAPON_SLOTS {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.weapons.len(),
            limit: MAX_WEAPON_SLOTS,
        });
    }

    writer.write_u16_le(value.player_target)?;
    writer.write_u16_le(value.actor_target)?;
    for weapon in &value.weapons {
        writer.write_u8(weapon.slot)?;
        writer.write_u8(weapon.weapon)?;
        writer.write_u16_le(weapon.ammo)?;
    }
    Ok(())
}

fn read_connection_accepted<R: BitRead>(
    reader: &mut R,
) -> Result<ConnectionAccepted, DecodeError<R::Error>> {
    Ok(ConnectionAccepted {
        ip: reader.read_i32_le()?,
        port: reader.read_u16_le()?,
        player_id: reader.read_u16_le()?,
        challenge: reader.read_i32_le()?,
    })
}

fn write_connection_accepted<W: BitWrite>(
    writer: &mut W,
    value: &ConnectionAccepted,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.ip)?;
    writer.write_u16_le(value.port)?;
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.challenge)
}

fn read_player_sync<R: BitRead>(reader: &mut R) -> Result<PlayerSync, DecodeError<R::Error>> {
    Ok(PlayerSync {
        left_right_keys: reader.read_u16_le()?,
        up_down_keys: reader.read_u16_le()?,
        key_data: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
        quaternion: read_quaternion(reader)?,
        health: reader.read_u8()?,
        armour: reader.read_u8()?,
        weapon_and_special_key: reader.read_u8()?,
        special_action: reader.read_u8()?,
        move_speed: reader.read_vector3_le()?,
        surfing_offsets: reader.read_vector3_le()?,
        surfing_vehicle_id: reader.read_u16_le()?,
        animation_id: reader.read_u16_le()?,
        animation_flags: reader.read_u16_le()?,
    })
}

fn write_player_sync<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.left_right_keys)?;
    writer.write_u16_le(value.up_down_keys)?;
    writer.write_u16_le(value.key_data)?;
    writer.write_vector3_le(&value.position)?;
    write_quaternion(writer, &value.quaternion)?;
    writer.write_u8(value.health)?;
    writer.write_u8(value.armour)?;
    writer.write_u8(value.weapon_and_special_key)?;
    writer.write_u8(value.special_action)?;
    writer.write_vector3_le(&value.move_speed)?;
    writer.write_vector3_le(&value.surfing_offsets)?;
    writer.write_u16_le(value.surfing_vehicle_id)?;
    writer.write_u16_le(value.animation_id)?;
    writer.write_u16_le(value.animation_flags)
}

fn read_vehicle_sync<R: BitRead>(reader: &mut R) -> Result<VehicleSync, DecodeError<R::Error>> {
    Ok(VehicleSync {
        vehicle_id: reader.read_u16_le()?,
        left_right_keys: reader.read_u16_le()?,
        up_down_keys: reader.read_u16_le()?,
        key_data: reader.read_u16_le()?,
        quaternion: read_quaternion(reader)?,
        position: reader.read_vector3_le()?,
        move_speed: reader.read_vector3_le()?,
        vehicle_health: reader.read_f32_le()?,
        player_health: reader.read_u8()?,
        armour: reader.read_u8()?,
        weapon_and_special_key: reader.read_u8()?,
        siren: reader.read_u8()?,
        landing_gear_state: reader.read_u8()?,
        trailer_id: reader.read_u16_le()?,
        vehicle_specific: read_fixed(reader)?,
    })
}

fn write_vehicle_sync<W: BitWrite>(
    writer: &mut W,
    value: &VehicleSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u16_le(value.left_right_keys)?;
    writer.write_u16_le(value.up_down_keys)?;
    writer.write_u16_le(value.key_data)?;
    write_quaternion(writer, &value.quaternion)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_vector3_le(&value.move_speed)?;
    writer.write_f32_le(value.vehicle_health)?;
    writer.write_u8(value.player_health)?;
    writer.write_u8(value.armour)?;
    writer.write_u8(value.weapon_and_special_key)?;
    writer.write_u8(value.siren)?;
    writer.write_u8(value.landing_gear_state)?;
    writer.write_u16_le(value.trailer_id)?;
    writer.write_bytes(&value.vehicle_specific)
}

fn read_passenger_sync<R: BitRead>(reader: &mut R) -> Result<PassengerSync, DecodeError<R::Error>> {
    Ok(PassengerSync {
        vehicle_id: reader.read_u16_le()?,
        seat_driveby_cuffed: reader.read_u8()?,
        weapon_and_special_key: reader.read_u8()?,
        health: reader.read_u8()?,
        armour: reader.read_u8()?,
        left_right_keys: reader.read_u16_le()?,
        up_down_keys: reader.read_u16_le()?,
        key_data: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_passenger_sync<W: BitWrite>(
    writer: &mut W,
    value: &PassengerSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(value.seat_driveby_cuffed)?;
    writer.write_u8(value.weapon_and_special_key)?;
    writer.write_u8(value.health)?;
    writer.write_u8(value.armour)?;
    writer.write_u16_le(value.left_right_keys)?;
    writer.write_u16_le(value.up_down_keys)?;
    writer.write_u16_le(value.key_data)?;
    writer.write_vector3_le(&value.position)
}

fn read_aim_sync<R: BitRead>(reader: &mut R) -> Result<AimSync, DecodeError<R::Error>> {
    Ok(AimSync {
        camera_mode: reader.read_u8()?,
        camera_front: reader.read_vector3_le()?,
        camera_position: reader.read_vector3_le()?,
        aim_z: reader.read_f32_le()?,
        zoom_and_weapon_state: reader.read_u8()?,
        aspect_ratio: reader.read_u8()?,
    })
}

fn write_aim_sync<W: BitWrite>(
    writer: &mut W,
    value: &AimSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.camera_mode)?;
    writer.write_vector3_le(&value.camera_front)?;
    writer.write_vector3_le(&value.camera_position)?;
    writer.write_f32_le(value.aim_z)?;
    writer.write_u8(value.zoom_and_weapon_state)?;
    writer.write_u8(value.aspect_ratio)
}

fn read_unoccupied_sync<R: BitRead>(
    reader: &mut R,
) -> Result<UnoccupiedSync, DecodeError<R::Error>> {
    Ok(UnoccupiedSync {
        vehicle_id: reader.read_u16_le()?,
        seat_id: reader.read_u8()?,
        roll: reader.read_vector3_le()?,
        direction: reader.read_vector3_le()?,
        position: reader.read_vector3_le()?,
        move_speed: reader.read_vector3_le()?,
        turn_speed: reader.read_vector3_le()?,
        vehicle_health: reader.read_f32_le()?,
    })
}

fn write_unoccupied_sync<W: BitWrite>(
    writer: &mut W,
    value: &UnoccupiedSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(value.seat_id)?;
    writer.write_vector3_le(&value.roll)?;
    writer.write_vector3_le(&value.direction)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_vector3_le(&value.move_speed)?;
    writer.write_vector3_le(&value.turn_speed)?;
    writer.write_f32_le(value.vehicle_health)
}

fn read_trailer_sync<R: BitRead>(reader: &mut R) -> Result<TrailerSync, DecodeError<R::Error>> {
    Ok(TrailerSync {
        trailer_id: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
        quaternion: read_quaternion(reader)?,
        move_speed: reader.read_vector3_le()?,
        turn_speed: reader.read_vector3_le()?,
    })
}

fn write_trailer_sync<W: BitWrite>(
    writer: &mut W,
    value: &TrailerSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.trailer_id)?;
    writer.write_vector3_le(&value.position)?;
    write_quaternion(writer, &value.quaternion)?;
    writer.write_vector3_le(&value.move_speed)?;
    writer.write_vector3_le(&value.turn_speed)
}

fn read_bullet_sync<R: BitRead>(reader: &mut R) -> Result<BulletSync, DecodeError<R::Error>> {
    Ok(BulletSync {
        target_type: reader.read_u8()?,
        target_id: reader.read_u16_le()?,
        origin: reader.read_vector3_le()?,
        target: reader.read_vector3_le()?,
        center: reader.read_vector3_le()?,
        weapon_id: reader.read_u8()?,
    })
}

fn write_bullet_sync<W: BitWrite>(
    writer: &mut W,
    value: &BulletSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.target_type)?;
    writer.write_u16_le(value.target_id)?;
    writer.write_vector3_le(&value.origin)?;
    writer.write_vector3_le(&value.target)?;
    writer.write_vector3_le(&value.center)?;
    writer.write_u8(value.weapon_id)
}

fn read_spectator_sync<R: BitRead>(reader: &mut R) -> Result<SpectatorSync, DecodeError<R::Error>> {
    Ok(SpectatorSync {
        left_right_keys: reader.read_u16_le()?,
        up_down_keys: reader.read_u16_le()?,
        key_data: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_spectator_sync<W: BitWrite>(
    writer: &mut W,
    value: &SpectatorSync,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.left_right_keys)?;
    writer.write_u16_le(value.up_down_keys)?;
    writer.write_u16_le(value.key_data)?;
    writer.write_vector3_le(&value.position)
}

fn read_quaternion<R: BitRead>(reader: &mut R) -> Result<[f32; 4], DecodeError<R::Error>> {
    Ok([
        reader.read_f32_le()?,
        reader.read_f32_le()?,
        reader.read_f32_le()?,
        reader.read_f32_le()?,
    ])
}

fn write_quaternion<W: BitWrite>(
    writer: &mut W,
    value: &[f32; 4],
) -> Result<(), EncodeError<W::Error>> {
    for component in value {
        writer.write_f32_le(*component)?;
    }
    Ok(())
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
