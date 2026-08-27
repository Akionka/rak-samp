//! Fixed-layout incoming RPC codecs.
//!
//! This module owns four bounded incoming batches: 29 descriptors from
//! `SERVER_MESSAGE` through `VEHICLE_STREAM_OUT`, 30 descriptors from
//! `SET_VEHICLE_POSITION` through `SHOW_PLAYER_NAME_TAG`, 26 descriptors from
//! `CLIENT_CHECK` through `SET_CAMERA_BEHIND`, and 29 descriptors from
//! `ATTACH_CAMERA_TO_OBJECT` through `PLAYER_EXIT_VEHICLE`. `SHOW_DIALOG`
//! remains in the SDK because it needs the later Native encoded-string
//! extension boundary.

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, IncomingRpc, TrailingPolicy, WireCodec,
    WireReadExt, WireWriteExt,
};

use crate::{limits::MAX_STRING32_BYTES, types::Vector3};

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

/// MoonLoader's `onSetVehiclePosition` payload (RPC 159).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehiclePosition {
    pub vehicle_id: u16,
    pub position: Vector3,
}

/// MoonLoader's `onSetVehicleAngle` payload (RPC 160).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleAngle {
    pub vehicle_id: u16,
    pub angle: f32,
}

/// MoonLoader's `onSetVehicleHealth` payload (RPC 147).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleHealth {
    pub vehicle_id: u16,
    pub health: f32,
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

/// MoonLoader's `onSetObjectPosition` payload (RPC 45).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectPosition {
    pub object_id: u16,
    pub position: Vector3,
}

/// MoonLoader's `onSetObjectRotation` payload (RPC 46).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectRotation {
    pub object_id: u16,
    pub rotation: Vector3,
}

/// MoonLoader's `onPlayerDeathNotification` payload (RPC 55).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerDeathNotification {
    pub killer_id: u16,
    pub killed_id: u16,
    pub reason: u8,
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

/// MoonLoader's `onRemoveVehicleComponent` payload (RPC 57).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleComponent {
    pub vehicle_id: u16,
    pub component_id: u16,
}

/// MoonLoader's `onLinkVehicleToInterior` payload (RPC 65).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleInterior {
    pub vehicle_id: u16,
    pub interior_id: u8,
}

/// MoonLoader's `onSetPlayerColor` payload (RPC 72).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerColor {
    pub player_id: u16,
    pub color: i32,
}

/// MoonLoader's `onSetPlayerSkillLevel` payload (RPC 34).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSkill {
    pub player_id: u16,
    pub skill: i32,
    pub level: u16,
}

/// MoonLoader's `onRemoveBuilding` payload (RPC 43).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoveBuilding {
    pub model_id: i32,
    pub position: Vector3,
    pub radius: f32,
}

/// MoonLoader's `onAttachObjectToPlayer` payload (RPC 75).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachObjectToPlayer {
    pub object_id: u16,
    pub player_id: u16,
    pub offsets: Vector3,
    pub rotation: Vector3,
}

/// MoonLoader's `onCreateExplosion` payload (RPC 79).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Explosion {
    pub position: Vector3,
    pub style: i32,
    pub radius: f32,
}

/// MoonLoader's `onShowPlayerNameTag` payload (RPC 80).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerNameTag {
    pub player_id: u16,
    pub show: bool,
}

/// MoonLoader's `onClientCheck` payload (RPC 103).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCheck {
    pub request_type: u8,
    pub subject: i32,
    pub offset: u16,
    pub length: u16,
}

/// MoonLoader's `onSetVehicleParamsEx` payload (RPC 24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleParamsEx {
    pub vehicle_id: u16,
    pub params: [u8; 8],
    pub doors: [u8; 4],
    pub windows: [u8; 4],
}

/// MoonLoader's `onVehicleTuningNotification` payload (RPC 96).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleTuningNotification {
    pub player_id: u16,
    pub event: i32,
    pub vehicle_id: i32,
    pub param1: i32,
    pub param2: i32,
}

/// MoonLoader's `onVehicleDamageStatusUpdate` payload (RPC 106).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleDamageStatus {
    pub vehicle_id: u16,
    pub panel_damage: i32,
    pub door_damage: i32,
    pub lights: u8,
    pub tires: u8,
}

