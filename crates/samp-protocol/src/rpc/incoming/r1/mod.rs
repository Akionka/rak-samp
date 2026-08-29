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

/// R1's `onEnterEditObject` payload (RPC 117).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnterEditObject {
    pub player_object: bool,
    pub object_id: u16,
}

struct EnterEditObjectCodec;
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
    EnterEditObjectRpc,
    ENTER_EDIT_OBJECT,
    117,
    EnterEditObjectCodec,
    EnterEditObject,
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

r1_codec!(
    EnterEditObjectCodec,
    EnterEditObject,
    decode_enter_edit_object,
    encode_enter_edit_object
);
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

mod camera;

pub use camera::{
    INTERPOLATE_CAMERA, InterpolateCamera, InterpolateCameraRpc, TOGGLE_CAMERA_TARGET_NOTIFYING,
    ToggleCameraTargetNotifyingRpc,
};
mod ui;

pub use ui::{
    INIT_MENU, InitMenu, InitMenuRpc, MAX_MENU_COLUMNS, MAX_MENU_ROWS, MenuColumn, SHOW_TEXT_DRAW,
    ShowTextDraw, ShowTextDrawRpc, TEXT_DRAW_HIDE, TOGGLE_SELECT_TEXT_DRAW, TextDraw,
    TextDrawHideRpc, ToggleSelectTextDraw, ToggleSelectTextDrawRpc,
};
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
