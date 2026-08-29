//! R1 incoming RPC codecs.

mod wire;

use wire::{
    decode_bit_bool, decode_bool32, encode_bit_bool, encode_bool32, read_bool8, read_bool32,
    read_fixed, write_bool8, write_bool32,
};

use crate::limits::{MAX_ENCODED_STRING_BYTES, MAX_STRING32_BYTES};
use crate::types::{Vector2, Vector3};
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, EncodedStringRead, EncodedStringWireCodec,
    EncodedStringWireDescriptor, EncodedStringWrite, ExactBitsPolicy, ExactBytesPolicy,
    TrailingPolicy, WireCodec, WireKind, WireReadExt, WireWriteExt,
    encoded_string::{read_encoded_string, write_encoded_string},
};

/// The server can send at most one score/ping entry for each R1 player slot.
pub const MAX_SCORE_PING_ENTRIES: usize = 1_000;
/// The maximum number of rows that R1 menus can expose per column.
pub const MAX_MENU_ROWS: usize = 12;
/// The R1 client accepts at most two menu columns.
pub const MAX_MENU_COLUMNS: usize = 2;
/// SA-MP objects expose at most sixteen material slots.
pub const MAX_OBJECT_MATERIALS: usize = 16;
/// R1 material text accepts at most 2,047 logical bytes.
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

/// Object attachment fields present only for an attached R1 object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectAttachment {
    pub offsets: Vector3,
    pub rotation: Vector3,
    pub sync_rotation: bool,
}

/// A texture-based R1 object material.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureMaterial {
    pub material_id: u8,
    pub model_id: u16,
    pub library_name: Vec<u8>,
    pub texture_name: Vec<u8>,
    pub color: i32,
}

/// A text-based R1 object material.
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

/// One R1 object material, preserving texture/text ordering.
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

/// R1's `onInitGame` payload (RPC 139).
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

/// R1's `onRequestClassResponse` payload (RPC 128).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestClassResponse {
    pub can_spawn: bool,
    pub spawn: SpawnInfo,
}

/// One score and ping record sent by RPC 155.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScorePing {
    pub player_id: u16,
    pub score: i32,
    pub ping: i32,
}

/// R1's `onUpdateScoresAndPings` payload (RPC 155), retained in wire order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoresAndPings {
    pub entries: Vec<ScorePing>,
}

/// One column in an R1 menu initialization payload.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuColumn {
    pub width: f32,
    pub title: [u8; 32],
    pub rows: Vec<[u8; 32]>,
}

/// R1's `onInitMenu` payload (RPC 76).
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

/// R1's `onInterpolateCamera` payload (RPC 82).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterpolateCamera {
    pub set_position: bool,
    pub from_position: Vector3,
    pub destination: Vector3,
    pub time_ms: i32,
    pub mode: u8,
}

/// R1's `onToggleSelectTextDraw` payload (RPC 83).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleSelectTextDraw {
    pub enabled: bool,
    pub hover_color: i32,
}

/// R1's `onEnterEditObject` payload (RPC 117).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// R1's `onShowTextDraw` payload (RPC 134).
#[derive(Clone, Debug, PartialEq)]
pub struct ShowTextDraw {
    pub textdraw_id: u16,
    pub textdraw: TextDraw,
}

/// R1's `onVehicleStreamIn` vehicle data (RPC 164).
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

/// R1's `onVehicleStreamIn` payload (RPC 164).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleStreamIn {
    pub vehicle_id: u16,
    pub vehicle: StreamedVehicle,
}

/// R1's `onApplyActorAnimation` payload (RPC 173).
#[derive(Clone, Debug, PartialEq)]
pub struct ActorAnimation {
    pub actor_id: u16,
    pub animation: Animation,
}

struct InitGameCodec;
struct RequestClassResponseCodec;
struct SpawnInfoCodec;
struct EnableStuntBonusCodec;
struct ScoresAndPingsCodec;
struct InitMenuCodec;
struct InterpolateCameraCodec;
struct ToggleSelectTextDrawCodec;
struct EnterEditObjectCodec;
struct ShowTextDrawCodec;
struct TextDrawHideCodec;
struct VehicleStreamInCodec;
struct DisableVehicleCollisionsCodec;
struct ToggleCameraTargetNotifyingCodec;
struct ActorAnimationCodec;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ident, $value:ty, $policy:ident) => {
        crate::wire::nominal_descriptor!(
            incoming rpc,
            $name,
            $constant,
            $id,
            $codec,
            $value,
            $policy
        );
    };
}

