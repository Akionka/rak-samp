use super::*;

#[derive(Debug)]
pub(in crate::platform::win32) enum NetworkCommand {
    SetSendRate {
        kind: u8,
        milliseconds: u32,
    },
    SendPacket {
        id: u8,
        payload: BitStream,
        options: SendOptions,
    },
    SendRpc {
        id: u8,
        payload: BitStream,
        options: SendOptions,
    },
    EmulateIncomingPacket {
        id: u8,
        payload: BitStream,
    },
    EmulateIncomingRpc {
        id: u8,
        payload: BitStream,
    },
}

impl BackendState {
    pub(in crate::platform::win32) fn queue_network_command(
        &self,
        command: GameCommand,
    ) -> Result<bool, SendError> {
        let id = self.submit_network_command(command)?;
        self.game_commands.detach(id).map_err(command_send_error)?;
        Ok(true)
    }

    pub(in crate::platform::win32) fn submit_network_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, SendError> {
        self.submit_game_command(command)
            .map_err(command_send_error)
    }

    pub(in crate::platform::win32) fn send_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::Network(NetworkCommand::SendPacket {
            id: packet_id,
            payload: payload.clone(),
            options,
        }))
    }

    pub(in crate::platform::win32) fn send_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::Network(NetworkCommand::SendRpc {
            id: rpc_id,
            payload: payload.clone(),
            options,
        }))
    }

    pub(in crate::platform::win32) fn submit_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::Network(NetworkCommand::SendPacket {
            id: packet_id,
            payload: payload.clone(),
            options,
        }))
    }

    pub(in crate::platform::win32) fn submit_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::Network(NetworkCommand::SendRpc {
            id: rpc_id,
            payload: payload.clone(),
            options,
        }))
    }

    pub(in crate::platform::win32) fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::Network(
            NetworkCommand::EmulateIncomingPacket {
                id: packet_id,
                payload,
            },
        ))
    }

    pub(in crate::platform::win32) fn emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::Network(NetworkCommand::EmulateIncomingRpc {
            id: rpc_id,
            payload,
        }))
    }

    pub(in crate::platform::win32) fn submit_emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::Network(
            NetworkCommand::EmulateIncomingPacket {
                id: packet_id,
                payload,
            },
        ))
    }

    pub(in crate::platform::win32) fn submit_emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::Network(NetworkCommand::EmulateIncomingRpc {
            id: rpc_id,
            payload,
        }))
    }

    pub(in crate::platform::win32) fn submit_send_rate(
        &self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !matches!(kind, 0..=2)
            || i32::try_from(milliseconds).is_err()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::Network(NetworkCommand::SetSendRate {
            kind,
            milliseconds,
        }))
    }

    pub(super) fn execute_network_command(
        &self,
        command: NetworkCommand,
    ) -> Result<(), CommandError> {
        match command {
            NetworkCommand::SetSendRate { kind, milliseconds } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_send_rate(kind, milliseconds)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            NetworkCommand::SendPacket {
                id,
                payload,
                options,
            } => self
                .send_packet_native(id, &payload, options)
                .and_then(super::super::sent_game_command_result)
                .map_err(|_| CommandError::NativeFailure),
            NetworkCommand::SendRpc {
                id,
                payload,
                options,
            } => self
                .send_rpc_native(id, &payload, options)
                .and_then(super::super::sent_game_command_result)
                .map_err(|_| CommandError::NativeFailure),
            NetworkCommand::EmulateIncomingPacket { id, payload } => self
                .emulate_incoming_packet_native(id, payload)
                .map(|_| ())
                .map_err(|_| CommandError::NativeFailure),
            NetworkCommand::EmulateIncomingRpc { id, payload } => self
                .emulate_incoming_rpc_native(id, payload)
                .map(|_| ())
                .map_err(|_| CommandError::NativeFailure),
        }
    }
}
