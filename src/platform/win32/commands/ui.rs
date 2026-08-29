use super::*;

impl BackendState {
    pub(in crate::platform::win32) fn queue_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::ShowDialog(request))
    }

    pub(in crate::platform::win32) fn queue_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::AddChatMessage(request))
    }

    pub(in crate::platform::win32) fn queue_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::AddDeathMessage(request))
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
        self.queue_game_command(GameCommand::SetCursorMode(mode))
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
        self.queue_game_command(GameCommand::SetChatDisplayMode(mode))
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
        self.queue_game_command(GameCommand::SetChatEntry {
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
        self.queue_game_command(GameCommand::CloseDialog(button))
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
        self.queue_game_command(GameCommand::SetChatInputText(text))
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
        self.queue_game_command(GameCommand::SetChatInputEnabled(enabled))
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
        self.queue_game_command(GameCommand::ProcessChatInput(text))
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
        self.queue_game_command(GameCommand::RegisterChatCommand {
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
        self.queue_game_command(GameCommand::UnregisterChatCommand { subscription, name })
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
        self.queue_game_command(GameCommand::ToggleCursor(show))
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
        self.queue_game_command(GameCommand::SetScoreboardOpen(open))
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
        self.queue_game_command(GameCommand::SetDialogClientSide(client_side))
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
        self.queue_game_command(GameCommand::SetDialogSelectedItem(selected))
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
        self.queue_game_command(GameCommand::SetDialogEditboxText(text))
    }
}