descriptor!(
    InitGameRpc,
    INIT_GAME,
    139,
    InitGameCodec,
    InitGame,
    ExactBitsPolicy
);
descriptor!(
    RequestClassResponseRpc,
    REQUEST_CLASS_RESPONSE,
    128,
    RequestClassResponseCodec,
    RequestClassResponse,
    ExactBitsPolicy
);
descriptor!(
    SpawnInfoRpc,
    SET_SPAWN_INFO,
    68,
    SpawnInfoCodec,
    SpawnInfo,
    ExactBitsPolicy
);
descriptor!(
    EnableStuntBonusRpc,
    ENABLE_STUNT_BONUS,
    104,
    EnableStuntBonusCodec,
    bool,
    ExactBitsPolicy
);
descriptor!(
    ScoresAndPingsRpc,
    UPDATE_SCORES_AND_PINGS,
    155,
    ScoresAndPingsCodec,
    ScoresAndPings,
    ExactBitsPolicy
);
descriptor!(
    InitMenuRpc,
    INIT_MENU,
    76,
    InitMenuCodec,
    InitMenu,
    ExactBytesPolicy
);
descriptor!(
    InterpolateCameraRpc,
    INTERPOLATE_CAMERA,
    82,
    InterpolateCameraCodec,
    InterpolateCamera,
    ExactBitsPolicy
);
descriptor!(
    ToggleSelectTextDrawRpc,
    TOGGLE_SELECT_TEXT_DRAW,
    83,
    ToggleSelectTextDrawCodec,
    ToggleSelectTextDraw,
    ExactBitsPolicy
);
descriptor!(
    EnterEditObjectRpc,
    ENTER_EDIT_OBJECT,
    117,
    EnterEditObjectCodec,
    EnterEditObject,
    ExactBitsPolicy
);
descriptor!(
    ShowTextDrawRpc,
    SHOW_TEXT_DRAW,
    134,
    ShowTextDrawCodec,
    ShowTextDraw,
    ExactBytesPolicy
);
descriptor!(
    TextDrawHideRpc,
    TEXT_DRAW_HIDE,
    135,
    TextDrawHideCodec,
    u16,
    ExactBytesPolicy
);
descriptor!(
    VehicleStreamInRpc,
    VEHICLE_STREAM_IN,
    164,
    VehicleStreamInCodec,
    VehicleStreamIn,
    ExactBytesPolicy
);
descriptor!(
    DisableVehicleCollisionsRpc,
    DISABLE_VEHICLE_COLLISIONS,
    167,
    DisableVehicleCollisionsCodec,
    bool,
    ExactBitsPolicy
);
descriptor!(
    ToggleCameraTargetNotifyingRpc,
    TOGGLE_CAMERA_TARGET_NOTIFYING,
    170,
    ToggleCameraTargetNotifyingCodec,
    bool,
    ExactBitsPolicy
);
descriptor!(
    ApplyActorAnimationRpc,
    APPLY_ACTOR_ANIMATION,
    173,
    ActorAnimationCodec,
    ActorAnimation,
    ExactBitsPolicy
);

