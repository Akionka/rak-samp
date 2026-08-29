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

struct InitMenuCodec;
struct InterpolateCameraCodec;
struct ToggleSelectTextDrawCodec;
struct EnterEditObjectCodec;
struct ShowTextDrawCodec;
struct TextDrawHideCodec;
struct ToggleCameraTargetNotifyingCodec;
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
    ToggleCameraTargetNotifyingRpc,
    TOGGLE_CAMERA_TARGET_NOTIFYING,
    170,
    ToggleCameraTargetNotifyingCodec,
    bool,
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
    ToggleCameraTargetNotifyingCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);
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

mod vehicle;

pub use vehicle::{
    DISABLE_VEHICLE_COLLISIONS, DisableVehicleCollisionsRpc, StreamedVehicle, VEHICLE_STREAM_IN,
    VehicleStreamIn, VehicleStreamInRpc,
};
mod session;

pub use session::{
    ENABLE_STUNT_BONUS, EnableStuntBonusRpc, GameSettings, INIT_GAME, InitGame, InitGameRpc,
    MAX_SCORE_PING_ENTRIES, REQUEST_CLASS_RESPONSE, RequestClassResponse, RequestClassResponseRpc,
    SET_SPAWN_INFO, ScorePing, ScoresAndPings, ScoresAndPingsRpc, SpawnInfo, SpawnInfoRpc,
    UPDATE_SCORES_AND_PINGS,
};
mod actor;

pub use actor::{APPLY_ACTOR_ANIMATION, ActorAnimation, ApplyActorAnimationRpc};
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
