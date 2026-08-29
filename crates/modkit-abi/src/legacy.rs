//! Migration-only legacy SA-MP service wrapper.
//!
//! This service exists only so the new host/service discovery path can be
//! introduced without rewriting the whole current SDK in the same commit. It
//! wraps a pointer to the existing `SampClientSdkApiV1` table. Do not add new
//! features to it; replace or remove it before `1.0.0`.

use crate::ServiceHeader;
use core::ffi::c_void;

/// Legacy SA-MP service v1 table.
///
/// `api` points to the existing legacy `SampClientSdkApiV1` table. It is
/// host-owned and valid until host shutdown.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LegacySampServiceV1 {
    pub header: ServiceHeader,
    pub api: *const c_void,
}

// The `api` pointer references a host-owned immutable static table and is never
// dereferenced by the host. Sharing the table across threads is sound.
unsafe impl Send for LegacySampServiceV1 {}
unsafe impl Sync for LegacySampServiceV1 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_service_layout_is_fixed() {
        assert_eq!(core::mem::offset_of!(LegacySampServiceV1, header), 0);
        assert_eq!(core::mem::offset_of!(LegacySampServiceV1, api), 16);
        assert_eq!(core::mem::size_of::<LegacySampServiceV1>(), 20);
        assert_eq!(core::mem::align_of::<LegacySampServiceV1>(), 4);
    }
}