macro_rules! r1_codec {
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

r1_codec!(InitGameCodec, InitGame, decode_init_game, encode_init_game);
r1_codec!(
    RequestClassResponseCodec,
    RequestClassResponse,
    decode_request_class_response,
    encode_request_class_response
);
r1_codec!(
    SpawnInfoCodec,
    SpawnInfo,
    decode_spawn_info,
    encode_spawn_info
);
r1_codec!(
    EnableStuntBonusCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);
r1_codec!(
    ScoresAndPingsCodec,
    ScoresAndPings,
    decode_scores_and_pings,
    encode_scores_and_pings
);
r1_codec!(InitMenuCodec, InitMenu, decode_init_menu, encode_init_menu);
r1_codec!(
    InterpolateCameraCodec,
    InterpolateCamera,
    decode_interpolate_camera,
    encode_interpolate_camera
);
r1_codec!(
    ToggleSelectTextDrawCodec,
    ToggleSelectTextDraw,
    decode_toggle_select_text_draw,
    encode_toggle_select_text_draw
);
r1_codec!(
    EnterEditObjectCodec,
    EnterEditObject,
    decode_enter_edit_object,
    encode_enter_edit_object
);
r1_codec!(
    ShowTextDrawCodec,
    ShowTextDraw,
    decode_show_text_draw,
    encode_show_text_draw
);
r1_codec!(TextDrawHideCodec, u16, decode_u16, encode_u16);
r1_codec!(
    VehicleStreamInCodec,
    VehicleStreamIn,
    decode_vehicle_stream_in,
    encode_vehicle_stream_in
);
r1_codec!(
    DisableVehicleCollisionsCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);
r1_codec!(
    ToggleCameraTargetNotifyingCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);
r1_codec!(
    ActorAnimationCodec,
    ActorAnimation,
    decode_actor_animation,
    encode_actor_animation
);

fn decode_init_game<R: BitRead>(reader: &mut R) -> Result<InitGame, DecodeError<R::Error>> {
    let mut settings = GameSettings {
        zone_names: reader.read_bit_bool()?,
        use_cj_walk: reader.read_bit_bool()?,
        allow_weapons: reader.read_bit_bool()?,
        limit_global_chat_radius: reader.read_bit_bool()?,
        global_chat_radius: reader.read_f32_le()?,
        stunt_bonus: reader.read_bit_bool()?,
        nametag_draw_distance: reader.read_f32_le()?,
        disable_enter_exits: reader.read_bit_bool()?,
        nametag_los: reader.read_bit_bool()?,
        tire_popping: reader.read_bit_bool()?,
        classes_available: reader.read_i32_le()?,
        show_player_tags: false,
        player_markers_mode: 0,
        world_time: 0,
        world_weather: 0,
        gravity: 0.0,
        lan_mode: false,
        death_money_drop: 0,
        instagib: false,
        normal_onfoot_send_rate: 0,
        normal_incar_send_rate: 0,
        normal_firing_send_rate: 0,
        send_multiplier: 0,
        lag_compensation_mode: 0,
        vehicle_friendly_fire: false,
    };
    let player_id = reader.read_u16_le()?;
    settings.show_player_tags = reader.read_bit_bool()?;
    settings.player_markers_mode = reader.read_i32_le()?;
    settings.world_time = reader.read_u8()?;
    settings.world_weather = reader.read_u8()?;
    settings.gravity = reader.read_f32_le()?;
    settings.lan_mode = reader.read_bit_bool()?;
    settings.death_money_drop = reader.read_i32_le()?;
    settings.instagib = reader.read_bit_bool()?;
    settings.normal_onfoot_send_rate = reader.read_i32_le()?;
    settings.normal_incar_send_rate = reader.read_i32_le()?;
    settings.normal_firing_send_rate = reader.read_i32_le()?;
    settings.send_multiplier = reader.read_i32_le()?;
    settings.lag_compensation_mode = reader.read_i32_le()?;
    let host_name = reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?;
    let vehicle_models = read_fixed(reader)?;
    settings.vehicle_friendly_fire = read_bool32(reader)?;
    Ok(InitGame {
        player_id,
        host_name,
        settings,
        vehicle_models,
    })
}

fn encode_init_game<W: BitWrite>(
    writer: &mut W,
    value: &InitGame,
) -> Result<(), EncodeError<W::Error>> {
    let settings = value.settings;
    writer.write_bit_bool(settings.zone_names)?;
    writer.write_bit_bool(settings.use_cj_walk)?;
    writer.write_bit_bool(settings.allow_weapons)?;
    writer.write_bit_bool(settings.limit_global_chat_radius)?;
    writer.write_f32_le(settings.global_chat_radius)?;
    writer.write_bit_bool(settings.stunt_bonus)?;
    writer.write_f32_le(settings.nametag_draw_distance)?;
    writer.write_bit_bool(settings.disable_enter_exits)?;
    writer.write_bit_bool(settings.nametag_los)?;
    writer.write_bit_bool(settings.tire_popping)?;
    writer.write_i32_le(settings.classes_available)?;
    writer.write_u16_le(value.player_id)?;
    writer.write_bit_bool(settings.show_player_tags)?;
    writer.write_i32_le(settings.player_markers_mode)?;
    writer.write_u8(settings.world_time)?;
    writer.write_u8(settings.world_weather)?;
    writer.write_f32_le(settings.gravity)?;
    writer.write_bit_bool(settings.lan_mode)?;
    writer.write_i32_le(settings.death_money_drop)?;
    writer.write_bit_bool(settings.instagib)?;
    writer.write_i32_le(settings.normal_onfoot_send_rate)?;
    writer.write_i32_le(settings.normal_incar_send_rate)?;
    writer.write_i32_le(settings.normal_firing_send_rate)?;
    writer.write_i32_le(settings.send_multiplier)?;
    writer.write_i32_le(settings.lag_compensation_mode)?;
    writer.write_len_prefixed_bytes_u8(&value.host_name, usize::from(u8::MAX))?;
    writer.write_bytes(&value.vehicle_models)?;
    write_bool32(writer, settings.vehicle_friendly_fire)
}

fn decode_request_class_response<R: BitRead>(
    reader: &mut R,
) -> Result<RequestClassResponse, DecodeError<R::Error>> {
    Ok(RequestClassResponse {
        can_spawn: read_bool8(reader)?,
        spawn: decode_spawn_info(reader)?,
    })
}

fn encode_request_class_response<W: BitWrite>(
    writer: &mut W,
    value: &RequestClassResponse,
) -> Result<(), EncodeError<W::Error>> {
    write_bool8(writer, value.can_spawn)?;
    encode_spawn_info(writer, &value.spawn)
}

fn decode_spawn_info<R: BitRead>(reader: &mut R) -> Result<SpawnInfo, DecodeError<R::Error>> {
    Ok(SpawnInfo {
        team: reader.read_u8()?,
        skin: reader.read_i32_le()?,
        unused: reader.read_u8()?,
        position: reader.read_vector3_le()?,
        rotation: reader.read_f32_le()?,
        weapons: [
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
        ],
        ammo: [
            reader.read_i32_le()?,
            reader.read_i32_le()?,
            reader.read_i32_le()?,
        ],
    })
}

fn encode_spawn_info<W: BitWrite>(
    writer: &mut W,
    value: &SpawnInfo,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.team)?;
    writer.write_i32_le(value.skin)?;
    writer.write_u8(value.unused)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.rotation)?;
    for weapon in value.weapons {
        writer.write_i32_le(weapon)?;
    }
    for ammo in value.ammo {
        writer.write_i32_le(ammo)?;
    }
    Ok(())
}

