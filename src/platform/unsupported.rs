use crate::{
    AttachError, BitStream, SendError, SendOptions, event::Registry, runtime::ClientHookStatus,
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

    pub(crate) fn shutdown(&mut self) {}
}
