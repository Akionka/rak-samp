//! Profile-neutral incoming RPC codecs.
//!
//! `SHOW_DIALOG` remains in the SDK because it crosses the Native
//! encoded-string boundary.

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, WireCodec, WireReadExt,
    WireWriteExt,
};

use crate::{
    limits::MAX_STRING32_BYTES,
    types::{Vector2, Vector3},
};

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

/// MoonLoader's `onSetPlayerFightingStyle` payload (RPC 89).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerFightingStyle {
    pub player_id: u16,
    pub style_id: u8,
}

/// MoonLoader's `onSetVehicleVelocity` payload (RPC 91).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleVelocity {
    pub turn: bool,
    pub velocity: Vector3,
}

/// MoonLoader's `onCreatePickup` payload (RPC 95).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pickup {
    pub id: i32,
    pub model: i32,
    pub pickup_type: i32,
    pub position: Vector3,
}

/// MoonLoader's `onMoveObject` payload (RPC 99).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveObject {
    pub object_id: u16,
    pub from_position: Vector3,
    pub destination: Vector3,
    pub speed: f32,
    pub rotation: Vector3,
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

/// MoonLoader's `onSetVehicleNumberPlate` payload (RPC 123).
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleNumberPlate {
    pub vehicle_id: u16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onSpectatePlayer` / `onSpectateVehicle` payload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spectate {
    pub target_id: u16,
    pub camera_type: u8,
}

/// MoonLoader's `onSetWeaponAmmo` payload (RPC 145).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponAmmo {
    pub weapon_id: u8,
    pub ammo: u16,
}

/// MoonLoader's `onAttachTrailerToVehicle` payload (RPC 148).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailerAttachment {
    pub trailer_id: u16,
    pub vehicle_id: u16,
}

/// MoonLoader's `onSetCameraLookAt` payload (RPC 158).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraLookAt {
    pub position: Vector3,
    pub cut_type: u8,
}

/// MoonLoader's `onSetVehicleParams` payload (RPC 161).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleParams {
    pub vehicle_id: u16,
    pub objective: bool,
    pub doors_locked: bool,
}

/// MoonLoader's `onPlayerEnterVehicle` payload (RPC 26).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerEnterVehicle {
    pub player_id: u16,
    pub vehicle_id: u16,
    pub passenger: bool,
}

/// MoonLoader's `onPlayerExitVehicle` payload (RPC 154).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerExitVehicle {
    pub player_id: u16,
    pub vehicle_id: u16,
}

struct Empty;
struct U8;
struct U16;
struct I32;
struct F32;
struct Bool8;
struct Vector3Codec;
struct ServerMessageCodec;
struct GameTextCodec;
struct PlaySoundCodec;
struct CheckpointCodec;
struct ChatMessageCodec;
struct ChatBubbleCodec;
struct PlayerJoinCodec;
struct PlayerQuitCodec;
struct PlayerNameCodec;
struct PlayerTimeCodec;
struct WorldBoundsCodec;
struct PlayerWeaponCodec;
struct PlayerTeamCodec;
struct PlayerSkinCodec;
struct PutPlayerInVehicleCodec;
struct VehiclePositionCodec;
struct VehicleAngleCodec;
struct VehicleHealthCodec;
struct RaceCheckpointCodec;
struct AudioStreamCodec;
struct ObjectPositionCodec;
struct ObjectRotationCodec;
struct PlayerDeathNotificationCodec;
struct MapIconCodec;
struct VehicleComponentCodec;
struct VehicleInteriorCodec;
struct PlayerColorCodec;
struct FixedString32Codec;
struct PlayerSkillCodec;
struct RemoveBuildingCodec;
struct AttachObjectToPlayerCodec;
struct ExplosionCodec;
struct PlayerNameTagCodec;
struct ClientCheckCodec;
struct VehicleParamsExCodec;
struct VehicleTuningNotificationCodec;
struct U16U8Codec;
struct VehicleDamageStatusCodec;
struct ActorCodec;
struct ActorAngleCodec;
struct ActorPositionCodec;
struct ActorHealthCodec;
struct PlayerFightingStyleCodec;
struct VehicleVelocityCodec;
struct PickupCodec;
struct MoveObjectCodec;
struct TextDrawStringCodec;
struct GangZoneCodec;
struct U16I32Codec;
struct VehicleNumberPlateCodec;
struct SpectateCodec;
struct WeaponAmmoCodec;
struct TrailerAttachmentCodec;
struct CameraLookAtCodec;
struct VehicleParamsCodec;
struct PlayerEnterVehicleCodec;
struct PlayerExitVehicleCodec;

macro_rules! descriptor_value {
    (Empty) => {
        ()
    };
    (U8) => {
        u8
    };
    (U16) => {
        u16
    };
    (I32) => {
        i32
    };
    (F32) => {
        f32
    };
    (Bool8) => {
        bool
    };
    (Vector3Codec) => {
        Vector3
    };
    (ServerMessageCodec) => {
        ServerMessage
    };
    (GameTextCodec) => {
        GameText
    };
    (PlaySoundCodec) => {
        PlaySound
    };
    (CheckpointCodec) => {
        Checkpoint
    };
    (ChatMessageCodec) => {
        ChatMessage
    };
    (ChatBubbleCodec) => {
        ChatBubble
    };
    (PlayerJoinCodec) => {
        PlayerJoin
    };
    (PlayerQuitCodec) => {
        PlayerQuit
    };
    (PlayerNameCodec) => {
        PlayerName
    };
    (PlayerTimeCodec) => {
        PlayerTime
    };
    (WorldBoundsCodec) => {
        WorldBounds
    };
    (PlayerWeaponCodec) => {
        PlayerWeapon
    };
    (PlayerTeamCodec) => {
        PlayerTeam
    };
    (PlayerSkinCodec) => {
        PlayerSkin
    };
    (PutPlayerInVehicleCodec) => {
        PutPlayerInVehicle
    };
    (VehiclePositionCodec) => {
        VehiclePosition
    };
    (VehicleAngleCodec) => {
        VehicleAngle
    };
    (VehicleHealthCodec) => {
        VehicleHealth
    };
    (RaceCheckpointCodec) => {
        RaceCheckpoint
    };
    (AudioStreamCodec) => {
        AudioStream
    };
    (ObjectPositionCodec) => {
        ObjectPosition
    };
    (ObjectRotationCodec) => {
        ObjectRotation
    };
    (PlayerDeathNotificationCodec) => {
        PlayerDeathNotification
    };
    (MapIconCodec) => {
        MapIcon
    };
    (VehicleComponentCodec) => {
        VehicleComponent
    };
    (VehicleInteriorCodec) => {
        VehicleInterior
    };
    (PlayerColorCodec) => {
        PlayerColor
    };
    (FixedString32Codec) => {
        [u8; 32]
    };
    (PlayerSkillCodec) => {
        PlayerSkill
    };
    (RemoveBuildingCodec) => {
        RemoveBuilding
    };
    (AttachObjectToPlayerCodec) => {
        AttachObjectToPlayer
    };
    (ExplosionCodec) => {
        Explosion
    };
    (PlayerNameTagCodec) => {
        PlayerNameTag
    };
    (ClientCheckCodec) => {
        ClientCheck
    };
    (VehicleParamsExCodec) => {
        VehicleParamsEx
    };
    (VehicleTuningNotificationCodec) => {
        VehicleTuningNotification
    };
    (U16U8Codec) => {
        (u16, u8)
    };
    (VehicleDamageStatusCodec) => {
        VehicleDamageStatus
    };
    (ActorCodec) => {
        Actor
    };
    (ActorAngleCodec) => {
        ActorAngle
    };
    (ActorPositionCodec) => {
        ActorPosition
    };
    (ActorHealthCodec) => {
        ActorHealth
    };
    (PlayerFightingStyleCodec) => {
        PlayerFightingStyle
    };
    (VehicleVelocityCodec) => {
        VehicleVelocity
    };
    (PickupCodec) => {
        Pickup
    };
    (MoveObjectCodec) => {
        MoveObject
    };
    (TextDrawStringCodec) => {
        TextDrawString
    };
    (GangZoneCodec) => {
        GangZone
    };
    (U16I32Codec) => {
        (u16, i32)
    };
    (VehicleNumberPlateCodec) => {
        VehicleNumberPlate
    };
    (SpectateCodec) => {
        Spectate
    };
    (WeaponAmmoCodec) => {
        WeaponAmmo
    };
    (TrailerAttachmentCodec) => {
        TrailerAttachment
    };
    (CameraLookAtCodec) => {
        CameraLookAt
    };
    (VehicleParamsCodec) => {
        VehicleParams
    };
    (PlayerEnterVehicleCodec) => {
        PlayerEnterVehicle
    };
    (PlayerExitVehicleCodec) => {
        PlayerExitVehicle
    };
}

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ident) => {
        crate::wire::nominal_descriptor!(
            incoming rpc,
            $name,
            $constant,
            $id,
            $codec,
            descriptor_value!($codec),
            ExactBytesPolicy
        );
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
descriptor!(AttachCameraToObject, ATTACH_CAMERA_TO_OBJECT, 81, U16);
descriptor!(GangZoneStopFlash, GANG_ZONE_STOP_FLASH, 85, U16);
descriptor!(ClearPlayerAnimation, CLEAR_PLAYER_ANIMATION, 87, U16);
descriptor!(SetPlayerSpecialAction, SET_PLAYER_SPECIAL_ACTION, 88, U8);
descriptor!(
    SetPlayerFightingStyle,
    SET_PLAYER_FIGHTING_STYLE,
    89,
    PlayerFightingStyleCodec
);
descriptor!(SetPlayerVelocity, SET_PLAYER_VELOCITY, 90, Vector3Codec);
descriptor!(
    SetVehicleVelocity,
    SET_VEHICLE_VELOCITY,
    91,
    VehicleVelocityCodec
);
descriptor!(CreatePickup, CREATE_PICKUP, 95, PickupCodec);
descriptor!(MoveObjectRpc, MOVE_OBJECT, 99, MoveObjectCodec);
descriptor!(
    TextDrawSetString,
    TEXT_DRAW_SET_STRING,
    105,
    TextDrawStringCodec
);
descriptor!(CreateGangZone, CREATE_GANG_ZONE, 108, GangZoneCodec);
descriptor!(GangZoneDestroy, GANG_ZONE_DESTROY, 120, U16);
descriptor!(GangZoneFlash, GANG_ZONE_FLASH, 121, U16I32Codec);
descriptor!(StopObject, STOP_OBJECT, 122, U16);
descriptor!(
    SetVehicleNumberPlate,
    SET_VEHICLE_NUMBER_PLATE,
    123,
    VehicleNumberPlateCodec
);
descriptor!(SpectatePlayer, SPECTATE_PLAYER, 126, SpectateCodec);
descriptor!(SpectateVehicle, SPECTATE_VEHICLE, 127, SpectateCodec);
descriptor!(ConnectionRejected, CONNECTION_REJECTED, 130, U8);
descriptor!(RemoveMapIcon, REMOVE_MAP_ICON, 144, U8);
descriptor!(SetWeaponAmmo, SET_WEAPON_AMMO, 145, WeaponAmmoCodec);
descriptor!(SetGravity, SET_GRAVITY, 146, F32);
descriptor!(
    AttachTrailerToVehicle,
    ATTACH_TRAILER_TO_VEHICLE,
    148,
    TrailerAttachmentCodec
);
descriptor!(
    DetachTrailerFromVehicle,
    DETACH_TRAILER_FROM_VEHICLE,
    149,
    U16
);
descriptor!(SetCameraPosition, SET_CAMERA_POSITION, 157, Vector3Codec);
descriptor!(SetCameraLookAt, SET_CAMERA_LOOK_AT, 158, CameraLookAtCodec);
descriptor!(
    SetVehicleParams,
    SET_VEHICLE_PARAMS,
    161,
    VehicleParamsCodec
);
descriptor!(PlayerDeath, PLAYER_DEATH, 166, U16);
descriptor!(
    PlayerEnterVehicleRpc,
    PLAYER_ENTER_VEHICLE,
    26,
    PlayerEnterVehicleCodec
);
descriptor!(
    PlayerExitVehicleRpc,
    PLAYER_EXIT_VEHICLE,
    154,
    PlayerExitVehicleCodec
);

macro_rules! wire_codec {
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

macro_rules! scalar_wire_codec {
    ($codec:ident, $value:ty, $read:ident, $write:ident) => {
        impl WireCodec for $codec {
            type Value = $value;
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

macro_rules! vector_wire_codec {
    ($codec:ident, $value:ty, $read:ident, $write:ident) => {
        impl WireCodec for $codec {
            type Value = $value;
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

wire_codec!(Empty, (), read_empty, write_empty);
scalar_wire_codec!(U8, u8, read_u8, write_u8);
scalar_wire_codec!(U16, u16, read_u16_le, write_u16_le);
scalar_wire_codec!(I32, i32, read_i32_le, write_i32_le);
scalar_wire_codec!(F32, f32, read_f32_le, write_f32_le);
wire_codec!(Bool8, bool, read_bool8, write_bool8);
vector_wire_codec!(Vector3Codec, Vector3, read_vector3_le, write_vector3_le);
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
    PlayerJoinCodec,
    PlayerJoin,
    read_player_join,
    write_player_join
);
wire_codec!(
    PlayerQuitCodec,
    PlayerQuit,
    read_player_quit,
    write_player_quit
);
wire_codec!(
    PlayerNameCodec,
    PlayerName,
    read_player_name,
    write_player_name
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
    PlayerWeaponCodec,
    PlayerWeapon,
    read_player_weapon,
    write_player_weapon
);
wire_codec!(
    PlayerTeamCodec,
    PlayerTeam,
    read_player_team,
    write_player_team
);
wire_codec!(
    PlayerSkinCodec,
    PlayerSkin,
    read_player_skin,
    write_player_skin
);
wire_codec!(
    PutPlayerInVehicleCodec,
    PutPlayerInVehicle,
    read_put_player_in_vehicle,
    write_put_player_in_vehicle
);
wire_codec!(
    VehiclePositionCodec,
    VehiclePosition,
    read_vehicle_position,
    write_vehicle_position
);
wire_codec!(
    VehicleAngleCodec,
    VehicleAngle,
    read_vehicle_angle,
    write_vehicle_angle
);
wire_codec!(
    VehicleHealthCodec,
    VehicleHealth,
    read_vehicle_health,
    write_vehicle_health
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
wire_codec!(
    ObjectPositionCodec,
    ObjectPosition,
    read_object_position,
    write_object_position
);
wire_codec!(
    ObjectRotationCodec,
    ObjectRotation,
    read_object_rotation,
    write_object_rotation
);
wire_codec!(
    PlayerDeathNotificationCodec,
    PlayerDeathNotification,
    read_player_death_notification,
    write_player_death_notification
);
wire_codec!(MapIconCodec, MapIcon, read_map_icon, write_map_icon);
wire_codec!(
    VehicleComponentCodec,
    VehicleComponent,
    read_vehicle_component,
    write_vehicle_component
);
wire_codec!(
    VehicleInteriorCodec,
    VehicleInterior,
    read_vehicle_interior,
    write_vehicle_interior
);
wire_codec!(
    PlayerColorCodec,
    PlayerColor,
    read_player_color,
    write_player_color
);
wire_codec!(
    FixedString32Codec,
    [u8; 32],
    read_fixed_string32,
    write_fixed_string32
);
wire_codec!(
    PlayerSkillCodec,
    PlayerSkill,
    read_player_skill,
    write_player_skill
);
wire_codec!(
    RemoveBuildingCodec,
    RemoveBuilding,
    read_remove_building,
    write_remove_building
);
wire_codec!(
    AttachObjectToPlayerCodec,
    AttachObjectToPlayer,
    read_attach_object_to_player,
    write_attach_object_to_player
);
wire_codec!(ExplosionCodec, Explosion, read_explosion, write_explosion);
wire_codec!(
    PlayerNameTagCodec,
    PlayerNameTag,
    read_player_name_tag,
    write_player_name_tag
);
wire_codec!(
    ClientCheckCodec,
    ClientCheck,
    read_client_check,
    write_client_check
);
wire_codec!(
    VehicleParamsExCodec,
    VehicleParamsEx,
    read_vehicle_params_ex,
    write_vehicle_params_ex
);
wire_codec!(
    VehicleTuningNotificationCodec,
    VehicleTuningNotification,
    read_vehicle_tuning_notification,
    write_vehicle_tuning_notification
);
wire_codec!(U16U8Codec, (u16, u8), read_u16_u8, write_u16_u8);
wire_codec!(
    VehicleDamageStatusCodec,
    VehicleDamageStatus,
    read_vehicle_damage_status,
    write_vehicle_damage_status
);
wire_codec!(ActorCodec, Actor, read_actor, write_actor);
wire_codec!(
    ActorAngleCodec,
    ActorAngle,
    read_actor_angle,
    write_actor_angle
);
wire_codec!(
    ActorPositionCodec,
    ActorPosition,
    read_actor_position,
    write_actor_position
);
wire_codec!(
    ActorHealthCodec,
    ActorHealth,
    read_actor_health,
    write_actor_health
);
wire_codec!(
    PlayerFightingStyleCodec,
    PlayerFightingStyle,
    read_player_fighting_style,
    write_player_fighting_style
);
wire_codec!(
    VehicleVelocityCodec,
    VehicleVelocity,
    read_vehicle_velocity,
    write_vehicle_velocity
);
wire_codec!(PickupCodec, Pickup, read_pickup, write_pickup);
wire_codec!(
    MoveObjectCodec,
    MoveObject,
    read_move_object,
    write_move_object
);
wire_codec!(
    TextDrawStringCodec,
    TextDrawString,
    read_text_draw_string,
    write_text_draw_string
);
wire_codec!(GangZoneCodec, GangZone, read_gang_zone, write_gang_zone);
wire_codec!(U16I32Codec, (u16, i32), read_u16_i32, write_u16_i32);
wire_codec!(
    VehicleNumberPlateCodec,
    VehicleNumberPlate,
    read_vehicle_number_plate,
    write_vehicle_number_plate
);
wire_codec!(SpectateCodec, Spectate, read_spectate, write_spectate);
wire_codec!(
    WeaponAmmoCodec,
    WeaponAmmo,
    read_weapon_ammo,
    write_weapon_ammo
);
wire_codec!(
    TrailerAttachmentCodec,
    TrailerAttachment,
    read_trailer_attachment,
    write_trailer_attachment
);
wire_codec!(
    CameraLookAtCodec,
    CameraLookAt,
    read_camera_look_at,
    write_camera_look_at
);
wire_codec!(
    VehicleParamsCodec,
    VehicleParams,
    read_vehicle_params,
    write_vehicle_params
);
wire_codec!(
    PlayerEnterVehicleCodec,
    PlayerEnterVehicle,
    read_player_enter_vehicle,
    write_player_enter_vehicle
);
wire_codec!(
    PlayerExitVehicleCodec,
    PlayerExitVehicle,
    read_player_exit_vehicle,
    write_player_exit_vehicle
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

fn read_player_fighting_style<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerFightingStyle, DecodeError<R::Error>> {
    Ok(PlayerFightingStyle {
        player_id: reader.read_u16_le()?,
        style_id: reader.read_u8()?,
    })
}

fn write_player_fighting_style<W: BitWrite>(
    writer: &mut W,
    value: &PlayerFightingStyle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.style_id)
}

fn read_vehicle_velocity<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleVelocity, DecodeError<R::Error>> {
    Ok(VehicleVelocity {
        turn: reader.read_u8()? != 0,
        velocity: reader.read_vector3_le()?,
    })
}

fn write_vehicle_velocity<W: BitWrite>(
    writer: &mut W,
    value: &VehicleVelocity,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(u8::from(value.turn))?;
    writer.write_vector3_le(&value.velocity)
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

fn read_move_object<R: BitRead>(reader: &mut R) -> Result<MoveObject, DecodeError<R::Error>> {
    Ok(MoveObject {
        object_id: reader.read_u16_le()?,
        from_position: reader.read_vector3_le()?,
        destination: reader.read_vector3_le()?,
        speed: reader.read_f32_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_move_object<W: BitWrite>(
    writer: &mut W,
    value: &MoveObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.from_position)?;
    writer.write_vector3_le(&value.destination)?;
    writer.write_f32_le(value.speed)?;
    writer.write_vector3_le(&value.rotation)
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

fn read_u16_i32<R: BitRead>(reader: &mut R) -> Result<(u16, i32), DecodeError<R::Error>> {
    Ok((reader.read_u16_le()?, reader.read_i32_le()?))
}

fn write_u16_i32<W: BitWrite>(
    writer: &mut W,
    value: &(u16, i32),
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.0)?;
    writer.write_i32_le(value.1)
}

fn read_vehicle_number_plate<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleNumberPlate, DecodeError<R::Error>> {
    Ok(VehicleNumberPlate {
        vehicle_id: reader.read_u16_le()?,
        text: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_vehicle_number_plate<W: BitWrite>(
    writer: &mut W,
    value: &VehicleNumberPlate,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_len_prefixed_bytes_u8(&value.text, usize::from(u8::MAX))
}

fn read_spectate<R: BitRead>(reader: &mut R) -> Result<Spectate, DecodeError<R::Error>> {
    Ok(Spectate {
        target_id: reader.read_u16_le()?,
        camera_type: reader.read_u8()?,
    })
}

fn write_spectate<W: BitWrite>(
    writer: &mut W,
    value: &Spectate,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.target_id)?;
    writer.write_u8(value.camera_type)
}

fn read_weapon_ammo<R: BitRead>(reader: &mut R) -> Result<WeaponAmmo, DecodeError<R::Error>> {
    Ok(WeaponAmmo {
        weapon_id: reader.read_u8()?,
        ammo: reader.read_u16_le()?,
    })
}

fn write_weapon_ammo<W: BitWrite>(
    writer: &mut W,
    value: &WeaponAmmo,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.weapon_id)?;
    writer.write_u16_le(value.ammo)
}

fn read_trailer_attachment<R: BitRead>(
    reader: &mut R,
) -> Result<TrailerAttachment, DecodeError<R::Error>> {
    Ok(TrailerAttachment {
        trailer_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
    })
}

fn write_trailer_attachment<W: BitWrite>(
    writer: &mut W,
    value: &TrailerAttachment,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.trailer_id)?;
    writer.write_u16_le(value.vehicle_id)
}

fn read_camera_look_at<R: BitRead>(reader: &mut R) -> Result<CameraLookAt, DecodeError<R::Error>> {
    Ok(CameraLookAt {
        position: reader.read_vector3_le()?,
        cut_type: reader.read_u8()?,
    })
}

fn write_camera_look_at<W: BitWrite>(
    writer: &mut W,
    value: &CameraLookAt,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_vector3_le(&value.position)?;
    writer.write_u8(value.cut_type)
}

fn read_vehicle_params<R: BitRead>(reader: &mut R) -> Result<VehicleParams, DecodeError<R::Error>> {
    Ok(VehicleParams {
        vehicle_id: reader.read_u16_le()?,
        objective: reader.read_u8()? != 0,
        doors_locked: reader.read_u8()? != 0,
    })
}

fn write_vehicle_params<W: BitWrite>(
    writer: &mut W,
    value: &VehicleParams,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(u8::from(value.objective))?;
    writer.write_u8(u8::from(value.doors_locked))
}

fn read_player_enter_vehicle<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerEnterVehicle, DecodeError<R::Error>> {
    Ok(PlayerEnterVehicle {
        player_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
        passenger: reader.read_u8()? != 0,
    })
}

fn write_player_enter_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &PlayerEnterVehicle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(u8::from(value.passenger))
}

fn read_player_exit_vehicle<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerExitVehicle, DecodeError<R::Error>> {
    Ok(PlayerExitVehicle {
        player_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
    })
}

fn write_player_exit_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &PlayerExitVehicle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u16_le(value.vehicle_id)
}
