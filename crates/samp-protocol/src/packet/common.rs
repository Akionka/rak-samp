//! Common byte-aligned Packet codecs.
//!
//! This module owns the local outgoing packets and non-R1 incoming packets whose
//! layouts are shared across supported client profiles. R1 remote player, remote
//! vehicle, and marker synchronization use the exact-bit layouts in [`super::r1`].

use core::marker::PhantomData;

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, IncomingPacket, OutgoingPacket, TrailingPolicy,
    WireCodec, WireReadExt, WireWriteExt,
};

pub use crate::rpc::incoming::{MAX_STRING32_BYTES, Vector3};

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

pub struct Empty;
pub struct String8;
pub struct String32;
pub struct StatsUpdateCodec;
pub struct WeaponsUpdateCodec;
pub struct ConnectionAcceptedCodec;
pub struct PlayerSyncCodec;
pub struct VehicleSyncCodec;
pub struct PassengerSyncCodec;
pub struct AimSyncCodec;
pub struct UnoccupiedSyncCodec;
pub struct TrailerSyncCodec;
pub struct BulletSyncCodec;
pub struct SpectatorSyncCodec;
pub struct RemoteCodec<C>(PhantomData<C>);

macro_rules! descriptor {
    ($direction:ident, $name:ident, $constant:ident, $id:literal, $codec:ty) => {
        pub type $name = $direction<$id, $codec>;
        pub const $constant: $name = $direction::new();
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
            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

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

byte_aligned_codec!(Empty, (), read_empty, write_empty);
byte_aligned_codec!(String8, Vec<u8>, read_string8, write_string8);
byte_aligned_codec!(String32, Vec<u8>, read_string32, write_string32);
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
    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        Ok(RemoteSync {
            player_id: read_u16(reader)?,
            data: C::decode(reader)?,
        })
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        write_u16(writer, &value.player_id)?;
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
        money: read_i32(reader)?,
        drunk_level: read_i32(reader)?,
    })
}

fn write_stats_update<W: BitWrite>(
    writer: &mut W,
    value: &StatsUpdate,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.money)?;
    write_i32(writer, &value.drunk_level)
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

    let player_target = read_u16(reader)?;
    let actor_target = read_u16(reader)?;
    let mut weapons = Vec::with_capacity(weapon_count);
    for _ in 0..weapon_count {
        weapons.push(WeaponSlot {
            slot: read_u8(reader)?,
            weapon: read_u8(reader)?,
            ammo: read_u16(reader)?,
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

    write_u16(writer, &value.player_target)?;
    write_u16(writer, &value.actor_target)?;
    for weapon in &value.weapons {
        write_u8(writer, &weapon.slot)?;
        write_u8(writer, &weapon.weapon)?;
        write_u16(writer, &weapon.ammo)?;
    }
    Ok(())
}

fn read_connection_accepted<R: BitRead>(
    reader: &mut R,
) -> Result<ConnectionAccepted, DecodeError<R::Error>> {
    Ok(ConnectionAccepted {
        ip: read_i32(reader)?,
        port: read_u16(reader)?,
        player_id: read_u16(reader)?,
        challenge: read_i32(reader)?,
    })
}

fn write_connection_accepted<W: BitWrite>(
    writer: &mut W,
    value: &ConnectionAccepted,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.ip)?;
    write_u16(writer, &value.port)?;
    write_u16(writer, &value.player_id)?;
    write_i32(writer, &value.challenge)
}

fn read_player_sync<R: BitRead>(reader: &mut R) -> Result<PlayerSync, DecodeError<R::Error>> {
    Ok(PlayerSync {
        left_right_keys: read_u16(reader)?,
        up_down_keys: read_u16(reader)?,
        key_data: read_u16(reader)?,
        position: read_vector3(reader)?,
        quaternion: read_quaternion(reader)?,
        health: read_u8(reader)?,
        armour: read_u8(reader)?,
        weapon_and_special_key: read_u8(reader)?,
        special_action: read_u8(reader)?,
        move_speed: read_vector3(reader)?,
        surfing_offsets: read_vector3(reader)?,
        surfing_vehicle_id: read_u16(reader)?,
        animation_id: read_u16(reader)?,
        animation_flags: read_u16(reader)?,
    })
}

fn write_player_sync<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.left_right_keys)?;
    write_u16(writer, &value.up_down_keys)?;
    write_u16(writer, &value.key_data)?;
    write_vector3(writer, &value.position)?;
    write_quaternion(writer, &value.quaternion)?;
    write_u8(writer, &value.health)?;
    write_u8(writer, &value.armour)?;
    write_u8(writer, &value.weapon_and_special_key)?;
    write_u8(writer, &value.special_action)?;
    write_vector3(writer, &value.move_speed)?;
    write_vector3(writer, &value.surfing_offsets)?;
    write_u16(writer, &value.surfing_vehicle_id)?;
    write_u16(writer, &value.animation_id)?;
    write_u16(writer, &value.animation_flags)
}

