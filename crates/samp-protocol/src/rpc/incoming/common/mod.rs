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

/// MoonLoader's `onTextDrawSetString` payload (RPC 105).
#[derive(Clone, Debug, PartialEq)]
pub struct TextDrawString {
    pub textdraw_id: u16,
    pub text: Vec<u8>,
}

struct ServerMessageCodec;
struct GameTextCodec;
struct ChatMessageCodec;
struct ChatBubbleCodec;
struct TextDrawStringCodec;
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

mod world;

pub use world::{
    AudioStream, CREATE_EXPLOSION, CREATE_GANG_ZONE, CREATE_PICKUP, Checkpoint, CreateExplosion,
    CreateGangZone, CreatePickup, DESTROY_PICKUP, DESTROY_WEAPON_PICKUP, DISABLE_CHECKPOINT,
    DISABLE_RACE_CHECKPOINT, DestroyPickup, DestroyWeaponPickup, DisableCheckpoint,
    DisableRaceCheckpoint, Explosion, GANG_ZONE_DESTROY, GANG_ZONE_FLASH, GANG_ZONE_STOP_FLASH,
    GangZone, GangZoneDestroy, GangZoneFlash, GangZoneStopFlash, MapIcon, PLAY_AUDIO_STREAM,
    PLAY_SOUND, Pickup, PlayAudioStream, PlaySound, PlaySoundRpc, PlayerTime, REMOVE_3D_TEXT_LABEL,
    REMOVE_BUILDING, REMOVE_MAP_ICON, RaceCheckpoint, Remove3DTextLabel, RemoveBuilding,
    RemoveBuildingRpc, RemoveMapIcon, SET_CHECKPOINT, SET_GRAVITY, SET_MAP_ICON, SET_PLAYER_TIME,
    SET_RACE_CHECKPOINT, SET_SHOP_NAME, SET_TOGGLE_CLOCK, SET_WEATHER, SET_WORLD_BOUNDS,
    SET_WORLD_TIME, STOP_AUDIO_STREAM, SetCheckpoint, SetGravity, SetMapIcon, SetPlayerTime,
    SetRaceCheckpoint, SetShopName, SetToggleClock, SetWeather, SetWorldBounds, SetWorldTime,
    StopAudioStream, UPDATE_GLOBAL_TIMER, UpdateGlobalTimer, WorldBounds,
};
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
descriptor!(
    ChatMessageRpc,
    CHAT_MESSAGE,
    101,
    ChatMessageCodec,
    ChatMessage
);
descriptor!(ChatBubbleRpc, CHAT_BUBBLE, 59, ChatBubbleCodec, ChatBubble);
descriptor!(ShowMenu, SHOW_MENU, 77, U8, u8);
descriptor!(HideMenu, HIDE_MENU, 78, U8, u8);
descriptor!(ToggleWidescreen, TOGGLE_WIDESCREEN, 111, Bool8, bool);
descriptor!(
    TextDrawSetString,
    TEXT_DRAW_SET_STRING,
    105,
    TextDrawStringCodec,
    TextDrawString
);
wire_codec!(
    ServerMessageCodec,
    ServerMessage,
    read_server_message,
    write_server_message
);
wire_codec!(GameTextCodec, GameText, read_game_text, write_game_text);
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
    TextDrawStringCodec,
    TextDrawString,
    read_text_draw_string,
    write_text_draw_string
);
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
