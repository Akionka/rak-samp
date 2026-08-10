//! Local chat and death display `HostApi` wrappers.

use crate::{
    CommandReceipt, HostApi, LocalChatMessage, LocalDeathMessage, SampClientSdkCommandReceipt,
    SampClientSdkResult, unit_command_result,
};

impl HostApi {
    /// Copies and queues a direct local R1 chat entry on the game thread.
    ///
    /// [`SampClientSdkResult::Ok`] confirms only that the host copied and queued the
    /// entry. It does not send a chat RPC or mean the player has seen it.
    pub fn show_local_chat_message(self, message: LocalChatMessage<'_>) -> SampClientSdkResult {
        if !message.is_valid() {
            return SampClientSdkResult::InvalidArgument;
        }
        unsafe {
            (self.raw.show_local_chat_message)(
                message.style.as_raw(),
                message.text.as_ptr(),
                message.text.len(),
                message.prefix.as_ptr(),
                message.prefix.len(),
                message.text_colour,
                message.prefix_colour,
            )
        }
    }

    /// Submits a direct local R1 chat entry and returns its completion receipt.
    pub fn submit_local_chat_message(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if !message.is_valid() {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut receipt = SampClientSdkCommandReceipt::default();
        match unsafe {
            (self.raw.submit_local_chat_message)(
                message.style.as_raw(),
                message.text.as_ptr(),
                message.text.len(),
                message.prefix.as_ptr(),
                message.prefix.len(),
                message.text_colour,
                message.prefix_colour,
                &mut receipt,
            )
        } {
            SampClientSdkResult::Ok if receipt.id != 0 => {
                Ok(CommandReceipt::new(self, receipt, unit_command_result))
            }
            SampClientSdkResult::Ok => Err(SampClientSdkResult::NativeCallFailed),
            error => Err(error),
        }
    }

    /// Copies and queues a direct local R1 death-window entry on the game thread.
    ///
    /// [`SampClientSdkResult::Ok`] confirms only that the host copied and queued the entry.
    /// It does not send any packet or RPC.
    pub fn show_local_death_message(self, message: LocalDeathMessage<'_>) -> SampClientSdkResult {
        if !message.is_valid() {
            return SampClientSdkResult::InvalidArgument;
        }
        unsafe {
            (self.raw.show_local_death_message)(
                message.killer.as_ptr(),
                message.killer.len(),
                message.victim.as_ptr(),
                message.victim.len(),
                message.killer_colour,
                message.victim_colour,
                message.weapon,
            )
        }
    }

    /// Submits a direct local R1 death-window entry and returns its completion receipt.
    pub fn submit_local_death_message(
        self,
        message: LocalDeathMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if !message.is_valid() {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut receipt = SampClientSdkCommandReceipt::default();
        match unsafe {
            (self.raw.submit_local_death_message)(
                message.killer.as_ptr(),
                message.killer.len(),
                message.victim.as_ptr(),
                message.victim.len(),
                message.killer_colour,
                message.victim_colour,
                message.weapon,
                &mut receipt,
            )
        } {
            SampClientSdkResult::Ok if receipt.id != 0 => {
                Ok(CommandReceipt::new(self, receipt, unit_command_result))
            }
            SampClientSdkResult::Ok => Err(SampClientSdkResult::NativeCallFailed),
            error => Err(error),
        }
    }
}
