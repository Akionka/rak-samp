//! Optional SAMPFUNCS host-ABI entry points.

use super::helpers::copied_nul_free_string;
use crate::platform::{
    SampfuncsLogError, sampfuncs_loaded as platform_sampfuncs_loaded,
    sampfuncs_log_console as platform_sampfuncs_log_console,
};
use sdk_abi::{SampClientSdkResult, limits::MAX_SAMPFUNCS_LOG_BYTES};

pub(super) extern "system" fn sampfuncs_loaded() -> u8 {
    u8::from(platform_sampfuncs_loaded())
}

/// # Safety
///
/// `text` must be readable for `text_len` bytes when non-null.
pub(super) unsafe extern "system" fn sampfuncs_log_console(
    text: *const u8,
    text_len: usize,
) -> SampClientSdkResult {
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, MAX_SAMPFUNCS_LOG_BYTES) })
    else {
        return SampClientSdkResult::InvalidArgument;
    };
    match platform_sampfuncs_log_console(&text) {
        Ok(()) => SampClientSdkResult::Ok,
        Err(SampfuncsLogError::NotLoaded) => SampClientSdkResult::NotReady,
        Err(SampfuncsLogError::Unsupported) => SampClientSdkResult::UnsupportedVersion,
        Err(SampfuncsLogError::Failed) => SampClientSdkResult::NativeCallFailed,
    }
}
