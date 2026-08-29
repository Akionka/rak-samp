use super::*;

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

/// MoonLoader's `onSetVehicleVelocity` payload (RPC 91).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleVelocity {
    pub turn: bool,
    pub velocity: Vector3,
}

/// MoonLoader's `onSetVehicleNumberPlate` payload (RPC 123).
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleNumberPlate {
    pub vehicle_id: u16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onAttachTrailerToVehicle` payload (RPC 148).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailerAttachment {
    pub trailer_id: u16,
    pub vehicle_id: u16,
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

struct PutPlayerInVehicleCodec;

struct VehiclePositionCodec;

struct VehicleAngleCodec;

struct VehicleHealthCodec;

struct VehicleComponentCodec;

struct VehicleInteriorCodec;

struct VehicleParamsExCodec;

struct VehicleTuningNotificationCodec;

struct VehicleDamageStatusCodec;

struct VehicleVelocityCodec;

struct VehicleNumberPlateCodec;

struct TrailerAttachmentCodec;

struct VehicleParamsCodec;

struct PlayerEnterVehicleCodec;

struct PlayerExitVehicleCodec;

descriptor!(
    PutPlayerInVehicleRpc,
    PUT_PLAYER_IN_VEHICLE,
    70,
    PutPlayerInVehicleCodec,
    PutPlayerInVehicle
);

descriptor!(VehicleStreamOut, VEHICLE_STREAM_OUT, 165, U16, u16);

descriptor!(
    SetVehiclePosition,
    SET_VEHICLE_POSITION,
    159,
    VehiclePositionCodec,
    VehiclePosition
);

descriptor!(
    SetVehicleAngle,
    SET_VEHICLE_ANGLE,
    160,
    VehicleAngleCodec,
    VehicleAngle
);

descriptor!(
    SetVehicleHealth,
    SET_VEHICLE_HEALTH,
    147,
    VehicleHealthCodec,
    VehicleHealth
);

descriptor!(
    RemoveVehicleComponent,
    REMOVE_VEHICLE_COMPONENT,
    57,
    VehicleComponentCodec,
    VehicleComponent
);

descriptor!(
    LinkVehicleToInterior,
    LINK_VEHICLE_TO_INTERIOR,
    65,
    VehicleInteriorCodec,
    VehicleInterior
);

descriptor!(
    SetVehicleParamsEx,
    SET_VEHICLE_PARAMS_EX,
    24,
    VehicleParamsExCodec,
    VehicleParamsEx
);

descriptor!(
    VehicleTuningNotificationRpc,
    VEHICLE_TUNING_NOTIFICATION,
    96,
    VehicleTuningNotificationCodec,
    VehicleTuningNotification
);

descriptor!(
    SetVehicleTires,
    SET_VEHICLE_TIRES,
    98,
    U16U8Codec,
    (u16, u8)
);

descriptor!(
    VehicleDamageStatusUpdate,
    VEHICLE_DAMAGE_STATUS_UPDATE,
    106,
    VehicleDamageStatusCodec,
    VehicleDamageStatus
);

descriptor!(
    RemovePlayerFromVehicle,
    REMOVE_PLAYER_FROM_VEHICLE,
    71,
    Empty,
    ()
);

descriptor!(
    SetVehicleVelocity,
    SET_VEHICLE_VELOCITY,
    91,
    VehicleVelocityCodec,
    VehicleVelocity
);

descriptor!(
    SetVehicleNumberPlate,
    SET_VEHICLE_NUMBER_PLATE,
    123,
    VehicleNumberPlateCodec,
    VehicleNumberPlate
);

descriptor!(
    AttachTrailerToVehicle,
    ATTACH_TRAILER_TO_VEHICLE,
    148,
    TrailerAttachmentCodec,
    TrailerAttachment
);

descriptor!(
    DetachTrailerFromVehicle,
    DETACH_TRAILER_FROM_VEHICLE,
    149,
    U16,
    u16
);

descriptor!(
    SetVehicleParams,
    SET_VEHICLE_PARAMS,
    161,
    VehicleParamsCodec,
    VehicleParams
);

descriptor!(
    PlayerEnterVehicleRpc,
    PLAYER_ENTER_VEHICLE,
    26,
    PlayerEnterVehicleCodec,
    PlayerEnterVehicle
);

descriptor!(
    PlayerExitVehicleRpc,
    PLAYER_EXIT_VEHICLE,
    154,
    PlayerExitVehicleCodec,
    PlayerExitVehicle
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

wire_codec!(
    VehicleDamageStatusCodec,
    VehicleDamageStatus,
    read_vehicle_damage_status,
    write_vehicle_damage_status
);

wire_codec!(
    VehicleVelocityCodec,
    VehicleVelocity,
    read_vehicle_velocity,
    write_vehicle_velocity
);

wire_codec!(
    VehicleNumberPlateCodec,
    VehicleNumberPlate,
    read_vehicle_number_plate,
    write_vehicle_number_plate
);

wire_codec!(
    TrailerAttachmentCodec,
    TrailerAttachment,
    read_trailer_attachment,
    write_trailer_attachment
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
