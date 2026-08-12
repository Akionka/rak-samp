//! Optional SAMPFUNCS `HostApi` wrappers.

use crate::{HostApi, SampClientSdkResult, limits::MAX_SAMPFUNCS_LOG_BYTES};

impl HostApi {
    /// Returns whether `SAMPFUNCS.asi` is currently loaded in the process.
    #[must_use]
    pub fn sampfuncs_loaded(self) -> bool {
        (self.raw.sampfuncs_loaded)() != 0
    }

    /// Writes a bounded NUL-free byte string through SAMPFUNCS's own console.
    pub fn sampfuncs_log_console(self, text: &[u8]) -> Result<(), SampClientSdkResult> {
        if text.len() > MAX_SAMPFUNCS_LOG_BYTES || text.contains(&0) {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        match unsafe { (self.raw.sampfuncs_log_console)(text.as_ptr(), text.len()) } {
            SampClientSdkResult::Ok => Ok(()),
            result => Err(result),
        }
    }
}