/// MoonLoader's `onCreateActor` payload (RPC 171).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Actor {
    pub actor_id: u16,
    pub skin_id: i32,
    pub position: Vector3,
    pub rotation: f32,
    pub health: f32,
}

/// MoonLoader's `onSetActorFacingAngle` payload (RPC 175).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorAngle {
    pub actor_id: u16,
    pub angle: f32,
}

/// MoonLoader's `onSetActorPos` payload (RPC 176).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorPosition {
    pub actor_id: u16,
    pub position: Vector3,
}

/// MoonLoader's `onSetActorHealth` payload (RPC 178).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorHealth {
    pub actor_id: u16,
    pub health: f32,
}

pub struct Empty;
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
pub struct VehiclePositionCodec;
pub struct VehicleAngleCodec;
pub struct VehicleHealthCodec;
pub struct RaceCheckpointCodec;
pub struct AudioStreamCodec;
pub struct ObjectPositionCodec;
pub struct ObjectRotationCodec;
pub struct PlayerDeathNotificationCodec;
pub struct MapIconCodec;
pub struct VehicleComponentCodec;
pub struct VehicleInteriorCodec;
pub struct PlayerColorCodec;
pub struct FixedString32Codec;
pub struct PlayerSkillCodec;
pub struct RemoveBuildingCodec;
pub struct AttachObjectToPlayerCodec;
pub struct ExplosionCodec;
pub struct PlayerNameTagCodec;
pub struct ClientCheckCodec;
pub struct VehicleParamsExCodec;
pub struct VehicleTuningNotificationCodec;
pub struct U16U8Codec;
pub struct VehicleDamageStatusCodec;
pub struct ActorCodec;
pub struct ActorAngleCodec;
pub struct ActorPositionCodec;
pub struct ActorHealthCodec;

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
descriptor!(
    SetVehiclePosition,
    SET_VEHICLE_POSITION,
    159,
    VehiclePositionCodec
);
descriptor!(SetVehicleAngle, SET_VEHICLE_ANGLE, 160, VehicleAngleCodec);
descriptor!(
    SetVehicleHealth,
    SET_VEHICLE_HEALTH,
    147,
    VehicleHealthCodec
);
descriptor!(ResetPlayerMoney, RESET_PLAYER_MONEY, 20, Empty);
descriptor!(ResetPlayerWeapons, RESET_PLAYER_WEAPONS, 21, Empty);
descriptor!(CancelEdit, CANCEL_EDIT, 28, Empty);
descriptor!(SetToggleClock, SET_TOGGLE_CLOCK, 30, Bool8);
descriptor!(SetPlayerDrunk, SET_PLAYER_DRUNK, 35, I32);
descriptor!(
    SetRaceCheckpoint,
    SET_RACE_CHECKPOINT,
    38,
    RaceCheckpointCodec
);
descriptor!(PlayAudioStream, PLAY_AUDIO_STREAM, 41, AudioStreamCodec);
descriptor!(
    SetObjectPosition,
    SET_OBJECT_POSITION,
    45,
    ObjectPositionCodec
);
descriptor!(
    SetObjectRotation,
    SET_OBJECT_ROTATION,
    46,
    ObjectRotationCodec
);
descriptor!(DestroyObject, DESTROY_OBJECT, 47, U16);
descriptor!(
    PlayerDeathNotificationRpc,
    PLAYER_DEATH_NOTIFICATION,
    55,
    PlayerDeathNotificationCodec
);
descriptor!(SetMapIcon, SET_MAP_ICON, 56, MapIconCodec);
descriptor!(
    RemoveVehicleComponent,
    REMOVE_VEHICLE_COMPONENT,
    57,
    VehicleComponentCodec
);
descriptor!(Remove3DTextLabel, REMOVE_3D_TEXT_LABEL, 58, U16);
descriptor!(UpdateGlobalTimer, UPDATE_GLOBAL_TIMER, 60, I32);
descriptor!(DestroyPickup, DESTROY_PICKUP, 63, I32);
descriptor!(
    LinkVehicleToInterior,
    LINK_VEHICLE_TO_INTERIOR,
    65,
    VehicleInteriorCodec
);
descriptor!(SetPlayerColor, SET_PLAYER_COLOR, 72, PlayerColorCodec);
descriptor!(RequestSpawnResponse, REQUEST_SPAWN_RESPONSE, 129, Bool8);
descriptor!(SetShopName, SET_SHOP_NAME, 33, FixedString32Codec);
descriptor!(
    SetPlayerSkillLevel,
    SET_PLAYER_SKILL_LEVEL,
    34,
    PlayerSkillCodec
);
descriptor!(RemoveBuildingRpc, REMOVE_BUILDING, 43, RemoveBuildingCodec);
descriptor!(
    AttachObjectToPlayerRpc,
    ATTACH_OBJECT_TO_PLAYER,
    75,
    AttachObjectToPlayerCodec
);
descriptor!(ShowMenu, SHOW_MENU, 77, U8);
descriptor!(HideMenu, HIDE_MENU, 78, U8);
descriptor!(CreateExplosion, CREATE_EXPLOSION, 79, ExplosionCodec);
descriptor!(
    ShowPlayerNameTag,
    SHOW_PLAYER_NAME_TAG,
    80,
    PlayerNameTagCodec
);
descriptor!(ClientCheckRpc, CLIENT_CHECK, 103, ClientCheckCodec);
descriptor!(
    SetVehicleParamsEx,
    SET_VEHICLE_PARAMS_EX,
    24,
    VehicleParamsExCodec
);
descriptor!(
    VehicleTuningNotificationRpc,
    VEHICLE_TUNING_NOTIFICATION,
    96,
    VehicleTuningNotificationCodec
);
descriptor!(SetVehicleTires, SET_VEHICLE_TIRES, 98, U16U8Codec);
descriptor!(
    VehicleDamageStatusUpdate,
    VEHICLE_DAMAGE_STATUS_UPDATE,
    106,
    VehicleDamageStatusCodec
);
descriptor!(ToggleWidescreen, TOGGLE_WIDESCREEN, 111, Bool8);
descriptor!(DestroyActor, DESTROY_ACTOR, 172, U16);
descriptor!(DestroyWeaponPickup, DESTROY_WEAPON_PICKUP, 151, U8);
descriptor!(EditAttachedObject, EDIT_ATTACHED_OBJECT, 116, I32);
descriptor!(EnterSelectObject, ENTER_SELECT_OBJECT, 27, Empty);
descriptor!(
    ServerStatisticsResponse,
    SERVER_STATISTICS_RESPONSE,
    102,
    Empty
);
descriptor!(SetPlayerDrunkVisuals, SET_PLAYER_DRUNK_VISUALS, 92, I32);
descriptor!(SetPlayerDrunkHandling, SET_PLAYER_DRUNK_HANDLING, 150, I32);
descriptor!(CreateActor, CREATE_ACTOR, 171, ActorCodec);
descriptor!(ClearActorAnimation, CLEAR_ACTOR_ANIMATION, 174, U16);
descriptor!(
    SetActorFacingAngle,
    SET_ACTOR_FACING_ANGLE,
    175,
    ActorAngleCodec
);
descriptor!(
    SetActorPosition,
    SET_ACTOR_POSITION,
    176,
    ActorPositionCodec
);
descriptor!(SetActorHealth, SET_ACTOR_HEALTH, 178, ActorHealthCodec);
descriptor!(
    SetPlayerObjectNoCameraCol,
    SET_PLAYER_OBJECT_NO_CAMERA_COL,
    169,
    U16
);
descriptor!(DisableCheckpoint, DISABLE_CHECKPOINT, 37, Empty);
descriptor!(DisableRaceCheckpoint, DISABLE_RACE_CHECKPOINT, 39, Empty);
descriptor!(GamemodeRestart, GAMEMODE_RESTART, 40, Empty);
descriptor!(StopAudioStream, STOP_AUDIO_STREAM, 42, Empty);
descriptor!(
    RemovePlayerFromVehicle,
    REMOVE_PLAYER_FROM_VEHICLE,
    71,
    Empty
);
descriptor!(ForceClassSelection, FORCE_CLASS_SELECTION, 74, Empty);
descriptor!(SetCameraBehind, SET_CAMERA_BEHIND, 162, Empty);

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

