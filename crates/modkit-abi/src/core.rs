//! Core service v1: cross-service lifetime primitives.

use crate::{CommandReceiptId, ModResult, ServiceHeader, SubscriptionId};

/// An ABI timeout that requests an unbounded wait.
pub const TIMEOUT_INFINITE: u32 = u32::MAX;

/// Maximum UTF-8 log payload accepted by Core v1.
pub const MAX_LOG_MESSAGE_BYTES: u32 = 4096;

pub const LOG_LEVEL_ERROR: u32 = 0;
pub const LOG_LEVEL_WARN: u32 = 1;
pub const LOG_LEVEL_INFO: u32 = 2;
pub const LOG_LEVEL_DEBUG: u32 = 3;

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

    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub host_status: unsafe extern "system" fn(out: *mut HostStatusV1) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; disables future callback starts without blocking.
    pub unregister: unsafe extern "system" fn(id: SubscriptionId) -> ModResult,
    /// `MAY_BLOCK`; callers must not invoke it from `DllMain`.
    pub unregister_and_wait:
        unsafe extern "system" fn(id: SubscriptionId, timeout_ms: u32) -> ModResult,

    /// `ANY_THREAD + CALLBACK_SAFE`; polls without blocking.
    pub receipt_poll:
        unsafe extern "system" fn(id: CommandReceiptId, out: *mut CommandCompletionV1) -> ModResult,
    /// `MAY_BLOCK`; callers must not invoke it from `DllMain`.
    pub receipt_wait: unsafe extern "system" fn(
        id: CommandReceiptId,
        timeout_ms: u32,
        out: *mut CommandCompletionV1,
    ) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; detaches without cancelling the command.
    pub receipt_release: unsafe extern "system" fn(id: CommandReceiptId) -> ModResult,

    /// `ANY_THREAD + CALLBACK_SAFE`; performs bounded copying work without blocking.
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
    fn log_levels_are_stable() {
        assert_eq!(LOG_LEVEL_ERROR, 0);
        assert_eq!(LOG_LEVEL_WARN, 1);
        assert_eq!(LOG_LEVEL_INFO, 2);
        assert_eq!(LOG_LEVEL_DEBUG, 3);
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
    fn core_value_layouts_are_fixed() {
        assert_eq!(core::mem::size_of::<HostStatusV1>(), 16);
        assert_eq!(core::mem::align_of::<HostStatusV1>(), 4);
        assert_eq!(core::mem::size_of::<CommandCompletionV1>(), 24);
        assert_eq!(
            core::mem::align_of::<CommandCompletionV1>(),
            core::mem::align_of::<u64>()
        );
    }

    #[test]
    fn core_service_layout_is_fixed() {
        let pointer_size = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(CoreServiceV1, header), 0);
        assert_eq!(core::mem::offset_of!(CoreServiceV1, host_status), 16);
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, unregister),
            16 + pointer_size
        );
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, unregister_and_wait),
            16 + 2 * pointer_size
        );
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, receipt_poll),
            16 + 3 * pointer_size
        );
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, receipt_wait),
            16 + 4 * pointer_size
        );
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, receipt_release),
            16 + 5 * pointer_size
        );
        assert_eq!(
            core::mem::offset_of!(CoreServiceV1, log_utf8),
            16 + 6 * pointer_size
        );
        assert_eq!(core::mem::size_of::<CoreServiceV1>(), 16 + 7 * pointer_size);
        assert_eq!(
            core::mem::align_of::<CoreServiceV1>(),
            core::mem::align_of::<usize>()
        );
    }
}
