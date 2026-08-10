//! Local chat-input command `HostApi` wrappers.

use crate::{CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult};

impl HostApi {
    /// Copies and queues a R1 chat-input text update.
    pub fn submit_local_chat_input_text(
        self,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_local_chat_input_text)(text.as_ptr(), text.len(), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues the native R1 chat-input open or close transition.
    pub fn submit_local_chat_input_enabled(
        self,
        enabled: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_local_chat_input_enabled)(u8::from(enabled), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Copies text and queues R1 chat-input command processing.
    pub fn submit_local_chat_input_process(
        self,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_local_chat_input_process)(text.as_ptr(), text.len(), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }
}
