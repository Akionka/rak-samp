//! Opaque native-address `HostApi` wrappers.

use core::{ffi::c_void, ptr::NonNull};

use crate::{HostApi, SampClientSdkResult};

impl HostApi {
    pub(crate) fn raw_rakclient(self) -> Result<NonNull<c_void>, SampClientSdkResult> {
        self.raw_native_address(self.raw.raw_rakclient)
    }

    pub(crate) fn raw_rakpeer(self) -> Result<NonNull<c_void>, SampClientSdkResult> {
        self.raw_native_address(self.raw.raw_rakpeer)
    }

    pub(crate) fn raw_player_pool(self) -> Result<NonNull<c_void>, SampClientSdkResult> {
        self.raw_native_address(self.raw.raw_player_pool)
    }

    pub(crate) fn raw_vehicle_pool(self) -> Result<NonNull<c_void>, SampClientSdkResult> {
        self.raw_native_address(self.raw.raw_vehicle_pool)
    }

    pub(crate) fn raw_local_player(self) -> Result<NonNull<c_void>, SampClientSdkResult> {
        self.raw_native_address(self.raw.raw_local_player)
    }

    fn raw_native_address(
        self,
        operation: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    ) -> Result<NonNull<c_void>, SampClientSdkResult> {
        let mut output = core::ptr::null_mut();
        match unsafe { operation(&mut output) } {
            SampClientSdkResult::Ok => {
                NonNull::new(output).ok_or(SampClientSdkResult::NativeCallFailed)
            }
            error => Err(error),
        }
    }
}