fn decode_scores_and_pings<R: BitRead>(
    reader: &mut R,
) -> Result<ScoresAndPings, DecodeError<R::Error>> {
    let remaining_bits = reader.remaining_bits();
    if !remaining_bits.is_multiple_of(80) {
        return Err(DecodeError::UnexpectedTrailingBits {
            remaining_bits,
            allowed_bits: 0,
        });
    }
    let count = remaining_bits / 80;
    if count > MAX_SCORE_PING_ENTRIES {
        return Err(DecodeError::LengthExceedsLimit {
            length: count,
            limit: MAX_SCORE_PING_ENTRIES,
        });
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(ScorePing {
            player_id: reader.read_u16_le()?,
            score: reader.read_i32_le()?,
            ping: reader.read_i32_le()?,
        });
    }
    Ok(ScoresAndPings { entries })
}

fn encode_scores_and_pings<W: BitWrite>(
    writer: &mut W,
    value: &ScoresAndPings,
) -> Result<(), EncodeError<W::Error>> {
    if value.entries.len() > MAX_SCORE_PING_ENTRIES {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.entries.len(),
            limit: MAX_SCORE_PING_ENTRIES,
        });
    }
    for entry in &value.entries {
        writer.write_u16_le(entry.player_id)?;
        writer.write_i32_le(entry.score)?;
        writer.write_i32_le(entry.ping)?;
    }
    Ok(())
}

fn decode_menu_column<R: BitRead>(
    reader: &mut R,
    width: f32,
) -> Result<MenuColumn, DecodeError<R::Error>> {
    let title = read_fixed(reader)?;
    let row_count = usize::from(reader.read_u8()?);
    if row_count > MAX_MENU_ROWS {
        return Err(DecodeError::LengthExceedsLimit {
            length: row_count,
            limit: MAX_MENU_ROWS,
        });
    }
    let mut rows = Vec::with_capacity(row_count);
    for _ in 0..row_count {
        rows.push(read_fixed(reader)?);
    }
    Ok(MenuColumn { width, title, rows })
}

fn encode_menu_column<W: BitWrite>(
    writer: &mut W,
    value: &MenuColumn,
) -> Result<(), EncodeError<W::Error>> {
    if value.rows.len() > MAX_MENU_ROWS {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.rows.len(),
            limit: MAX_MENU_ROWS,
        });
    }
    writer.write_bytes(&value.title)?;
    writer.write_u8(value.rows.len() as u8)?;
    for row in &value.rows {
        writer.write_bytes(row)?;
    }
    Ok(())
}

fn decode_init_menu<R: BitRead>(reader: &mut R) -> Result<InitMenu, DecodeError<R::Error>> {
    let menu_id = reader.read_u8()?;
    let two_columns = read_bool32(reader)?;
    let title = read_fixed(reader)?;
    let position = reader.read_vector2_le()?;
    let first_width = reader.read_f32_le()?;
    let second_width = two_columns.then(|| reader.read_f32_le()).transpose()?;
    let menu = read_bool32(reader)?;
    let mut rows = [0; MAX_MENU_ROWS];
    for row in &mut rows {
        *row = reader.read_i32_le()?;
    }
    let mut columns = Vec::with_capacity(if two_columns { 2 } else { 1 });
    columns.push(decode_menu_column(reader, first_width)?);
    if let Some(width) = second_width {
        columns.push(decode_menu_column(reader, width)?);
    }
    Ok(InitMenu {
        menu_id,
        two_columns,
        title,
        position,
        columns,
        rows,
        menu,
    })
}

