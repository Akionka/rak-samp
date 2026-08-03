//! Stable C ABI definitions and host-discovery helpers for `rak-samp` plugins.
//!
//! Depend on this crate from an independently loaded ASI plugin. Do **not**
//! depend on the `rak_samp` host crate: that would embed a second hook engine in
//! the process instead of communicating with `rak_samp.asi`.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_plugin_api supports only 32-bit Windows x86 targets");

pub mod events;

use core::{ffi::c_void, fmt, mem, ptr::NonNull};
use std::time::{Duration, Instant};

pub const ABI_VERSION_V1: u32 = 1;
pub const DEFAULT_HOST_MODULE: &[u8] = b"rak_samp.asi\0";

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RakSampResult {
    Ok = 0,
    NotReady = 1,
    InvalidArgument = 2,
    UnsupportedVersion = 3,
    SubscriptionNotFound = 4,
    ReadOutOfBounds = 5,
    PayloadTooLarge = 6,
    NativeCallFailed = 7,
    CallbackInProgress = 8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RakSampHostStatus {
    WaitingForSamp = 0,
    Ready = 1,
    Failed = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RakSampDirection {
    Incoming = 0,
    Outgoing = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RakSampHookAction {
    Continue = 0,
    Block = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RakSampSubscription {
    pub id: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RakSampSendOptions {
    pub priority: u32,
    pub reliability: u32,
    pub ordering_channel: u8,
    pub timestamp: bool,
}

/// A RakNet encoded string represented as left-aligned bytes and an exact bit length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RakSampEncodedString {
    bytes: Vec<u8>,
    bit_len: usize,
}

impl RakSampEncodedString {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn len_bits(&self) -> usize {
        self.bit_len
    }
}

impl Default for RakSampSendOptions {
    fn default() -> Self {
        Self {
            priority: 1,
            reliability: 9,
            ordering_channel: 0,
            timestamp: false,
        }
    }
}

/// An opaque event that is valid only for the duration of a plugin callback.
#[repr(C)]
pub struct RakSampEventV1 {
    _private: [u8; 0],
}

pub type RakSampEventCallbackV1 = unsafe extern "system" fn(
    user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction;

/// The process-wide API exported by `rak_samp.asi`.
///
/// Fields are append-only. Check `size` before accessing fields added by a
/// newer ABI version.
#[repr(C)]
pub struct RakSampApiV1 {
    pub abi_version: u32,
    pub size: u32,
    pub host_status: extern "system" fn() -> RakSampHostStatus,
    pub register_packet: unsafe extern "system" fn(
        RakSampDirection,
        Option<RakSampEventCallbackV1>,
        *mut c_void,
        *mut RakSampSubscription,
    ) -> RakSampResult,
    pub register_rpc: unsafe extern "system" fn(
        RakSampDirection,
        Option<RakSampEventCallbackV1>,
        *mut c_void,
        *mut RakSampSubscription,
    ) -> RakSampResult,
    pub unregister: unsafe extern "system" fn(RakSampSubscription) -> RakSampResult,
    pub event_id: unsafe extern "system" fn(*const RakSampEventV1) -> u8,
    pub event_reset_read: unsafe extern "system" fn(*mut RakSampEventV1) -> RakSampResult,
    pub event_clear: unsafe extern "system" fn(*mut RakSampEventV1) -> RakSampResult,
    pub event_read_u8: unsafe extern "system" fn(*mut RakSampEventV1, *mut u8) -> RakSampResult,
    pub event_read_u16: unsafe extern "system" fn(*mut RakSampEventV1, *mut u16) -> RakSampResult,
    pub event_read_u32: unsafe extern "system" fn(*mut RakSampEventV1, *mut u32) -> RakSampResult,
    pub event_read_f32: unsafe extern "system" fn(*mut RakSampEventV1, *mut f32) -> RakSampResult,
    pub event_read_bytes:
        unsafe extern "system" fn(*mut RakSampEventV1, *mut u8, usize) -> RakSampResult,
    pub event_write_u8: unsafe extern "system" fn(*mut RakSampEventV1, u8) -> RakSampResult,
    pub event_write_u16: unsafe extern "system" fn(*mut RakSampEventV1, u16) -> RakSampResult,
    pub event_write_u32: unsafe extern "system" fn(*mut RakSampEventV1, u32) -> RakSampResult,
    pub event_write_f32: unsafe extern "system" fn(*mut RakSampEventV1, f32) -> RakSampResult,
    pub event_write_bytes:
        unsafe extern "system" fn(*mut RakSampEventV1, *const u8, usize) -> RakSampResult,
    pub send_packet:
        unsafe extern "system" fn(u8, *const u8, usize, usize, RakSampSendOptions) -> RakSampResult,
    pub send_rpc:
        unsafe extern "system" fn(u8, *const u8, usize, usize, RakSampSendOptions) -> RakSampResult,
    /// Atomically replaces a byte-aligned callback payload. This field was appended to ABI v1.
    pub event_replace_bytes:
        unsafe extern "system" fn(*mut RakSampEventV1, *const u8, usize) -> RakSampResult,
    /// Removes a listener and waits for callbacks already running on other threads.
    pub unregister_and_wait: unsafe extern "system" fn(RakSampSubscription) -> RakSampResult,
    /// Queues a locally generated incoming packet. `data` excludes the packet ID.
    pub emulate_incoming_packet:
        unsafe extern "system" fn(u8, *const u8, usize, usize) -> RakSampResult,
    /// Dispatches a locally generated incoming RPC. `data` excludes the RPC ID.
    pub emulate_incoming_rpc:
        unsafe extern "system" fn(u8, *const u8, usize, usize) -> RakSampResult,
    /// Returns unread bits in a callback-local event. This field was appended to ABI v1.
    pub event_remaining_bits: unsafe extern "system" fn(*mut RakSampEventV1) -> usize,
    /// Reads exact bits into a left-aligned byte buffer. This field was appended to ABI v1.
    pub event_read_bits:
        unsafe extern "system" fn(*mut RakSampEventV1, *mut u8, usize) -> RakSampResult,
    /// Atomically replaces a callback payload with an exact bit length.
    pub event_replace_bits:
        unsafe extern "system" fn(*mut RakSampEventV1, *const u8, usize, usize) -> RakSampResult,
    /// Encodes one string with SA-MP's native RakNet compressor.
    pub encode_string:
        unsafe extern "system" fn(*const u8, usize, *mut u8, usize, *mut usize) -> RakSampResult,
    /// Decodes one string from a callback event and advances its read cursor.
    pub event_read_encoded_string:
        unsafe extern "system" fn(*mut RakSampEventV1, *mut u8, usize, *mut usize) -> RakSampResult,
}

pub type RakSampGetApiV1 = unsafe extern "system" fn(u32) -> *const RakSampApiV1;

/// A validated reference to the host API table.
#[derive(Clone, Copy)]
pub struct HostApi {
    raw: &'static RakSampApiV1,
}

impl HostApi {
    /// # Safety
    ///
    /// `raw` must point to a live API table exported by a compatible host.
    pub unsafe fn from_raw(raw: *const RakSampApiV1) -> Result<Self, ResolveError> {
        let raw = NonNull::new(raw.cast_mut()).ok_or(ResolveError::MissingApi)?;
        let raw = unsafe { raw.as_ref() };
        if raw.abi_version != ABI_VERSION_V1 || raw.size < mem::size_of::<RakSampApiV1>() as u32 {
            return Err(ResolveError::UnsupportedAbi);
        }
        Ok(Self { raw })
    }

    #[must_use]
    pub fn raw(self) -> &'static RakSampApiV1 {
        self.raw
    }

    #[must_use]
    pub fn status(self) -> RakSampHostStatus {
        (self.raw.host_status)()
    }

    /// Removes a subscription and waits until no callback can still execute it.
    ///
    /// Call this from the plugin's shutdown worker before unloading its ASI. Calling it from a
    /// rak-samp callback returns [`RakSampResult::CallbackInProgress`] instead of deadlocking.
    pub fn unregister_and_wait(self, subscription: RakSampSubscription) -> RakSampResult {
        unsafe { (self.raw.unregister_and_wait)(subscription) }
    }

    /// Sends a packet through SA-MP's original RakClient method.
    ///
    /// `payload` excludes the packet ID. Outgoing listeners are bypassed to prevent recursive
    /// dispatch. Timestamped packet options are currently rejected as invalid.
    pub fn send_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: RakSampSendOptions,
    ) -> RakSampResult {
        unsafe {
            (self.raw.send_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len, options)
        }
    }

    /// Sends an RPC through SA-MP's original RakClient method.
    ///
    /// `payload` excludes the RPC ID. Outgoing listeners are bypassed to prevent recursive
    /// dispatch.
    pub fn send_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: RakSampSendOptions,
    ) -> RakSampResult {
        unsafe { (self.raw.send_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len, options) }
    }

    /// Queues an incoming packet for SA-MP after incoming plugin listeners run.
    ///
    /// `payload` excludes the packet ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`RakSampResult::Ok`].
    pub fn emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> RakSampResult {
        unsafe {
            (self.raw.emulate_incoming_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len)
        }
    }

    /// Dispatches an incoming RPC to plugin listeners and then SA-MP unless blocked.
    ///
    /// `payload` excludes the RPC ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`RakSampResult::Ok`].
    pub fn emulate_incoming_rpc(self, rpc_id: u8, payload: &[u8], bit_len: usize) -> RakSampResult {
        unsafe { (self.raw.emulate_incoming_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len) }
    }

    /// Encodes a NUL-free byte string with the current SA-MP client's RakNet compressor.
    pub fn encode_string(self, value: &[u8]) -> Result<RakSampEncodedString, RakSampResult> {
        let capacity_bits = value
            .len()
            .checked_mul(16)
            .and_then(|bits| bits.checked_add(16))
            .ok_or(RakSampResult::PayloadTooLarge)?;
        let mut bytes = vec![0_u8; capacity_bits.div_ceil(u8::BITS as usize)];
        let mut bit_len = 0;
        let result = unsafe {
            (self.raw.encode_string)(
                value.as_ptr(),
                value.len(),
                bytes.as_mut_ptr(),
                bytes.len(),
                &raw mut bit_len,
            )
        };
        if result != RakSampResult::Ok {
            return Err(result);
        }
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(RakSampResult::NativeCallFailed);
        }
        bytes.truncate(bit_len.div_ceil(u8::BITS as usize));
        Ok(RakSampEncodedString { bytes, bit_len })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    UnsupportedPlatform,
    HostNotLoaded,
    MissingApi,
    UnsupportedAbi,
    HostFailed,
    TimedOut,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => formatter.write_str("rak-samp plugins require Windows"),
            Self::HostNotLoaded => formatter.write_str("rak-samp host module is not loaded"),
            Self::MissingApi => {
                formatter.write_str("rak-samp host does not export RakSamp_GetApiV1")
            }
            Self::UnsupportedAbi => formatter.write_str("rak-samp host ABI v1 is unavailable"),
            Self::HostFailed => formatter.write_str("rak-samp host failed to initialize"),
            Self::TimedOut => formatter.write_str("timed out waiting for rak-samp host readiness"),
        }
    }
}

