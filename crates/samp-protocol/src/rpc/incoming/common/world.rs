use super::*;

/// MoonLoader's `onPlaySound` payload (RPC 16).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaySound {
    pub sound_id: i32,
    pub position: Vector3,
}

/// MoonLoader's `onSetCheckpoint` payload (RPC 107).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Checkpoint {
    pub position: Vector3,
    pub radius: f32,
}

/// MoonLoader's `onSetPlayerTime` payload (RPC 29).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerTime {
    pub hour: u8,
    pub minute: u8,
}

/// MoonLoader's `onSetWorldBounds` payload (RPC 17).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBounds {
    pub max_x: f32,
    pub min_x: f32,
    pub max_y: f32,
    pub min_y: f32,
}

/// MoonLoader's `onSetRaceCheckpoint` payload (RPC 38).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaceCheckpoint {
    pub checkpoint_type: u8,
    pub position: Vector3,
    pub next_position: Vector3,
    pub size: f32,
}

/// MoonLoader's `onPlayAudioStream` payload (RPC 41).
#[derive(Clone, Debug, PartialEq)]
pub struct AudioStream {
    pub url: Vec<u8>,
    pub position: Vector3,
    pub radius: f32,
    pub use_position: bool,
}

/// MoonLoader's `onSetMapIcon` payload (RPC 56).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapIcon {
    pub icon_id: u8,
    pub position: Vector3,
    pub icon_type: u8,
    pub color: i32,
    pub style: u8,
}

/// MoonLoader's `onRemoveBuilding` payload (RPC 43).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoveBuilding {
    pub model_id: i32,
    pub position: Vector3,
    pub radius: f32,
}

/// MoonLoader's `onCreateExplosion` payload (RPC 79).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Explosion {
    pub position: Vector3,
    pub style: i32,
    pub radius: f32,
}

/// MoonLoader's `onCreatePickup` payload (RPC 95).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pickup {
    pub id: i32,
    pub model: i32,
    pub pickup_type: i32,
    pub position: Vector3,
}

/// MoonLoader's `onCreateGangZone` payload (RPC 108).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GangZone {
    pub zone_id: u16,
    pub square_start: Vector2,
    pub square_end: Vector2,
    pub color: i32,
}

struct PlaySoundCodec;

struct CheckpointCodec;

struct PlayerTimeCodec;

struct WorldBoundsCodec;

struct RaceCheckpointCodec;

struct AudioStreamCodec;

struct MapIconCodec;

struct RemoveBuildingCodec;

struct ExplosionCodec;

struct PickupCodec;

struct GangZoneCodec;

descriptor!(PlaySoundRpc, PLAY_SOUND, 16, PlaySoundCodec, PlaySound);

descriptor!(
    SetCheckpoint,
    SET_CHECKPOINT,
    107,
    CheckpointCodec,
    Checkpoint
);

descriptor!(
    SetPlayerTime,
    SET_PLAYER_TIME,
    29,
    PlayerTimeCodec,
    PlayerTime
);

descriptor!(
    SetWorldBounds,
    SET_WORLD_BOUNDS,
    17,
    WorldBoundsCodec,
    WorldBounds
);

descriptor!(SetWorldTime, SET_WORLD_TIME, 94, U8, u8);

descriptor!(SetWeather, SET_WEATHER, 152, U8, u8);

descriptor!(SetToggleClock, SET_TOGGLE_CLOCK, 30, Bool8, bool);

descriptor!(
    SetRaceCheckpoint,
    SET_RACE_CHECKPOINT,
    38,
    RaceCheckpointCodec,
    RaceCheckpoint
);

descriptor!(
    PlayAudioStream,
    PLAY_AUDIO_STREAM,
    41,
    AudioStreamCodec,
    AudioStream
);

descriptor!(SetMapIcon, SET_MAP_ICON, 56, MapIconCodec, MapIcon);

descriptor!(Remove3DTextLabel, REMOVE_3D_TEXT_LABEL, 58, U16, u16);

descriptor!(UpdateGlobalTimer, UPDATE_GLOBAL_TIMER, 60, I32, i32);

descriptor!(DestroyPickup, DESTROY_PICKUP, 63, I32, i32);

