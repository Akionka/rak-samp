//! R1-specific SDK-owned incoming RPC descriptors and payloads.

mod types;

pub use types::{
    MAX_OBJECT_MATERIAL_TEXT_BYTES, MAX_OBJECT_MATERIALS, Object, ObjectAttachment, ObjectMaterial,
    ObjectMaterialUpdate, TextLabel3D, TextMaterial, TextureMaterial,
};

use crate::events::core::PayloadWriter;
use crate::{
    HostApi,
    events::{EncodedPayload, Event, EventError, IncomingRpc, MAX_ENCODED_STRING_BYTES, Vector3},
};

fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    Ok(Vector3 {
        x: event.read_f32()?,
        y: event.read_f32()?,
        z: event.read_f32()?,
    })
}

fn decode_bool8(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_u8()? != 0)
}

fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

/// The R1 `onCreate3DText` descriptor.
pub const CREATE_3D_TEXT: IncomingRpc<TextLabel3D> =
    IncomingRpc::new_bits(36, decode_text_label_3d, encode_text_label_3d);
/// The R1 `onCreateObject` descriptor.
pub const CREATE_OBJECT: IncomingRpc<Object> =
    IncomingRpc::new_bits(44, decode_object, encode_object);
/// The R1 object material descriptor. [`ObjectMaterial`] preserves either material variant.
pub const SET_OBJECT_MATERIAL: IncomingRpc<ObjectMaterialUpdate> = IncomingRpc::new_bits(
    84,
    decode_object_material_update,
    encode_object_material_update,
);

fn write_i32(writer: &mut PayloadWriter, value: i32) {
    writer.u32(value as u32);
}

fn decode_text_label_3d(event: &mut Event<'_>) -> Result<TextLabel3D, EventError> {
    Ok(TextLabel3D {
        id: event.read_u16()?,
        color: decode_i32(event)?,
        position: decode_vector3(event)?,
        distance: event.read_f32()?,
        test_los: decode_bool8(event)?,
        attached_player_id: event.read_u16()?,
        attached_vehicle_id: event.read_u16()?,
        text: event.read_encoded_string(MAX_ENCODED_STRING_BYTES + 1)?,
    })
}

fn encode_text_label_3d(api: HostApi, value: TextLabel3D) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.id);
    write_i32(&mut writer, value.color);
    writer.vector3(value.position);
    writer.f32(value.distance);
    writer.u8(u8::from(value.test_los));
    writer.u16(value.attached_player_id);
    writer.u16(value.attached_vehicle_id);
    writer.encoded_string(api, &value.text)?;
    Ok(writer.finish_bits())
}

fn decode_texture_material(event: &mut Event<'_>) -> Result<TextureMaterial, EventError> {
    Ok(TextureMaterial {
        material_id: event.read_u8()?,
        model_id: event.read_u16()?,
        library_name: event.read_string8()?,
        texture_name: event.read_string8()?,
        color: decode_i32(event)?,
    })
}

fn encode_texture_material(
    writer: &mut PayloadWriter,
    value: TextureMaterial,
) -> Result<(), EventError> {
    writer.u8(1);
    writer.u8(value.material_id);
    writer.u16(value.model_id);
    writer.string8(&value.library_name)?;
    writer.string8(&value.texture_name)?;
    write_i32(writer, value.color);
    Ok(())
}

fn decode_text_material(event: &mut Event<'_>) -> Result<TextMaterial, EventError> {
    Ok(TextMaterial {
        material_id: event.read_u8()?,
        material_size: event.read_u8()?,
        font_name: event.read_string8()?,
        font_size: event.read_u8()?,
        bold: event.read_u8()?,
        font_color: decode_i32(event)?,
        background_color: decode_i32(event)?,
        align: event.read_u8()?,
        text: event.read_encoded_string(MAX_OBJECT_MATERIAL_TEXT_BYTES + 1)?,
    })
}

