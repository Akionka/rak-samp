//! Profile-neutral incoming RPC codecs.

#[macro_use]
mod wire;

use wire::{
    Bool8, Empty, F32, FixedString32Codec, I32, U8, U16, U16I32Codec, U16U8Codec, Vector3Codec,
    read_bool8, read_fixed, write_bool8,
};

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, EncodedStringRead, EncodedStringWireCodec,
    EncodedStringWireDescriptor, EncodedStringWrite, ExactBytesPolicy, TrailingPolicy, WireCodec,
    WireKind, WireReadExt, WireWriteExt,
};

use crate::{
    encoded_string::{read_encoded_string, write_encoded_string},
    limits::{MAX_ENCODED_STRING_BYTES, MAX_STRING32_BYTES},
    types::{Vector2, Vector3},
};

/// MoonLoader's `onShowDialog` payload (RPC 61).
#[derive(Clone, Debug, PartialEq)]
pub struct ShowDialog {
    pub dialog_id: u16,
    pub style: u8,
    pub title: Vec<u8>,
    pub button1: Vec<u8>,
    pub button2: Vec<u8>,
    pub text: Vec<u8>,
}

struct ShowDialogCodec;

impl EncodedStringWireCodec for ShowDialogCodec {
    type Value = ShowDialog;

    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        Ok(ShowDialog {
            dialog_id: reader.read_u16_le()?,
            style: reader.read_u8()?,
            title: reader.read_len_prefixed_bytes_u8(u8::MAX as usize)?,
            button1: reader.read_len_prefixed_bytes_u8(u8::MAX as usize)?,
            button2: reader.read_len_prefixed_bytes_u8(u8::MAX as usize)?,
            text: read_encoded_string(reader, MAX_ENCODED_STRING_BYTES)?,
        })
    }

    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.dialog_id)?;
        writer.write_u8(value.style)?;
        writer.write_len_prefixed_bytes_u8(&value.title, u8::MAX as usize)?;
        writer.write_len_prefixed_bytes_u8(&value.button1, u8::MAX as usize)?;
        writer.write_len_prefixed_bytes_u8(&value.button2, u8::MAX as usize)?;
        write_encoded_string(writer, &value.text, MAX_ENCODED_STRING_BYTES)
    }
}

/// The profile-neutral incoming `SHOW_DIALOG` Wire descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShowDialogRpc;

/// The profile-neutral incoming `SHOW_DIALOG` Wire descriptor value.
pub const SHOW_DIALOG: ShowDialogRpc = ShowDialogRpc;

impl crate::encoded_string::sealed::EncodedStringWireDescriptor<ShowDialog> for ShowDialogRpc {
    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<ShowDialog, DecodeError<R::Error>> {
        ShowDialogCodec::decode(reader)
    }

    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &ShowDialog,
    ) -> Result<(), EncodeError<W::Error>> {
        ShowDialogCodec::encode(writer, value)
    }
}

impl EncodedStringWireDescriptor for ShowDialogRpc {
    type Value = ShowDialog;

    const ID: u8 = 61;
    const KIND: WireKind = WireKind::Rpc;
    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;
}

impl crate::wire::sealed::IncomingRpcDescriptor for ShowDialogRpc {}

impl crate::IncomingRpcDescriptor for ShowDialogRpc {
    type Value = ShowDialog;
    type Capability = crate::EncodedStringWire;