descriptor!(SetShopName, SET_SHOP_NAME, 33, FixedString32Codec, [u8; 32]);

descriptor!(
    RemoveBuildingRpc,
    REMOVE_BUILDING,
    43,
    RemoveBuildingCodec,
    RemoveBuilding
);

descriptor!(
    CreateExplosion,
    CREATE_EXPLOSION,
    79,
    ExplosionCodec,
    Explosion
);

descriptor!(DestroyWeaponPickup, DESTROY_WEAPON_PICKUP, 151, U8, u8);

descriptor!(DisableCheckpoint, DISABLE_CHECKPOINT, 37, Empty, ());

descriptor!(
    DisableRaceCheckpoint,
    DISABLE_RACE_CHECKPOINT,
    39,
    Empty,
    ()
);

descriptor!(StopAudioStream, STOP_AUDIO_STREAM, 42, Empty, ());

descriptor!(GangZoneStopFlash, GANG_ZONE_STOP_FLASH, 85, U16, u16);

descriptor!(CreatePickup, CREATE_PICKUP, 95, PickupCodec, Pickup);

descriptor!(
    CreateGangZone,
    CREATE_GANG_ZONE,
    108,
    GangZoneCodec,
    GangZone
);

descriptor!(GangZoneDestroy, GANG_ZONE_DESTROY, 120, U16, u16);

descriptor!(GangZoneFlash, GANG_ZONE_FLASH, 121, U16I32Codec, (u16, i32));

descriptor!(RemoveMapIcon, REMOVE_MAP_ICON, 144, U8, u8);

descriptor!(SetGravity, SET_GRAVITY, 146, F32, f32);

wire_codec!(PlaySoundCodec, PlaySound, read_play_sound, write_play_sound);

wire_codec!(
    CheckpointCodec,
    Checkpoint,
    read_checkpoint,
    write_checkpoint
);

wire_codec!(
    PlayerTimeCodec,
    PlayerTime,
    read_player_time,
    write_player_time
);

wire_codec!(
    WorldBoundsCodec,
    WorldBounds,
    read_world_bounds,
    write_world_bounds
);

wire_codec!(
    RaceCheckpointCodec,
    RaceCheckpoint,
    read_race_checkpoint,
    write_race_checkpoint
);

wire_codec!(
    AudioStreamCodec,
    AudioStream,
    read_audio_stream,
    write_audio_stream
);

wire_codec!(MapIconCodec, MapIcon, read_map_icon, write_map_icon);

wire_codec!(
    RemoveBuildingCodec,
    RemoveBuilding,
    read_remove_building,
    write_remove_building
);

wire_codec!(ExplosionCodec, Explosion, read_explosion, write_explosion);

wire_codec!(PickupCodec, Pickup, read_pickup, write_pickup);

wire_codec!(GangZoneCodec, GangZone, read_gang_zone, write_gang_zone);

