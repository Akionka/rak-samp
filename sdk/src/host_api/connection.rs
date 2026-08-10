use crate::{CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult};

impl HostApi {
    /// Copies and queues the documented R1 reconnect sequence.
    pub fn submit_connect_to_server(
        self,
        address: &[u8],
        port: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_connect_to_server)(address.as_ptr(), address.len(), port, &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues the documented R1 RakClient disconnect and restart sequence.
    pub fn submit_disconnect_with_reason(
        self,
        block_duration: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_disconnect_with_reason)(block_duration, &mut receipt) };
        self.command_receipt(result, receipt)
    }
}