    const ID: u8 = 61;
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

/// MoonLoader's `onTextDrawSetString` payload (RPC 105).
#[derive(Clone, Debug, PartialEq)]
pub struct TextDrawString {
    pub textdraw_id: u16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onCreateGangZone` payload (RPC 108).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GangZone {
    pub zone_id: u16,
    pub square_start: Vector2,
    pub square_end: Vector2,
    pub color: i32,
}

struct ServerMessageCodec;
struct GameTextCodec;
struct PlaySoundCodec;
struct CheckpointCodec;
struct ChatMessageCodec;
struct ChatBubbleCodec;
struct PlayerTimeCodec;
struct WorldBoundsCodec;
struct RaceCheckpointCodec;
struct AudioStreamCodec;
struct MapIconCodec;
struct RemoveBuildingCodec;
struct ExplosionCodec;
struct PickupCodec;
struct TextDrawStringCodec;
struct GangZoneCodec;
macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ident, $value:ty) => {
        crate::wire::nominal_descriptor!(
            incoming rpc,
            $name,
            $constant,
            $id,
            $codec,
            $value,
            ExactBytesPolicy
        );
    };
}

mod player;

pub use player::{
    CLEAR_PLAYER_ANIMATION, ClearPlayerAnimation, GIVE_PLAYER_MONEY, GIVE_PLAYER_WEAPON,
    GivePlayerMoney, GivePlayerWeapon, PLAYER_DEATH, PLAYER_DEATH_NOTIFICATION, PLAYER_STREAM_OUT,
    PlayerColor, PlayerDeath, PlayerDeathNotification, PlayerDeathNotificationRpc,
    PlayerFightingStyle, PlayerName, PlayerNameTag, PlayerSkill, PlayerSkin, PlayerStreamOut,
    PlayerTeam, PlayerWeapon, RESET_PLAYER_MONEY, RESET_PLAYER_WEAPONS, ResetPlayerMoney,
    ResetPlayerWeapons, SET_INTERIOR, SET_PLAYER_ARMED_WEAPON, SET_PLAYER_ARMOUR, SET_PLAYER_COLOR,
    SET_PLAYER_DRUNK, SET_PLAYER_DRUNK_HANDLING, SET_PLAYER_DRUNK_VISUALS, SET_PLAYER_FACING_ANGLE,
    SET_PLAYER_FIGHTING_STYLE, SET_PLAYER_HEALTH, SET_PLAYER_NAME, SET_PLAYER_POS,
    SET_PLAYER_POS_FIND_Z, SET_PLAYER_SKILL_LEVEL, SET_PLAYER_SKIN, SET_PLAYER_SPECIAL_ACTION,
    SET_PLAYER_TEAM, SET_PLAYER_VELOCITY, SET_PLAYER_WANTED_LEVEL, SET_WEAPON_AMMO,
    SHOW_PLAYER_NAME_TAG, SetInterior, SetPlayerArmedWeapon, SetPlayerArmour, SetPlayerColor,
    SetPlayerDrunk, SetPlayerDrunkHandling, SetPlayerDrunkVisuals, SetPlayerFacingAngle,
    SetPlayerFightingStyle, SetPlayerHealth, SetPlayerName, SetPlayerPos, SetPlayerPosFindZ,
    SetPlayerSkillLevel, SetPlayerSkin, SetPlayerSpecialAction, SetPlayerTeam, SetPlayerVelocity,
    SetPlayerWantedLevel, SetWeaponAmmo, ShowPlayerNameTag, TOGGLE_PLAYER_CONTROLLABLE,
    TogglePlayerControllable, WeaponAmmo,
};
mod vehicle;

pub use vehicle::{
    ATTACH_TRAILER_TO_VEHICLE, AttachTrailerToVehicle, DETACH_TRAILER_FROM_VEHICLE,
    DetachTrailerFromVehicle, LINK_VEHICLE_TO_INTERIOR, LinkVehicleToInterior,
    PLAYER_ENTER_VEHICLE, PLAYER_EXIT_VEHICLE, PUT_PLAYER_IN_VEHICLE, PlayerEnterVehicle,
    PlayerEnterVehicleRpc, PlayerExitVehicle, PlayerExitVehicleRpc, PutPlayerInVehicle,
    PutPlayerInVehicleRpc, REMOVE_PLAYER_FROM_VEHICLE, REMOVE_VEHICLE_COMPONENT,
    RemovePlayerFromVehicle, RemoveVehicleComponent, SET_VEHICLE_ANGLE, SET_VEHICLE_HEALTH,
    SET_VEHICLE_NUMBER_PLATE, SET_VEHICLE_PARAMS, SET_VEHICLE_PARAMS_EX, SET_VEHICLE_POSITION,
    SET_VEHICLE_TIRES, SET_VEHICLE_VELOCITY, SetVehicleAngle, SetVehicleHealth,
    SetVehicleNumberPlate, SetVehicleParams, SetVehicleParamsEx, SetVehiclePosition,
    SetVehicleTires, SetVehicleVelocity, TrailerAttachment, VEHICLE_DAMAGE_STATUS_UPDATE,
    VEHICLE_STREAM_OUT, VEHICLE_TUNING_NOTIFICATION, VehicleAngle, VehicleComponent,
    VehicleDamageStatus, VehicleDamageStatusUpdate, VehicleHealth, VehicleInterior,
    VehicleNumberPlate, VehicleParams, VehicleParamsEx, VehiclePosition, VehicleStreamOut,
    VehicleTuningNotification, VehicleTuningNotificationRpc, VehicleVelocity,
};
mod object;

pub use object::{
    ATTACH_OBJECT_TO_PLAYER, AttachObjectToPlayer, AttachObjectToPlayerRpc, CANCEL_EDIT,
    CancelEdit, DESTROY_OBJECT, DestroyObject, EDIT_ATTACHED_OBJECT, ENTER_SELECT_OBJECT,
    EditAttachedObject, EnterSelectObject, MOVE_OBJECT, MoveObject, MoveObjectRpc, ObjectPosition,
    ObjectRotation, SET_OBJECT_POSITION, SET_OBJECT_ROTATION, SET_PLAYER_OBJECT_NO_CAMERA_COL,
    STOP_OBJECT, SetObjectPosition, SetObjectRotation, SetPlayerObjectNoCameraCol, StopObject,
};
mod camera;

pub use camera::{
    ATTACH_CAMERA_TO_OBJECT, AttachCameraToObject, CameraLookAt, SET_CAMERA_BEHIND,
    SET_CAMERA_LOOK_AT, SET_CAMERA_POSITION, SPECTATE_PLAYER, SPECTATE_VEHICLE, SetCameraBehind,
    SetCameraLookAt, SetCameraPosition, Spectate, SpectatePlayer, SpectateVehicle,
};
mod session;

pub use session::{
    CLIENT_CHECK, CONNECTION_REJECTED, ClientCheck, ClientCheckRpc, ConnectionRejected,
    FORCE_CLASS_SELECTION, ForceClassSelection, GAMEMODE_RESTART, GamemodeRestart, PLAYER_JOIN,
    PLAYER_QUIT, PlayerJoin, PlayerJoinRpc, PlayerQuit, PlayerQuitRpc, REQUEST_SPAWN_RESPONSE,
    RequestSpawnResponse, SERVER_STATISTICS_RESPONSE, ServerStatisticsResponse,
};
mod actor;

pub use actor::{
    Actor, ActorAngle, ActorHealth, ActorPosition, CLEAR_ACTOR_ANIMATION, CREATE_ACTOR,
    ClearActorAnimation, CreateActor, DESTROY_ACTOR, DestroyActor, SET_ACTOR_FACING_ANGLE,
    SET_ACTOR_HEALTH, SET_ACTOR_POSITION, SetActorFacingAngle, SetActorHealth, SetActorPosition,
};
descriptor!(
    ServerMessageRpc,
    SERVER_MESSAGE,
    93,
    ServerMessageCodec,
    ServerMessage
);
descriptor!(
    DisplayGameText,
    DISPLAY_GAME_TEXT,
    73,
    GameTextCodec,
    GameText
);
descriptor!(PlaySoundRpc, PLAY_SOUND, 16, PlaySoundCodec, PlaySound);
descriptor!(
    SetCheckpoint,
    SET_CHECKPOINT,
    107,
    CheckpointCodec,
    Checkpoint
);
descriptor!(
    ChatMessageRpc,
    CHAT_MESSAGE,
    101,
    ChatMessageCodec,
    ChatMessage
);
descriptor!(ChatBubbleRpc, CHAT_BUBBLE, 59, ChatBubbleCodec, ChatBubble);
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
descriptor!(ShowMenu, SHOW_MENU, 77, U8, u8);
descriptor!(HideMenu, HIDE_MENU, 78, U8, u8);
descriptor!(
    CreateExplosion,
    CREATE_EXPLOSION,
    79,
    ExplosionCodec,
    Explosion
);
descriptor!(ToggleWidescreen, TOGGLE_WIDESCREEN, 111, Bool8, bool);
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
    TextDrawSetString,
    TEXT_DRAW_SET_STRING,
    105,
    TextDrawStringCodec,
    TextDrawString
);
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
wire_codec!(
    ServerMessageCodec,
    ServerMessage,
    read_server_message,
    write_server_message
);
wire_codec!(GameTextCodec, GameText, read_game_text, write_game_text);
wire_codec!(PlaySoundCodec, PlaySound, read_play_sound, write_play_sound);
wire_codec!(
    CheckpointCodec,
    Checkpoint,
    read_checkpoint,
    write_checkpoint
);
wire_codec!(
    ChatMessageCodec,
    ChatMessage,
    read_chat_message,
    write_chat_message
);
wire_codec!(
    ChatBubbleCodec,
    ChatBubble,
    read_chat_bubble,
    write_chat_bubble
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
wire_codec!(
    TextDrawStringCodec,
    TextDrawString,
    read_text_draw_string,
    write_text_draw_string
);
wire_codec!(GangZoneCodec, GangZone, read_gang_zone, write_gang_zone);
fn read_server_message<R: BitRead>(reader: &mut R) -> Result<ServerMessage, DecodeError<R::Error>> {
    Ok(ServerMessage {
        color: reader.read_u32_le()?,
        text: reader.read_len_prefixed_bytes_u32_le(MAX_STRING32_BYTES)?,
    })
}

fn write_server_message<W: BitWrite>(
    writer: &mut W,
    value: &ServerMessage,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u32_le(value.color)?;
    writer.write_len_prefixed_bytes_u32_le(&value.text, MAX_STRING32_BYTES)
}

fn read_game_text<R: BitRead>(reader: &mut R) -> Result<GameText, DecodeError<R::Error>> {
    Ok(GameText {
        style: reader.read_i32_le()?,
        time_ms: reader.read_i32_le()?,
        text: reader.read_len_prefixed_bytes_u32_le(MAX_STRING32_BYTES)?,
    })
}

fn write_game_text<W: BitWrite>(
    writer: &mut W,
    value: &GameText,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.style)?;
    writer.write_i32_le(value.time_ms)?;
    writer.write_len_prefixed_bytes_u32_le(&value.text, MAX_STRING32_BYTES)
}

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

fn read_chat_message<R: BitRead>(reader: &mut R) -> Result<ChatMessage, DecodeError<R::Error>> {
    Ok(ChatMessage {
        player_id: reader.read_u16_le()?,
        text: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_chat_message<W: BitWrite>(
    writer: &mut W,
    value: &ChatMessage,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_len_prefixed_bytes_u8(&value.text, usize::from(u8::MAX))
}

fn read_chat_bubble<R: BitRead>(reader: &mut R) -> Result<ChatBubble, DecodeError<R::Error>> {
    Ok(ChatBubble {
        player_id: reader.read_u16_le()?,
        color: reader.read_u32_le()?,
        draw_distance: reader.read_f32_le()?,
        duration_ms: reader.read_i32_le()?,
        text: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_chat_bubble<W: BitWrite>(
    writer: &mut W,
    value: &ChatBubble,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u32_le(value.color)?;
    writer.write_f32_le(value.draw_distance)?;
    writer.write_i32_le(value.duration_ms)?;
    writer.write_len_prefixed_bytes_u8(&value.text, usize::from(u8::MAX))
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

fn read_text_draw_string<R: BitRead>(
    reader: &mut R,
) -> Result<TextDrawString, DecodeError<R::Error>> {
    Ok(TextDrawString {
        textdraw_id: reader.read_u16_le()?,
        text: reader.read_len_prefixed_bytes_u16_le(MAX_STRING32_BYTES)?,
    })
}

fn write_text_draw_string<W: BitWrite>(
    writer: &mut W,
    value: &TextDrawString,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.textdraw_id)?;
    writer.write_len_prefixed_bytes_u16_le(&value.text, MAX_STRING32_BYTES)
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