fn encode_init_menu<W: BitWrite>(
    writer: &mut W,
    value: &InitMenu,
) -> Result<(), EncodeError<W::Error>> {
    if value.columns.len() > MAX_MENU_COLUMNS {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.columns.len(),
            limit: MAX_MENU_COLUMNS,
        });
    }
    let expected_columns = if value.two_columns { 2 } else { 1 };
    if value.columns.len() != expected_columns {
        return Err(EncodeError::InvalidCollectionLength {
            length: value.columns.len(),
            expected: expected_columns,
        });
    }
    let first = &value.columns[0];
    writer.write_u8(value.menu_id)?;
    write_bool32(writer, value.two_columns)?;
    writer.write_bytes(&value.title)?;
    writer.write_vector2_le(&value.position)?;
    writer.write_f32_le(first.width)?;
    if let Some(second) = value.columns.get(1) {
        writer.write_f32_le(second.width)?;
    }
    write_bool32(writer, value.menu)?;
    for row in value.rows {
        writer.write_i32_le(row)?;
    }
    encode_menu_column(writer, first)?;
    if let Some(second) = value.columns.get(1) {
        encode_menu_column(writer, second)?;
    }
    Ok(())
}

fn decode_interpolate_camera<R: BitRead>(
    reader: &mut R,
) -> Result<InterpolateCamera, DecodeError<R::Error>> {
    Ok(InterpolateCamera {
        set_position: reader.read_bit_bool()?,
        from_position: reader.read_vector3_le()?,
        destination: reader.read_vector3_le()?,
        time_ms: reader.read_i32_le()?,
        mode: reader.read_u8()?,
    })
}

fn encode_interpolate_camera<W: BitWrite>(
    writer: &mut W,
    value: &InterpolateCamera,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bit_bool(value.set_position)?;
    writer.write_vector3_le(&value.from_position)?;
    writer.write_vector3_le(&value.destination)?;
    writer.write_i32_le(value.time_ms)?;
    writer.write_u8(value.mode)
}

fn decode_toggle_select_text_draw<R: BitRead>(
    reader: &mut R,
) -> Result<ToggleSelectTextDraw, DecodeError<R::Error>> {
    Ok(ToggleSelectTextDraw {
        enabled: reader.read_bit_bool()?,
        hover_color: reader.read_i32_le()?,
    })
}

fn encode_toggle_select_text_draw<W: BitWrite>(
    writer: &mut W,
    value: &ToggleSelectTextDraw,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bit_bool(value.enabled)?;
    writer.write_i32_le(value.hover_color)
}

fn decode_enter_edit_object<R: BitRead>(
    reader: &mut R,
) -> Result<EnterEditObject, DecodeError<R::Error>> {
    Ok(EnterEditObject {
        player_object: reader.read_bit_bool()?,
        object_id: reader.read_u16_le()?,
    })
}

fn encode_enter_edit_object<W: BitWrite>(
    writer: &mut W,
    value: &EnterEditObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bit_bool(value.player_object)?;
    writer.write_u16_le(value.object_id)
}

fn decode_show_text_draw<R: BitRead>(
    reader: &mut R,
) -> Result<ShowTextDraw, DecodeError<R::Error>> {
    Ok(ShowTextDraw {
        textdraw_id: reader.read_u16_le()?,
        textdraw: TextDraw {
            flags: reader.read_u8()?,
            letter_width: reader.read_f32_le()?,
            letter_height: reader.read_f32_le()?,
            letter_color: reader.read_i32_le()?,
            line_width: reader.read_f32_le()?,
            line_height: reader.read_f32_le()?,
            box_color: reader.read_i32_le()?,
            shadow: reader.read_u8()?,
            outline: reader.read_u8()?,
            background_color: reader.read_i32_le()?,
            style: reader.read_u8()?,
            selectable: reader.read_u8()?,
            position: reader.read_vector2_le()?,
            model_id: reader.read_u16_le()?,
            rotation: reader.read_vector3_le()?,
            zoom: reader.read_f32_le()?,
            color1: reader.read_i16_le()?,
            color2: reader.read_i16_le()?,
            text: reader.read_len_prefixed_bytes_u16_le(MAX_STRING32_BYTES)?,
        },
    })
}

