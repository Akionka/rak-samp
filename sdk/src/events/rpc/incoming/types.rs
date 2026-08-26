use crate::events::{Vector2, Vector3};

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

/// MoonLoader's `onClientCheck` payload (RPC 103).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCheck {
    pub request_type: u8,
    pub subject: i32,
    pub offset: u16,
    pub length: u16,
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

/// MoonLoader's `onSetVehicleParamsEx` payload (RPC 24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleParamsEx {
    pub vehicle_id: u16,
    pub params: [u8; 8],
    pub doors: [u8; 4],
    pub windows: [u8; 4],
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

/// The maximum number of rows that R1 menus can expose per column.
pub const MAX_MENU_ROWS: usize = 12;
/// The R1 client accepts at most two menu columns.
pub const MAX_MENU_COLUMNS: usize = 2;
/// SA-MP objects expose at most sixteen material slots.
pub const MAX_OBJECT_MATERIALS: usize = 16;
/// The server can send at most one score/ping entry for each R1 player slot.
pub const MAX_SCORE_PING_ENTRIES: usize = 1_000;
/// R1's material-text codec accepts at most 2,047 payload bytes.
pub const MAX_OBJECT_MATERIAL_TEXT_BYTES: usize = 2_047;

/// Settings supplied by `onInitGame` (RPC 139).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameSettings {
    pub zone_names: bool,
    pub use_cj_walk: bool,
    pub allow_weapons: bool,
    pub limit_global_chat_radius: bool,
    pub global_chat_radius: f32,
    pub stunt_bonus: bool,
    pub nametag_draw_distance: f32,
    pub disable_enter_exits: bool,
    pub nametag_los: bool,
    pub tire_popping: bool,
    pub classes_available: i32,
    pub show_player_tags: bool,
    pub player_markers_mode: i32,
    pub world_time: u8,
    pub world_weather: u8,
    pub gravity: f32,
    pub lan_mode: bool,
    pub death_money_drop: i32,
    pub instagib: bool,
    pub normal_onfoot_send_rate: i32,
    pub normal_incar_send_rate: i32,
    pub normal_firing_send_rate: i32,
    pub send_multiplier: i32,
    pub lag_compensation_mode: i32,
    pub vehicle_friendly_fire: bool,
}

/// MoonLoader's `onInitGame` payload (RPC 139).
#[derive(Clone, Debug, PartialEq)]
pub struct InitGame {
    pub player_id: u16,
    pub host_name: Vec<u8>,
    pub settings: GameSettings,
    /// R1's 212 vehicle-model capability flags, retained byte-for-byte.
    pub vehicle_models: [u8; 212],
}

/// A class preview or spawn definition shared by the class and spawn RPCs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnInfo {
    pub team: u8,
    pub skin: i32,
    /// R1 serializes this byte between the skin and position. Its purpose is unknown.
    pub unused: u8,
    pub position: Vector3,
    pub rotation: f32,
    pub weapons: [i32; 3],
    pub ammo: [i32; 3],
}

/// MoonLoader's `onRequestClassResponse` payload (RPC 128).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestClassResponse {
    pub can_spawn: bool,
    pub spawn: SpawnInfo,
}

/// MoonLoader's `onPlayerStreamIn` payload (RPC 32).
///
/// R1 sends one skill level for each of the eleven weapon-skill categories after the fixed
/// player data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerStreamIn {
    pub player_id: u16,
    pub team: u8,
    pub model: i32,
    pub position: Vector3,
    pub rotation: f32,
    pub color: i32,
    pub fighting_style: u8,
    pub weapon_skill_levels: [u16; 11],
}

/// MoonLoader's `onCreate3DText` payload (RPC 36).
#[derive(Clone, Debug, PartialEq)]
pub struct TextLabel3D {
    pub id: u16,
    pub color: i32,
    pub position: Vector3,
    pub distance: f32,
    pub test_los: bool,
    pub attached_player_id: u16,
    pub attached_vehicle_id: u16,
    pub text: Vec<u8>,
}

/// Object attachment fields that are present only when an object has an attachment target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectAttachment {
    pub offsets: Vector3,
    pub rotation: Vector3,
    pub sync_rotation: bool,
}

/// A texture-based object material. The field order is the R1 wire order.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureMaterial {
    pub material_id: u8,
    pub model_id: u16,
    pub library_name: Vec<u8>,
    pub texture_name: Vec<u8>,
    pub color: i32,
}

/// A text-based object material. The encoded text deliberately remains bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct TextMaterial {
    pub material_id: u8,
    pub material_size: u8,
    pub font_name: Vec<u8>,
    pub font_size: u8,
    pub bold: u8,
    pub font_color: i32,
    pub background_color: i32,
    pub align: u8,
    pub text: Vec<u8>,
}

/// One object material, preserving texture/text ordering during a replacement.
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectMaterial {
    Texture(TextureMaterial),
    Text(TextMaterial),
}

