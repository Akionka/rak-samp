//! Host lifecycle-status `HostApi` wrappers.

use crate::{HostApi, SampClientSdkHostStatus};

impl HostApi {
    #[must_use]
    pub fn status(self) -> SampClientSdkHostStatus {
        (self.raw.host_status)()
    }

    /// Returns whether the host attached to a recognized SA-MP client and its
    /// RakClient hooks are ready.
    ///
    /// This is the safe host-level equivalent of SF.lua's
    /// `isSampAvailable`; it does not dereference `CNetGame` on the plugin
    /// thread.
    pub fn is_samp_available(self) -> bool {
        self.status() == SampClientSdkHostStatus::Ready
    }

    /// Returns whether the host has attached to and recognized `samp.dll`.
    ///
    /// This is the safe equivalent of SF.lua's `isSampLoaded`. Unlike
    /// [`Self::is_samp_available`], it can be true while the host is still
    /// installing its RakClient hooks. It never returns a module base or reads
    /// client memory from the plugin thread.
    #[must_use]
    pub fn is_samp_loaded(self) -> bool {
        self.samp_version().is_ok()
    }
}
