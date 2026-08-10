//! Low-level packet/RPC send and incoming-emulation `HostApi` wrappers.

use crate::raknet::BitStream;
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

    /// Sends a complete owned plugin-side bit stream as a packet payload.
    pub fn send_packet_stream(
        self,
        packet_id: u8,
        payload: &BitStream,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        self.send_packet(packet_id, payload.as_bytes(), payload.len_bits(), options)
    }

    /// Sends a complete owned plugin-side bit stream as an RPC payload.
    pub fn send_rpc_stream(
        self,
        rpc_id: u8,
        payload: &BitStream,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        self.send_rpc(rpc_id, payload.as_bytes(), payload.len_bits(), options)
    }

    /// Queues an incoming packet for SA-MP after incoming plugin listeners run.
    ///
    /// `payload` excludes the packet ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`SampClientSdkResult::Ok`].
    pub fn emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> SampClientSdkResult {
        unsafe {
            (self.raw.emulate_incoming_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len)
        }
    }

    /// Dispatches an incoming RPC to plugin listeners and then SA-MP unless blocked.
    ///
    /// `payload` excludes the RPC ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`SampClientSdkResult::Ok`].
    pub fn emulate_incoming_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> SampClientSdkResult {
        unsafe { (self.raw.emulate_incoming_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len) }
    }

    /// Copies and queues a locally generated incoming packet, returning its completion receipt.
    pub fn submit_emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_emulate_incoming_packet)(
                packet_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a locally generated incoming RPC, returning its completion receipt.
    pub fn submit_emulate_incoming_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_emulate_incoming_rpc)(
                rpc_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
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