macro_rules! fixed_scalar_codec {
    ($codec:ident, $value:ty, $read:ident, $write:ident) => {
        impl WireCodec for $codec {
            type Value = $value;
            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                reader.$read()
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                writer.$write(*value)
            }
        }
    };
}

macro_rules! fixed_vector_codec {
    ($codec:ident, $value:ty, $read:ident, $write:ident) => {
        impl WireCodec for $codec {
            type Value = $value;
            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                reader.$read()
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                writer.$write(value)
            }
        }
    };
}

fixed_codec!(Empty, (), read_empty, write_empty);
fixed_scalar_codec!(U8, u8, read_u8, write_u8);
fixed_scalar_codec!(U16, u16, read_u16_le, write_u16_le);
fixed_scalar_codec!(I32, i32, read_i32_le, write_i32_le);
fixed_scalar_codec!(F32, f32, read_f32_le, write_f32_le);
fixed_codec!(Bool8, bool, read_bool8, write_bool8);
fixed_vector_codec!(Vector3Codec, Vector3, read_vector3_le, write_vector3_le);
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
fixed_codec!(
    VehiclePositionCodec,
    VehiclePosition,
    read_vehicle_position,
    write_vehicle_position
);
fixed_codec!(
    VehicleAngleCodec,
    VehicleAngle,
    read_vehicle_angle,
    write_vehicle_angle
);
fixed_codec!(
    VehicleHealthCodec,
    VehicleHealth,
    read_vehicle_health,
    write_vehicle_health
);
fixed_codec!(
    RaceCheckpointCodec,
    RaceCheckpoint,
    read_race_checkpoint,
    write_race_checkpoint
);
fixed_codec!(
    AudioStreamCodec,
    AudioStream,
    read_audio_stream,
    write_audio_stream
);
fixed_codec!(
    ObjectPositionCodec,
    ObjectPosition,
    read_object_position,
    write_object_position
);
fixed_codec!(
    ObjectRotationCodec,
    ObjectRotation,
    read_object_rotation,
    write_object_rotation
);
fixed_codec!(
    PlayerDeathNotificationCodec,
    PlayerDeathNotification,
    read_player_death_notification,
    write_player_death_notification
);
fixed_codec!(MapIconCodec, MapIcon, read_map_icon, write_map_icon);
fixed_codec!(
    VehicleComponentCodec,
    VehicleComponent,
    read_vehicle_component,
    write_vehicle_component
);
fixed_codec!(
    VehicleInteriorCodec,
    VehicleInterior,
    read_vehicle_interior,
    write_vehicle_interior
);
fixed_codec!(
    PlayerColorCodec,
    PlayerColor,
    read_player_color,
    write_player_color
);
fixed_codec!(
    FixedString32Codec,
    [u8; 32],
    read_fixed_string32,
    write_fixed_string32
);
fixed_codec!(
    PlayerSkillCodec,
    PlayerSkill,
    read_player_skill,
    write_player_skill
);
fixed_codec!(
    RemoveBuildingCodec,
    RemoveBuilding,
    read_remove_building,
    write_remove_building
);
fixed_codec!(
    AttachObjectToPlayerCodec,
    AttachObjectToPlayer,
    read_attach_object_to_player,
    write_attach_object_to_player
);
fixed_codec!(ExplosionCodec, Explosion, read_explosion, write_explosion);
fixed_codec!(
    PlayerNameTagCodec,
    PlayerNameTag,
    read_player_name_tag,
    write_player_name_tag
);
fixed_codec!(
    ClientCheckCodec,
    ClientCheck,
    read_client_check,
    write_client_check
);
fixed_codec!(
    VehicleParamsExCodec,
    VehicleParamsEx,
    read_vehicle_params_ex,
    write_vehicle_params_ex
);
fixed_codec!(
    VehicleTuningNotificationCodec,
    VehicleTuningNotification,
    read_vehicle_tuning_notification,
    write_vehicle_tuning_notification
);
fixed_codec!(U16U8Codec, (u16, u8), read_u16_u8, write_u16_u8);
fixed_codec!(
    VehicleDamageStatusCodec,
    VehicleDamageStatus,
    read_vehicle_damage_status,
    write_vehicle_damage_status
);
fixed_codec!(ActorCodec, Actor, read_actor, write_actor);
fixed_codec!(
    ActorAngleCodec,
    ActorAngle,
    read_actor_angle,
    write_actor_angle
);
fixed_codec!(
    ActorPositionCodec,
    ActorPosition,
    read_actor_position,
    write_actor_position
);
fixed_codec!(
    ActorHealthCodec,
    ActorHealth,
    read_actor_health,
    write_actor_health
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

fn read_player_join<R: BitRead>(reader: &mut R) -> Result<PlayerJoin, DecodeError<R::Error>> {
    Ok(PlayerJoin {
        player_id: reader.read_u16_le()?,
        color: reader.read_u32_le()?,
        is_npc: read_bool8(reader)?,
        nickname: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_player_join<W: BitWrite>(
    writer: &mut W,
    value: &PlayerJoin,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u32_le(value.color)?;
    write_bool8(writer, &value.is_npc)?;
    writer.write_len_prefixed_bytes_u8(&value.nickname, usize::from(u8::MAX))
}

fn read_player_quit<R: BitRead>(reader: &mut R) -> Result<PlayerQuit, DecodeError<R::Error>> {
    Ok(PlayerQuit {
        player_id: reader.read_u16_le()?,
        reason: reader.read_u8()?,
    })
}

fn write_player_quit<W: BitWrite>(
    writer: &mut W,
    value: &PlayerQuit,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.reason)
}

fn read_player_name<R: BitRead>(reader: &mut R) -> Result<PlayerName, DecodeError<R::Error>> {
    Ok(PlayerName {
        player_id: reader.read_u16_le()?,
        name: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        success: read_bool8(reader)?,
    })
}

fn write_player_name<W: BitWrite>(
    writer: &mut W,
    value: &PlayerName,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_len_prefixed_bytes_u8(&value.name, usize::from(u8::MAX))?;
    write_bool8(writer, &value.success)
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

fn read_player_weapon<R: BitRead>(reader: &mut R) -> Result<PlayerWeapon, DecodeError<R::Error>> {
    Ok(PlayerWeapon {
        weapon_id: reader.read_i32_le()?,
        ammo: reader.read_i32_le()?,
    })
}

fn write_player_weapon<W: BitWrite>(
    writer: &mut W,
    value: &PlayerWeapon,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.weapon_id)?;
    writer.write_i32_le(value.ammo)
}

fn read_player_team<R: BitRead>(reader: &mut R) -> Result<PlayerTeam, DecodeError<R::Error>> {
    Ok(PlayerTeam {
        player_id: reader.read_u16_le()?,
        team_id: reader.read_u8()?,
    })
}

fn write_player_team<W: BitWrite>(
    writer: &mut W,
    value: &PlayerTeam,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.team_id)
}

fn read_player_skin<R: BitRead>(reader: &mut R) -> Result<PlayerSkin, DecodeError<R::Error>> {
    Ok(PlayerSkin {
        player_id: reader.read_i32_le()?,
        skin_id: reader.read_i32_le()?,
    })
}

fn write_player_skin<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSkin,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.player_id)?;
    writer.write_i32_le(value.skin_id)
}

