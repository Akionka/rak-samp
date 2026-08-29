//! Game-thread command submission and execution helpers.

use super::*;

mod connection;
mod network;
mod players;
mod text_labels;
mod textdraws;
mod ui;

use connection::ConnectionCommand;
pub(in crate::platform::win32) use network::NetworkCommand;
use players::PlayerCommand;
pub(in crate::platform::win32) use text_labels::TextLabelCommand;
use textdraws::TextdrawCommand;
pub(in crate::platform::win32) use ui::UiCommand;
#[derive(Debug)]
pub(super) enum GameCommand {
    Ui(UiCommand),
    Connection(ConnectionCommand),
    TextLabel(TextLabelCommand),
    Textdraw(TextdrawCommand),
    Player(PlayerCommand),
    Network(NetworkCommand),
}

impl BackendState {
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

    pub(super) fn execute_game_commands(&self, commands: Vec<QueuedCommand<GameCommand>>) {
        for queued in commands {
            let result = match queued.command {
                GameCommand::Ui(command) => self.execute_ui_command(command),
                GameCommand::Connection(command) => self.execute_connection_command(command),
                GameCommand::Textdraw(command) => self.execute_textdraw_command(command),
                GameCommand::TextLabel(command) => {
                    self.execute_text_label_command(queued.id, command)
                }
                GameCommand::Player(command) => self.execute_player_command(command),
                GameCommand::Network(command) => self.execute_network_command(command),
            };
            if !self
                .game_command_completion_diagnostic_logged
                .swap(true, Ordering::AcqRel)
            {
                // Do not include the command variant or its payload: plugins
                // can own text, packet, or RPC data.
                log::debug!(
                    "completed first game command: id={}, success={}",
                    queued.id,
                    result.is_ok(),
                );
            }
            match result {
                Ok(()) => self.game_commands.complete(queued.id, Ok(())),
                Err(error) => {
                    // Every command owns its plugin-provided payload. Keep logs
                    // free of dialog text, chat text, and death-window names.
                    log::debug!("game command failed: {error:?}");
                    self.game_commands
                        .complete(queued.id, Err(CommandError::NativeFailure));
                }
            }
        }
    }
}
