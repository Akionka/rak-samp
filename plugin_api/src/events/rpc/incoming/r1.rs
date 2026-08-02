use super::{fixed::*, types::*};
use crate::events::core::PayloadWriter;
use crate::{
    HostApi,
    events::{
        EncodedPayload, Event, EventError, MAX_ENCODED_STRING_BYTES, MAX_STRING32_BYTES, Rpc,
    },
};

/// The R1 `onInitGame` descriptor.
pub const INIT_GAME: Rpc<InitGame> = Rpc::new_bits(139, decode_init_game, encode_init_game);
/// The R1 `onRequestClassResponse` descriptor.
pub const REQUEST_CLASS_RESPONSE: Rpc<RequestClassResponse> = Rpc::new_bits(
    128,
    decode_request_class_response,
    encode_request_class_response,
);
/// The R1 `onPlayerStreamIn` descriptor.
pub const PLAYER_STREAM_IN: Rpc<PlayerStreamIn> =
    Rpc::new_bits(32, decode_player_stream_in, encode_player_stream_in);
/// The R1 `onCreate3DText` descriptor.
pub const CREATE_3D_TEXT: Rpc<TextLabel3D> =
    Rpc::new_bits(36, decode_text_label_3d, encode_text_label_3d);
/// The R1 `onCreateObject` descriptor.
pub const CREATE_OBJECT: Rpc<Object> = Rpc::new_bits(44, decode_object, encode_object);
/// The R1 `onSetSpawnInfo` descriptor.
pub const SET_SPAWN_INFO: Rpc<SpawnInfo> = Rpc::new_bits(68, decode_spawn_info, encode_spawn_info);
/// The R1 `onInitMenu` descriptor.
pub const INIT_MENU: Rpc<InitMenu> = Rpc::new_bits(76, decode_init_menu, encode_init_menu);
/// The R1 `onInterpolateCamera` descriptor.
pub const INTERPOLATE_CAMERA: Rpc<InterpolateCamera> =
    Rpc::new_bits(82, decode_interpolate_camera, encode_interpolate_camera);
/// The R1 `onToggleSelectTextDraw` descriptor.
pub const TOGGLE_SELECT_TEXT_DRAW: Rpc<ToggleSelectTextDraw> = Rpc::new_bits(
    83,
    decode_toggle_select_text_draw,
    encode_toggle_select_text_draw,
);
/// The R1 object material descriptor. [`ObjectMaterial`] preserves either material variant.
pub const SET_OBJECT_MATERIAL: Rpc<ObjectMaterialUpdate> = Rpc::new_bits(
    84,
    decode_object_material_update,
    encode_object_material_update,
);
/// The R1 `onApplyPlayerAnimation` descriptor.
pub const APPLY_PLAYER_ANIMATION: Rpc<PlayerAnimation> =
    Rpc::new_bits(86, decode_player_animation, encode_player_animation);
/// The R1 `onEnableStuntBonus` descriptor.
pub const ENABLE_STUNT_BONUS: Rpc<bool> = Rpc::new_bits(104, decode_bit_bool, encode_bit_bool);
/// The R1 `onPlayCrimeReport` descriptor.
pub const PLAY_CRIME_REPORT: Rpc<CrimeReport> =
    Rpc::new_bits(112, decode_crime_report, encode_crime_report);
/// The R1 `onSetPlayerAttachedObject` descriptor.
pub const SET_PLAYER_ATTACHED_OBJECT: Rpc<PlayerAttachedObject> = Rpc::new_bits(
    113,
    decode_player_attached_object,
    encode_player_attached_object,
);
/// The R1 `onEnterEditObject` descriptor.
pub const ENTER_EDIT_OBJECT: Rpc<EnterEditObject> =
    Rpc::new_bits(117, decode_enter_edit_object, encode_enter_edit_object);
/// The R1 `onTogglePlayerSpectating` descriptor.
pub const TOGGLE_PLAYER_SPECTATING: Rpc<bool> = Rpc::new_bits(124, decode_bool32, encode_bool32);
/// The R1 `onShowTextDraw` descriptor.
pub const SHOW_TEXT_DRAW: Rpc<ShowTextDraw> =
    Rpc::new_bits(134, decode_show_text_draw, encode_show_text_draw);
