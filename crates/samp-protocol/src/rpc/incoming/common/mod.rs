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

mod ui;

pub use ui::{
    CHAT_BUBBLE, CHAT_MESSAGE, ChatBubble, ChatBubbleRpc, ChatMessage, ChatMessageRpc,
    DISPLAY_GAME_TEXT, DisplayGameText, GameText, HIDE_MENU, HideMenu, SERVER_MESSAGE, SHOW_DIALOG,
    SHOW_MENU, ServerMessage, ServerMessageRpc, ShowDialog, ShowDialogRpc, ShowMenu,
    TEXT_DRAW_SET_STRING, TOGGLE_WIDESCREEN, TextDrawSetString, TextDrawString, ToggleWidescreen,
};
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
