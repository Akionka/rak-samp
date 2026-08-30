//! Host bootstrap ABI: exact-version service discovery.

use crate::{ModResult, ServiceHeader, ServiceId};

/// The published `ModHostApiV1` ABI version.
pub const MOD_HOST_ABI_VERSION_V1: u32 = 1;

/// The host bootstrap table exported by `GtaModHost_GetApiV1`.
///
/// The primary job of this table is exact-version service discovery. It is
/// host-owned, immutable, and valid until host shutdown/process termination.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModHostApiV1 {
    pub abi_version: u32,
    pub size: u32,
    /// `ANY_THREAD + CALLBACK_SAFE`; performs non-blocking exact lookup.
    pub query_service: unsafe extern "system" fn(
        service: ServiceId,
        requested_version: u32,
        out_service: *mut *const ServiceHeader,
    ) -> ModResult,
}

/// The `GtaModHost_GetApiV1` export signature.
///
/// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
pub type GetModHostApiV1 =
    unsafe extern "system" fn(out_api: *mut *const ModHostApiV1) -> ModResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_abi_version_is_one() {
        assert_eq!(MOD_HOST_ABI_VERSION_V1, 1);
    }

    #[test]
    fn bootstrap_table_layout_is_fixed() {
        assert_eq!(
            core::mem::size_of::<ModHostApiV1>(),
            8 + core::mem::size_of::<usize>()
        );
        assert_eq!(
            core::mem::align_of::<ModHostApiV1>(),
            core::mem::align_of::<usize>()
        );
        assert_eq!(core::mem::offset_of!(ModHostApiV1, abi_version), 0);
        assert_eq!(core::mem::offset_of!(ModHostApiV1, size), 4);
        assert_eq!(core::mem::offset_of!(ModHostApiV1, query_service), 8);
    }
}
