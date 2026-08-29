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
pub(in crate::platform::win32) use text_labels::TextLabelCommand;
use textdraws::TextdrawCommand;
pub(in crate::platform::win32) use ui::UiCommand;
#[derive(Debug)]
pub(super) enum GameCommand {
    Ui(UiCommand),
    Connection(ConnectionCommand),
    TextLabel(TextLabelCommand),
    Textdraw(TextdrawCommand),
    SpawnLocalPlayer,
    SetLocalPlayerSpecialAction(u8),
    SetLocalPlayerName(Vec<u8>),
    ForceUnoccupiedSync { vehicle: u16, seat: u8 },
    ForceAimSync,
    ForceOnfootSync,
    ForceStatsSync,
    ForceTrailerSync { trailer: u16 },
    ForcePassengerSync { vehicle: u16, seat: u8 },
    ForceWeaponsSync,
    ForceVehicleSync { vehicle: u16 },
    SetPlayerColour { id: u16, colour: u32 },
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
                GameCommand::SpawnLocalPlayer => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .spawn_local_player()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetLocalPlayerSpecialAction(action) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_local_player_special_action(action)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetLocalPlayerName(name) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_local_player_name(&name)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceUnoccupiedSync { vehicle, seat } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_unoccupied_sync(vehicle, seat)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceAimSync => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_aim_sync()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceOnfootSync => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_onfoot_sync()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceStatsSync => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_stats_sync()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceTrailerSync { trailer } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_trailer_sync(trailer)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceVehicleSync { vehicle } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_vehicle_sync(vehicle)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForcePassengerSync { vehicle, seat } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_passenger_sync(vehicle, seat)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceWeaponsSync => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_weapons_sync()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetPlayerColour { id, colour } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_player_colour(id, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
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
