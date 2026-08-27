use super::{fixed::*, types::*};
use crate::events::core::PayloadWriter;
use crate::{
    HostApi, SampClientSdkResult,
    events::{
        EncodedPayload, Event, EventError, IncomingRpc, MAX_ENCODED_STRING_BYTES,
        MAX_STRING32_BYTES,
    },
};

fn read_array<const N: usize>(event: &mut Event<'_>) -> Result<[u8; N], EventError> {
    event
        .read_bytes(N)?
        .try_into()
        .map_err(|_| EventError::Host(SampClientSdkResult::NativeCallFailed))
}

/// The R1 `onCreate3DText` descriptor.
pub const CREATE_3D_TEXT: IncomingRpc<TextLabel3D> =
    IncomingRpc::new_bits(36, decode_text_label_3d, encode_text_label_3d);
/// The R1 `onCreateObject` descriptor.
pub const CREATE_OBJECT: IncomingRpc<Object> =
    IncomingRpc::new_bits(44, decode_object, encode_object);
/// The R1 `onInitMenu` descriptor.
pub const INIT_MENU: IncomingRpc<InitMenu> =
    IncomingRpc::new_bits(76, decode_init_menu, encode_init_menu);
/// The R1 `onInterpolateCamera` descriptor.
pub const INTERPOLATE_CAMERA: IncomingRpc<InterpolateCamera> =
    IncomingRpc::new_bits(82, decode_interpolate_camera, encode_interpolate_camera);
/// The R1 `onToggleSelectTextDraw` descriptor.
pub const TOGGLE_SELECT_TEXT_DRAW: IncomingRpc<ToggleSelectTextDraw> = IncomingRpc::new_bits(
    83,
    decode_toggle_select_text_draw,
    encode_toggle_select_text_draw,
);
/// The R1 object material descriptor. [`ObjectMaterial`] preserves either material variant.
pub const SET_OBJECT_MATERIAL: IncomingRpc<ObjectMaterialUpdate> = IncomingRpc::new_bits(
    84,
    decode_object_material_update,
    encode_object_material_update,
);
/// The R1 `onEnterEditObject` descriptor.
pub const ENTER_EDIT_OBJECT: IncomingRpc<EnterEditObject> =
    IncomingRpc::new_bits(117, decode_enter_edit_object, encode_enter_edit_object);
/// The R1 `onShowTextDraw` descriptor.
pub const SHOW_TEXT_DRAW: IncomingRpc<ShowTextDraw> =
    IncomingRpc::new_bits(134, decode_show_text_draw, encode_show_text_draw);
/// The R1 `onTextDrawHide` descriptor.
pub const TEXT_DRAW_HIDE: IncomingRpc<u16> = IncomingRpc::new(135, decode_u16, encode_u16);
/// The R1 `onVehicleStreamIn` descriptor.
pub const VEHICLE_STREAM_IN: IncomingRpc<VehicleStreamIn> =
    IncomingRpc::new_bits(164, decode_vehicle_stream_in, encode_vehicle_stream_in);
/// The R1 `onDisableVehicleCollisions` descriptor.
pub const DISABLE_VEHICLE_COLLISIONS: IncomingRpc<bool> =
    IncomingRpc::new_bits(167, decode_bit_bool, encode_bit_bool);
/// The R1 `onToggleCameraTargetNotifying` descriptor.
pub const TOGGLE_CAMERA_TARGET_NOTIFYING: IncomingRpc<bool> =
    IncomingRpc::new_bits(170, decode_bit_bool, encode_bit_bool);
/// The R1 `onApplyActorAnimation` descriptor.
pub const APPLY_ACTOR_ANIMATION: IncomingRpc<ActorAnimation> =
    IncomingRpc::new_bits(173, decode_actor_animation, encode_actor_animation);

fn read_bit_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_bits(1)?[0] & 0x80 != 0)
}

fn decode_bit_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
    read_bit_bool(event)
}

fn encode_bit_bool(_api: HostApi, value: bool) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bool(value);
    Ok(writer.finish_bits())
}

fn decode_bool32(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_u32()? != 0)
}

fn read_i16(event: &mut Event<'_>) -> Result<i16, EventError> {
    Ok(event.read_u16()? as i16)
}

fn write_i32(writer: &mut PayloadWriter, value: i32) {
    writer.u32(value as u32);
}

fn read_fixed_string32(event: &mut Event<'_>) -> Result<[u8; 32], EventError> {
    read_array(event)
}

