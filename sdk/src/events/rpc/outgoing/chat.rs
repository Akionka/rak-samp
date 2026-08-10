//! Outgoing chat and slash-command RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, MAX_STRING32_BYTES, Rpc, RpcAction},
};

/// The `onSendChat` descriptor.
pub const SEND_CHAT: Rpc<Vec<u8>> = Rpc::new(101, decode_string8, encode_string8);
/// The `onSendCommand` descriptor.
pub const SEND_COMMAND: Rpc<Vec<u8>> = Rpc::new(50, decode_string32, encode_string32);

/// Handles `onSendChat` from an outgoing raw RPC callback.
///
/// # Safety
///
/// See [`crate::events::handle`].
#[allow(dead_code)]
pub(crate) unsafe fn on_send_chat(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(Vec<u8>) -> RpcAction<Vec<u8>>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_CHAT, handler) }
}

/// Handles `onSendCommand` from an outgoing raw RPC callback.
///
/// # Safety
///
/// See [`crate::events::handle`].
#[allow(dead_code)]
pub(crate) unsafe fn on_send_command(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(Vec<u8>) -> RpcAction<Vec<u8>>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_COMMAND, handler) }
}

fn decode_string8(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
    event.read_string8()
}

fn encode_string8(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.string8(&value)?;
    Ok(writer.finish())
}

fn decode_string32(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
    event.read_string32(MAX_STRING32_BYTES)
}

fn encode_string32(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.string32(&value)?;
    Ok(writer.finish())
}