fn encode_show_text_draw<W: BitWrite>(
    writer: &mut W,
    value: &ShowTextDraw,
) -> Result<(), EncodeError<W::Error>> {
    let textdraw = &value.textdraw;
    writer.write_u16_le(value.textdraw_id)?;
    writer.write_u8(textdraw.flags)?;
    writer.write_f32_le(textdraw.letter_width)?;
    writer.write_f32_le(textdraw.letter_height)?;
    writer.write_i32_le(textdraw.letter_color)?;
    writer.write_f32_le(textdraw.line_width)?;
    writer.write_f32_le(textdraw.line_height)?;
    writer.write_i32_le(textdraw.box_color)?;
    writer.write_u8(textdraw.shadow)?;
    writer.write_u8(textdraw.outline)?;
    writer.write_i32_le(textdraw.background_color)?;
    writer.write_u8(textdraw.style)?;
    writer.write_u8(textdraw.selectable)?;
    writer.write_vector2_le(&textdraw.position)?;
    writer.write_u16_le(textdraw.model_id)?;
    writer.write_vector3_le(&textdraw.rotation)?;
    writer.write_f32_le(textdraw.zoom)?;
    writer.write_i16_le(textdraw.color1)?;
    writer.write_i16_le(textdraw.color2)?;
    writer.write_len_prefixed_bytes_u16_le(&textdraw.text, MAX_STRING32_BYTES)
}

fn decode_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    reader.read_u16_le()
}

fn encode_u16<W: BitWrite>(writer: &mut W, value: &u16) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(*value)
}

fn decode_vehicle_stream_in<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleStreamIn, DecodeError<R::Error>> {
    Ok(VehicleStreamIn {
        vehicle_id: reader.read_u16_le()?,
        vehicle: StreamedVehicle {
            model: reader.read_i32_le()?,
            position: reader.read_vector3_le()?,
            rotation: reader.read_f32_le()?,
            body_color1: reader.read_u8()?,
            body_color2: reader.read_u8()?,
            health: reader.read_f32_le()?,
            interior_id: reader.read_u8()?,
            door_damage_status: reader.read_i32_le()?,
            panel_damage_status: reader.read_i32_le()?,
            light_damage_status: reader.read_u8()?,
            tire_damage_status: reader.read_u8()?,
            add_siren: reader.read_u8()?,
            mod_slots: read_fixed(reader)?,
            paint_job: reader.read_u8()?,
            interior_color1: reader.read_i32_le()?,
            interior_color2: reader.read_i32_le()?,
        },
    })
}

fn encode_vehicle_stream_in<W: BitWrite>(
    writer: &mut W,
    value: &VehicleStreamIn,
) -> Result<(), EncodeError<W::Error>> {
    let vehicle = &value.vehicle;
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_i32_le(vehicle.model)?;
    writer.write_vector3_le(&vehicle.position)?;
    writer.write_f32_le(vehicle.rotation)?;
    writer.write_u8(vehicle.body_color1)?;
    writer.write_u8(vehicle.body_color2)?;
    writer.write_f32_le(vehicle.health)?;
    writer.write_u8(vehicle.interior_id)?;
    writer.write_i32_le(vehicle.door_damage_status)?;
    writer.write_i32_le(vehicle.panel_damage_status)?;
    writer.write_u8(vehicle.light_damage_status)?;
    writer.write_u8(vehicle.tire_damage_status)?;
    writer.write_u8(vehicle.add_siren)?;
    writer.write_bytes(&vehicle.mod_slots)?;
    writer.write_u8(vehicle.paint_job)?;
    writer.write_i32_le(vehicle.interior_color1)?;
    writer.write_i32_le(vehicle.interior_color2)
}

fn decode_actor_animation<R: BitRead>(
    reader: &mut R,
) -> Result<ActorAnimation, DecodeError<R::Error>> {
    Ok(ActorAnimation {
        actor_id: reader.read_u16_le()?,
        animation: decode_animation(reader)?,
    })
}

fn encode_actor_animation<W: BitWrite>(
    writer: &mut W,
    value: &ActorAnimation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    encode_animation(writer, &value.animation)
}

struct Create3DTextCodec;
struct CreateObjectCodec;
struct SetObjectMaterialCodec;

