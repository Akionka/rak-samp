use super::core::{PayloadWriter, handle};
/// Typed fixed-layout RakNet packet helpers.
///
/// The helpers in this module operate on packet callback events, not RPC callback events. They
/// cover packet layouts that are fixed and byte-aligned in the SA-MP protocol. The packed flag
/// bytes are intentionally exposed without splitting bit fields: their bit order is protocol
/// data, not a Rust memory layout.
use super::{EncodedPayload, Event, EventError, MAX_STRING32_BYTES, Packet, RpcAction, Vector3};
use crate::{HostApi, SampClientSdkEventV1, SampClientSdkHookAction, SampClientSdkResult};

/// SA-MP sends at most 13 weapon slots in one weapons-update packet.
pub const MAX_WEAPON_SLOTS: usize = 13;
/// R1 marker packets cannot contain more players than the protocol player-slot limit.
pub const MAX_MARKERS: usize = 1_000;

pub const AUTHENTICATION_ID: u8 = 12;
pub const CONNECTION_ATTEMPT_FAILED_ID: u8 = 29;
pub const NO_FREE_INCOMING_CONNECTIONS_ID: u8 = 31;
pub const DISCONNECTION_NOTIFICATION_ID: u8 = 32;
pub const CONNECTION_LOST_ID: u8 = 33;
pub const CONNECTION_REQUEST_ACCEPTED_ID: u8 = 34;
pub const CONNECTION_BANNED_ID: u8 = 36;
pub const INVALID_PASSWORD_ID: u8 = 37;
pub const VEHICLE_SYNC_ID: u8 = 200;
pub const RCON_COMMAND_ID: u8 = 201;
pub const AIM_SYNC_ID: u8 = 203;
pub const STATS_UPDATE_ID: u8 = 205;
pub const BULLET_SYNC_ID: u8 = 206;
pub const PLAYER_SYNC_ID: u8 = 207;
pub const MARKERS_SYNC_ID: u8 = 208;
pub const UNOCCUPIED_SYNC_ID: u8 = 209;
pub const TRAILER_SYNC_ID: u8 = 210;
pub const PASSENGER_SYNC_ID: u8 = 211;
pub const SPECTATOR_SYNC_ID: u8 = 212;

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

pub(super) fn require_exact_bytes(event: &Event<'_>, bytes: usize) -> Result<(), EventError> {
    let expected = bytes * u8::BITS as usize;
    let bit_len = event.remaining_bits();
    if bit_len == expected {
        Ok(())
    } else {
        Err(EventError::UnexpectedBitLength { bit_len, expected })
    }
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

pub(super) fn decode_quaternion(event: &mut Event<'_>) -> Result<[f32; 4], EventError> {
    Ok([
        event.read_f32()?,
        event.read_f32()?,
        event.read_f32()?,
        event.read_f32()?,
    ])
}

pub(super) fn write_quaternion(writer: &mut PayloadWriter, value: [f32; 4]) {
    for component in value {
        writer.f32(component);
    }
}

pub(super) fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

pub(super) fn decode_stats_update(event: &mut Event<'_>) -> Result<StatsUpdate, EventError> {
    require_exact_bytes(event, 8)?;
    Ok(StatsUpdate {
        money: decode_i32(event)?,
        drunk_level: decode_i32(event)?,
    })
}

pub(super) fn encode_stats_update(value: StatsUpdate) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.money as u32);
    writer.u32(value.drunk_level as u32);
    Ok(writer.finish())
}

pub(super) fn decode_weapons_update(event: &mut Event<'_>) -> Result<WeaponsUpdate, EventError> {
    let bit_len = event.remaining_bits();
    if bit_len < 32 || !(bit_len - 32).is_multiple_of(32) {
        return Err(EventError::UnexpectedBitLength {
            bit_len,
            expected: 32,
        });
    }
    let weapon_count = (bit_len - 32) / 32;
    if weapon_count > MAX_WEAPON_SLOTS {
        return Err(EventError::LengthExceedsLimit {
            length: weapon_count,
            limit: MAX_WEAPON_SLOTS,
        });
    }
    let player_target = event.read_u16()?;
    let actor_target = event.read_u16()?;
    let mut weapons = Vec::with_capacity(weapon_count);
    for _ in 0..weapon_count {
        weapons.push(WeaponSlot {
            slot: event.read_u8()?,
            weapon: event.read_u8()?,
            ammo: event.read_u16()?,
        });
    }
    Ok(WeaponsUpdate {
        player_target,
        actor_target,
        weapons,
    })
}

