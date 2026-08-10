//! Outgoing object-editing RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{EncodedPayload, Event, EventError, Rpc, RpcAction, Vector3},
};

/// MoonLoader's `onSendEnterEditObject` payload (RPC 27).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnterEditObject {
    pub object_type: i32,
    pub object_id: u16,
    pub model_id: i32,
    pub position: Vector3,
}

/// MoonLoader's `onSendEditAttachedObject` payload (RPC 116).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditAttachedObject {
    pub response: i32,
    pub index: i32,
    pub model_id: i32,
    pub bone: i32,
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
    pub color1: i32,
    pub color2: i32,
}

/// MoonLoader's `onSendEditObject` payload (RPC 117).
///
/// `player_object` is a one-bit RakNet boolean; replacements preserve that exact-bit layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditObject {
    pub player_object: bool,
    pub object_id: u16,
    pub response: i32,
    pub position: Vector3,
    pub rotation: Vector3,
}

/// The `onSendEnterEditObject` descriptor.
pub const SEND_ENTER_EDIT_OBJECT: Rpc<EnterEditObject> =
    Rpc::new(27, decode_enter_edit_object, encode_enter_edit_object);
/// The `onSendEditAttachedObject` descriptor.
pub const SEND_EDIT_ATTACHED_OBJECT: Rpc<EditAttachedObject> = Rpc::new(
    116,
    decode_edit_attached_object,
    encode_edit_attached_object,
);
/// The `onSendEditObject` descriptor.
pub const SEND_EDIT_OBJECT: Rpc<EditObject> =
    Rpc::new_bits(117, decode_edit_object, encode_edit_object);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_enter_edit_object(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(EnterEditObject) -> RpcAction<EnterEditObject>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_ENTER_EDIT_OBJECT, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_edit_attached_object(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(EditAttachedObject) -> RpcAction<EditAttachedObject>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_EDIT_ATTACHED_OBJECT, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_edit_object(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(EditObject) -> RpcAction<EditObject>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_EDIT_OBJECT, handler) }
}

fn decode_enter_edit_object(event: &mut Event<'_>) -> Result<EnterEditObject, EventError> {
    Ok(EnterEditObject {
        object_type: event.read_u32()? as i32,
        object_id: event.read_u16()?,
        model_id: event.read_u32()? as i32,
        position: decode_vector3(event)?,
    })
}

fn encode_enter_edit_object(value: EnterEditObject) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.object_type as u32);
    writer.u16(value.object_id);
    writer.u32(value.model_id as u32);
    writer.vector3(value.position);
    Ok(writer.finish())
}

fn decode_edit_attached_object(event: &mut Event<'_>) -> Result<EditAttachedObject, EventError> {
    Ok(EditAttachedObject {
        response: event.read_u32()? as i32,
        index: event.read_u32()? as i32,
        model_id: event.read_u32()? as i32,
        bone: event.read_u32()? as i32,
        position: decode_vector3(event)?,
        rotation: decode_vector3(event)?,
        scale: decode_vector3(event)?,
        color1: event.read_u32()? as i32,
        color2: event.read_u32()? as i32,
    })
}

fn encode_edit_attached_object(value: EditAttachedObject) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.response as u32);
    writer.u32(value.index as u32);
    writer.u32(value.model_id as u32);
    writer.u32(value.bone as u32);
    writer.vector3(value.position);
    writer.vector3(value.rotation);
    writer.vector3(value.scale);
    writer.u32(value.color1 as u32);
    writer.u32(value.color2 as u32);
    Ok(writer.finish())
}

fn decode_edit_object(event: &mut Event<'_>) -> Result<EditObject, EventError> {
    Ok(EditObject {
        player_object: event.read_bits(1)?[0] & 0x80 != 0,
        object_id: event.read_u16()?,
        response: event.read_u32()? as i32,
        position: decode_vector3(event)?,
        rotation: decode_vector3(event)?,
    })
}

fn encode_edit_object(_api: HostApi, value: EditObject) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bit(value.player_object);
    writer.u16(value.object_id);
    writer.u32(value.response as u32);
    writer.vector3(value.position);
    writer.vector3(value.rotation);
    Ok(writer.finish_bits())
}

fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    Ok(Vector3 {
        x: event.read_f32()?,
        y: event.read_f32()?,
        z: event.read_f32()?,
    })
}