fn read_vehicle_sync<R: BitRead>(reader: &mut R) -> Result<VehicleSync, DecodeError<R::Error>> {
    Ok(VehicleSync {
        vehicle_id: read_u16(reader)?,
        left_right_keys: read_u16(reader)?,
        up_down_keys: read_u16(reader)?,
        key_data: read_u16(reader)?,
        quaternion: read_quaternion(reader)?,
        position: read_vector3(reader)?,
        move_speed: read_vector3(reader)?,
        vehicle_health: read_f32(reader)?,
        player_health: read_u8(reader)?,
        armour: read_u8(reader)?,
        weapon_and_special_key: read_u8(reader)?,
        siren: read_u8(reader)?,
        landing_gear_state: read_u8(reader)?,
        trailer_id: read_u16(reader)?,
        vehicle_specific: read_fixed(reader)?,
    })
}

fn write_vehicle_sync<W: BitWrite>(
    writer: &mut W,
    value: &VehicleSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.vehicle_id)?;
    write_u16(writer, &value.left_right_keys)?;
    write_u16(writer, &value.up_down_keys)?;
    write_u16(writer, &value.key_data)?;
    write_quaternion(writer, &value.quaternion)?;
    write_vector3(writer, &value.position)?;
    write_vector3(writer, &value.move_speed)?;
    write_f32(writer, &value.vehicle_health)?;
    write_u8(writer, &value.player_health)?;
    write_u8(writer, &value.armour)?;
    write_u8(writer, &value.weapon_and_special_key)?;
    write_u8(writer, &value.siren)?;
    write_u8(writer, &value.landing_gear_state)?;
    write_u16(writer, &value.trailer_id)?;
    write_bytes(writer, &value.vehicle_specific)
}

fn read_passenger_sync<R: BitRead>(reader: &mut R) -> Result<PassengerSync, DecodeError<R::Error>> {
    Ok(PassengerSync {
        vehicle_id: read_u16(reader)?,
        seat_driveby_cuffed: read_u8(reader)?,
        weapon_and_special_key: read_u8(reader)?,
        health: read_u8(reader)?,
        armour: read_u8(reader)?,
        left_right_keys: read_u16(reader)?,
        up_down_keys: read_u16(reader)?,
        key_data: read_u16(reader)?,
        position: read_vector3(reader)?,
    })
}

fn write_passenger_sync<W: BitWrite>(
    writer: &mut W,
    value: &PassengerSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.vehicle_id)?;
    write_u8(writer, &value.seat_driveby_cuffed)?;
    write_u8(writer, &value.weapon_and_special_key)?;
    write_u8(writer, &value.health)?;
    write_u8(writer, &value.armour)?;
    write_u16(writer, &value.left_right_keys)?;
    write_u16(writer, &value.up_down_keys)?;
    write_u16(writer, &value.key_data)?;
    write_vector3(writer, &value.position)
}

fn read_aim_sync<R: BitRead>(reader: &mut R) -> Result<AimSync, DecodeError<R::Error>> {
    Ok(AimSync {
        camera_mode: read_u8(reader)?,
        camera_front: read_vector3(reader)?,
        camera_position: read_vector3(reader)?,
        aim_z: read_f32(reader)?,
        zoom_and_weapon_state: read_u8(reader)?,
        aspect_ratio: read_u8(reader)?,
    })
}

fn write_aim_sync<W: BitWrite>(
    writer: &mut W,
    value: &AimSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, &value.camera_mode)?;
    write_vector3(writer, &value.camera_front)?;
    write_vector3(writer, &value.camera_position)?;
    write_f32(writer, &value.aim_z)?;
    write_u8(writer, &value.zoom_and_weapon_state)?;
    write_u8(writer, &value.aspect_ratio)
}

fn read_unoccupied_sync<R: BitRead>(
    reader: &mut R,
) -> Result<UnoccupiedSync, DecodeError<R::Error>> {
    Ok(UnoccupiedSync {
        vehicle_id: read_u16(reader)?,
        seat_id: read_u8(reader)?,
        roll: read_vector3(reader)?,
        direction: read_vector3(reader)?,
        position: read_vector3(reader)?,
        move_speed: read_vector3(reader)?,
        turn_speed: read_vector3(reader)?,
        vehicle_health: read_f32(reader)?,
    })
}

fn write_unoccupied_sync<W: BitWrite>(
    writer: &mut W,
    value: &UnoccupiedSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.vehicle_id)?;
    write_u8(writer, &value.seat_id)?;
    write_vector3(writer, &value.roll)?;
    write_vector3(writer, &value.direction)?;
    write_vector3(writer, &value.position)?;
    write_vector3(writer, &value.move_speed)?;
    write_vector3(writer, &value.turn_speed)?;
    write_f32(writer, &value.vehicle_health)
}

