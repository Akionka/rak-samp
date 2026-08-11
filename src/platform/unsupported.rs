use crate::{
    AttachError, BitStream, SampVersion, SendError, SendOptions,
    event::Registry,
    runtime::{
        AnimationSnapshot, ClientHookStatus, CodecError, DirectClientError, GangzoneSnapshot,
        InCarSyncSnapshot, LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest,
        LocalDialogSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot, PassengerSyncSnapshot,
        PlayerInfoSnapshot, ServerInfoSnapshot, TextLabelSnapshot, TextdrawSnapshot,
        TrailerSyncSnapshot,
    },
};
use std::sync::Arc;

pub(crate) struct Backend;

pub(crate) fn attach(_registry: Arc<Registry>) -> Result<Backend, AttachError> {
    Err(AttachError::UnsupportedPlatform)
}

impl Backend {
    pub(crate) fn client_hook_status(&self) -> ClientHookStatus {
        ClientHookStatus::Failed
    }

    pub(crate) fn samp_version(&self) -> SampVersion {
        unreachable!("the unsupported platform backend cannot be constructed")
    }

    pub(crate) fn encode_string(&self, _value: &[u8]) -> Result<BitStream, CodecError> {
        Err(CodecError::ClientNotReady)
    }

    pub(crate) fn decode_string(
        &self,
        _payload: &mut BitStream,
        _output: &mut [u8],
    ) -> Result<usize, CodecError> {
        Err(CodecError::ClientNotReady)
    }

    pub(crate) fn send_packet(
        &self,
        _packet_id: u8,
        _payload: &BitStream,
        _options: SendOptions,
    ) -> Result<bool, SendError> {
        Err(SendError::ClientNotReady)
    }

    pub(crate) fn send_rpc(
        &self,
        _rpc_id: u8,
        _payload: &BitStream,
        _options: SendOptions,
    ) -> Result<bool, SendError> {
        Err(SendError::ClientNotReady)
    }

    pub(crate) fn emulate_incoming_packet(
        &self,
        _packet_id: u8,
        _payload: BitStream,
    ) -> Result<bool, SendError> {
        Err(SendError::ClientNotReady)
    }

    pub(crate) fn emulate_incoming_rpc(
        &self,
        _rpc_id: u8,
        _payload: BitStream,
    ) -> Result<bool, SendError> {
        Err(SendError::ClientNotReady)
    }

    pub(crate) fn show_local_dialog(
        &self,
        _request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn player_info(
        &self,
        _id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn onfoot_sync(
        &self,
        _id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn vehicle_sync(
        &self,
        _id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn passenger_sync(
        &self,
        _id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn trailer_sync(
        &self,
        _id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn player_defined(&self, _id: u16) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn player_paused(&self, _id: u16) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn player_count(&self, _include_npcs: bool) -> Result<u16, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn vehicle_exists(&self, _id: u16) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn text_label_exists(&self, _id: u16) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn textdraw_exists(&self, _pool_index: u16) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn object_exists(&self, _id: u16) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn gangzone(&self, _id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn text_label(
        &self,
        _id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn textdraw(
        &self,
        _pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn show_local_chat_message(
        &self,
        _request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn show_local_death_message(
        &self,
        _request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_animation(&self, _id: u16) -> Result<AnimationSnapshot, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn local_animation_id(
        &self,
        _name: &[u8],
        _file: &[u8],
    ) -> Result<Option<u16>, DirectClientError> {
        Err(DirectClientError::UnsupportedVersion)
    }

    pub(crate) fn shutdown(&mut self) {}
}
