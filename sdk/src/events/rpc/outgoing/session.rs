//! Outgoing session and class-selection RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, Rpc, RpcAction},
};

/// MoonLoader's `onSendClientCheckResponse` payload (RPC 103).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCheckResponse {
    pub request_type: u8,
    pub result1: i32,
    pub result2: u8,
}

/// The `onSendSpawn` descriptor.
pub const SEND_SPAWN: Rpc<()> = Rpc::new(52, decode_empty, encode_empty);
/// The `onSendRequestClass` descriptor.
pub const SEND_REQUEST_CLASS: Rpc<i32> = Rpc::new(128, decode_i32, encode_i32);
/// The `onSendRequestSpawn` descriptor.
pub const SEND_REQUEST_SPAWN: Rpc<()> = Rpc::new(129, decode_empty, encode_empty);
/// The `onSendServerStatisticsRequest` descriptor.
pub const SEND_SERVER_STATISTICS_REQUEST: Rpc<()> = Rpc::new(102, decode_empty, encode_empty);
/// The `onSendClientCheckResponse` descriptor.
pub const SEND_CLIENT_CHECK_RESPONSE: Rpc<ClientCheckResponse> = Rpc::new(
    103,
    decode_client_check_response,
    encode_client_check_response,
);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_spawn(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(()) -> RpcAction<()>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_SPAWN, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_request_class(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(i32) -> RpcAction<i32>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_REQUEST_CLASS, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_request_spawn(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(()) -> RpcAction<()>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_REQUEST_SPAWN, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_server_statistics_request(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(()) -> RpcAction<()>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_SERVER_STATISTICS_REQUEST, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_client_check_response(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(ClientCheckResponse) -> RpcAction<ClientCheckResponse>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_CLIENT_CHECK_RESPONSE, handler) }
}

fn decode_client_check_response(event: &mut Event<'_>) -> Result<ClientCheckResponse, EventError> {
    Ok(ClientCheckResponse {
        request_type: event.read_u8()?,
        result1: event.read_u32()? as i32,
        result2: event.read_u8()?,
    })
}

fn encode_client_check_response(value: ClientCheckResponse) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.request_type);
    writer.u32(value.result1 as u32);
    writer.u8(value.result2);
    Ok(writer.finish())
}

fn decode_empty(_event: &mut Event<'_>) -> Result<(), EventError> {
    Ok(())
}

fn encode_empty(_: ()) -> Result<Vec<u8>, EventError> {
    Ok(Vec::new())
}

fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

fn encode_i32(value: i32) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value as u32);
    Ok(writer.finish())
}
