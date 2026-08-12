use crate::{
    CommandReceipt, HostApi, LocalDialog, LocalDialogResponse, LocalDialogState,
    SampClientSdkCommandReceipt, SampClientSdkDialogResponseV1, SampClientSdkDialogSnapshotV1,
    SampClientSdkResult, local_dialog_response_from_abi, local_dialog_state_from_abi,
    unit_command_result,
};

impl HostApi {
    /// Queues one R1 dialog close with the selected response button.
    pub fn submit_local_dialog_close(
        self,
        button: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_local_dialog_close)(button, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a direct local dialog on the verified R1 game thread.
    ///
    /// [`SampClientSdkResult::Ok`] confirms only that the host copied and queued the
    /// request; it does not mean the player has seen or dismissed the dialog.
    pub fn show_local_dialog(self, dialog: LocalDialog<'_>) -> SampClientSdkResult {
        if !dialog.is_valid() {
            return SampClientSdkResult::InvalidArgument;
        }
        unsafe {
            (self.raw.show_local_dialog)(
                dialog.id,
                dialog.style.as_raw(),
                dialog.title.as_ptr(),
                dialog.title.len(),
                dialog.text.as_ptr(),
                dialog.text.len(),
                dialog.button1.as_ptr(),
                dialog.button1.len(),
                dialog.button2.as_ptr(),
                dialog.button2.len(),
            )
        }
    }

    /// Submits a direct local R1 dialog and returns its game-thread completion.
    ///
    /// The returned receipt does not indicate whether the player interacted
    /// with the dialog; it reports only whether the host-native call ran.
    pub fn submit_local_dialog(
        self,
        dialog: LocalDialog<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if !dialog.is_valid() {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut receipt = SampClientSdkCommandReceipt::default();
        match unsafe {
            (self.raw.submit_local_dialog)(
                dialog.id,
                dialog.style.as_raw(),
                dialog.title.as_ptr(),
                dialog.title.len(),
                dialog.text.as_ptr(),
                dialog.text.len(),
                dialog.button1.as_ptr(),
                dialog.button1.len(),
                dialog.button2.as_ptr(),
                dialog.button2.len(),
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

    /// Queues one R1 dialog client-side write and returns its completion receipt.
    pub fn submit_local_dialog_client_side(
        self,
        client_side: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_local_dialog_client_side)(u8::from(client_side), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Returns the copied active R1 dialog list selection.
    pub fn local_dialog_selected_item(self) -> Result<i32, SampClientSdkResult> {
        let mut output = 0;
        match unsafe { (self.raw.local_dialog_selected_item)(&mut output) } {
            SampClientSdkResult::Ok => Ok(output),
            error => Err(error),
        }
    }

    /// Queues an R1 dialog list-selection write.
    pub fn submit_local_dialog_selected_item(
        self,
        selected: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_local_dialog_selected_item)(selected, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues a bounded R1 dialog editbox text write.
    pub fn submit_local_dialog_editbox_text(
        self,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_local_dialog_editbox_text)(text.as_ptr(), text.len(), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Returns the copied count of items in the active R1 dialog list.
    pub fn local_dialog_list_item_count(self) -> Result<i32, SampClientSdkResult> {
        let mut output = 0;
        match unsafe { (self.raw.local_dialog_list_item_count)(&mut output) } {
            SampClientSdkResult::Ok => Ok(output),
            error => Err(error),
        }
    }

    /// Returns a cloned, nonblocking snapshot of the active local R1 dialog.
    ///
    /// This returns `Ok(None)` once the game-thread cache confirms that no
    /// dialog is active, and `NotReady` before the first cache publication.
    pub fn active_local_dialog(self) -> Result<Option<LocalDialogState>, SampClientSdkResult> {
        let mut raw = SampClientSdkDialogSnapshotV1::default();
        match unsafe { (self.raw.local_dialog_snapshot)(&mut raw) } {
            SampClientSdkResult::Ok => {}
            result => return Err(result),
        }
        local_dialog_state_from_abi(raw)
    }

    /// Takes the newest captured R1 client-side dialog response, if one is pending.
    pub fn take_local_dialog_response(
        self,
    ) -> Result<Option<LocalDialogResponse>, SampClientSdkResult> {
        let mut raw = SampClientSdkDialogResponseV1::default();
        match unsafe { (self.raw.take_local_dialog_response)(&mut raw) } {
            SampClientSdkResult::Ok => local_dialog_response_from_abi(raw),
            result => Err(result),
        }
    }
}