fn read_trailer_sync<R: BitRead>(reader: &mut R) -> Result<TrailerSync, DecodeError<R::Error>> {
    Ok(TrailerSync {
        trailer_id: read_u16(reader)?,
        position: read_vector3(reader)?,
        quaternion: read_quaternion(reader)?,
        move_speed: read_vector3(reader)?,
        turn_speed: read_vector3(reader)?,
    })
}

fn write_trailer_sync<W: BitWrite>(
    writer: &mut W,
    value: &TrailerSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.trailer_id)?;
    write_vector3(writer, &value.position)?;
    write_quaternion(writer, &value.quaternion)?;
    write_vector3(writer, &value.move_speed)?;
    write_vector3(writer, &value.turn_speed)
}

fn read_bullet_sync<R: BitRead>(reader: &mut R) -> Result<BulletSync, DecodeError<R::Error>> {
    Ok(BulletSync {
        target_type: read_u8(reader)?,
        target_id: read_u16(reader)?,
        origin: read_vector3(reader)?,
        target: read_vector3(reader)?,
        center: read_vector3(reader)?,
        weapon_id: read_u8(reader)?,
    })
}

fn write_bullet_sync<W: BitWrite>(
    writer: &mut W,
    value: &BulletSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, &value.target_type)?;
    write_u16(writer, &value.target_id)?;
    write_vector3(writer, &value.origin)?;
    write_vector3(writer, &value.target)?;
    write_vector3(writer, &value.center)?;
    write_u8(writer, &value.weapon_id)
}

fn read_spectator_sync<R: BitRead>(reader: &mut R) -> Result<SpectatorSync, DecodeError<R::Error>> {
    Ok(SpectatorSync {
        left_right_keys: read_u16(reader)?,
        up_down_keys: read_u16(reader)?,
        key_data: read_u16(reader)?,
        position: read_vector3(reader)?,
    })
}

fn write_spectator_sync<W: BitWrite>(
    writer: &mut W,
    value: &SpectatorSync,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.left_right_keys)?;
    write_u16(writer, &value.up_down_keys)?;
    write_u16(writer, &value.key_data)?;
    write_vector3(writer, &value.position)
}

fn read_u8<R: BitRead>(reader: &mut R) -> Result<u8, DecodeError<R::Error>> {
    WireReadExt::read_u8(reader)
}

fn write_u8<W: BitWrite>(writer: &mut W, value: &u8) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_u8(writer, *value)
}

fn read_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    WireReadExt::read_u16_le(reader)
}

fn write_u16<W: BitWrite>(writer: &mut W, value: &u16) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_u16_le(writer, *value)
}

fn read_i32<R: BitRead>(reader: &mut R) -> Result<i32, DecodeError<R::Error>> {
    WireReadExt::read_i32_le(reader)
}

fn write_i32<W: BitWrite>(writer: &mut W, value: &i32) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_i32_le(writer, *value)
}

fn read_f32<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    WireReadExt::read_f32_le(reader)
}

fn write_f32<W: BitWrite>(writer: &mut W, value: &f32) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_f32_le(writer, *value)
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

fn read_quaternion<R: BitRead>(reader: &mut R) -> Result<[f32; 4], DecodeError<R::Error>> {
    Ok([
        read_f32(reader)?,
        read_f32(reader)?,
        read_f32(reader)?,
        read_f32(reader)?,
    ])
}

fn write_quaternion<W: BitWrite>(
    writer: &mut W,
    value: &[f32; 4],
) -> Result<(), EncodeError<W::Error>> {
    for component in value {
        write_f32(writer, component)?;
    }
    Ok(())
}

fn read_string8<R: BitRead>(reader: &mut R) -> Result<Vec<u8>, DecodeError<R::Error>> {
    WireReadExt::read_len_prefixed_bytes_u8(reader, usize::from(u8::MAX))
}

fn write_string8<W: BitWrite>(writer: &mut W, value: &[u8]) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_len_prefixed_bytes_u8(writer, value, usize::from(u8::MAX))
}

fn read_string32<R: BitRead>(reader: &mut R) -> Result<Vec<u8>, DecodeError<R::Error>> {
    WireReadExt::read_len_prefixed_bytes_u32_le(reader, MAX_STRING32_BYTES)
}

fn write_string32<W: BitWrite>(writer: &mut W, value: &[u8]) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_len_prefixed_bytes_u32_le(writer, value, MAX_STRING32_BYTES)
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

fn read_bytes<R: BitRead>(reader: &mut R, length: usize) -> Result<Vec<u8>, DecodeError<R::Error>> {
    WireReadExt::read_bytes(reader, length)
}

fn write_bytes<W: BitWrite>(writer: &mut W, bytes: &[u8]) -> Result<(), EncodeError<W::Error>> {
    WireWriteExt::write_bytes(writer, bytes)
}
