//! Core service v1: cross-service lifetime primitives.

use crate::{CommandReceiptId, ModResult, ServiceHeader, SubscriptionId};

/// Host lifecycle status reported by the Core service.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostStatusV1 {
    pub state: u32,
    pub reserved: [u32; 3],
}

impl HostStatusV1 {
    /// The host is still initializing and not yet ready.
    pub const STATE_WAITING: u32 = 0;
    /// The host is ready to serve plugins.
    pub const STATE_READY: u32 = 1;
    /// The host failed to initialize.
    pub const STATE_FAILED: u32 = 2;
    /// The host has begun shutdown.
    pub const STATE_SHUTTING_DOWN: u32 = 3;
}

/// Fixed C-compatible completion storage for a command receipt.
///
/// `value0`/`value1` are only for compact scalar/handle results. Large owned
/// results use fixed service-specific output structs, snapshots, or
/// caller-provided buffers.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandCompletionV1 {
    pub status: ModResult,
    pub reserved: u32,
    pub value0: u64,
    pub value1: u64,
}

impl Default for CommandCompletionV1 {
    fn default() -> Self {
        Self {
            status: crate::MOD_OK,
            reserved: 0,
            value0: 0,
            value1: 0,
        }
    }
}

/// Core service v1 table.
///
/// This layout is frozen once published as stable V1; do not append fields.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CoreServiceV1 {
    pub header: ServiceHeader,

    pub host_status: unsafe extern "system" fn(out: *mut HostStatusV1) -> ModResult,
    pub unregister: unsafe extern "system" fn(id: SubscriptionId) -> ModResult,
    pub unregister_and_wait:
        unsafe extern "system" fn(id: SubscriptionId, timeout_ms: u32) -> ModResult,

    pub receipt_poll:
        unsafe extern "system" fn(id: CommandReceiptId, out: *mut CommandCompletionV1) -> ModResult,
    pub receipt_wait: unsafe extern "system" fn(
        id: CommandReceiptId,
        timeout_ms: u32,
        out: *mut CommandCompletionV1,
    ) -> ModResult,
    pub receipt_release: unsafe extern "system" fn(id: CommandReceiptId) -> ModResult,

    pub log_utf8: unsafe extern "system" fn(level: u32, ptr: *const u8, len: u32) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_status_states_are_stable() {
        assert_eq!(HostStatusV1::STATE_WAITING, 0);
        assert_eq!(HostStatusV1::STATE_READY, 1);
        assert_eq!(HostStatusV1::STATE_FAILED, 2);
        assert_eq!(HostStatusV1::STATE_SHUTTING_DOWN, 3);
    }

    #[test]
    fn command_completion_defaults_to_ok() {
        let completion = CommandCompletionV1::default();
        assert!(completion.status.is_ok());
        assert_eq!(completion.reserved, 0);
        assert_eq!(completion.value0, 0);
        assert_eq!(completion.value1, 0);
    }

    #[test]
    fn core_service_layout_is_fixed() {
        assert_eq!(core::mem::offset_of!(CoreServiceV1, header), 0);
        assert_eq!(core::mem::offset_of!(CoreServiceV1, host_status), 16);
        assert_eq!(core::mem::offset_of!(CoreServiceV1, unregister), 20);
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, unregister_and_wait),
            24
        );
        assert_eq!(core::mem::offset_of!(CoreServiceV1, receipt_poll), 28);
        assert_eq!(core::mem::offset_of!(CoreServiceV1, receipt_wait), 32);
        assert_eq!(core::mem::offset_of!(CoreServiceV1, receipt_release), 36);
        assert_eq!(core::mem::offset_of!(CoreServiceV1, log_utf8), 40);
        assert_eq!(core::mem::size_of::<CoreServiceV1>(), 44);
        assert_eq!(core::mem::align_of::<CoreServiceV1>(), 4);
    }
}
