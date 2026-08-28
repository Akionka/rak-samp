use crate::events::{Vector2, Vector3};

/// The maximum number of rows that R1 menus can expose per column.
pub const MAX_MENU_ROWS: usize = 12;
/// The R1 client accepts at most two menu columns.
pub const MAX_MENU_COLUMNS: usize = 2;
/// SA-MP objects expose at most sixteen material slots.
pub const MAX_OBJECT_MATERIALS: usize = 16;
/// R1's material-text codec accepts at most 2,047 payload bytes.
pub const MAX_OBJECT_MATERIAL_TEXT_BYTES: usize = 2_047;

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

/// MoonLoader's `onApplyActorAnimation` payload (RPC 173).
#[derive(Clone, Debug, PartialEq)]
pub struct ActorAnimation {
    pub actor_id: u16,
    pub animation: Animation,
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
