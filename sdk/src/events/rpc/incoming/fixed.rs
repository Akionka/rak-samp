use super::types::*;
use crate::events::core::PayloadWriter;
use crate::{
    HostApi,
    events::{
        EncodedPayload, Event, EventError, IncomingRpc, MAX_ENCODED_STRING_BYTES,
        MAX_STRING32_BYTES, Vector2, Vector3,
    },
};

/// The `onShowDialog` descriptor.
pub const SHOW_DIALOG: IncomingRpc<ShowDialog> =
    IncomingRpc::new_bits(61, decode_show_dialog, encode_show_dialog);
/// The `onAttachCameraToObject` descriptor.
pub const ATTACH_CAMERA_TO_OBJECT: IncomingRpc<u16> = IncomingRpc::new(81, decode_u16, encode_u16);
/// The `onGangZoneStopFlash` descriptor.
pub const GANG_ZONE_STOP_FLASH: IncomingRpc<u16> = IncomingRpc::new(85, decode_u16, encode_u16);
/// The `onClearPlayerAnimation` descriptor.
pub const CLEAR_PLAYER_ANIMATION: IncomingRpc<u16> = IncomingRpc::new(87, decode_u16, encode_u16);
/// The `onSetPlayerSpecialAction` descriptor.
pub const SET_PLAYER_SPECIAL_ACTION: IncomingRpc<u8> = IncomingRpc::new(88, decode_u8, encode_u8);
/// The `onSetPlayerFightingStyle` descriptor.
pub const SET_PLAYER_FIGHTING_STYLE: IncomingRpc<PlayerFightingStyle> = IncomingRpc::new(
    89,
    decode_player_fighting_style,
    encode_player_fighting_style,
);
/// The `onSetPlayerVelocity` descriptor.
pub const SET_PLAYER_VELOCITY: IncomingRpc<Vector3> =
    IncomingRpc::new(90, decode_vector3, encode_vector3);
/// The `onSetVehicleVelocity` descriptor.
pub const SET_VEHICLE_VELOCITY: IncomingRpc<VehicleVelocity> =
    IncomingRpc::new(91, decode_vehicle_velocity, encode_vehicle_velocity);
/// The `onCreatePickup` descriptor.
pub const CREATE_PICKUP: IncomingRpc<Pickup> = IncomingRpc::new(95, decode_pickup, encode_pickup);
/// The `onMoveObject` descriptor.
pub const MOVE_OBJECT: IncomingRpc<MoveObject> =
    IncomingRpc::new(99, decode_move_object, encode_move_object);
/// The `onTextDrawSetString` descriptor.
pub const TEXT_DRAW_SET_STRING: IncomingRpc<TextDrawString> =
    IncomingRpc::new(105, decode_text_draw_string, encode_text_draw_string);
/// The `onCreateGangZone` descriptor.
pub const CREATE_GANG_ZONE: IncomingRpc<GangZone> =
    IncomingRpc::new(108, decode_gang_zone, encode_gang_zone);
/// The `onGangZoneDestroy` descriptor.
pub const GANG_ZONE_DESTROY: IncomingRpc<u16> = IncomingRpc::new(120, decode_u16, encode_u16);
/// The `onGangZoneFlash` descriptor.
pub const GANG_ZONE_FLASH: IncomingRpc<(u16, i32)> =
    IncomingRpc::new(121, decode_u16_i32, encode_u16_i32);
/// The `onStopObject` descriptor.
pub const STOP_OBJECT: IncomingRpc<u16> = IncomingRpc::new(122, decode_u16, encode_u16);
/// The `onSetVehicleNumberPlate` descriptor.
pub const SET_VEHICLE_NUMBER_PLATE: IncomingRpc<VehicleNumberPlate> = IncomingRpc::new(
    123,
    decode_vehicle_number_plate,
    encode_vehicle_number_plate,
);
/// The `onSpectatePlayer` descriptor.
pub const SPECTATE_PLAYER: IncomingRpc<Spectate> =
    IncomingRpc::new(126, decode_spectate, encode_spectate);
