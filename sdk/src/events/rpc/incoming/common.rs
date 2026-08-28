//! Profile-neutral SDK-owned incoming RPC descriptors and payloads.

use crate::events::core::PayloadWriter;
use crate::{
    HostApi,
    events::{EncodedPayload, Event, EventError, IncomingRpc, MAX_ENCODED_STRING_BYTES},
};

/// MoonLoader's `onShowDialog` payload (RPC 61).
#[derive(Clone, Debug, PartialEq)]
pub struct ShowDialog {
    pub dialog_id: u16,
    pub style: u8,
    pub title: Vec<u8>,
    pub button1: Vec<u8>,
    pub button2: Vec<u8>,
    pub text: Vec<u8>,
}

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
