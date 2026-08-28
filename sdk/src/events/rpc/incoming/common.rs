use super::types::ShowDialog;
use crate::events::core::PayloadWriter;
use crate::{
    HostApi,
    events::{
        EncodedPayload, Event, EventError, IncomingRpc, MAX_ENCODED_STRING_BYTES, Vector2, Vector3,
    },
};

/// The `onShowDialog` descriptor.
pub const SHOW_DIALOG: IncomingRpc<ShowDialog> =
    IncomingRpc::new_bits(61, decode_show_dialog, encode_show_dialog);

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

pub(super) fn decode_bool8(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_u8()? != 0)
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

pub(super) fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

pub(super) fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
    event.read_u16()
}

pub(super) fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
    Ok(value.to_le_bytes().to_vec())
}
