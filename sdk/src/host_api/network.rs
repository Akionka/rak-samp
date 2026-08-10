//! Low-level packet/RPC send `HostApi` wrappers.

use crate::{
    CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult,
    SampClientSdkSendOptions,
};

impl HostApi {
    /// Sends a packet through SA-MP's original RakClient method.
    ///
    /// `payload` excludes the packet ID. Outgoing listeners are bypassed to prevent recursive
    /// dispatch. Timestamped packet options are currently rejected as invalid.
    pub fn send_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        unsafe {
            (self.raw.send_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len, options)
        }
    }

    /// Sends an RPC through SA-MP's original RakClient method.
    ///
    /// `payload` excludes the RPC ID. Outgoing listeners are bypassed to prevent recursive
    /// dispatch.
    pub fn send_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        unsafe { (self.raw.send_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len, options) }
    }

    /// Copies and queues a server-bound packet, returning its game-thread completion receipt.
    pub fn submit_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_packet)(
                packet_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                options,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a server-bound RPC, returning its game-thread completion receipt.
    pub fn submit_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_rpc)(
                rpc_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                options,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }
}