/// The `onSpectateVehicle` descriptor.
pub const SPECTATE_VEHICLE: IncomingRpc<Spectate> =
    IncomingRpc::new(127, decode_spectate, encode_spectate);
/// The `onConnectionRejected` descriptor.
pub const CONNECTION_REJECTED: IncomingRpc<u8> = IncomingRpc::new(130, decode_u8, encode_u8);
/// The `onRemoveMapIcon` descriptor.
pub const REMOVE_MAP_ICON: IncomingRpc<u8> = IncomingRpc::new(144, decode_u8, encode_u8);
/// The `onSetWeaponAmmo` descriptor.
pub const SET_WEAPON_AMMO: IncomingRpc<WeaponAmmo> =
    IncomingRpc::new(145, decode_weapon_ammo, encode_weapon_ammo);
/// The `onSetGravity` descriptor.
pub const SET_GRAVITY: IncomingRpc<f32> = IncomingRpc::new(146, decode_f32, encode_f32);
/// The `onAttachTrailerToVehicle` descriptor.
pub const ATTACH_TRAILER_TO_VEHICLE: IncomingRpc<TrailerAttachment> =
    IncomingRpc::new(148, decode_trailer_attachment, encode_trailer_attachment);
/// The `onDetachTrailerFromVehicle` descriptor.
pub const DETACH_TRAILER_FROM_VEHICLE: IncomingRpc<u16> =
    IncomingRpc::new(149, decode_u16, encode_u16);
/// The `onSetCameraPosition` descriptor.
pub const SET_CAMERA_POSITION: IncomingRpc<Vector3> =
    IncomingRpc::new(157, decode_vector3, encode_vector3);
/// The `onSetCameraLookAt` descriptor.
pub const SET_CAMERA_LOOK_AT: IncomingRpc<CameraLookAt> =
    IncomingRpc::new(158, decode_camera_look_at, encode_camera_look_at);
/// The `onSetVehicleParams` descriptor.
pub const SET_VEHICLE_PARAMS: IncomingRpc<VehicleParams> =
    IncomingRpc::new(161, decode_vehicle_params, encode_vehicle_params);
/// The `onPlayerDeath` descriptor.
pub const PLAYER_DEATH: IncomingRpc<u16> = IncomingRpc::new(166, decode_u16, encode_u16);
/// The `onPlayerEnterVehicle` descriptor.
pub const PLAYER_ENTER_VEHICLE: IncomingRpc<PlayerEnterVehicle> =
    IncomingRpc::new(26, decode_player_enter_vehicle, encode_player_enter_vehicle);
/// The `onPlayerExitVehicle` descriptor.
pub const PLAYER_EXIT_VEHICLE: IncomingRpc<PlayerExitVehicle> =
    IncomingRpc::new(154, decode_player_exit_vehicle, encode_player_exit_vehicle);

fn decode_show_dialog(event: &mut Event<'_>) -> Result<ShowDialog, EventError> {
    Ok(ShowDialog {
        dialog_id: event.read_u16()?,
        style: event.read_u8()?,
        title: event.read_string8()?,
        button1: event.read_string8()?,
        button2: event.read_string8()?,
        text: event.read_encoded_string(MAX_ENCODED_STRING_BYTES + 1)?,
    })
}

fn encode_show_dialog(api: HostApi, value: ShowDialog) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.dialog_id);
    writer.u8(value.style);
    writer.string8(&value.title)?;
    writer.string8(&value.button1)?;
    writer.string8(&value.button2)?;
    writer.encoded_string(api, &value.text)?;
    Ok(writer.finish_bits())
}

pub(super) fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    Ok(Vector3 {
        x: event.read_f32()?,
        y: event.read_f32()?,
        z: event.read_f32()?,
    })
}

fn encode_vector3(value: Vector3) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.vector3(value);
    Ok(writer.finish())
}