pub(super) fn encode_weapons_update(value: WeaponsUpdate) -> Result<Vec<u8>, EventError> {
    if value.weapons.len() > MAX_WEAPON_SLOTS {
        return Err(EventError::LengthExceedsLimit {
            length: value.weapons.len(),
            limit: MAX_WEAPON_SLOTS,
        });
    }
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_target);
    writer.u16(value.actor_target);
    for weapon in value.weapons {
        writer.u8(weapon.slot);
        writer.u8(weapon.weapon);
        writer.u16(weapon.ammo);
    }
    Ok(writer.finish())
}

pub(super) fn decode_string8(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
    let value = event.read_string8()?;
    let bit_len = event.remaining_bits();
    if bit_len == 0 {
        Ok(value)
    } else {
        Err(EventError::UnexpectedBitLength {
            bit_len,
            expected: 0,
        })
    }
}

pub(super) fn encode_string8(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.string8(&value)?;
    Ok(writer.finish())
}

pub(super) fn decode_connection_accepted(
    event: &mut Event<'_>,
) -> Result<ConnectionAccepted, EventError> {
    require_exact_bytes(event, 12)?;
    Ok(ConnectionAccepted {
        ip: decode_i32(event)?,
        port: event.read_u16()?,
        player_id: event.read_u16()?,
        challenge: decode_i32(event)?,
    })
}

pub(super) fn encode_connection_accepted(value: ConnectionAccepted) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.ip as u32);
    writer.u16(value.port);
    writer.u16(value.player_id);
    writer.u32(value.challenge as u32);
    Ok(writer.finish())
}

pub(super) fn decode_empty(event: &mut Event<'_>) -> Result<(), EventError> {
    require_exact_bytes(event, 0)
}

pub(super) fn encode_empty(_value: ()) -> Result<Vec<u8>, EventError> {
    Ok(Vec::new())
}

pub(super) fn decode_player_sync(event: &mut Event<'_>) -> Result<PlayerSync, EventError> {
    require_exact_bytes(event, 68)?;
    Ok(PlayerSync {
        left_right_keys: event.read_u16()?,
        up_down_keys: event.read_u16()?,
        key_data: event.read_u16()?,
        position: decode_vector3(event)?,
        quaternion: decode_quaternion(event)?,
        health: event.read_u8()?,
        armour: event.read_u8()?,
        weapon_and_special_key: event.read_u8()?,
        special_action: event.read_u8()?,
        move_speed: decode_vector3(event)?,
        surfing_offsets: decode_vector3(event)?,
        surfing_vehicle_id: event.read_u16()?,
        animation_id: event.read_u16()?,
        animation_flags: event.read_u16()?,
    })
}

pub(super) fn encode_player_sync(value: PlayerSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.left_right_keys);
    writer.u16(value.up_down_keys);
    writer.u16(value.key_data);
    write_vector3(&mut writer, value.position);
    write_quaternion(&mut writer, value.quaternion);
    writer.u8(value.health);
    writer.u8(value.armour);
    writer.u8(value.weapon_and_special_key);
    writer.u8(value.special_action);
    write_vector3(&mut writer, value.move_speed);
    write_vector3(&mut writer, value.surfing_offsets);
    writer.u16(value.surfing_vehicle_id);
    writer.u16(value.animation_id);
    writer.u16(value.animation_flags);
    Ok(writer.finish())
}

pub(super) fn decode_vehicle_sync(event: &mut Event<'_>) -> Result<VehicleSync, EventError> {
    require_exact_bytes(event, 63)?;
    Ok(VehicleSync {
        vehicle_id: event.read_u16()?,
        left_right_keys: event.read_u16()?,
        up_down_keys: event.read_u16()?,
        key_data: event.read_u16()?,
        quaternion: decode_quaternion(event)?,
        position: decode_vector3(event)?,
        move_speed: decode_vector3(event)?,
        vehicle_health: event.read_f32()?,
        player_health: event.read_u8()?,
        armour: event.read_u8()?,
        weapon_and_special_key: event.read_u8()?,
        siren: event.read_u8()?,
        landing_gear_state: event.read_u8()?,
        trailer_id: event.read_u16()?,
        vehicle_specific: event
            .read_bytes(4)?
            .try_into()
            .map_err(|_| EventError::Host(SampClientSdkResult::NativeCallFailed))?,
    })
}

