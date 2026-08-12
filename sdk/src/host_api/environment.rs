//! Environment and server-state `HostApi` wrappers.

use crate::{
    HostApi, SampClientSdkClientVersion, SampClientSdkResult, SampClientSdkServerInfoV1, ServerInfo,
};

impl HostApi {
    /// Returns a cloned, nonblocking current-server snapshot.
    ///
    /// This returns [`SampClientSdkResult::NotReady`] until the verified R1 game
    /// thread has published a valid address and port. It returns
    /// [`SampClientSdkResult::Busy`] when another thread is publishing the
    /// nonblocking snapshot; callers may retry later.
    pub fn server_info(self) -> Result<ServerInfo, SampClientSdkResult> {
        let mut raw = SampClientSdkServerInfoV1::default();
        match unsafe { (self.raw.server_info)(&mut raw) } {
            SampClientSdkResult::Ok => {}
            result => return Err(result),
        }
        let address_len = usize::from(raw.address_len);
        let hostname_len = usize::from(raw.hostname_len);
        if address_len > raw.address.len() || hostname_len > raw.hostname.len() || raw.port == 0 {
            return Err(SampClientSdkResult::NativeCallFailed);
        }
        Ok(ServerInfo {
            address: raw.address[..address_len].to_vec(),
            hostname: raw.hostname[..hostname_len].to_vec(),
            port: raw.port,
        })
    }

    /// Returns the cached native `CNetGame` state for a verified client profile.
    ///
    /// The value is deliberately an opaque scalar: SA-MP has no stable public
    /// enum ABI for it. Like [`Self::local_player`], this never calls client
    /// code from the plugin thread and returns `NotReady` before publication.
    pub fn samp_game_state(self) -> Result<i32, SampClientSdkResult> {
        let mut state = 0_i32;
        match unsafe { (self.raw.samp_game_state)(&mut state) } {
            SampClientSdkResult::Ok => Ok(state),
            result => Err(result),
        }
    }

    /// Returns the version identity obtained when the host attached to `samp.dll`.
    ///
    /// This is a detection result, not a client-memory read, so it is available
    /// for every recognized client build once the host runtime is ready.
    pub fn samp_version(self) -> Result<SampClientSdkClientVersion, SampClientSdkResult> {
        let mut version = 0_u32;
        match unsafe { (self.raw.samp_version)(&mut version) } {
            SampClientSdkResult::Ok => SampClientSdkClientVersion::from_raw(version)
                .ok_or(SampClientSdkResult::NativeCallFailed),
            result => Err(result),
        }
    }
}
