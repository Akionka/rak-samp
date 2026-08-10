//! Outgoing vehicle-interaction RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, Rpc, RpcAction},
};

/// MoonLoader's `onSendEnterVehicle` payload (RPC 26).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnterVehicle {
    pub vehicle_id: u16,
    pub passenger: bool,
}

/// MoonLoader's `onSendVehicleTuningNotification` payload (RPC 96).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleTuning {
    pub vehicle_id: i32,
    pub param1: i32,
    pub param2: i32,
    pub event: i32,
}

/// The `onSendEnterVehicle` descriptor.
pub const SEND_ENTER_VEHICLE: Rpc<EnterVehicle> =
    Rpc::new(26, decode_enter_vehicle, encode_enter_vehicle);
/// The `onSendExitVehicle` descriptor.
pub const SEND_EXIT_VEHICLE: Rpc<u16> = Rpc::new(154, decode_u16, encode_u16);
/// The `onSendVehicleDestroyed` descriptor.
pub const SEND_VEHICLE_DESTROYED: Rpc<u16> = Rpc::new(136, decode_u16, encode_u16);
/// The `onSendVehicleTuningNotification` descriptor.
pub const SEND_VEHICLE_TUNING: Rpc<VehicleTuning> =
    Rpc::new(96, decode_vehicle_tuning, encode_vehicle_tuning);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_enter_vehicle(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(EnterVehicle) -> RpcAction<EnterVehicle>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_ENTER_VEHICLE, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_exit_vehicle(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(u16) -> RpcAction<u16>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_EXIT_VEHICLE, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_vehicle_destroyed(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(u16) -> RpcAction<u16>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_VEHICLE_DESTROYED, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_vehicle_tuning(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(VehicleTuning) -> RpcAction<VehicleTuning>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_VEHICLE_TUNING, handler) }
}

fn decode_enter_vehicle(event: &mut Event<'_>) -> Result<EnterVehicle, EventError> {
    Ok(EnterVehicle {
        vehicle_id: event.read_u16()?,
        passenger: event.read_u8()? != 0,
    })
}

fn encode_enter_vehicle(value: EnterVehicle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(u8::from(value.passenger));
    Ok(writer.finish())
}

fn decode_vehicle_tuning(event: &mut Event<'_>) -> Result<VehicleTuning, EventError> {
    Ok(VehicleTuning {
        vehicle_id: event.read_u32()? as i32,
        param1: event.read_u32()? as i32,
        param2: event.read_u32()? as i32,
        event: event.read_u32()? as i32,
    })
}

fn encode_vehicle_tuning(value: VehicleTuning) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.vehicle_id as u32);
    writer.u32(value.param1 as u32);
    writer.u32(value.param2 as u32);
    writer.u32(value.event as u32);
    Ok(writer.finish())
}

fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
    event.read_u16()
}

fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value);
    Ok(writer.finish())
}