/// The R1 `onTextDrawHide` descriptor.
pub const TEXT_DRAW_HIDE: Rpc<u16> = Rpc::new(135, decode_u16, encode_u16);
/// The R1 `onInitGame` score/ping update descriptor.
pub const UPDATE_SCORES_AND_PINGS: Rpc<ScoresAndPings> =
    Rpc::new_bits(155, decode_scores_and_pings, encode_scores_and_pings);
/// The R1 `onVehicleStreamIn` descriptor.
pub const VEHICLE_STREAM_IN: Rpc<VehicleStreamIn> =
    Rpc::new_bits(164, decode_vehicle_stream_in, encode_vehicle_stream_in);
/// The R1 `onDisableVehicleCollisions` descriptor.
pub const DISABLE_VEHICLE_COLLISIONS: Rpc<bool> =
    Rpc::new_bits(167, decode_bit_bool, encode_bit_bool);
/// The R1 `onToggleCameraTargetNotifying` descriptor.
pub const TOGGLE_CAMERA_TARGET_NOTIFYING: Rpc<bool> =
    Rpc::new_bits(170, decode_bit_bool, encode_bit_bool);
/// The R1 `onApplyActorAnimation` descriptor.
pub const APPLY_ACTOR_ANIMATION: Rpc<ActorAnimation> =
    Rpc::new_bits(173, decode_actor_animation, encode_actor_animation);

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

fn encode_bool32(_api: HostApi, value: bool) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(u32::from(value));
    Ok(writer.finish_bits())
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

fn decode_spawn_info_fields(event: &mut Event<'_>) -> Result<SpawnInfo, EventError> {
    Ok(SpawnInfo {
        team: event.read_u8()?,
        skin: decode_i32(event)?,
        unused: event.read_u8()?,
        position: decode_vector3(event)?,
        rotation: event.read_f32()?,
        weapons: [decode_i32(event)?, decode_i32(event)?, decode_i32(event)?],
        ammo: [decode_i32(event)?, decode_i32(event)?, decode_i32(event)?],
    })
}

fn encode_spawn_info_fields(writer: &mut PayloadWriter, value: SpawnInfo) {
    writer.u8(value.team);
    write_i32(writer, value.skin);
    writer.u8(value.unused);
    writer.vector3(value.position);
    writer.f32(value.rotation);
    for weapon in value.weapons {
        write_i32(writer, weapon);
    }
    for ammo in value.ammo {
        write_i32(writer, ammo);
    }
}

