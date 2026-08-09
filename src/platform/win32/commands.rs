//! Producer-side game-thread command submission.

use super::{BackendState, GameCommand, command_send_error};
use crate::{
    BitStream, SendError, SendOptions,
    command::{CommandError, CommandId},
    runtime::{
        DirectClientError, LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest,
    },
};

impl BackendState {
    pub(super) fn queue_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::ShowDialog(request))
    }

    pub(super) fn queue_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::AddChatMessage(request))
    }

    pub(super) fn queue_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::AddDeathMessage(request))
    }

    pub(super) fn queue_game_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, DirectClientError> {
        self.submit_game_command(command)
            .map_err(|error| match error {
                CommandError::QueueFull => DirectClientError::QueueFull,
                CommandError::ShuttingDown
                | CommandError::NativeFailure
                | CommandError::UnknownReceipt
                | CommandError::TimedOut
                | CommandError::WaitRejected => DirectClientError::NotReady,
            })
    }

    pub(super) fn submit_game_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, CommandError> {
        self.game_commands.submit(command)
    }

    pub(super) fn queue_network_command(&self, command: GameCommand) -> Result<bool, SendError> {
        let id = self.submit_network_command(command)?;
        self.game_commands.detach(id).map_err(command_send_error)?;
        Ok(true)
    }

    pub(super) fn submit_network_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, SendError> {
        self.submit_game_command(command)
            .map_err(command_send_error)
    }

    pub(super) fn send_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::SendPacket {
            id: packet_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn send_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::SendRpc {
            id: rpc_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn submit_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::SendPacket {
            id: packet_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn submit_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::SendRpc {
            id: rpc_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::EmulateIncomingPacket {
            id: packet_id,
            payload,
        })
    }

    pub(super) fn emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::EmulateIncomingRpc {
            id: rpc_id,
            payload,
        })
    }

    pub(super) fn submit_emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::EmulateIncomingPacket {
            id: packet_id,
            payload,
        })
    }

    pub(super) fn submit_emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::EmulateIncomingRpc {
            id: rpc_id,
            payload,
        })
    }
}