fn decode_f32(event: &mut Event<'_>) -> Result<f32, EventError> {
    event.read_f32()
}

fn encode_f32(value: f32) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.f32(value);
    Ok(writer.finish())
}

pub(super) fn decode_bool8(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_u8()? != 0)
}

fn decode_player_fighting_style(event: &mut Event<'_>) -> Result<PlayerFightingStyle, EventError> {
    Ok(PlayerFightingStyle {
        player_id: event.read_u16()?,
        style_id: event.read_u8()?,
    })
}

fn encode_player_fighting_style(value: PlayerFightingStyle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u8(value.style_id);
    Ok(writer.finish())
}

fn decode_vehicle_velocity(event: &mut Event<'_>) -> Result<VehicleVelocity, EventError> {
    Ok(VehicleVelocity {
        turn: decode_bool8(event)?,
        velocity: decode_vector3(event)?,
    })
}

fn encode_vehicle_velocity(value: VehicleVelocity) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(u8::from(value.turn));
    writer.vector3(value.velocity);
    Ok(writer.finish())
}

fn decode_pickup(event: &mut Event<'_>) -> Result<Pickup, EventError> {
    Ok(Pickup {
        id: decode_i32(event)?,
        model: decode_i32(event)?,
        pickup_type: decode_i32(event)?,
        position: decode_vector3(event)?,
    })
}

fn encode_pickup(value: Pickup) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.id as u32);
    writer.u32(value.model as u32);
    writer.u32(value.pickup_type as u32);
    writer.vector3(value.position);
    Ok(writer.finish())
}

fn decode_move_object(event: &mut Event<'_>) -> Result<MoveObject, EventError> {
    Ok(MoveObject {
        object_id: event.read_u16()?,
        from_position: decode_vector3(event)?,
        destination: decode_vector3(event)?,
        speed: event.read_f32()?,
        rotation: decode_vector3(event)?,
    })
}

fn encode_move_object(value: MoveObject) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    writer.vector3(value.from_position);
    writer.vector3(value.destination);
    writer.f32(value.speed);
    writer.vector3(value.rotation);
    Ok(writer.finish())
}

fn decode_text_draw_string(event: &mut Event<'_>) -> Result<TextDrawString, EventError> {
    let textdraw_id = event.read_u16()?;
    let length = usize::from(event.read_u16()?);
    if length > MAX_STRING32_BYTES {
        return Err(EventError::LengthExceedsLimit {
            length,
            limit: MAX_STRING32_BYTES,
        });
    }
    Ok(TextDrawString {
        textdraw_id,
        text: event.read_bytes(length)?,
    })
}

fn encode_text_draw_string(value: TextDrawString) -> Result<Vec<u8>, EventError> {
    if value.text.len() > MAX_STRING32_BYTES {
        return Err(EventError::LengthExceedsLimit {
            length: value.text.len(),
            limit: MAX_STRING32_BYTES,
        });
    }
    let mut writer = PayloadWriter::new();
    writer.u16(value.textdraw_id);
    writer.u16(value.text.len() as u16);
    writer.bytes(&value.text);
    Ok(writer.finish())
}

pub(super) fn decode_vector2(event: &mut Event<'_>) -> Result<Vector2, EventError> {
    Ok(Vector2 {
        x: event.read_f32()?,
        y: event.read_f32()?,
    })
}

pub(super) fn encode_vector2(writer: &mut PayloadWriter, value: Vector2) {
    writer.f32(value.x);
    writer.f32(value.y);
}

fn decode_gang_zone(event: &mut Event<'_>) -> Result<GangZone, EventError> {
    Ok(GangZone {
        zone_id: event.read_u16()?,
        square_start: decode_vector2(event)?,
        square_end: decode_vector2(event)?,
        color: decode_i32(event)?,
    })
}

