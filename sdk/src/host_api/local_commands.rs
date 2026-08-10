//! Local UI command `HostApi` wrappers.

use crate::{
    CommandReceipt, HostApi, LocalChatDisplayMode, LocalCursorMode, SampClientSdkCommandReceipt,
    SampClientSdkResult,
};

impl HostApi {
    /// Queues one validated R1 cursor-mode write and returns its completion receipt.
    pub fn submit_local_cursor_mode(
        self,
        mode: LocalCursorMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_local_cursor_mode)(mode.as_raw(), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one R1 scoreboard-enabled write and returns its completion receipt.
    pub fn submit_local_scoreboard_open(
        self,
        open: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_local_scoreboard_open)(u8::from(open), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues the R1 cursor toggle transition.
    pub fn submit_local_cursor_toggle(
        self,
        show: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_local_cursor_toggle)(u8::from(show), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one R1 chat display-mode write.
    pub fn submit_local_chat_display_mode(
        self,
        mode: LocalChatDisplayMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_local_chat_display_mode)(mode.raw(), &mut receipt) };
        self.command_receipt(result, receipt)
    }
}
