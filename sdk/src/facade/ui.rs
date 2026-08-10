use crate::{
    ChatEntry, CommandReceipt, HostApi, LocalChatDisplayMode, LocalChatMessage, LocalCursorMode,
    LocalDeathMessage, LocalDialog, LocalDialogState, MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES,
    SampClientSdkResult,
};

#[derive(Clone, Copy)]
pub struct Dialogs {
    api: HostApi,
}

impl Dialogs {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn active(self) -> Result<Option<LocalDialogState>, SampClientSdkResult> {
        self.api.active_local_dialog()
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_dialog_active()
    }

    /// Returns the copied selected index for an active R1 list dialog.
    pub fn selected_item(self) -> Result<i32, SampClientSdkResult> {
        self.api.local_dialog_selected_item()
    }

    /// Returns the copied count of items in the active R1 dialog list.
    pub fn list_item_count(self) -> Result<i32, SampClientSdkResult> {
        self.api.local_dialog_list_item_count()
    }

    /// Queues selection of an item in the active R1 list dialog.
    pub fn set_selected_item(
        self,
        selected: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog_selected_item(selected)
    }

    pub fn show(self, dialog: LocalDialog<'_>) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog(dialog)
    }

    /// Queues an R1 write that marks the current dialog as client-side or
    /// server-side on the game thread.
    pub fn set_client_side(
        self,
        client_side: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog_client_side(client_side)
    }

    /// Queues closure of the active R1 dialog with its first (`0`) or second
    /// (`1`) response button.
    pub fn close_with_button(self, button: u8) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if button > 1 {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        self.api.submit_local_dialog_close(button)
    }

    /// Queues a bounded R1 dialog editbox text replacement on the game thread.
    pub fn set_editbox_text(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if text.len() > MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES || text.contains(&0) {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        self.api.submit_local_dialog_editbox_text(text)
    }
}

#[derive(Clone, Copy)]
pub struct Chat {
    api: HostApi,
}

impl Chat {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn display_mode(self) -> Result<LocalChatDisplayMode, SampClientSdkResult> {
        self.api.local_chat_display_mode()
    }

    /// Returns one copied fixed R1 chat-history entry.
    pub fn entry(self, id: u16) -> Result<ChatEntry, SampClientSdkResult> {
        self.api.chat_entry(id)
    }

    /// Queues one R1 chat display-mode write.
    pub fn set_display_mode(
        self,
        mode: LocalChatDisplayMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_display_mode(mode)
    }

    /// Queues one bounded R1 chat-history entry replacement.
    pub fn set_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_local_chat_entry(id, text, prefix, text_colour, prefix_colour)
    }

    pub fn is_visible(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_chat_visible()
    }

    #[allow(clippy::should_implement_trait)] // Mirrors the documented `Chat::add` SDK verb.
    pub fn add(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_message(message)
    }

    /// Alias for [`Self::add`] that emphasizes the request's explicit native style.
    #[allow(clippy::should_implement_trait)] // Mirrors the documented `Chat::add_with_style` SDK verb.
    pub fn add_with_style(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.add(message)
    }

    pub fn death_window(self) -> DeathWindow {
        DeathWindow::from_api(self.api)
    }
}

#[derive(Clone, Copy)]
pub struct DeathWindow {
    api: HostApi,
}

/// Safe cached state for SA-MP's local chat-input UI.
#[derive(Clone, Copy)]
pub struct ChatInput {
    api: HostApi,
}

impl ChatInput {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_chat_input_active()
    }

    /// Returns the owned game-thread-cached R1 chat-input text.
    pub fn text(self) -> Result<Vec<u8>, SampClientSdkResult> {
        self.api.local_chat_input_text()
    }

    /// Queues a copied R1 chat-input text update. Text is limited to 128 bytes
    /// and cannot contain an interior NUL.
    pub fn set_text(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_text(text)
    }

    /// Queues R1's native chat-input open or close transition.
    pub fn set_enabled(self, enabled: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_enabled(enabled)
    }

    /// Queues a copied R1 chat-input text update followed by native command
    /// processing. Text is limited to 128 bytes and cannot contain an interior
    /// NUL.
    pub fn process(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_process(text)
    }
}

impl DeathWindow {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    #[allow(clippy::should_implement_trait)] // Mirrors the documented death-window `add` verb.
    pub fn add(
        self,
        message: LocalDeathMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_death_message(message)
    }
}

#[derive(Clone, Copy)]
pub struct Cursor {
    api: HostApi,
}

impl Cursor {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn mode(self) -> Result<LocalCursorMode, SampClientSdkResult> {
        self.api.local_cursor_mode()
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_cursor_active()
    }

    /// Queues one validated R1 cursor-mode change on the game thread.
    pub fn set_mode(
        self,
        mode: LocalCursorMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_cursor_mode(mode)
    }

    /// Queues SF.lua-compatible R1 cursor visibility behavior, including input
    /// re-enabling when hiding the cursor.
    pub fn toggle(self, show: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_cursor_toggle(show)
    }
}

#[derive(Clone, Copy)]
pub struct Scoreboard {
    api: HostApi,
}

impl Scoreboard {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn is_open(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_scoreboard_open()
    }

    /// Queues one R1 scoreboard visibility change on the game thread.
    pub fn toggle(self, open: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_scoreboard_open(open)
    }
}