impl EncodedStringWireCodec for Create3DTextCodec {
    type Value = TextLabel3D;

    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        Ok(TextLabel3D {
            id: reader.read_u16_le()?,
            color: reader.read_i32_le()?,
            position: reader.read_vector3_le()?,
            distance: reader.read_f32_le()?,
            test_los: read_bool8(reader)?,
            attached_player_id: reader.read_u16_le()?,
            attached_vehicle_id: reader.read_u16_le()?,
            text: read_encoded_string(reader, MAX_ENCODED_STRING_BYTES)?,
        })
    }

    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.id)?;
        writer.write_i32_le(value.color)?;
        writer.write_vector3_le(&value.position)?;
        writer.write_f32_le(value.distance)?;
        write_bool8(writer, value.test_los)?;
        writer.write_u16_le(value.attached_player_id)?;
        writer.write_u16_le(value.attached_vehicle_id)?;
        write_encoded_string(writer, &value.text, MAX_ENCODED_STRING_BYTES)
    }
}

fn decode_texture_material<R: EncodedStringRead>(
    reader: &mut R,
) -> Result<TextureMaterial, DecodeError<R::Error>> {
    Ok(TextureMaterial {
        material_id: reader.read_u8()?,
        model_id: reader.read_u16_le()?,
        library_name: reader.read_len_prefixed_bytes_u8(u8::MAX as usize)?,
        texture_name: reader.read_len_prefixed_bytes_u8(u8::MAX as usize)?,
        color: reader.read_i32_le()?,
    })
}

fn encode_texture_material<W: EncodedStringWrite>(
    writer: &mut W,
    value: &TextureMaterial,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(1)?;
    writer.write_u8(value.material_id)?;
    writer.write_u16_le(value.model_id)?;
    writer.write_len_prefixed_bytes_u8(&value.library_name, u8::MAX as usize)?;
    writer.write_len_prefixed_bytes_u8(&value.texture_name, u8::MAX as usize)?;
    writer.write_i32_le(value.color)
}

fn decode_text_material<R: EncodedStringRead>(
    reader: &mut R,
) -> Result<TextMaterial, DecodeError<R::Error>> {
    Ok(TextMaterial {
        material_id: reader.read_u8()?,
        material_size: reader.read_u8()?,
        font_name: reader.read_len_prefixed_bytes_u8(u8::MAX as usize)?,
        font_size: reader.read_u8()?,
        bold: reader.read_u8()?,
        font_color: reader.read_i32_le()?,
        background_color: reader.read_i32_le()?,
        align: reader.read_u8()?,
        text: read_encoded_string(reader, MAX_OBJECT_MATERIAL_TEXT_BYTES)?,
    })
}

fn encode_text_material<W: EncodedStringWrite>(
    writer: &mut W,
    value: &TextMaterial,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(2)?;
    writer.write_u8(value.material_id)?;
    writer.write_u8(value.material_size)?;
    writer.write_len_prefixed_bytes_u8(&value.font_name, u8::MAX as usize)?;
    writer.write_u8(value.font_size)?;
    writer.write_u8(value.bold)?;
    writer.write_i32_le(value.font_color)?;
    writer.write_i32_le(value.background_color)?;
    writer.write_u8(value.align)?;
    write_encoded_string(writer, &value.text, MAX_OBJECT_MATERIAL_TEXT_BYTES)
}

fn decode_object_material<R: EncodedStringRead>(
    reader: &mut R,
) -> Result<ObjectMaterial, DecodeError<R::Error>> {
    match reader.read_u8()? {
        1 => Ok(ObjectMaterial::Texture(decode_texture_material(reader)?)),
        2 => Ok(ObjectMaterial::Text(decode_text_material(reader)?)),
        value => Err(DecodeError::InvalidDiscriminant { value }),
    }
}

fn encode_object_material<W: EncodedStringWrite>(
    writer: &mut W,
    value: &ObjectMaterial,
) -> Result<(), EncodeError<W::Error>> {
    match value {
        ObjectMaterial::Texture(value) => encode_texture_material(writer, value),
        ObjectMaterial::Text(value) => encode_text_material(writer, value),
    }
}

