//! Exact-bit outgoing object-editing OutgoingRpc codec.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{EncodedPayload, Event, EventError, OutgoingRpc, RpcAction, Vector3},
};

/// MoonLoader's `onSendEditObject` payload (OutgoingRpc 117).
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

/// The `onSendEditObject` descriptor.
pub const SEND_EDIT_OBJECT: OutgoingRpc<EditObject> =
    OutgoingRpc::new_bits(117, decode_edit_object, encode_edit_object);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_edit_object(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(EditObject) -> RpcAction<EditObject>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_EDIT_OBJECT, handler) }
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