fn read_put_player_in_vehicle<R: BitRead>(
    reader: &mut R,
) -> Result<PutPlayerInVehicle, DecodeError<R::Error>> {
    Ok(PutPlayerInVehicle {
        vehicle_id: reader.read_u16_le()?,
        seat_id: reader.read_u8()?,
    })
}

fn write_put_player_in_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &PutPlayerInVehicle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(value.seat_id)
}

fn read_vehicle_position<R: BitRead>(
    reader: &mut R,
) -> Result<VehiclePosition, DecodeError<R::Error>> {
    Ok(VehiclePosition {
        vehicle_id: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_vehicle_position<W: BitWrite>(
    writer: &mut W,
    value: &VehiclePosition,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_vehicle_angle<R: BitRead>(reader: &mut R) -> Result<VehicleAngle, DecodeError<R::Error>> {
    Ok(VehicleAngle {
        vehicle_id: reader.read_u16_le()?,
        angle: reader.read_f32_le()?,
    })
}

fn write_vehicle_angle<W: BitWrite>(
    writer: &mut W,
    value: &VehicleAngle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_f32_le(value.angle)
}

fn read_vehicle_health<R: BitRead>(reader: &mut R) -> Result<VehicleHealth, DecodeError<R::Error>> {
    Ok(VehicleHealth {
        vehicle_id: reader.read_u16_le()?,
        health: reader.read_f32_le()?,
    })
}

fn write_vehicle_health<W: BitWrite>(
    writer: &mut W,
    value: &VehicleHealth,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_f32_le(value.health)
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

fn read_object_position<R: BitRead>(
    reader: &mut R,
) -> Result<ObjectPosition, DecodeError<R::Error>> {
    Ok(ObjectPosition {
        object_id: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_object_position<W: BitWrite>(
    writer: &mut W,
    value: &ObjectPosition,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_object_rotation<R: BitRead>(
    reader: &mut R,
) -> Result<ObjectRotation, DecodeError<R::Error>> {
    Ok(ObjectRotation {
        object_id: reader.read_u16_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_object_rotation<W: BitWrite>(
    writer: &mut W,
    value: &ObjectRotation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.rotation)
}

fn read_player_death_notification<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerDeathNotification, DecodeError<R::Error>> {
    Ok(PlayerDeathNotification {
        killer_id: reader.read_u16_le()?,
        killed_id: reader.read_u16_le()?,
        reason: reader.read_u8()?,
    })
}

fn write_player_death_notification<W: BitWrite>(
    writer: &mut W,
    value: &PlayerDeathNotification,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.killer_id)?;
    writer.write_u16_le(value.killed_id)?;
    writer.write_u8(value.reason)
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

fn read_vehicle_component<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleComponent, DecodeError<R::Error>> {
    Ok(VehicleComponent {
        vehicle_id: reader.read_u16_le()?,
        component_id: reader.read_u16_le()?,
    })
}

fn write_vehicle_component<W: BitWrite>(
    writer: &mut W,
    value: &VehicleComponent,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u16_le(value.component_id)
}

fn read_vehicle_interior<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleInterior, DecodeError<R::Error>> {
    Ok(VehicleInterior {
        vehicle_id: reader.read_u16_le()?,
        interior_id: reader.read_u8()?,
    })
}

fn write_vehicle_interior<W: BitWrite>(
    writer: &mut W,
    value: &VehicleInterior,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(value.interior_id)
}

fn read_player_color<R: BitRead>(reader: &mut R) -> Result<PlayerColor, DecodeError<R::Error>> {
    Ok(PlayerColor {
        player_id: reader.read_u16_le()?,
        color: reader.read_i32_le()?,
    })
}

fn write_player_color<W: BitWrite>(
    writer: &mut W,
    value: &PlayerColor,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.color)
}

fn read_fixed_string32<R: BitRead>(reader: &mut R) -> Result<[u8; 32], DecodeError<R::Error>> {
    read_fixed(reader)
}

fn write_fixed_string32<W: BitWrite>(
    writer: &mut W,
    value: &[u8; 32],
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bytes(value)
}

fn read_player_skill<R: BitRead>(reader: &mut R) -> Result<PlayerSkill, DecodeError<R::Error>> {
    Ok(PlayerSkill {
        player_id: reader.read_u16_le()?,
        skill: reader.read_i32_le()?,
        level: reader.read_u16_le()?,
    })
}

fn write_player_skill<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSkill,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.skill)?;
    writer.write_u16_le(value.level)
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

fn read_attach_object_to_player<R: BitRead>(
    reader: &mut R,
) -> Result<AttachObjectToPlayer, DecodeError<R::Error>> {
    Ok(AttachObjectToPlayer {
        object_id: reader.read_u16_le()?,
        player_id: reader.read_u16_le()?,
        offsets: reader.read_vector3_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_attach_object_to_player<W: BitWrite>(
    writer: &mut W,
    value: &AttachObjectToPlayer,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_u16_le(value.player_id)?;
    writer.write_vector3_le(&value.offsets)?;
    writer.write_vector3_le(&value.rotation)
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

fn read_player_name_tag<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerNameTag, DecodeError<R::Error>> {
    Ok(PlayerNameTag {
        player_id: reader.read_u16_le()?,
        show: read_bool8(reader)?,
    })
}

fn write_player_name_tag<W: BitWrite>(
    writer: &mut W,
    value: &PlayerNameTag,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    write_bool8(writer, &value.show)
}

fn read_client_check<R: BitRead>(reader: &mut R) -> Result<ClientCheck, DecodeError<R::Error>> {
    Ok(ClientCheck {
        request_type: reader.read_u8()?,
        subject: reader.read_i32_le()?,
        offset: reader.read_u16_le()?,
        length: reader.read_u16_le()?,
    })
}

fn write_client_check<W: BitWrite>(
    writer: &mut W,
    value: &ClientCheck,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.request_type)?;
    writer.write_i32_le(value.subject)?;
    writer.write_u16_le(value.offset)?;
    writer.write_u16_le(value.length)
}

fn read_vehicle_params_ex<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleParamsEx, DecodeError<R::Error>> {
    Ok(VehicleParamsEx {
        vehicle_id: reader.read_u16_le()?,
        params: read_fixed(reader)?,
        doors: read_fixed(reader)?,
        windows: read_fixed(reader)?,
    })
}

fn write_vehicle_params_ex<W: BitWrite>(
    writer: &mut W,
    value: &VehicleParamsEx,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_bytes(&value.params)?;
    writer.write_bytes(&value.doors)?;
    writer.write_bytes(&value.windows)
}

fn read_vehicle_tuning_notification<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleTuningNotification, DecodeError<R::Error>> {
    Ok(VehicleTuningNotification {
        player_id: reader.read_u16_le()?,
        event: reader.read_i32_le()?,
        vehicle_id: reader.read_i32_le()?,
        param1: reader.read_i32_le()?,
        param2: reader.read_i32_le()?,
    })
}

fn write_vehicle_tuning_notification<W: BitWrite>(
    writer: &mut W,
    value: &VehicleTuningNotification,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.event)?;
    writer.write_i32_le(value.vehicle_id)?;
    writer.write_i32_le(value.param1)?;
    writer.write_i32_le(value.param2)
}

fn read_u16_u8<R: BitRead>(reader: &mut R) -> Result<(u16, u8), DecodeError<R::Error>> {
    Ok((reader.read_u16_le()?, reader.read_u8()?))
}

fn write_u16_u8<W: BitWrite>(
    writer: &mut W,
    value: &(u16, u8),
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.0)?;
    writer.write_u8(value.1)
}

fn read_vehicle_damage_status<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleDamageStatus, DecodeError<R::Error>> {
    Ok(VehicleDamageStatus {
        vehicle_id: reader.read_u16_le()?,
        panel_damage: reader.read_i32_le()?,
        door_damage: reader.read_i32_le()?,
        lights: reader.read_u8()?,
        tires: reader.read_u8()?,
    })
}

fn write_vehicle_damage_status<W: BitWrite>(
    writer: &mut W,
    value: &VehicleDamageStatus,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_i32_le(value.panel_damage)?;
    writer.write_i32_le(value.door_damage)?;
    writer.write_u8(value.lights)?;
    writer.write_u8(value.tires)
}

fn read_actor<R: BitRead>(reader: &mut R) -> Result<Actor, DecodeError<R::Error>> {
    Ok(Actor {
        actor_id: reader.read_u16_le()?,
        skin_id: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
        rotation: reader.read_f32_le()?,
        health: reader.read_f32_le()?,
    })
}

fn write_actor<W: BitWrite>(writer: &mut W, value: &Actor) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_i32_le(value.skin_id)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.rotation)?;
    writer.write_f32_le(value.health)
}

fn read_actor_angle<R: BitRead>(reader: &mut R) -> Result<ActorAngle, DecodeError<R::Error>> {
    Ok(ActorAngle {
        actor_id: reader.read_u16_le()?,
        angle: reader.read_f32_le()?,
    })
}

fn write_actor_angle<W: BitWrite>(
    writer: &mut W,
    value: &ActorAngle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_f32_le(value.angle)
}

fn read_actor_position<R: BitRead>(reader: &mut R) -> Result<ActorPosition, DecodeError<R::Error>> {
    Ok(ActorPosition {
        actor_id: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_actor_position<W: BitWrite>(
    writer: &mut W,
    value: &ActorPosition,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_actor_health<R: BitRead>(reader: &mut R) -> Result<ActorHealth, DecodeError<R::Error>> {
    Ok(ActorHealth {
        actor_id: reader.read_u16_le()?,
        health: reader.read_f32_le()?,
    })
}

fn write_actor_health<W: BitWrite>(
    writer: &mut W,
    value: &ActorHealth,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_f32_le(value.health)
}

fn read_empty<R: BitRead>(_reader: &mut R) -> Result<(), DecodeError<R::Error>> {
    Ok(())
}

fn write_empty<W: BitWrite>(_writer: &mut W, _value: &()) -> Result<(), EncodeError<W::Error>> {
    Ok(())
}

fn read_bool8<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u8()? != 0)
}

fn write_bool8<W: BitWrite>(writer: &mut W, value: &bool) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(u8::from(*value))
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

mod phase15;

pub use phase15::*;
