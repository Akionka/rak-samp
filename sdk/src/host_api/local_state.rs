//! Cached local UI and chat-input `HostApi` wrappers.

use crate::{
    HostApi, LocalChatDisplayMode, LocalCursorMode, SampClientSdkChatInputTextV1,
    SampClientSdkResult,
};

impl HostApi {
    /// Returns the cached R1 local chat-window display mode.
    pub fn local_chat_display_mode(self) -> Result<LocalChatDisplayMode, SampClientSdkResult> {
        let mut raw = 0;
        match unsafe { (self.raw.local_chat_display_mode)(&mut raw) } {
            SampClientSdkResult::Ok => {
                LocalChatDisplayMode::from_raw(raw).ok_or(SampClientSdkResult::NativeCallFailed)
            }
            result => Err(result),
        }
    }

    /// Returns whether the cached R1 local chat window is visible.
    pub fn is_local_chat_visible(self) -> Result<bool, SampClientSdkResult> {
        self.local_chat_display_mode()
            .map(|mode| mode != LocalChatDisplayMode::Off)
    }

    /// Returns the cached R1 local cursor mode.
    pub fn local_cursor_mode(self) -> Result<LocalCursorMode, SampClientSdkResult> {
        let mut raw = 0;
        match unsafe { (self.raw.local_cursor_mode)(&mut raw) } {
            SampClientSdkResult::Ok => {
                LocalCursorMode::from_raw(raw).ok_or(SampClientSdkResult::NativeCallFailed)
            }
            result => Err(result),
        }
    }

    /// Returns whether the cached R1 local cursor mode is active.
    pub fn is_local_cursor_active(self) -> Result<bool, SampClientSdkResult> {
        self.local_cursor_mode()
            .map(|mode| mode != LocalCursorMode::None)
    }

    /// Returns whether the cached R1 local scoreboard is open.
    pub fn is_local_scoreboard_open(self) -> Result<bool, SampClientSdkResult> {
        self.cached_boolean(self.raw.local_scoreboard_open)
    }

    /// Returns whether the cached local dialog is active.
    pub fn is_local_dialog_active(self) -> Result<bool, SampClientSdkResult> {
        self.cached_boolean(self.raw.local_dialog_active)
    }

    /// Returns whether the cached local chat input is active.
    pub fn is_local_chat_input_active(self) -> Result<bool, SampClientSdkResult> {
        self.cached_boolean(self.raw.local_chat_input_active)
    }

    /// Returns a copied, game-thread-cached R1 chat-input string.
    pub fn local_chat_input_text(self) -> Result<Vec<u8>, SampClientSdkResult> {
        let mut output = SampClientSdkChatInputTextV1::default();
        match unsafe { (self.raw.local_chat_input_text)(&mut output) } {
            SampClientSdkResult::Ok if usize::from(output.len) <= output.bytes.len() => {
                Ok(output.bytes[..usize::from(output.len)].to_vec())
            }
            SampClientSdkResult::Ok => Err(SampClientSdkResult::NativeCallFailed),
            error => Err(error),
        }
    }

    fn cached_boolean(
        self,
        callback: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    ) -> Result<bool, SampClientSdkResult> {
        let mut raw = 0;
        match unsafe { callback(&mut raw) } {
            SampClientSdkResult::Ok => match raw {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
}