pub(super) fn encode_vehicle_sync(value: VehicleSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u16(value.left_right_keys);
    writer.u16(value.up_down_keys);
    writer.u16(value.key_data);
    write_quaternion(&mut writer, value.quaternion);
    write_vector3(&mut writer, value.position);
    write_vector3(&mut writer, value.move_speed);
    writer.f32(value.vehicle_health);
    writer.u8(value.player_health);
    writer.u8(value.armour);
    writer.u8(value.weapon_and_special_key);
    writer.u8(value.siren);
    writer.u8(value.landing_gear_state);
    writer.u16(value.trailer_id);
    writer.bytes(&value.vehicle_specific);
    Ok(writer.finish())
}

pub(super) fn decode_passenger_sync(event: &mut Event<'_>) -> Result<PassengerSync, EventError> {
    require_exact_bytes(event, 24)?;
    Ok(PassengerSync {
        vehicle_id: event.read_u16()?,
        seat_driveby_cuffed: event.read_u8()?,
        weapon_and_special_key: event.read_u8()?,
        health: event.read_u8()?,
        armour: event.read_u8()?,
        left_right_keys: event.read_u16()?,
        up_down_keys: event.read_u16()?,
        key_data: event.read_u16()?,
        position: decode_vector3(event)?,
    })
}

pub(super) fn encode_passenger_sync(value: PassengerSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(value.seat_driveby_cuffed);
    writer.u8(value.weapon_and_special_key);
    writer.u8(value.health);
    writer.u8(value.armour);
    writer.u16(value.left_right_keys);
    writer.u16(value.up_down_keys);
    writer.u16(value.key_data);
    write_vector3(&mut writer, value.position);
    Ok(writer.finish())
}

pub(super) fn decode_aim_sync(event: &mut Event<'_>) -> Result<AimSync, EventError> {
    require_exact_bytes(event, 31)?;
    Ok(AimSync {
        camera_mode: event.read_u8()?,
        camera_front: decode_vector3(event)?,
        camera_position: decode_vector3(event)?,
        aim_z: event.read_f32()?,
        zoom_and_weapon_state: event.read_u8()?,
        aspect_ratio: event.read_u8()?,
    })
}

pub(super) fn encode_aim_sync(value: AimSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.camera_mode);
    write_vector3(&mut writer, value.camera_front);
    write_vector3(&mut writer, value.camera_position);
    writer.f32(value.aim_z);
    writer.u8(value.zoom_and_weapon_state);
    writer.u8(value.aspect_ratio);
    Ok(writer.finish())
}

pub(super) fn decode_unoccupied_sync(event: &mut Event<'_>) -> Result<UnoccupiedSync, EventError> {
    require_exact_bytes(event, 67)?;
    Ok(UnoccupiedSync {
        vehicle_id: event.read_u16()?,
        seat_id: event.read_u8()?,
        roll: decode_vector3(event)?,
        direction: decode_vector3(event)?,
        position: decode_vector3(event)?,
        move_speed: decode_vector3(event)?,
        turn_speed: decode_vector3(event)?,
        vehicle_health: event.read_f32()?,
    })
}

pub(super) fn encode_unoccupied_sync(value: UnoccupiedSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(value.seat_id);
    write_vector3(&mut writer, value.roll);
    write_vector3(&mut writer, value.direction);
    write_vector3(&mut writer, value.position);
    write_vector3(&mut writer, value.move_speed);
    write_vector3(&mut writer, value.turn_speed);
    writer.f32(value.vehicle_health);
    Ok(writer.finish())
}

pub(super) fn decode_trailer_sync(event: &mut Event<'_>) -> Result<TrailerSync, EventError> {
    require_exact_bytes(event, 54)?;
    Ok(TrailerSync {
        trailer_id: event.read_u16()?,
        position: decode_vector3(event)?,
        quaternion: decode_quaternion(event)?,
        move_speed: decode_vector3(event)?,
        turn_speed: decode_vector3(event)?,
    })
}

pub(super) fn encode_trailer_sync(value: TrailerSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.trailer_id);
    write_vector3(&mut writer, value.position);
    write_quaternion(&mut writer, value.quaternion);
    write_vector3(&mut writer, value.move_speed);
    write_vector3(&mut writer, value.turn_speed);
    Ok(writer.finish())
}

pub(super) fn decode_bullet_sync(event: &mut Event<'_>) -> Result<BulletSync, EventError> {
    require_exact_bytes(event, 40)?;
    Ok(BulletSync {
        target_type: event.read_u8()?,
        target_id: event.read_u16()?,
        origin: decode_vector3(event)?,
        target: decode_vector3(event)?,
        center: decode_vector3(event)?,
        weapon_id: event.read_u8()?,
    })
}

