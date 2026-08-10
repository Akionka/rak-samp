//! Outgoing dialog, textdraw, player-click, and menu RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, Rpc, RpcAction},
};

/// MoonLoader's `onSendDialogResponse` payload (RPC 62).
#[derive(Clone, Debug, PartialEq)]
pub struct DialogResponse {
    pub dialog_id: u16,
    pub button: u8,
    pub list_item: u16,
    pub input: Vec<u8>,
}

/// MoonLoader's `onSendClickPlayer` payload (RPC 23).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickPlayer {
    pub player_id: u16,
    pub source: u8,
}

/// The `onSendDialogResponse` descriptor.
pub const SEND_DIALOG_RESPONSE: Rpc<DialogResponse> =
    Rpc::new(62, decode_dialog_response, encode_dialog_response);
/// The `onSendClickPlayer` descriptor.
pub const SEND_CLICK_PLAYER: Rpc<ClickPlayer> =
    Rpc::new(23, decode_click_player, encode_click_player);
/// The `onSendClickTextDraw` descriptor.
pub const SEND_CLICK_TEXT_DRAW: Rpc<u16> = Rpc::new(83, decode_u16, encode_u16);
/// The `onSendMenuSelect` descriptor.
pub const SEND_MENU_SELECT: Rpc<u8> = Rpc::new(132, decode_u8, encode_u8);
/// The `onSendQuitMenu` descriptor.
pub const SEND_QUIT_MENU: Rpc<()> = Rpc::new(140, decode_empty, encode_empty);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_dialog_response(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(DialogResponse) -> RpcAction<DialogResponse>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_DIALOG_RESPONSE, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_click_player(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(ClickPlayer) -> RpcAction<ClickPlayer>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_CLICK_PLAYER, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_click_text_draw(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(u16) -> RpcAction<u16>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_CLICK_TEXT_DRAW, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_menu_select(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(u8) -> RpcAction<u8>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_MENU_SELECT, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_quit_menu(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(()) -> RpcAction<()>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_QUIT_MENU, handler) }
}

fn decode_dialog_response(event: &mut Event<'_>) -> Result<DialogResponse, EventError> {
    Ok(DialogResponse {
        dialog_id: event.read_u16()?,
        button: event.read_u8()?,
        list_item: event.read_u16()?,
        input: event.read_string8()?,
    })
}

fn encode_dialog_response(value: DialogResponse) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.dialog_id);
    writer.u8(value.button);
    writer.u16(value.list_item);
    writer.string8(&value.input)?;
    Ok(writer.finish())
}

fn decode_click_player(event: &mut Event<'_>) -> Result<ClickPlayer, EventError> {
    Ok(ClickPlayer {
        player_id: event.read_u16()?,
        source: event.read_u8()?,
    })
}

fn encode_click_player(value: ClickPlayer) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u8(value.source);
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

fn decode_u8(event: &mut Event<'_>) -> Result<u8, EventError> {
    event.read_u8()
}

fn encode_u8(value: u8) -> Result<Vec<u8>, EventError> {
    Ok(vec![value])
}

fn decode_empty(_event: &mut Event<'_>) -> Result<(), EventError> {
    Ok(())
}

fn encode_empty(_: ()) -> Result<Vec<u8>, EventError> {
    Ok(Vec::new())
}