/// MoonLoader's `onCreateObject` payload (RPC 44).
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub object_id: u16,
    pub model_id: i32,
    pub position: Vector3,
    pub rotation: Vector3,
    pub draw_distance: f32,
    pub no_camera_collision: bool,
    pub attach_to_vehicle_id: u16,
    pub attach_to_object_id: u16,
    pub attachment: Option<ObjectAttachment>,
    /// R1's original material-count field, retained independently of the decoded sequence.
    pub textures_count: u8,
    pub materials: Vec<ObjectMaterial>,
}

/// One update from RPC 84, which can carry either material variant.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectMaterialUpdate {
    pub object_id: u16,
    pub material: ObjectMaterial,
}

/// One column in an R1 menu initialization payload.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuColumn {
    pub width: f32,
    pub title: [u8; 32],
    pub rows: Vec<[u8; 32]>,
}

/// MoonLoader's `onInitMenu` payload (RPC 76).
#[derive(Clone, Debug, PartialEq)]
pub struct InitMenu {
    pub menu_id: u8,
    pub two_columns: bool,
    pub title: [u8; 32],
    pub position: Vector2,
    pub columns: Vec<MenuColumn>,
    pub rows: [i32; MAX_MENU_ROWS],
    pub menu: bool,
}

/// MoonLoader's `onInterpolateCamera` payload (RPC 82).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterpolateCamera {
    pub set_position: bool,
    pub from_position: Vector3,
    pub destination: Vector3,
    pub time_ms: i32,
    pub mode: u8,
}

/// MoonLoader's `onToggleSelectTextDraw` payload (RPC 83).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleSelectTextDraw {
    pub enabled: bool,
    pub hover_color: i32,
}

/// MoonLoader's player or actor animation payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub animation_library: Vec<u8>,
    pub animation_name: Vec<u8>,
    pub frame_delta: f32,
    pub looped: bool,
    pub lock_x: bool,
    pub lock_y: bool,
    pub freeze: bool,
    pub time: i32,
}

/// MoonLoader's `onApplyPlayerAnimation` payload (RPC 86).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerAnimation {
    pub player_id: u16,
    pub animation: Animation,
}

/// MoonLoader's `onApplyActorAnimation` payload (RPC 173).
#[derive(Clone, Debug, PartialEq)]
pub struct ActorAnimation {
    pub actor_id: u16,
    pub animation: Animation,
}

/// MoonLoader's `onPlayCrimeReport` payload (RPC 112).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrimeReport {
    pub suspect_id: u16,
    pub in_vehicle: bool,
    pub vehicle_model: i32,
    pub vehicle_color: i32,
    pub crime: i32,
    pub coordinates: Vector3,
}

/// An attached player object, present only when `create` is true.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachedObject {
    pub model_id: i32,
    pub bone: i32,
    pub offset: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
    pub color1: i32,
    pub color2: i32,
}

/// MoonLoader's `onSetPlayerAttachedObject` payload (RPC 113).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerAttachedObject {
    pub player_id: u16,
    pub index: i32,
    pub object: Option<AttachedObject>,
}

/// MoonLoader's `onEnterEditObject` payload (RPC 117).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnterEditObject {
    pub player_object: bool,
    pub object_id: u16,
}

/// The R1 textdraw shape and content sent by `onShowTextDraw`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextDraw {
    pub flags: u8,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_color: i32,
    pub line_width: f32,
    pub line_height: f32,
    pub box_color: i32,
    pub shadow: u8,
    pub outline: u8,
    pub background_color: i32,
    pub style: u8,
    pub selectable: u8,
    pub position: Vector2,
    pub model_id: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub color1: i16,
    pub color2: i16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onShowTextDraw` payload (RPC 134).
#[derive(Clone, Debug, PartialEq)]
pub struct ShowTextDraw {
    pub textdraw_id: u16,
    pub textdraw: TextDraw,
}

/// One score and ping record sent by RPC 155.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScorePing {
    pub player_id: u16,
    pub score: i32,
    pub ping: i32,
}

/// MoonLoader's `onUpdateScoresAndPings` payload (RPC 155), retained in wire order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoresAndPings {
    pub entries: Vec<ScorePing>,
}

/// MoonLoader's `onVehicleStreamIn` vehicle data (RPC 164).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreamedVehicle {
    pub model: i32,
    pub position: Vector3,
    pub rotation: f32,
    pub body_color1: u8,
    pub body_color2: u8,
    pub health: f32,
    pub interior_id: u8,
    pub door_damage_status: i32,
    pub panel_damage_status: i32,
    pub light_damage_status: u8,
    pub tire_damage_status: u8,
    pub add_siren: u8,
    pub mod_slots: [u8; 14],
    pub paint_job: u8,
    pub interior_color1: i32,
    pub interior_color2: i32,
}

/// MoonLoader's `onVehicleStreamIn` payload (RPC 164).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleStreamIn {
    pub vehicle_id: u16,
    pub vehicle: StreamedVehicle,
}