impl EncodedStringWireCodec for CreateObjectCodec {
    type Value = Object;

    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let object_id = reader.read_u16_le()?;
        let model_id = reader.read_i32_le()?;
        let position = reader.read_vector3_le()?;
        let rotation = reader.read_vector3_le()?;
        let draw_distance = reader.read_f32_le()?;
        let no_camera_collision = read_bool8(reader)?;
        let attach_to_vehicle_id = reader.read_u16_le()?;
        let attach_to_object_id = reader.read_u16_le()?;
        let attachment = (attach_to_vehicle_id != u16::MAX || attach_to_object_id != u16::MAX)
            .then(|| {
                Ok(ObjectAttachment {
                    offsets: reader.read_vector3_le()?,
                    rotation: reader.read_vector3_le()?,
                    sync_rotation: read_bool8(reader)?,
                })
            })
            .transpose()?;
        let textures_count = reader.read_u8()?;
        let mut materials = Vec::new();
        while reader.remaining_bits() != 0 {
            if materials.len() == MAX_OBJECT_MATERIALS {
                return Err(DecodeError::LengthExceedsLimit {
                    length: materials.len() + 1,
                    limit: MAX_OBJECT_MATERIALS,
                });
            }
            materials.push(decode_object_material(reader)?);
        }
        Ok(Object {
            object_id,
            model_id,
            position,
            rotation,
            draw_distance,
            no_camera_collision,
            attach_to_vehicle_id,
            attach_to_object_id,
            attachment,
            textures_count,
            materials,
        })
    }

    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        if value.materials.len() > MAX_OBJECT_MATERIALS {
            return Err(EncodeError::LengthExceedsLimit {
                length: value.materials.len(),
                limit: MAX_OBJECT_MATERIALS,
            });
        }
        let attachment_required =
            value.attach_to_vehicle_id != u16::MAX || value.attach_to_object_id != u16::MAX;
        if attachment_required != value.attachment.is_some() {
            return Err(EncodeError::InvalidFieldCombination {
                field: "attachment",
            });
        }
        writer.write_u16_le(value.object_id)?;
        writer.write_i32_le(value.model_id)?;
        writer.write_vector3_le(&value.position)?;
        writer.write_vector3_le(&value.rotation)?;
        writer.write_f32_le(value.draw_distance)?;
        write_bool8(writer, value.no_camera_collision)?;
        writer.write_u16_le(value.attach_to_vehicle_id)?;
        writer.write_u16_le(value.attach_to_object_id)?;
        if let Some(attachment) = value.attachment {
            writer.write_vector3_le(&attachment.offsets)?;
            writer.write_vector3_le(&attachment.rotation)?;
            write_bool8(writer, attachment.sync_rotation)?;
        }
        writer.write_u8(value.textures_count)?;
        for material in &value.materials {
            encode_object_material(writer, material)?;
        }
        Ok(())
    }
}

impl EncodedStringWireCodec for SetObjectMaterialCodec {
    type Value = ObjectMaterialUpdate;

    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        Ok(ObjectMaterialUpdate {
            object_id: reader.read_u16_le()?,
            material: decode_object_material(reader)?,
        })
    }

    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.object_id)?;
        encode_object_material(writer, &value.material)
    }
}

macro_rules! encoded_string_rpc_descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty, $value:ty) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        pub const $constant: $name = $name;

        impl crate::encoded_string::sealed::EncodedStringWireDescriptor<$value> for $name {
            fn decode<R: EncodedStringRead>(
                reader: &mut R,
            ) -> Result<$value, DecodeError<R::Error>> {
                <$codec as EncodedStringWireCodec>::decode(reader)
            }

            fn encode<W: EncodedStringWrite>(
                writer: &mut W,
                value: &$value,
            ) -> Result<(), EncodeError<W::Error>> {
                <$codec as EncodedStringWireCodec>::encode(writer, value)
            }
        }

        impl EncodedStringWireDescriptor for $name {
            type Value = $value;

            const ID: u8 = $id;
            const KIND: WireKind = WireKind::Rpc;
            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;
        }

        impl crate::wire::sealed::IncomingRpcDescriptor for $name {}

        impl crate::IncomingRpcDescriptor for $name {
            type Value = $value;
            type Capability = crate::EncodedStringWire;

            const ID: u8 = $id;
        }
    };
}

mod player;

pub use player::{
    APPLY_PLAYER_ANIMATION, Animation, AttachedObject, CrimeReport, CrimeReportRpc,
    PLAY_CRIME_REPORT, PLAYER_STREAM_IN, PlayerAnimation, PlayerAnimationRpc, PlayerAttachedObject,
    PlayerAttachedObjectRpc, PlayerStreamIn, PlayerStreamInRpc, SET_PLAYER_ATTACHED_OBJECT,
    TOGGLE_PLAYER_SPECTATING, TogglePlayerSpectatingRpc,
};

use player::{decode_animation, encode_animation};
encoded_string_rpc_descriptor!(
    Create3DTextRpc,
    CREATE_3D_TEXT,
    36,
    Create3DTextCodec,
    TextLabel3D
);
encoded_string_rpc_descriptor!(
    CreateObjectRpc,
    CREATE_OBJECT,
    44,
    CreateObjectCodec,
    Object
);
encoded_string_rpc_descriptor!(
    SetObjectMaterialRpc,
    SET_OBJECT_MATERIAL,
    84,
    SetObjectMaterialCodec,
    ObjectMaterialUpdate
);