fn read_play_sound<R: BitRead>(reader: &mut R) -> Result<PlaySound, DecodeError<R::Error>> {
    Ok(PlaySound {
        sound_id: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_play_sound<W: BitWrite>(
    writer: &mut W,
    value: &PlaySound,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.sound_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_checkpoint<R: BitRead>(reader: &mut R) -> Result<Checkpoint, DecodeError<R::Error>> {
    Ok(Checkpoint {
        position: reader.read_vector3_le()?,
        radius: reader.read_f32_le()?,
    })
}

fn write_checkpoint<W: BitWrite>(
    writer: &mut W,
    value: &Checkpoint,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.radius)
}

fn read_player_time<R: BitRead>(reader: &mut R) -> Result<PlayerTime, DecodeError<R::Error>> {
    Ok(PlayerTime {
        hour: reader.read_u8()?,
        minute: reader.read_u8()?,
    })
}

fn write_player_time<W: BitWrite>(
    writer: &mut W,
    value: &PlayerTime,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.hour)?;
    writer.write_u8(value.minute)
}

fn read_world_bounds<R: BitRead>(reader: &mut R) -> Result<WorldBounds, DecodeError<R::Error>> {
    Ok(WorldBounds {
        max_x: reader.read_f32_le()?,
        min_x: reader.read_f32_le()?,
        max_y: reader.read_f32_le()?,
        min_y: reader.read_f32_le()?,
    })
}

fn write_world_bounds<W: BitWrite>(
    writer: &mut W,
    value: &WorldBounds,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_f32_le(value.max_x)?;
    writer.write_f32_le(value.min_x)?;
    writer.write_f32_le(value.max_y)?;
    writer.write_f32_le(value.min_y)
}

fn read_race_checkpoint<R: BitRead>(
    reader: &mut R,
) -> Result<RaceCheckpoint, DecodeError<R::Error>> {
    Ok(RaceCheckpoint {
        checkpoint_type: reader.read_u8()?,
        position: reader.read_vector3_le()?,
        next_position: reader.read_vector3_le()?,
        size: reader.read_f32_le()?,
    })
}

fn write_race_checkpoint<W: BitWrite>(
    writer: &mut W,
    value: &RaceCheckpoint,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.checkpoint_type)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_vector3_le(&value.next_position)?;
    writer.write_f32_le(value.size)
}

fn read_audio_stream<R: BitRead>(reader: &mut R) -> Result<AudioStream, DecodeError<R::Error>> {
    Ok(AudioStream {
        url: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        position: reader.read_vector3_le()?,
        radius: reader.read_f32_le()?,
        use_position: read_bool8(reader)?,
    })
}

fn write_audio_stream<W: BitWrite>(
    writer: &mut W,
    value: &AudioStream,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_len_prefixed_bytes_u8(&value.url, usize::from(u8::MAX))?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.radius)?;
    write_bool8(writer, &value.use_position)
}

fn read_map_icon<R: BitRead>(reader: &mut R) -> Result<MapIcon, DecodeError<R::Error>> {
    Ok(MapIcon {
        icon_id: reader.read_u8()?,
        position: reader.read_vector3_le()?,
        icon_type: reader.read_u8()?,
        color: reader.read_i32_le()?,
        style: reader.read_u8()?,
    })
}

fn write_map_icon<W: BitWrite>(
    writer: &mut W,
    value: &MapIcon,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.icon_id)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_u8(value.icon_type)?;
    writer.write_i32_le(value.color)?;
    writer.write_u8(value.style)
}

fn read_remove_building<R: BitRead>(
    reader: &mut R,
) -> Result<RemoveBuilding, DecodeError<R::Error>> {
    Ok(RemoveBuilding {
        model_id: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
        radius: reader.read_f32_le()?,
    })
}

fn write_remove_building<W: BitWrite>(
    writer: &mut W,
    value: &RemoveBuilding,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.model_id)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.radius)
}

fn read_explosion<R: BitRead>(reader: &mut R) -> Result<Explosion, DecodeError<R::Error>> {
    Ok(Explosion {
        position: reader.read_vector3_le()?,
        style: reader.read_i32_le()?,
        radius: reader.read_f32_le()?,
    })
}

fn write_explosion<W: BitWrite>(
    writer: &mut W,
    value: &Explosion,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_vector3_le(&value.position)?;
    writer.write_i32_le(value.style)?;
    writer.write_f32_le(value.radius)
}

fn read_pickup<R: BitRead>(reader: &mut R) -> Result<Pickup, DecodeError<R::Error>> {
    Ok(Pickup {
        id: reader.read_i32_le()?,
        model: reader.read_i32_le()?,
        pickup_type: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_pickup<W: BitWrite>(writer: &mut W, value: &Pickup) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.id)?;
    writer.write_i32_le(value.model)?;
    writer.write_i32_le(value.pickup_type)?;
    writer.write_vector3_le(&value.position)
}

fn read_gang_zone<R: BitRead>(reader: &mut R) -> Result<GangZone, DecodeError<R::Error>> {
    Ok(GangZone {
        zone_id: reader.read_u16_le()?,
        square_start: reader.read_vector2_le()?,
        square_end: reader.read_vector2_le()?,
        color: reader.read_i32_le()?,
    })
}

fn write_gang_zone<W: BitWrite>(
    writer: &mut W,
    value: &GangZone,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.zone_id)?;
    writer.write_vector2_le(&value.square_start)?;
    writer.write_vector2_le(&value.square_end)?;
    writer.write_i32_le(value.color)
}
