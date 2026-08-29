use super::*;

impl BackendState {
    pub(in crate::platform::win32) fn submit_samp_game_state(
        &self,
        state: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !matches!(state, 0 | 9 | 13 | 14 | 15 | 18)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetGameState(state))
    }

    pub(in crate::platform::win32) fn submit_connect_to_server(
        &self,
        address: Vec<u8>,
        port: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || address.is_empty()
            || address.len() > 256
            || address.contains(&0)
            || port == 0
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ConnectToServer { address, port })
    }

    pub(in crate::platform::win32) fn submit_disconnect_with_reason(
        &self,
        block_duration: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DisconnectWithReason(block_duration))
    }
}