fn decode_init_game(event: &mut Event<'_>) -> Result<InitGame, EventError> {
    let mut settings = GameSettings {
        zone_names: read_bit_bool(event)?,
        use_cj_walk: read_bit_bool(event)?,
        allow_weapons: read_bit_bool(event)?,
        limit_global_chat_radius: read_bit_bool(event)?,
        global_chat_radius: event.read_f32()?,
        stunt_bonus: read_bit_bool(event)?,
        nametag_draw_distance: event.read_f32()?,
        disable_enter_exits: read_bit_bool(event)?,
        nametag_los: read_bit_bool(event)?,
        tire_popping: read_bit_bool(event)?,
        classes_available: decode_i32(event)?,
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
    let player_id = event.read_u16()?;
    settings.show_player_tags = read_bit_bool(event)?;
    settings.player_markers_mode = decode_i32(event)?;
    settings.world_time = event.read_u8()?;
    settings.world_weather = event.read_u8()?;
    settings.gravity = event.read_f32()?;
    settings.lan_mode = read_bit_bool(event)?;
    settings.death_money_drop = decode_i32(event)?;
    settings.instagib = read_bit_bool(event)?;
    settings.normal_onfoot_send_rate = decode_i32(event)?;
    settings.normal_incar_send_rate = decode_i32(event)?;
    settings.normal_firing_send_rate = decode_i32(event)?;
    settings.send_multiplier = decode_i32(event)?;
    settings.lag_compensation_mode = decode_i32(event)?;
    let host_name = event.read_string8()?;
    let vehicle_models = read_array::<212>(event)?;
    settings.vehicle_friendly_fire = decode_bool32(event)?;
    Ok(InitGame {
        player_id,
        host_name,
        settings,
        vehicle_models,
    })
}

fn encode_init_game(api: HostApi, value: InitGame) -> Result<EncodedPayload, EventError> {
    let _ = api;
    let settings = value.settings;
    let mut writer = PayloadWriter::new();
    writer.bool(settings.zone_names);
    writer.bool(settings.use_cj_walk);
    writer.bool(settings.allow_weapons);
    writer.bool(settings.limit_global_chat_radius);
    writer.f32(settings.global_chat_radius);
    writer.bool(settings.stunt_bonus);
    writer.f32(settings.nametag_draw_distance);
    writer.bool(settings.disable_enter_exits);
    writer.bool(settings.nametag_los);
    writer.bool(settings.tire_popping);
    write_i32(&mut writer, settings.classes_available);
    writer.u16(value.player_id);
    writer.bool(settings.show_player_tags);
    write_i32(&mut writer, settings.player_markers_mode);
    writer.u8(settings.world_time);
    writer.u8(settings.world_weather);
    writer.f32(settings.gravity);
    writer.bool(settings.lan_mode);
    write_i32(&mut writer, settings.death_money_drop);
    writer.bool(settings.instagib);
    write_i32(&mut writer, settings.normal_onfoot_send_rate);
    write_i32(&mut writer, settings.normal_incar_send_rate);
    write_i32(&mut writer, settings.normal_firing_send_rate);
    write_i32(&mut writer, settings.send_multiplier);
    write_i32(&mut writer, settings.lag_compensation_mode);
    writer.string8(&value.host_name)?;
    writer.bytes(&value.vehicle_models);
    writer.u32(u32::from(settings.vehicle_friendly_fire));
    Ok(writer.finish_bits())
}

fn decode_request_class_response(
    event: &mut Event<'_>,
) -> Result<RequestClassResponse, EventError> {
    Ok(RequestClassResponse {
        can_spawn: decode_bool8(event)?,
        spawn: decode_spawn_info_fields(event)?,
    })
}

fn encode_request_class_response(
    _api: HostApi,
    value: RequestClassResponse,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(u8::from(value.can_spawn));
    encode_spawn_info_fields(&mut writer, value.spawn);
    Ok(writer.finish_bits())
}

fn decode_player_stream_in(event: &mut Event<'_>) -> Result<PlayerStreamIn, EventError> {
    let player_id = event.read_u16()?;
    let team = event.read_u8()?;
    let model = decode_i32(event)?;
    let position = decode_vector3(event)?;
    let rotation = event.read_f32()?;
    let color = decode_i32(event)?;
    let fighting_style = event.read_u8()?;
    let mut weapon_skill_levels = [0; 11];
    for skill_level in &mut weapon_skill_levels {
        *skill_level = event.read_u16()?;
    }
    Ok(PlayerStreamIn {
        player_id,
        team,
        model,
        position,
        rotation,
        color,
        fighting_style,
        weapon_skill_levels,
    })
}

fn encode_player_stream_in(
    _api: HostApi,
    value: PlayerStreamIn,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u8(value.team);
    write_i32(&mut writer, value.model);
    writer.vector3(value.position);
    writer.f32(value.rotation);
    write_i32(&mut writer, value.color);
    writer.u8(value.fighting_style);
    for skill_level in value.weapon_skill_levels {
        writer.u16(skill_level);
    }
    Ok(writer.finish_bits())
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

fn decode_spawn_info(event: &mut Event<'_>) -> Result<SpawnInfo, EventError> {
    decode_spawn_info_fields(event)
}

fn encode_spawn_info(api: HostApi, value: SpawnInfo) -> Result<EncodedPayload, EventError> {
    let _ = api;
    let mut writer = PayloadWriter::new();
    encode_spawn_info_fields(&mut writer, value);
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

fn decode_player_animation(event: &mut Event<'_>) -> Result<PlayerAnimation, EventError> {
    Ok(PlayerAnimation {
        player_id: event.read_u16()?,
        animation: decode_animation(event)?,
    })
}

fn encode_player_animation(
    _api: HostApi,
    value: PlayerAnimation,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    encode_animation(&mut writer, value.animation)?;
    Ok(writer.finish_bits())
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

fn decode_crime_report(event: &mut Event<'_>) -> Result<CrimeReport, EventError> {
    Ok(CrimeReport {
        suspect_id: event.read_u16()?,
        in_vehicle: decode_bool32(event)?,
        vehicle_model: decode_i32(event)?,
        vehicle_color: decode_i32(event)?,
        crime: decode_i32(event)?,
        coordinates: decode_vector3(event)?,
    })
}

fn encode_crime_report(_api: HostApi, value: CrimeReport) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.suspect_id);
    writer.u32(u32::from(value.in_vehicle));
    write_i32(&mut writer, value.vehicle_model);
    write_i32(&mut writer, value.vehicle_color);
    write_i32(&mut writer, value.crime);
    writer.vector3(value.coordinates);
    Ok(writer.finish_bits())
}

fn decode_attached_object(event: &mut Event<'_>) -> Result<AttachedObject, EventError> {
    Ok(AttachedObject {
        model_id: decode_i32(event)?,
        bone: decode_i32(event)?,
        offset: decode_vector3(event)?,
        rotation: decode_vector3(event)?,
        scale: decode_vector3(event)?,
        color1: decode_i32(event)?,
        color2: decode_i32(event)?,
    })
}

fn encode_attached_object(writer: &mut PayloadWriter, value: AttachedObject) {
    write_i32(writer, value.model_id);
    write_i32(writer, value.bone);
    writer.vector3(value.offset);
    writer.vector3(value.rotation);
    writer.vector3(value.scale);
    write_i32(writer, value.color1);
    write_i32(writer, value.color2);
}

fn decode_player_attached_object(
    event: &mut Event<'_>,
) -> Result<PlayerAttachedObject, EventError> {
    let player_id = event.read_u16()?;
    let index = decode_i32(event)?;
    let create = read_bit_bool(event)?;
    let object = create.then(|| decode_attached_object(event)).transpose()?;
    Ok(PlayerAttachedObject {
        player_id,
        index,
        object,
    })
}

fn encode_player_attached_object(
    _api: HostApi,
    value: PlayerAttachedObject,
) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    write_i32(&mut writer, value.index);
    writer.bool(value.object.is_some());
    if let Some(object) = value.object {
        encode_attached_object(&mut writer, object);
    }
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

fn decode_scores_and_pings(event: &mut Event<'_>) -> Result<ScoresAndPings, EventError> {
    let bit_len = event.remaining_bits();
    if !bit_len.is_multiple_of(80) {
        return Err(EventError::UnexpectedBitLength {
            bit_len,
            expected: 80,
        });
    }
    let count = bit_len / 80;
    if count > MAX_SCORE_PING_ENTRIES {
        return Err(EventError::LengthExceedsLimit {
            length: count,
            limit: MAX_SCORE_PING_ENTRIES,
        });
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(ScorePing {
            player_id: event.read_u16()?,
            score: decode_i32(event)?,
            ping: decode_i32(event)?,
        });
    }
    Ok(ScoresAndPings { entries })
}

fn encode_scores_and_pings(
    _api: HostApi,
    value: ScoresAndPings,
) -> Result<EncodedPayload, EventError> {
    if value.entries.len() > MAX_SCORE_PING_ENTRIES {
        return Err(EventError::LengthExceedsLimit {
            length: value.entries.len(),
            limit: MAX_SCORE_PING_ENTRIES,
        });
    }
    let mut writer = PayloadWriter::new();
    for entry in value.entries {
        writer.u16(entry.player_id);
        write_i32(&mut writer, entry.score);
        write_i32(&mut writer, entry.ping);
    }
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
