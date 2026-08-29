use super::*;

#[derive(Debug)]
pub(in crate::platform::win32) enum ConnectionCommand {
    SetGameState(i32),
    ConnectToServer { address: Vec<u8>, port: u16 },
    DisconnectWithReason(u32),
}

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
        self.queue_game_command(GameCommand::Connection(ConnectionCommand::SetGameState(
            state,
        )))
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
        self.queue_game_command(GameCommand::Connection(
            ConnectionCommand::ConnectToServer { address, port },
        ))
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
        self.queue_game_command(GameCommand::Connection(
            ConnectionCommand::DisconnectWithReason(block_duration),
        ))
    }

    pub(super) fn execute_connection_command(
        &self,
        command: ConnectionCommand,
    ) -> Result<(), CommandError> {
        match command {
            ConnectionCommand::SetGameState(state) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_game_state(state)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            ConnectionCommand::ConnectToServer { address, port } => {
                let result = self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .connect_to_server(&address, port)
                            .map_err(|_| CommandError::NativeFailure)
                    });
                if result.is_ok() {
                    self.invalidate_connection_state();
                }
                result
            }
            ConnectionCommand::DisconnectWithReason(block_duration) => {
                let rak_client = self.rak_client.load(Ordering::Acquire) as *mut c_void;
                let result = self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .disconnect_with_reason(rak_client, block_duration)
                            .map_err(|_| CommandError::NativeFailure)
                    });
                if result.is_ok() {
                    self.invalidate_after_disconnect();
                }
                result
            }
        }
    }
}
