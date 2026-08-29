use super::*;

#[derive(Debug)]
pub(in crate::platform::win32) enum UiCommand {
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
}

impl BackendState {
    fn queue_ui_command(&self, command: UiCommand) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::Ui(command))
    }
    pub(in crate::platform::win32) fn queue_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_ui_command(UiCommand::ShowDialog(request))
    }

    pub(in crate::platform::win32) fn queue_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_ui_command(UiCommand::AddChatMessage(request))
    }

    pub(in crate::platform::win32) fn queue_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_ui_command(UiCommand::AddDeathMessage(request))
    }

    pub(in crate::platform::win32) fn show_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_dialog(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(in crate::platform::win32) fn show_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_chat_message(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(in crate::platform::win32) fn show_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_death_message(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(in crate::platform::win32) fn submit_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_dialog(request)
    }

    pub(in crate::platform::win32) fn submit_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_chat_message(request)
    }

    pub(in crate::platform::win32) fn submit_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_death_message(request)
    }

    pub(in crate::platform::win32) fn submit_local_cursor_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetCursorMode(mode))
    }

    pub(in crate::platform::win32) fn submit_local_chat_display_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(mode, 0..=2) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetChatDisplayMode(mode))
    }

    pub(in crate::platform::win32) fn submit_local_chat_entry(
        &self,
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || id >= 100
            || text.len() >= 144
            || prefix.len() >= 28
            || text.contains(&0)
            || prefix.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetChatEntry {
            id,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
    }

    pub(in crate::platform::win32) fn submit_local_dialog_close(
        &self,
        button: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || button > 1 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::CloseDialog(button))
    }

    pub(in crate::platform::win32) fn submit_local_chat_input_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetChatInputText(text))
    }

    pub(in crate::platform::win32) fn submit_local_chat_input_enabled(
        &self,
        enabled: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetChatInputEnabled(enabled))
    }

    pub(in crate::platform::win32) fn submit_local_chat_input_process(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::ProcessChatInput(text))
    }

    pub(in crate::platform::win32) fn submit_register_chat_command(
        &self,
        subscription: u64,
        slot: u8,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || subscription == 0
            || usize::from(slot) >= 144
            || name.is_empty()
            || name.len() > 32
            || name.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::RegisterChatCommand {
            subscription,
            slot,
            name,
        })
    }

    pub(in crate::platform::win32) fn submit_unregister_chat_command(
        &self,
        subscription: u64,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || subscription == 0
            || name.is_empty()
            || name.len() > 32
            || name.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::UnregisterChatCommand { subscription, name })
    }

    pub(in crate::platform::win32) fn submit_local_cursor_toggle(
        &self,
        show: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::ToggleCursor(show))
    }

    pub(in crate::platform::win32) fn submit_local_scoreboard_open(
        &self,
        open: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetScoreboardOpen(open))
    }

    pub(in crate::platform::win32) fn submit_local_dialog_client_side(
        &self,
        client_side: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetDialogClientSide(client_side))
    }

    pub(in crate::platform::win32) fn submit_local_dialog_selected_item(
        &self,
        selected: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetDialogSelectedItem(selected))
    }

    pub(in crate::platform::win32) fn submit_local_dialog_editbox_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_ui_command(UiCommand::SetDialogEditboxText(text))
    }

    pub(super) fn execute_ui_command(&self, command: UiCommand) -> Result<(), CommandError> {
        match command {
            UiCommand::ShowDialog(request) => self
                .connection_profile()
                .filter(|profile| profile.dialog_is_ready())
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .show_dialog(request)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::CloseDialog(button) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .close_dialog(button)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetChatInputText(text) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_chat_input_text(&text)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetChatInputEnabled(enabled) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_chat_input_enabled(enabled)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::ProcessChatInput(text) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .process_chat_input(&text)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::RegisterChatCommand {
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
                crate::host_api::chat_commands::finish_registration(subscription, result.is_ok());
                result
            }
            UiCommand::UnregisterChatCommand { subscription, name } => {
                let result = self
                    .connection_profile()
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .unregister_chat_command(&name)
                            .map_err(|_| CommandError::NativeFailure)
                    });
                crate::host_api::chat_commands::finish_unregistration(subscription, result.is_ok());
                result
            }
            UiCommand::SetChatDisplayMode(mode) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_chat_display_mode(mode)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetChatEntry {
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
            UiCommand::AddChatMessage(request) => self
                .connection_profile()
                .filter(|profile| profile.chat_is_ready())
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .show_chat_message(request)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::AddDeathMessage(request) => self
                .connection_profile()
                .filter(|profile| profile.death_window_is_ready())
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .show_death_message(request)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetCursorMode(mode) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_cursor_mode(mode)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::ToggleCursor(show) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .toggle_cursor(show)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetScoreboardOpen(open) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_scoreboard_open(open)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetDialogClientSide(client_side) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_dialog_client_side(client_side)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetDialogSelectedItem(selected) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_dialog_selected_item(selected)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            UiCommand::SetDialogEditboxText(text) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_dialog_editbox_text(&text)
                        .map_err(|_| CommandError::NativeFailure)
                }),
        }
    }
}