pub(super) fn encode_bullet_sync(value: BulletSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.target_type);
    writer.u16(value.target_id);
    write_vector3(&mut writer, value.origin);
    write_vector3(&mut writer, value.target);
    write_vector3(&mut writer, value.center);
    writer.u8(value.weapon_id);
    Ok(writer.finish())
}

pub(super) fn decode_spectator_sync(event: &mut Event<'_>) -> Result<SpectatorSync, EventError> {
    require_exact_bytes(event, 18)?;
    Ok(SpectatorSync {
        left_right_keys: event.read_u16()?,
        up_down_keys: event.read_u16()?,
        key_data: event.read_u16()?,
        position: decode_vector3(event)?,
    })
}

pub(super) fn encode_spectator_sync(value: SpectatorSync) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.left_right_keys);
    writer.u16(value.up_down_keys);
    writer.u16(value.key_data);
    write_vector3(&mut writer, value.position);
    Ok(writer.finish())
}

pub(super) fn decode_rcon_command(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
    let command = event.read_string32(MAX_STRING32_BYTES)?;
    if event.remaining_bits() == 0 {
        Ok(command)
    } else {
        Err(EventError::UnexpectedBitLength {
            bit_len: event.remaining_bits(),
            expected: 0,
        })
    }
}

pub(super) fn encode_rcon_command(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.string32(&value)?;
    Ok(writer.finish())
}

pub(super) fn decode_remote<T>(
    event: &mut Event<'_>,
    decode: fn(&mut Event<'_>) -> Result<T, EventError>,
) -> Result<RemoteSync<T>, EventError> {
    let player_id = event.read_u16()?;
    let data = decode(event)?;
    Ok(RemoteSync { player_id, data })
}

pub(super) fn encode_remote<T>(
    value: RemoteSync<T>,
    encode: fn(T) -> Result<Vec<u8>, EventError>,
) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.bytes(&encode(value.data)?);
    Ok(writer.finish())
}

pub(super) fn decode_remote_aim_sync(
    event: &mut Event<'_>,
) -> Result<RemoteSync<AimSync>, EventError> {
    require_exact_bytes(event, 33)?;
    decode_remote(event, decode_aim_sync)
}

pub(super) fn encode_remote_aim_sync(value: RemoteSync<AimSync>) -> Result<Vec<u8>, EventError> {
    encode_remote(value, encode_aim_sync)
}

pub(super) fn decode_remote_bullet_sync(
    event: &mut Event<'_>,
) -> Result<RemoteSync<BulletSync>, EventError> {
    require_exact_bytes(event, 42)?;
    decode_remote(event, decode_bullet_sync)
}

pub(super) fn encode_remote_bullet_sync(
    value: RemoteSync<BulletSync>,
) -> Result<Vec<u8>, EventError> {
    encode_remote(value, encode_bullet_sync)
}

pub(super) fn decode_remote_unoccupied_sync(
    event: &mut Event<'_>,
) -> Result<RemoteSync<UnoccupiedSync>, EventError> {
    require_exact_bytes(event, 69)?;
    decode_remote(event, decode_unoccupied_sync)
}

pub(super) fn encode_remote_unoccupied_sync(
    value: RemoteSync<UnoccupiedSync>,
) -> Result<Vec<u8>, EventError> {
    encode_remote(value, encode_unoccupied_sync)
}

pub(super) fn decode_remote_trailer_sync(
    event: &mut Event<'_>,
) -> Result<RemoteSync<TrailerSync>, EventError> {
    require_exact_bytes(event, 56)?;
    decode_remote(event, decode_trailer_sync)
}

pub(super) fn encode_remote_trailer_sync(
    value: RemoteSync<TrailerSync>,
) -> Result<Vec<u8>, EventError> {
    encode_remote(value, encode_trailer_sync)
}

pub(super) fn decode_remote_passenger_sync(
    event: &mut Event<'_>,
) -> Result<RemoteSync<PassengerSync>, EventError> {
    require_exact_bytes(event, 26)?;
    decode_remote(event, decode_passenger_sync)
}

pub(super) fn encode_remote_passenger_sync(
    value: RemoteSync<PassengerSync>,
) -> Result<Vec<u8>, EventError> {
    encode_remote(value, encode_passenger_sync)
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
pub mod outgoing;

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
