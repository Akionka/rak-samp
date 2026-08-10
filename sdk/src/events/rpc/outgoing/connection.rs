//! Outgoing client and NPC join RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, Rpc, RpcAction},
};

/// MoonLoader's `onSendClientJoin` payload (RPC 25).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientJoin {
    pub version: i32,
    pub mod_id: u8,
    pub nickname: Vec<u8>,
    pub challenge_response: i32,
    pub join_auth_key: Vec<u8>,
    pub client_version: Vec<u8>,
    pub challenge_response2: i32,
}

/// MoonLoader's `onSendNPCJoin` payload (RPC 54).
#[derive(Clone, Debug, PartialEq)]
pub struct NpcJoin {
    pub version: i32,
    pub mod_id: u8,
    pub nickname: Vec<u8>,
    pub challenge_response: i32,
}

/// The `onSendClientJoin` descriptor.
pub const SEND_CLIENT_JOIN: Rpc<ClientJoin> = Rpc::new(25, decode_client_join, encode_client_join);
/// The `onSendNPCJoin` descriptor.
pub const SEND_NPC_JOIN: Rpc<NpcJoin> = Rpc::new(54, decode_npc_join, encode_npc_join);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_client_join(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(ClientJoin) -> RpcAction<ClientJoin>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_CLIENT_JOIN, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_npc_join(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(NpcJoin) -> RpcAction<NpcJoin>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_NPC_JOIN, handler) }
}

fn decode_client_join(event: &mut Event<'_>) -> Result<ClientJoin, EventError> {
    Ok(ClientJoin {
        version: event.read_u32()? as i32,
        mod_id: event.read_u8()?,
        nickname: event.read_string8()?,
        challenge_response: event.read_u32()? as i32,
        join_auth_key: event.read_string8()?,
        client_version: event.read_string8()?,
        challenge_response2: event.read_u32()? as i32,
    })
}
fn encode_client_join(value: ClientJoin) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.version as u32);
    writer.u8(value.mod_id);
    writer.string8(&value.nickname)?;
    writer.u32(value.challenge_response as u32);
    writer.string8(&value.join_auth_key)?;
    writer.string8(&value.client_version)?;
    writer.u32(value.challenge_response2 as u32);
    Ok(writer.finish())
}
fn decode_npc_join(event: &mut Event<'_>) -> Result<NpcJoin, EventError> {
    Ok(NpcJoin {
        version: event.read_u32()? as i32,
        mod_id: event.read_u8()?,
        nickname: event.read_string8()?,
        challenge_response: event.read_u32()? as i32,
    })
}
fn encode_npc_join(value: NpcJoin) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.version as u32);
    writer.u8(value.mod_id);
    writer.string8(&value.nickname)?;
    writer.u32(value.challenge_response as u32);
    Ok(writer.finish())
}
