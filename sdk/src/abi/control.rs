//! C ABI result, control, event, and callback types.

use super::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampClientSdkResult {
    Ok = 0,
    NotReady = 1,
    InvalidArgument = 2,
    UnsupportedVersion = 3,
    SubscriptionNotFound = 4,
    ReadOutOfBounds = 5,
    PayloadTooLarge = 6,
    NativeCallFailed = 7,
    CallbackInProgress = 8,
    /// A bounded host-side request queue could not accept another request.
    QueueFull = 9,
    /// A receipt remains pending after a non-blocking poll.
    CommandPending = 10,
    /// A timed receipt wait expired without consuming the receipt.
    TimedOut = 11,
    /// Waiting would deadlock because the caller is on a host callback or game thread.
    WaitRejected = 12,
    /// The host is shutting down and completed the receipt without native execution.
    ShuttingDown = 13,
    /// A nonblocking direct-client operation could not acquire its cache or request lock.
    /// Retry the operation later.
    Busy = 14,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampClientSdkHostStatus {
    WaitingForSamp = 0,
    Ready = 1,
    Failed = 2,
}

/// A detected SA-MP client build.
///
/// Values are host-defined version identities, not PE entry-point RVAs. The
/// host reports this only after it has recognized the loaded `samp.dll`.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampClientSdkClientVersion {
    R1 = 1,
    R2 = 2,
    R3_1 = 3,
    R4_2 = 4,
    R5_1 = 5,
    Dl = 6,
}

impl SampClientSdkClientVersion {
    pub(crate) const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::R1),
            2 => Some(Self::R2),
            3 => Some(Self::R3_1),
            4 => Some(Self::R4_2),
            5 => Some(Self::R5_1),
            6 => Some(Self::Dl),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampClientSdkDirection {
    Incoming = 0,
    Outgoing = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampClientSdkHookAction {
    Continue = 0,
    Block = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SampClientSdkSubscription {
    pub id: u64,
}

/// Opaque, host-owned identity for one queued game-thread command.
///
/// Receipts are single-consumer: a successful poll or wait consumes the
/// completion. Call `command_release` when abandoning a pending receipt; doing
/// so detaches the caller without cancelling the owned command.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SampClientSdkCommandReceipt {
    pub id: u64,
}

/// Fixed C-compatible completion storage for commands that return `()`.
///
/// `status` is meaningful only when `command_try_take` or `command_wait`
/// returns [`SampClientSdkResult::Ok`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkCommandResultV1 {
    pub status: SampClientSdkResult,
}

impl Default for SampClientSdkCommandResultV1 {
    fn default() -> Self {
        Self {
            status: SampClientSdkResult::Ok,
        }
    }
}

/// Fixed C-compatible completion storage for a game-thread-created R1 3D label.
///
/// `status` is meaningful only when the dedicated text-label completion call
/// returns [`SampClientSdkResult::Ok`]. `id` is meaningful only when `status`
/// is also [`SampClientSdkResult::Ok`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkTextLabelCreateResultV1 {
    pub status: SampClientSdkResult,
    pub id: u16,
    pub reserved: u16,
}

impl Default for SampClientSdkTextLabelCreateResultV1 {
    fn default() -> Self {
        Self {
            status: SampClientSdkResult::Ok,
            id: 0,
            reserved: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkSendOptions {
    pub priority: u32,
    pub reliability: u32,
    pub ordering_channel: u8,
    pub timestamp: bool,
}

/// A RakNet encoded string represented as left-aligned bytes and an exact bit length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SampClientSdkEncodedString {
    pub(crate) bytes: Vec<u8>,
    pub(crate) bit_len: usize,
}

impl SampClientSdkEncodedString {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn len_bits(&self) -> usize {
        self.bit_len
    }
}

impl Default for SampClientSdkSendOptions {
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
pub struct SampClientSdkEventV1 {
    _private: [u8; 0],
}

pub type SampClientSdkEventCallbackV1 = unsafe extern "system" fn(
    user_data: *mut c_void,
    event: *mut SampClientSdkEventV1,
) -> SampClientSdkHookAction;

/// One copied local chat-command argument callback.
///
/// The host owns `args` for the duration of the callback only. Implementations
/// must copy any bytes they need after returning.
pub type SampClientSdkChatCommandCallbackV1 =
    unsafe extern "system" fn(user_data: *mut c_void, args: *const u8, args_len: usize);
