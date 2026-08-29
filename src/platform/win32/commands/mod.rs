//! Game-thread command submission and execution helpers.

use super::*;

mod connection;
mod network;
mod players;
mod text_labels;
mod textdraws;
mod ui;
#[derive(Debug)]
pub(super) enum GameCommand {
    ShowDialog(LocalDialogRequest),
    AddChatMessage(LocalChatMessageRequest),
    AddDeathMessage(LocalDeathMessageRequest),
    CloseDialog(u8),
    SetChatInputText(Vec<u8>),
    SetChatInputEnabled(bool),
    ProcessChatInput(Vec<u8>),
    RegisterChatCommand {
        subscription: u64,
        slot: u8,
        name: Vec<u8>,
    },
    UnregisterChatCommand {
        subscription: u64,
        name: Vec<u8>,
    },
    SetChatDisplayMode(i32),
    SetChatEntry {
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    },
    SetCursorMode(i32),
    ToggleCursor(bool),
    SetScoreboardOpen(bool),
    SetDialogClientSide(bool),
    SetDialogSelectedItem(i32),
    SetDialogEditboxText(Vec<u8>),
    SetGameState(i32),
    ConnectToServer {
        address: Vec<u8>,
        port: u16,
    },
    DisconnectWithReason(u32),
    DeleteTextLabel(u16),
    CreateTextLabel {
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    },
    CreateTextLabelAuto {
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    },
    SetTextLabelText {
        id: u16,
        text: Vec<u8>,
    },
    CreateTextdraw {
        id: u16,
        text: Vec<u8>,
        x: f32,
        y: f32,
    },
    DeleteTextdraw(u16),
    SetTextdrawPosition {
        id: u16,
        x: f32,
        y: f32,
    },
    SetTextdrawStyle {
        id: u16,
        style: i32,
    },
    SetTextdrawLetterStyle {
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    },
    SetTextdrawProportional {
        id: u16,
        proportional: bool,
    },
    SetTextdrawShadow {
        id: u16,
        shadow: u8,
        colour: u32,
    },
    SetTextdrawOutline {
        id: u16,
        outline: u8,
        colour: u32,
    },
    SetTextdrawBox {
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    },
    SetTextdrawAlignment {
        id: u16,
        alignment: u8,
    },
    SetTextdrawString {
        id: u16,
        text: Vec<u8>,
    },
    SetTextdrawModelStyle {
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    },
    SpawnLocalPlayer,
    SetLocalPlayerSpecialAction(u8),
    SetLocalPlayerName(Vec<u8>),
    ForceUnoccupiedSync {
        vehicle: u16,
        seat: u8,
    },
    ForceAimSync,
    ForceOnfootSync,
    ForceStatsSync,
    ForceTrailerSync {
        trailer: u16,
    },
    ForcePassengerSync {
        vehicle: u16,
        seat: u8,
    },
    ForceWeaponsSync,
    ForceVehicleSync {
        vehicle: u16,
    },
    SetPlayerColour {
        id: u16,
        colour: u32,
    },
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
                GameCommand::ShowDialog(request) => self
                    .connection_profile()
                    .filter(|profile| profile.dialog_is_ready())
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .show_dialog(request)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::CloseDialog(button) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .close_dialog(button)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatInputText(text) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_input_text(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatInputEnabled(enabled) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_input_enabled(enabled)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ProcessChatInput(text) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .process_chat_input(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::RegisterChatCommand {
                    subscription,
                    slot,
                    name,
                } => {
                    let result = self
                        .connection_profile()
                        .ok_or(CommandError::NativeFailure)
                        .and_then(|profile| {
                            profile
                                .register_chat_command(
                                    &name,
                                    crate::host_api::chat_commands::trampoline(slot),
                                )
                                .map_err(|_| CommandError::NativeFailure)
                        });
                    crate::host_api::chat_commands::finish_registration(
                        subscription,
                        result.is_ok(),
                    );
                    result
                }
                GameCommand::UnregisterChatCommand { subscription, name } => {
                    let result = self
                        .connection_profile()
                        .ok_or(CommandError::NativeFailure)
                        .and_then(|profile| {
                            profile
                                .unregister_chat_command(&name)
                                .map_err(|_| CommandError::NativeFailure)
                        });
                    crate::host_api::chat_commands::finish_unregistration(
                        subscription,
                        result.is_ok(),
                    );
                    result
                }
                GameCommand::SetChatDisplayMode(mode) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_display_mode(mode)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatEntry {
                    id,
                    text,
                    prefix,
                    text_colour,
                    prefix_colour,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_entry(id, &text, &prefix, text_colour, prefix_colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::AddChatMessage(request) => self
                    .connection_profile()
                    .filter(|profile| profile.chat_is_ready())
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .show_chat_message(request)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::AddDeathMessage(request) => self
                    .connection_profile()
                    .filter(|profile| profile.death_window_is_ready())
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .show_death_message(request)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetCursorMode(mode) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_cursor_mode(mode)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ToggleCursor(show) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .toggle_cursor(show)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetScoreboardOpen(open) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_scoreboard_open(open)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogClientSide(client_side) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_client_side(client_side)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogSelectedItem(selected) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_selected_item(selected)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogEditboxText(text) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_editbox_text(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetGameState(state) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_game_state(state)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ConnectToServer { address, port } => {
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
                GameCommand::DisconnectWithReason(block_duration) => {
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
                GameCommand::DeleteTextdraw(id) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .delete_textdraw(id)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.publish_deleted_textdraw(id);
                        Ok(())
                    }),
                GameCommand::CreateTextdraw { id, text, x, y } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .create_textdraw(id, &text, x, y)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.publish_created_textdraw(id);
                        Ok(())
                    }),
                GameCommand::DeleteTextLabel(id) => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .delete_text_label(id)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.publish_deleted_text_label(id);
                        Ok(())
                    }),
                GameCommand::CreateTextLabel {
                    id,
                    text,
                    colour,
                    position,
                    draw_distance,
                    behind_walls,
                    attached_player_id,
                    attached_vehicle_id,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .create_text_label(
                                id,
                                &text,
                                colour,
                                position,
                                draw_distance,
                                behind_walls,
                                attached_player_id,
                                attached_vehicle_id,
                            )
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::CreateTextLabelAuto {
                    text,
                    colour,
                    position,
                    draw_distance,
                    behind_walls,
                    attached_player_id,
                    attached_vehicle_id,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        let id = profile
                            .first_free_text_label_id()
                            .map_err(|_| CommandError::NativeFailure)?;
                        profile
                            .create_text_label(
                                id,
                                &text,
                                colour,
                                position,
                                draw_distance,
                                behind_walls,
                                attached_player_id,
                                attached_vehicle_id,
                            )
                            .map_err(|_| CommandError::NativeFailure)?;
                        let snapshot = profile
                            .text_label(id)
                            .map_err(|_| CommandError::NativeFailure)?
                            .ok_or(CommandError::NativeFailure)?;
                        self.publish_created_text_label(id, snapshot);
                        self.complete_created_text_label(queued.id, id);
                        Ok(())
                    }),
                GameCommand::SetTextLabelText { id, text } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        let label = profile
                            .text_label(id)
                            .map_err(|_| CommandError::NativeFailure)?
                            .ok_or(CommandError::NativeFailure)?;
                        profile
                            .create_text_label(
                                id,
                                &text,
                                label.colour,
                                label.position,
                                label.draw_distance,
                                label.behind_walls,
                                label.attached_player_id.unwrap_or(u16::MAX),
                                label.attached_vehicle_id.unwrap_or(u16::MAX),
                            )
                            .map_err(|_| CommandError::NativeFailure)?;
                        let snapshot = profile
                            .text_label(id)
                            .map_err(|_| CommandError::NativeFailure)?
                            .ok_or(CommandError::NativeFailure)?;
                        self.publish_created_text_label(id, snapshot);
                        Ok(())
                    }),
                GameCommand::SetTextdrawPosition { id, x, y } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_position(id, x, y)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawStyle { id, style } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_style(id, style)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawLetterStyle {
                    id,
                    width,
                    height,
                    colour,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_letter_style(id, width, height, colour)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawProportional { id, proportional } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_proportional(id, proportional)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawShadow { id, shadow, colour } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_shadow(id, shadow, colour)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawOutline {
                    id,
                    outline,
                    colour,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_outline(id, outline, colour)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawBox {
                    id,
                    enabled,
                    colour,
                    width,
                    height,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_box(id, enabled, colour, width, height)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawAlignment { id, alignment } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_alignment(id, alignment)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawString { id, text } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_string(id, &text)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
                GameCommand::SetTextdrawModelStyle {
                    id,
                    rotation,
                    zoom,
                    colour1,
                    colour2,
                } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
                            .map_err(|_| CommandError::NativeFailure)?;
                        self.invalidate_textdraw_snapshot(id);
                        Ok(())
                    }),
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
                GameCommand::SetSendRate { kind, milliseconds } => self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_send_rate(kind, milliseconds)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SendPacket {
                    id,
                    payload,
                    options,
                } => self
                    .send_packet_native(id, &payload, options)
                    .and_then(sent_game_command_result)
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::SendRpc {
                    id,
                    payload,
                    options,
                } => self
                    .send_rpc_native(id, &payload, options)
                    .and_then(sent_game_command_result)
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::EmulateIncomingPacket { id, payload } => self
                    .emulate_incoming_packet_native(id, payload)
                    .map(|_| ())
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::EmulateIncomingRpc { id, payload } => self
                    .emulate_incoming_rpc_native(id, payload)
                    .map(|_| ())
                    .map_err(|_| CommandError::NativeFailure),
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
