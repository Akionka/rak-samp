//! Optional safe SAMPFUNCS interop.

use crate::{HostApi, SampClientSdkResult};

/// Access to the SAMPFUNCS ASI when it is already loaded alongside the host.
#[derive(Clone, Copy)]
pub struct Sampfuncs {
    api: HostApi,
}

impl Sampfuncs {
    pub(crate) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    /// Returns whether `SAMPFUNCS.asi` is currently loaded in the process.
    #[must_use]
    pub fn is_loaded(self) -> bool {
        self.api.sampfuncs_loaded()
    }

    /// Writes a bounded NUL-free byte string through SAMPFUNCS's own console.
    ///
    /// The call does not load or initialize SAMPFUNCS. It returns `NotReady`
    /// when SAMPFUNCS is absent and `UnsupportedVersion` when its expected
    /// 5.7.1-compatible console export is unavailable.
    pub fn log_console(self, text: &[u8]) -> Result<(), SampClientSdkResult> {
        self.api.sampfuncs_log_console(text)
    }
}
