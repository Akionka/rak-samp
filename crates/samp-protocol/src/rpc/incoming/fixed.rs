//! Fixed-layout incoming RPC codecs.
//!
//! This module owns the first bounded incoming batch: 29 descriptors from
//! `SERVER_MESSAGE` through `VEHICLE_STREAM_OUT`. `SHOW_DIALOG` remains in the
//! SDK because it needs the later Native encoded-string extension boundary.

use crate::{BitRead, BitWrite, DecodeError, EncodeError, IncomingRpc, TrailingPolicy, WireCodec};

/// Maximum bytes accepted by a 32-bit length-prefixed SA-MP text field.
pub const MAX_STRING32_BYTES: usize = 4096;

/// A three-dimensional SA-MP coordinate or velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MoonLoader's `onServerMessage` payload (RPC 93).
#[derive(Clone, Debug, PartialEq)]
pub struct ServerMessage {
    pub color: u32,
    pub text: Vec<u8>,
}

/// MoonLoader's `onDisplayGameText` payload (RPC 73).
#[derive(Clone, Debug, PartialEq)]
pub struct GameText {
    pub style: i32,
    pub time_ms: i32,
    pub text: Vec<u8>,
}

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

/// MoonLoader's `onChatMessage` payload (RPC 101).
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub player_id: u16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onPlayerChatBubble` payload (RPC 59).
#[derive(Clone, Debug, PartialEq)]
pub struct ChatBubble {
    pub player_id: u16,
    pub color: u32,
    pub draw_distance: f32,
    pub duration_ms: i32,
    pub text: Vec<u8>,
}

/// MoonLoader's `onPlayerJoin` payload (RPC 137).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerJoin {
    pub player_id: u16,
    pub color: u32,
    pub is_npc: bool,
    pub nickname: Vec<u8>,
}

/// MoonLoader's `onPlayerQuit` payload (RPC 138).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerQuit {
    pub player_id: u16,
    pub reason: u8,
}

/// MoonLoader's `onSetPlayerName` payload (RPC 11).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerName {
    pub player_id: u16,
    pub name: Vec<u8>,
    pub success: bool,
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

/// MoonLoader's `onGivePlayerWeapon` payload (RPC 22).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerWeapon {
    pub weapon_id: i32,
    pub ammo: i32,
}

/// MoonLoader's `onSetPlayerTeam` payload (RPC 69).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerTeam {
    pub player_id: u16,
    pub team_id: u8,
}

/// MoonLoader's `onSetPlayerSkin` payload (RPC 153).
///
/// Both fields stay signed so unknown skin IDs remain observable without lossy validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSkin {
    pub player_id: i32,
    pub skin_id: i32,
}

/// MoonLoader's `onPutPlayerInVehicle` payload (RPC 70).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PutPlayerInVehicle {
    pub vehicle_id: u16,
    pub seat_id: u8,
}

pub struct U8;
pub struct U16;
pub struct I32;
pub struct F32;
pub struct Bool8;
pub struct Vector3Codec;
pub struct ServerMessageCodec;
pub struct GameTextCodec;
pub struct PlaySoundCodec;
pub struct CheckpointCodec;
pub struct ChatMessageCodec;
pub struct ChatBubbleCodec;
pub struct PlayerJoinCodec;
pub struct PlayerQuitCodec;
pub struct PlayerNameCodec;
pub struct PlayerTimeCodec;
pub struct WorldBoundsCodec;
pub struct PlayerWeaponCodec;
pub struct PlayerTeamCodec;
pub struct PlayerSkinCodec;
pub struct PutPlayerInVehicleCodec;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty) => {
        pub type $name = IncomingRpc<$id, $codec>;
        pub const $constant: $name = IncomingRpc::new();
    };
}

descriptor!(ServerMessageRpc, SERVER_MESSAGE, 93, ServerMessageCodec);
descriptor!(DisplayGameText, DISPLAY_GAME_TEXT, 73, GameTextCodec);
descriptor!(SetPlayerPos, SET_PLAYER_POS, 12, Vector3Codec);
descriptor!(SetPlayerPosFindZ, SET_PLAYER_POS_FIND_Z, 13, Vector3Codec);
descriptor!(SetPlayerHealth, SET_PLAYER_HEALTH, 14, F32);
descriptor!(SetPlayerArmour, SET_PLAYER_ARMOUR, 66, F32);
descriptor!(SetPlayerFacingAngle, SET_PLAYER_FACING_ANGLE, 19, F32);
descriptor!(
    TogglePlayerControllable,
    TOGGLE_PLAYER_CONTROLLABLE,
    15,
    Bool8
);
descriptor!(PlaySoundRpc, PLAY_SOUND, 16, PlaySoundCodec);
descriptor!(SetCheckpoint, SET_CHECKPOINT, 107, CheckpointCodec);
descriptor!(ChatMessageRpc, CHAT_MESSAGE, 101, ChatMessageCodec);
descriptor!(ChatBubbleRpc, CHAT_BUBBLE, 59, ChatBubbleCodec);
descriptor!(PlayerJoinRpc, PLAYER_JOIN, 137, PlayerJoinCodec);
descriptor!(PlayerQuitRpc, PLAYER_QUIT, 138, PlayerQuitCodec);
descriptor!(SetPlayerName, SET_PLAYER_NAME, 11, PlayerNameCodec);
descriptor!(SetPlayerTime, SET_PLAYER_TIME, 29, PlayerTimeCodec);
descriptor!(SetWorldBounds, SET_WORLD_BOUNDS, 17, WorldBoundsCodec);
descriptor!(GivePlayerMoney, GIVE_PLAYER_MONEY, 18, I32);
descriptor!(GivePlayerWeapon, GIVE_PLAYER_WEAPON, 22, PlayerWeaponCodec);
descriptor!(SetWorldTime, SET_WORLD_TIME, 94, U8);
descriptor!(SetWeather, SET_WEATHER, 152, U8);
descriptor!(SetPlayerSkin, SET_PLAYER_SKIN, 153, PlayerSkinCodec);
descriptor!(SetInterior, SET_INTERIOR, 156, U8);
descriptor!(SetPlayerArmedWeapon, SET_PLAYER_ARMED_WEAPON, 67, I32);
descriptor!(SetPlayerWantedLevel, SET_PLAYER_WANTED_LEVEL, 133, U8);
descriptor!(SetPlayerTeam, SET_PLAYER_TEAM, 69, PlayerTeamCodec);
descriptor!(
    PutPlayerInVehicleRpc,
    PUT_PLAYER_IN_VEHICLE,
    70,
    PutPlayerInVehicleCodec
);
descriptor!(PlayerStreamOut, PLAYER_STREAM_OUT, 163, U16);
descriptor!(VehicleStreamOut, VEHICLE_STREAM_OUT, 165, U16);

macro_rules! fixed_codec {
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

fixed_codec!(U8, u8, read_u8, write_u8);
fixed_codec!(U16, u16, read_u16, write_u16);
fixed_codec!(I32, i32, read_i32, write_i32);
fixed_codec!(F32, f32, read_f32, write_f32);
fixed_codec!(Bool8, bool, read_bool8, write_bool8);
fixed_codec!(Vector3Codec, Vector3, read_vector3, write_vector3);
fixed_codec!(
    ServerMessageCodec,
    ServerMessage,
    read_server_message,
    write_server_message
);
fixed_codec!(GameTextCodec, GameText, read_game_text, write_game_text);
fixed_codec!(PlaySoundCodec, PlaySound, read_play_sound, write_play_sound);
fixed_codec!(
    CheckpointCodec,
    Checkpoint,
    read_checkpoint,
    write_checkpoint
);
fixed_codec!(
    ChatMessageCodec,
    ChatMessage,
    read_chat_message,
    write_chat_message
);
fixed_codec!(
    ChatBubbleCodec,
    ChatBubble,
    read_chat_bubble,
    write_chat_bubble
);
fixed_codec!(
    PlayerJoinCodec,
    PlayerJoin,
    read_player_join,
    write_player_join
);
fixed_codec!(
    PlayerQuitCodec,
    PlayerQuit,
    read_player_quit,
    write_player_quit
);
fixed_codec!(
    PlayerNameCodec,
    PlayerName,
    read_player_name,
    write_player_name
);
fixed_codec!(
    PlayerTimeCodec,
    PlayerTime,
    read_player_time,
    write_player_time
);
fixed_codec!(
    WorldBoundsCodec,
    WorldBounds,
    read_world_bounds,
    write_world_bounds
);
fixed_codec!(
    PlayerWeaponCodec,
    PlayerWeapon,
    read_player_weapon,
    write_player_weapon
);
fixed_codec!(
    PlayerTeamCodec,
    PlayerTeam,
    read_player_team,
    write_player_team
);
fixed_codec!(
    PlayerSkinCodec,
    PlayerSkin,
    read_player_skin,
    write_player_skin
);
fixed_codec!(
    PutPlayerInVehicleCodec,
    PutPlayerInVehicle,
    read_put_player_in_vehicle,
    write_put_player_in_vehicle
);

fn read_server_message<R: BitRead>(reader: &mut R) -> Result<ServerMessage, DecodeError<R::Error>> {
    Ok(ServerMessage {
        color: read_u32(reader)?,
        text: read_string32(reader)?,
    })
}

fn write_server_message<W: BitWrite>(
    writer: &mut W,
    value: &ServerMessage,
) -> Result<(), EncodeError<W::Error>> {
    write_u32(writer, &value.color)?;
    write_string32(writer, &value.text)
}

fn read_game_text<R: BitRead>(reader: &mut R) -> Result<GameText, DecodeError<R::Error>> {
    Ok(GameText {
        style: read_i32(reader)?,
        time_ms: read_i32(reader)?,
        text: read_string32(reader)?,
    })
}

fn write_game_text<W: BitWrite>(
    writer: &mut W,
    value: &GameText,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.style)?;
    write_i32(writer, &value.time_ms)?;
    write_string32(writer, &value.text)
}

fn read_play_sound<R: BitRead>(reader: &mut R) -> Result<PlaySound, DecodeError<R::Error>> {
    Ok(PlaySound {
        sound_id: read_i32(reader)?,
        position: read_vector3(reader)?,
    })
}

fn write_play_sound<W: BitWrite>(
    writer: &mut W,
    value: &PlaySound,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.sound_id)?;
    write_vector3(writer, &value.position)
}

fn read_checkpoint<R: BitRead>(reader: &mut R) -> Result<Checkpoint, DecodeError<R::Error>> {
    Ok(Checkpoint {
        position: read_vector3(reader)?,
        radius: read_f32(reader)?,
    })
}

fn write_checkpoint<W: BitWrite>(
    writer: &mut W,
    value: &Checkpoint,
) -> Result<(), EncodeError<W::Error>> {
    write_vector3(writer, &value.position)?;
    write_f32(writer, &value.radius)
}

fn read_chat_message<R: BitRead>(reader: &mut R) -> Result<ChatMessage, DecodeError<R::Error>> {
    Ok(ChatMessage {
        player_id: read_u16(reader)?,
        text: read_string8(reader)?,
    })
}

fn write_chat_message<W: BitWrite>(
    writer: &mut W,
    value: &ChatMessage,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_string8(writer, &value.text)
}

fn read_chat_bubble<R: BitRead>(reader: &mut R) -> Result<ChatBubble, DecodeError<R::Error>> {
    Ok(ChatBubble {
        player_id: read_u16(reader)?,
        color: read_u32(reader)?,
        draw_distance: read_f32(reader)?,
        duration_ms: read_i32(reader)?,
        text: read_string8(reader)?,
    })
}

fn write_chat_bubble<W: BitWrite>(
    writer: &mut W,
    value: &ChatBubble,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_u32(writer, &value.color)?;
    write_f32(writer, &value.draw_distance)?;
    write_i32(writer, &value.duration_ms)?;
    write_string8(writer, &value.text)
}

fn read_player_join<R: BitRead>(reader: &mut R) -> Result<PlayerJoin, DecodeError<R::Error>> {
    Ok(PlayerJoin {
        player_id: read_u16(reader)?,
        color: read_u32(reader)?,
        is_npc: read_bool8(reader)?,
        nickname: read_string8(reader)?,
    })
}

fn write_player_join<W: BitWrite>(
    writer: &mut W,
    value: &PlayerJoin,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_u32(writer, &value.color)?;
    write_bool8(writer, &value.is_npc)?;
    write_string8(writer, &value.nickname)
}

fn read_player_quit<R: BitRead>(reader: &mut R) -> Result<PlayerQuit, DecodeError<R::Error>> {
    Ok(PlayerQuit {
        player_id: read_u16(reader)?,
        reason: read_u8(reader)?,
    })
}

fn write_player_quit<W: BitWrite>(
    writer: &mut W,
    value: &PlayerQuit,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_u8(writer, &value.reason)
}

fn read_player_name<R: BitRead>(reader: &mut R) -> Result<PlayerName, DecodeError<R::Error>> {
    Ok(PlayerName {
        player_id: read_u16(reader)?,
        name: read_string8(reader)?,
        success: read_bool8(reader)?,
    })
}

fn write_player_name<W: BitWrite>(
    writer: &mut W,
    value: &PlayerName,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_string8(writer, &value.name)?;
    write_bool8(writer, &value.success)
}

fn read_player_time<R: BitRead>(reader: &mut R) -> Result<PlayerTime, DecodeError<R::Error>> {
    Ok(PlayerTime {
        hour: read_u8(reader)?,
        minute: read_u8(reader)?,
    })
}

fn write_player_time<W: BitWrite>(
    writer: &mut W,
    value: &PlayerTime,
) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, &value.hour)?;
    write_u8(writer, &value.minute)
}

fn read_world_bounds<R: BitRead>(reader: &mut R) -> Result<WorldBounds, DecodeError<R::Error>> {
    Ok(WorldBounds {
        max_x: read_f32(reader)?,
        min_x: read_f32(reader)?,
        max_y: read_f32(reader)?,
        min_y: read_f32(reader)?,
    })
}

fn write_world_bounds<W: BitWrite>(
    writer: &mut W,
    value: &WorldBounds,
) -> Result<(), EncodeError<W::Error>> {
    write_f32(writer, &value.max_x)?;
    write_f32(writer, &value.min_x)?;
    write_f32(writer, &value.max_y)?;
    write_f32(writer, &value.min_y)
}

fn read_player_weapon<R: BitRead>(reader: &mut R) -> Result<PlayerWeapon, DecodeError<R::Error>> {
    Ok(PlayerWeapon {
        weapon_id: read_i32(reader)?,
        ammo: read_i32(reader)?,
    })
}

fn write_player_weapon<W: BitWrite>(
    writer: &mut W,
    value: &PlayerWeapon,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.weapon_id)?;
    write_i32(writer, &value.ammo)
}

fn read_player_team<R: BitRead>(reader: &mut R) -> Result<PlayerTeam, DecodeError<R::Error>> {
    Ok(PlayerTeam {
        player_id: read_u16(reader)?,
        team_id: read_u8(reader)?,
    })
}

fn write_player_team<W: BitWrite>(
    writer: &mut W,
    value: &PlayerTeam,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_u8(writer, &value.team_id)
}

fn read_player_skin<R: BitRead>(reader: &mut R) -> Result<PlayerSkin, DecodeError<R::Error>> {
    Ok(PlayerSkin {
        player_id: read_i32(reader)?,
        skin_id: read_i32(reader)?,
    })
}

fn write_player_skin<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSkin,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.player_id)?;
    write_i32(writer, &value.skin_id)
}

fn read_put_player_in_vehicle<R: BitRead>(
    reader: &mut R,
) -> Result<PutPlayerInVehicle, DecodeError<R::Error>> {
    Ok(PutPlayerInVehicle {
        vehicle_id: read_u16(reader)?,
        seat_id: read_u8(reader)?,
    })
}

fn write_put_player_in_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &PutPlayerInVehicle,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.vehicle_id)?;
    write_u8(writer, &value.seat_id)
}

fn read_u8<R: BitRead>(reader: &mut R) -> Result<u8, DecodeError<R::Error>> {
    Ok(read_fixed::<R, 1>(reader)?[0])
}

fn write_u8<W: BitWrite>(writer: &mut W, value: &u8) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &[*value])
}

fn read_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    Ok(u16::from_le_bytes(read_fixed::<R, 2>(reader)?))
}

fn write_u16<W: BitWrite>(writer: &mut W, value: &u16) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_u32<R: BitRead>(reader: &mut R) -> Result<u32, DecodeError<R::Error>> {
    Ok(u32::from_le_bytes(read_fixed::<R, 4>(reader)?))
}

fn write_u32<W: BitWrite>(writer: &mut W, value: &u32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_i32<R: BitRead>(reader: &mut R) -> Result<i32, DecodeError<R::Error>> {
    Ok(i32::from_le_bytes(read_fixed::<R, 4>(reader)?))
}

fn write_i32<W: BitWrite>(writer: &mut W, value: &i32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_f32<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    Ok(f32::from_le_bytes(read_fixed::<R, 4>(reader)?))
}

fn write_f32<W: BitWrite>(writer: &mut W, value: &f32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_bool8<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(read_u8(reader)? != 0)
}

fn write_bool8<W: BitWrite>(writer: &mut W, value: &bool) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, &u8::from(*value))
}

fn read_vector3<R: BitRead>(reader: &mut R) -> Result<Vector3, DecodeError<R::Error>> {
    Ok(Vector3 {
        x: read_f32(reader)?,
        y: read_f32(reader)?,
        z: read_f32(reader)?,
    })
}

fn write_vector3<W: BitWrite>(
    writer: &mut W,
    value: &Vector3,
) -> Result<(), EncodeError<W::Error>> {
    write_f32(writer, &value.x)?;
    write_f32(writer, &value.y)?;
    write_f32(writer, &value.z)
}

fn read_string8<R: BitRead>(reader: &mut R) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let length = usize::from(read_u8(reader)?);
    read_bytes(reader, length)
}

fn write_string8<W: BitWrite>(writer: &mut W, value: &[u8]) -> Result<(), EncodeError<W::Error>> {
    if value.len() > u8::MAX as usize {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.len(),
            limit: u8::MAX as usize,
        });
    }
    write_u8(writer, &(value.len() as u8))?;
    write_bytes(writer, value)
}

fn read_string32<R: BitRead>(reader: &mut R) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let length = read_u32(reader)? as usize;
    if length > MAX_STRING32_BYTES {
        return Err(DecodeError::LengthExceedsLimit {
            length,
            limit: MAX_STRING32_BYTES,
        });
    }
    read_bytes(reader, length)
}

fn write_string32<W: BitWrite>(writer: &mut W, value: &[u8]) -> Result<(), EncodeError<W::Error>> {
    if value.len() > MAX_STRING32_BYTES {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.len(),
            limit: MAX_STRING32_BYTES,
        });
    }
    write_u32(writer, &(value.len() as u32))?;
    write_bytes(writer, value)
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
    let requested_bits = length * u8::BITS as usize;
    let available_bits = reader.remaining_bits();
    if requested_bits > available_bits {
        return Err(DecodeError::OutOfBounds {
            requested_bits,
            available_bits,
        });
    }
    reader
        .read_left_aligned_bits(requested_bits)
        .map_err(DecodeError::Source)
}

fn write_bytes<W: BitWrite>(writer: &mut W, bytes: &[u8]) -> Result<(), EncodeError<W::Error>> {
    writer
        .write_left_aligned_bits(bytes, bytes.len() * u8::BITS as usize)
        .map_err(EncodeError::Source)
}