fn encode_text_material(
    api: HostApi,
    writer: &mut PayloadWriter,
    value: TextMaterial,
) -> Result<(), EventError> {
    writer.u8(2);
    writer.u8(value.material_id);
    writer.u8(value.material_size);
    writer.string8(&value.font_name)?;
    writer.u8(value.font_size);
    writer.u8(value.bold);
    write_i32(writer, value.font_color);
    write_i32(writer, value.background_color);
    writer.u8(value.align);
    writer.encoded_string_with_limit(api, &value.text, MAX_OBJECT_MATERIAL_TEXT_BYTES)
}

fn decode_object_material(event: &mut Event<'_>) -> Result<ObjectMaterial, EventError> {
    match event.read_u8()? {
        1 => Ok(ObjectMaterial::Texture(decode_texture_material(event)?)),
        2 => Ok(ObjectMaterial::Text(decode_text_material(event)?)),
        value => Err(EventError::InvalidDiscriminant { value }),
    }
}

fn encode_object_material(
    api: HostApi,
    writer: &mut PayloadWriter,
    value: ObjectMaterial,
) -> Result<(), EventError> {
    match value {
        ObjectMaterial::Texture(value) => encode_texture_material(writer, value),
        ObjectMaterial::Text(value) => encode_text_material(api, writer, value),
    }
}

fn decode_object(event: &mut Event<'_>) -> Result<Object, EventError> {
    let object_id = event.read_u16()?;
    let model_id = decode_i32(event)?;
    let position = decode_vector3(event)?;
    let rotation = decode_vector3(event)?;
    let draw_distance = event.read_f32()?;
    let no_camera_collision = decode_bool8(event)?;
    let attach_to_vehicle_id = event.read_u16()?;
    let attach_to_object_id = event.read_u16()?;
    let attachment = (attach_to_vehicle_id != u16::MAX || attach_to_object_id != u16::MAX)
        .then(|| {
            Ok(ObjectAttachment {
                offsets: decode_vector3(event)?,
                rotation: decode_vector3(event)?,
                sync_rotation: decode_bool8(event)?,
            })
        })
        .transpose()?;
    let textures_count = event.read_u8()?;
    let mut materials = Vec::new();
    while event.remaining_bits() != 0 {
        if materials.len() == MAX_OBJECT_MATERIALS {
            return Err(EventError::LengthExceedsLimit {
                length: materials.len() + 1,
                limit: MAX_OBJECT_MATERIALS,
            });
        }
        materials.push(decode_object_material(event)?);
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

fn encode_object(api: HostApi, value: Object) -> Result<EncodedPayload, EventError> {
    if value.materials.len() > MAX_OBJECT_MATERIALS {
        return Err(EventError::LengthExceedsLimit {
            length: value.materials.len(),
            limit: MAX_OBJECT_MATERIALS,
        });
    }
    let attachment_required =
        value.attach_to_vehicle_id != u16::MAX || value.attach_to_object_id != u16::MAX;
    if attachment_required != value.attachment.is_some() {
        return Err(EventError::ValueOutOfRange {
            value: usize::from(value.attachment.is_some()),
            maximum: usize::from(attachment_required),
        });
    }
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    write_i32(&mut writer, value.model_id);
    writer.vector3(value.position);
    writer.vector3(value.rotation);
    writer.f32(value.draw_distance);
    writer.u8(u8::from(value.no_camera_collision));
    writer.u16(value.attach_to_vehicle_id);
    writer.u16(value.attach_to_object_id);
    if let Some(attachment) = value.attachment {
        writer.vector3(attachment.offsets);
        writer.vector3(attachment.rotation);
        writer.u8(u8::from(attachment.sync_rotation));
    }
    writer.u8(value.textures_count);
    for material in value.materials {
        encode_object_material(api, &mut writer, material)?;
    }
    Ok(writer.finish_bits())
}

fn decode_object_material_update(
    event: &mut Event<'_>,
) -> Result<ObjectMaterialUpdate, EventError> {
    Ok(ObjectMaterialUpdate {
        object_id: event.read_u16()?,
        material: decode_object_material(event)?,
    })
}

fn encode_object_material_update(
    api: HostApi,
    value: ObjectMaterialUpdate,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    encode_object_material(api, &mut writer, value.material)?;
    Ok(writer.finish_bits())
}