fn encode_gang_zone(value: GangZone) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.zone_id);
    encode_vector2(&mut writer, value.square_start);
    encode_vector2(&mut writer, value.square_end);
    writer.u32(value.color as u32);
    Ok(writer.finish())
}

fn decode_u16_i32(event: &mut Event<'_>) -> Result<(u16, i32), EventError> {
    Ok((event.read_u16()?, decode_i32(event)?))
}

fn encode_u16_i32(value: (u16, i32)) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.0);
    writer.u32(value.1 as u32);
    Ok(writer.finish())
}

fn decode_vehicle_number_plate(event: &mut Event<'_>) -> Result<VehicleNumberPlate, EventError> {
    Ok(VehicleNumberPlate {
        vehicle_id: event.read_u16()?,
        text: event.read_string8()?,
    })
}

fn encode_vehicle_number_plate(value: VehicleNumberPlate) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.string8(&value.text)?;
    Ok(writer.finish())
}

fn decode_spectate(event: &mut Event<'_>) -> Result<Spectate, EventError> {
    Ok(Spectate {
        target_id: event.read_u16()?,
        camera_type: event.read_u8()?,
    })
}

fn encode_spectate(value: Spectate) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.target_id);
    writer.u8(value.camera_type);
    Ok(writer.finish())
}

fn decode_weapon_ammo(event: &mut Event<'_>) -> Result<WeaponAmmo, EventError> {
    Ok(WeaponAmmo {
        weapon_id: event.read_u8()?,
        ammo: event.read_u16()?,
    })
}

fn encode_weapon_ammo(value: WeaponAmmo) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.weapon_id);
    writer.u16(value.ammo);
    Ok(writer.finish())
}

fn decode_trailer_attachment(event: &mut Event<'_>) -> Result<TrailerAttachment, EventError> {
    Ok(TrailerAttachment {
        trailer_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
    })
}

fn encode_trailer_attachment(value: TrailerAttachment) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.trailer_id);
    writer.u16(value.vehicle_id);
    Ok(writer.finish())
}

fn decode_camera_look_at(event: &mut Event<'_>) -> Result<CameraLookAt, EventError> {
    Ok(CameraLookAt {
        position: decode_vector3(event)?,
        cut_type: event.read_u8()?,
    })
}

fn encode_camera_look_at(value: CameraLookAt) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.vector3(value.position);
    writer.u8(value.cut_type);
    Ok(writer.finish())
}

fn decode_vehicle_params(event: &mut Event<'_>) -> Result<VehicleParams, EventError> {
    Ok(VehicleParams {
        vehicle_id: event.read_u16()?,
        objective: decode_bool8(event)?,
        doors_locked: decode_bool8(event)?,
    })
}

fn encode_vehicle_params(value: VehicleParams) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(u8::from(value.objective));
    writer.u8(u8::from(value.doors_locked));
    Ok(writer.finish())
}

fn decode_player_enter_vehicle(event: &mut Event<'_>) -> Result<PlayerEnterVehicle, EventError> {
    Ok(PlayerEnterVehicle {
        player_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
        passenger: decode_bool8(event)?,
    })
}

fn encode_player_enter_vehicle(value: PlayerEnterVehicle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u16(value.vehicle_id);
    writer.u8(u8::from(value.passenger));
    Ok(writer.finish())
}

fn decode_player_exit_vehicle(event: &mut Event<'_>) -> Result<PlayerExitVehicle, EventError> {
    Ok(PlayerExitVehicle {
        player_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
    })
}

fn encode_player_exit_vehicle(value: PlayerExitVehicle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u16(value.vehicle_id);
    Ok(writer.finish())
}

pub(super) fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

fn decode_u8(event: &mut Event<'_>) -> Result<u8, EventError> {
    event.read_u8()
}

fn encode_u8(value: u8) -> Result<Vec<u8>, EventError> {
    Ok(vec![value])
}

pub(super) fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
    event.read_u16()
}

pub(super) fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
    Ok(value.to_le_bytes().to_vec())
}