fn write_fixed_string32(writer: &mut PayloadWriter, value: [u8; 32]) {
    writer.bytes(&value);
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

fn decode_menu_column(event: &mut Event<'_>, width: f32) -> Result<MenuColumn, EventError> {
    let title = read_fixed_string32(event)?;
    let row_count = usize::from(event.read_u8()?);
    if row_count > MAX_MENU_ROWS {
        return Err(EventError::LengthExceedsLimit {
            length: row_count,
            limit: MAX_MENU_ROWS,
        });
    }
    let mut rows = Vec::with_capacity(row_count);
    for _ in 0..row_count {
        rows.push(read_fixed_string32(event)?);
    }
    Ok(MenuColumn { width, title, rows })
}

fn encode_menu_column(writer: &mut PayloadWriter, value: MenuColumn) -> Result<(), EventError> {
    if value.rows.len() > MAX_MENU_ROWS {
        return Err(EventError::LengthExceedsLimit {
            length: value.rows.len(),
            limit: MAX_MENU_ROWS,
        });
    }
    write_fixed_string32(writer, value.title);
    writer.u8(value.rows.len() as u8);
    for row in value.rows {
        write_fixed_string32(writer, row);
    }
    Ok(())
}

fn decode_init_menu(event: &mut Event<'_>) -> Result<InitMenu, EventError> {
    let menu_id = event.read_u8()?;
    let two_columns = decode_bool32(event)?;
    let title = read_fixed_string32(event)?;
    let position = decode_vector2(event)?;
    let width1 = event.read_f32()?;
    let width2 = two_columns.then(|| event.read_f32()).transpose()?;
    let menu = decode_bool32(event)?;
    let mut rows = [0_i32; MAX_MENU_ROWS];
    for row in &mut rows {
        *row = decode_i32(event)?;
    }
    let mut columns = Vec::with_capacity(if two_columns { 2 } else { 1 });
    columns.push(decode_menu_column(event, width1)?);
    if let Some(width2) = width2 {
        columns.push(decode_menu_column(event, width2)?);
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

fn encode_init_menu(api: HostApi, value: InitMenu) -> Result<EncodedPayload, EventError> {
    let _ = api;
    let expected_columns = if value.two_columns { 2 } else { 1 };
    if value.columns.len() != expected_columns || value.columns.len() > MAX_MENU_COLUMNS {
        return Err(EventError::ValueOutOfRange {
            value: value.columns.len(),
            maximum: expected_columns,
        });
    }
    let mut columns = value.columns.into_iter();
    let first = columns.next().ok_or(EventError::ValueOutOfRange {
        value: 0,
        maximum: 1,
    })?;
    let second = columns.next();
    let mut writer = PayloadWriter::new();
    writer.u8(value.menu_id);
    writer.u32(u32::from(value.two_columns));
    write_fixed_string32(&mut writer, value.title);
    encode_vector2(&mut writer, value.position);
    writer.f32(first.width);
    if let Some(column) = second.as_ref() {
        writer.f32(column.width);
    }
    writer.u32(u32::from(value.menu));
    for row in value.rows {
        write_i32(&mut writer, row);
    }
    encode_menu_column(&mut writer, first)?;
    if let Some(column) = second {
        encode_menu_column(&mut writer, column)?;
    }
    Ok(writer.finish_bits())
}

fn decode_interpolate_camera(event: &mut Event<'_>) -> Result<InterpolateCamera, EventError> {
    Ok(InterpolateCamera {
        set_position: read_bit_bool(event)?,
        from_position: decode_vector3(event)?,
        destination: decode_vector3(event)?,
        time_ms: decode_i32(event)?,
        mode: event.read_u8()?,
    })
}

fn encode_interpolate_camera(
    _api: HostApi,
    value: InterpolateCamera,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bool(value.set_position);
    writer.vector3(value.from_position);
    writer.vector3(value.destination);
    write_i32(&mut writer, value.time_ms);
    writer.u8(value.mode);
    Ok(writer.finish_bits())
}

fn decode_toggle_select_text_draw(
    event: &mut Event<'_>,
) -> Result<ToggleSelectTextDraw, EventError> {
    Ok(ToggleSelectTextDraw {
        enabled: read_bit_bool(event)?,
        hover_color: decode_i32(event)?,
    })
}

fn encode_toggle_select_text_draw(
    _api: HostApi,
    value: ToggleSelectTextDraw,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bool(value.enabled);
    write_i32(&mut writer, value.hover_color);
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

fn decode_animation(event: &mut Event<'_>) -> Result<Animation, EventError> {
    Ok(Animation {
        animation_library: event.read_string8()?,
        animation_name: event.read_string8()?,
        frame_delta: event.read_f32()?,
        looped: read_bit_bool(event)?,
        lock_x: read_bit_bool(event)?,
        lock_y: read_bit_bool(event)?,
        freeze: read_bit_bool(event)?,
        time: decode_i32(event)?,
    })
}

fn encode_animation(writer: &mut PayloadWriter, value: Animation) -> Result<(), EventError> {
    writer.string8(&value.animation_library)?;
    writer.string8(&value.animation_name)?;
    writer.f32(value.frame_delta);
    writer.bool(value.looped);
    writer.bool(value.lock_x);
    writer.bool(value.lock_y);
    writer.bool(value.freeze);
    write_i32(writer, value.time);
    Ok(())
}

fn decode_actor_animation(event: &mut Event<'_>) -> Result<ActorAnimation, EventError> {
    Ok(ActorAnimation {
        actor_id: event.read_u16()?,
        animation: decode_animation(event)?,
    })
}

fn encode_actor_animation(
    _api: HostApi,
    value: ActorAnimation,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.actor_id);
    encode_animation(&mut writer, value.animation)?;
    Ok(writer.finish_bits())
}

fn decode_enter_edit_object(event: &mut Event<'_>) -> Result<EnterEditObject, EventError> {
    Ok(EnterEditObject {
        player_object: read_bit_bool(event)?,
        object_id: event.read_u16()?,
    })
}

fn encode_enter_edit_object(
    _api: HostApi,
    value: EnterEditObject,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bool(value.player_object);
    writer.u16(value.object_id);
    Ok(writer.finish_bits())
}

fn decode_show_text_draw(event: &mut Event<'_>) -> Result<ShowTextDraw, EventError> {
    let textdraw_id = event.read_u16()?;
    let textdraw = TextDraw {
        flags: event.read_u8()?,
        letter_width: event.read_f32()?,
        letter_height: event.read_f32()?,
        letter_color: decode_i32(event)?,
        line_width: event.read_f32()?,
        line_height: event.read_f32()?,
        box_color: decode_i32(event)?,
        shadow: event.read_u8()?,
        outline: event.read_u8()?,
        background_color: decode_i32(event)?,
        style: event.read_u8()?,
        selectable: event.read_u8()?,
        position: decode_vector2(event)?,
        model_id: event.read_u16()?,
        rotation: decode_vector3(event)?,
        zoom: event.read_f32()?,
        color1: read_i16(event)?,
        color2: read_i16(event)?,
        text: {
            let length = usize::from(event.read_u16()?);
            if length > MAX_STRING32_BYTES {
                return Err(EventError::LengthExceedsLimit {
                    length,
                    limit: MAX_STRING32_BYTES,
                });
            }
            event.read_bytes(length)?
        },
    };
    Ok(ShowTextDraw {
        textdraw_id,
        textdraw,
    })
}

fn encode_show_text_draw(_api: HostApi, value: ShowTextDraw) -> Result<EncodedPayload, EventError> {
    if value.textdraw.text.len() > MAX_STRING32_BYTES {
        return Err(EventError::LengthExceedsLimit {
            length: value.textdraw.text.len(),
            limit: MAX_STRING32_BYTES,
        });
    }
    let textdraw = value.textdraw;
    let mut writer = PayloadWriter::new();
    writer.u16(value.textdraw_id);
    writer.u8(textdraw.flags);
    writer.f32(textdraw.letter_width);
    writer.f32(textdraw.letter_height);
    write_i32(&mut writer, textdraw.letter_color);
    writer.f32(textdraw.line_width);
    writer.f32(textdraw.line_height);
    write_i32(&mut writer, textdraw.box_color);
    writer.u8(textdraw.shadow);
    writer.u8(textdraw.outline);
    write_i32(&mut writer, textdraw.background_color);
    writer.u8(textdraw.style);
    writer.u8(textdraw.selectable);
    encode_vector2(&mut writer, textdraw.position);
    writer.u16(textdraw.model_id);
    writer.vector3(textdraw.rotation);
    writer.f32(textdraw.zoom);
    writer.i16(textdraw.color1);
    writer.i16(textdraw.color2);
    writer.u16(textdraw.text.len() as u16);
    writer.bytes(&textdraw.text);
    Ok(writer.finish_bits())
}

fn decode_vehicle_stream_in(event: &mut Event<'_>) -> Result<VehicleStreamIn, EventError> {
    Ok(VehicleStreamIn {
        vehicle_id: event.read_u16()?,
        vehicle: StreamedVehicle {
            model: decode_i32(event)?,
            position: decode_vector3(event)?,
            rotation: event.read_f32()?,
            body_color1: event.read_u8()?,
            body_color2: event.read_u8()?,
            health: event.read_f32()?,
            interior_id: event.read_u8()?,
            door_damage_status: decode_i32(event)?,
            panel_damage_status: decode_i32(event)?,
            light_damage_status: event.read_u8()?,
            tire_damage_status: event.read_u8()?,
            add_siren: event.read_u8()?,
            mod_slots: read_array(event)?,
            paint_job: event.read_u8()?,
            interior_color1: decode_i32(event)?,
            interior_color2: decode_i32(event)?,
        },
    })
}

fn encode_vehicle_stream_in(
    _api: HostApi,
    value: VehicleStreamIn,
) -> Result<EncodedPayload, EventError> {
    let vehicle = value.vehicle;
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    write_i32(&mut writer, vehicle.model);
    writer.vector3(vehicle.position);
    writer.f32(vehicle.rotation);
    writer.u8(vehicle.body_color1);
    writer.u8(vehicle.body_color2);
    writer.f32(vehicle.health);
    writer.u8(vehicle.interior_id);
    write_i32(&mut writer, vehicle.door_damage_status);
    write_i32(&mut writer, vehicle.panel_damage_status);
    writer.u8(vehicle.light_damage_status);
    writer.u8(vehicle.tire_damage_status);
    writer.u8(vehicle.add_siren);
    writer.bytes(&vehicle.mod_slots);
    writer.u8(vehicle.paint_job);
    write_i32(&mut writer, vehicle.interior_color1);
    write_i32(&mut writer, vehicle.interior_color2);
    Ok(writer.finish_bits())
}