impl std::error::Error for ResolveError {}

/// Waits for the default `rak_samp.asi` host to expose a ready v1 API.
///
/// Call this from a plugin worker thread, never from `DllMain`.
pub fn wait_for_default_host(timeout: Duration) -> Result<HostApi, ResolveError> {
    wait_for_host(DEFAULT_HOST_MODULE, timeout)
}

/// Waits for a named host module to expose a ready v1 API.
///
/// `module_name` must be NUL-terminated, for example `b"rak_samp.asi\\0"`.
pub fn wait_for_host(module_name: &[u8], timeout: Duration) -> Result<HostApi, ResolveError> {
    if module_name.last() != Some(&0) {
        return Err(ResolveError::HostNotLoaded);
    }
    let started = Instant::now();
    loop {
        match resolve_host(module_name) {
            Ok(api) => match api.status() {
                RakSampHostStatus::Ready => return Ok(api),
                RakSampHostStatus::Failed => return Err(ResolveError::HostFailed),
                RakSampHostStatus::WaitingForSamp => {}
            },
            Err(ResolveError::HostNotLoaded | ResolveError::MissingApi) => {}
            Err(error) => return Err(error),
        }
        if started.elapsed() >= timeout {
            return Err(ResolveError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(all(windows, target_arch = "x86"))]
fn resolve_host(module_name: &[u8]) -> Result<HostApi, ResolveError> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };
    if module.is_null() {
        return Err(ResolveError::HostNotLoaded);
    }
    let symbol = unsafe { GetProcAddress(module, c"RakSamp_GetApiV1".as_ptr().cast()) };
    let Some(symbol) = symbol else {
        return Err(ResolveError::MissingApi);
    };
    let get_api: RakSampGetApiV1 = unsafe { mem::transmute(symbol) };
    let raw = unsafe { get_api(ABI_VERSION_V1) };
    unsafe { HostApi::from_raw(raw) }
}

#[cfg(not(all(windows, target_arch = "x86")))]
fn resolve_host(_module_name: &[u8]) -> Result<HostApi, ResolveError> {
    Err(ResolveError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_match_raknet_defaults() {
        assert_eq!(RakSampSendOptions::default().priority, 1);
        assert_eq!(RakSampSendOptions::default().reliability, 9);
    }

    #[test]
    fn default_host_module_matches_the_deploy_artifact() {
        assert_eq!(DEFAULT_HOST_MODULE, b"rak_samp.asi\0");
    }

    #[test]
    fn newer_functions_are_appended_to_abi_v1() {
        let function_size = mem::size_of::<*const c_void>();
        assert_eq!(
            mem::offset_of!(RakSampApiV1, emulate_incoming_packet),
            mem::offset_of!(RakSampApiV1, unregister_and_wait) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, emulate_incoming_rpc),
            mem::offset_of!(RakSampApiV1, emulate_incoming_packet) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, event_remaining_bits),
            mem::offset_of!(RakSampApiV1, emulate_incoming_rpc) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, event_read_bits),
            mem::offset_of!(RakSampApiV1, event_remaining_bits) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, event_replace_bits),
            mem::offset_of!(RakSampApiV1, event_read_bits) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, encode_string),
            mem::offset_of!(RakSampApiV1, event_replace_bits) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, event_read_encoded_string),
            mem::offset_of!(RakSampApiV1, encode_string) + function_size
        );
        assert_eq!(
            mem::size_of::<RakSampApiV1>(),
            mem::offset_of!(RakSampApiV1, event_read_encoded_string) + function_size
        );
    }
}
