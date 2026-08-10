use crate::{
    CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult, SpecialAction,
};

impl HostApi {
    /// Queues the R1 local-player spawn path and returns its completion receipt.
    pub fn submit_local_player_spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_local_player_spawn)(&mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one established R1 local-player special action.
    pub fn submit_local_player_special_action(
        self,
        action: SpecialAction,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_local_player_special_action)(action.raw(), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues a documented R1 local- or remote-player colour change.
    pub fn submit_player_colour(
        self,
        id: u16,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_player_colour)(id, colour, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a documented R1 local-player nickname update.
    pub fn submit_local_player_name(
        self,
        name: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_local_player_name)(name.as_ptr(), name.len(), &mut receipt) };
        self.command_receipt(result, receipt)
    }
}
