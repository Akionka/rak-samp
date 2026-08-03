//! Stable C ABI definitions and safe host-discovery helpers for `rak-samp` plugins.
//!
//! Depend on this crate from an independently loaded ASI plugin. Do **not**
//! depend on the `rak_samp` host crate: that would embed a second hook engine in
//! the process instead of communicating with `rak_samp.asi`. Register callbacks with
//! [`HostApi::on_packet`] or [`HostApi::on_rpc`]. Use their ID-filtered and typed variants when
//! one handler owns one protocol message, and [`register_handlers!`] to keep a group in one
//! [`SubscriptionSet`]. Synchronize subscriptions before unloading the plugin.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_plugin_api supports only 32-bit Windows x86 targets");

pub mod events;
pub mod raknet;

use core::{ffi::c_void, fmt, mem, ptr::NonNull};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    time::{Duration, Instant},
};

pub const ABI_VERSION_V1: u32 = 1;
pub const DEFAULT_HOST_MODULE: &[u8] = b"rak_samp.asi\0";
/// Maximum decoded byte length accepted by [`HostApi::decode_string`].
///
/// The extra byte used by the host's native decoder is reserved for its NUL
/// terminator and is not included in this limit.
pub const MAX_RAKNET_DECODED_STRING_BYTES: usize = 4_095;

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
    /// A bounded host-side request queue could not accept another request.
    QueueFull = 9,
}

/// The six dialog styles understood by SA-MP's local dialog implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDialogStyle {
    MessageBox,
    Input,
    List,
    Password,
    TabList,
    HeadersList,
}

impl LocalDialogStyle {
    const fn as_raw(self) -> u32 {
        match self {
            Self::MessageBox => 0,
            Self::Input => 1,
            Self::List => 2,
            Self::Password => 3,
            Self::TabList => 4,
            Self::HeadersList => 5,
        }
    }
}

/// A copied-and-queued local dialog request.
///
/// The host copies all borrowed strings before this call returns. Strings must
/// not contain NUL bytes; title and buttons are limited to 255 bytes and text
/// is limited to 4,095 bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDialog<'a> {
    pub id: u16,
    pub style: LocalDialogStyle,
    pub title: &'a [u8],
    pub text: &'a [u8],
    pub button1: &'a [u8],
    pub button2: &'a [u8],
}

impl LocalDialog<'_> {
    const MAX_TITLE_OR_BUTTON_BYTES: usize = 255;
    const MAX_TEXT_BYTES: usize = 4_095;

    fn is_valid(self) -> bool {
        [self.title, self.text, self.button1, self.button2]
            .into_iter()
            .all(|value| !value.contains(&0))
            && self.title.len() <= Self::MAX_TITLE_OR_BUTTON_BYTES
            && self.button1.len() <= Self::MAX_TITLE_OR_BUTTON_BYTES
            && self.button2.len() <= Self::MAX_TITLE_OR_BUTTON_BYTES
            && self.text.len() <= Self::MAX_TEXT_BYTES
    }
}

/// The three local chat entry styles accepted by SA-MP's R1 chat window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalChatMessageStyle {
    Chat,
    Info,
    Debug,
}

impl LocalChatMessageStyle {
    const fn as_raw(self) -> u32 {
        match self {
            Self::Chat => 2,
            Self::Info => 4,
            Self::Debug => 8,
        }
    }
}

/// The three R1 chat-window display modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalChatDisplayMode {
    Off,
    NoShadow,
    Normal,
}

impl LocalChatDisplayMode {
    const fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::NoShadow),
            2 => Some(Self::Normal),
            _ => None,
        }
    }
}

/// The five R1 local-cursor modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalCursorMode {
    None,
    LockKeysNoCursor,
    LockCameraAndControl,
    LockCamera,
    LockCameraNoCursor,
}

impl LocalCursorMode {
    const fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::LockKeysNoCursor),
            2 => Some(Self::LockCameraAndControl),
            3 => Some(Self::LockCamera),
            4 => Some(Self::LockCameraNoCursor),
            _ => None,
        }
    }
}

/// A copied-and-queued local chat message.
///
/// The host copies both borrowed byte strings before this call returns. They
/// must not contain NUL bytes. R1 chat entries retain at most 143 text bytes
/// and 27 prefix bytes, excluding their native terminators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalChatMessage<'a> {
    pub style: LocalChatMessageStyle,
    pub text: &'a [u8],
    pub prefix: &'a [u8],
    pub text_colour: u32,
    pub prefix_colour: u32,
}

impl LocalChatMessage<'_> {
    const MAX_TEXT_BYTES: usize = 143;
    const MAX_PREFIX_BYTES: usize = 27;

    fn is_valid(self) -> bool {
        !self.text.contains(&0)
            && !self.prefix.contains(&0)
            && self.text.len() <= Self::MAX_TEXT_BYTES
            && self.prefix.len() <= Self::MAX_PREFIX_BYTES
    }
}

/// A copied-and-queued local death-window entry.
///
/// The host copies both names before this call returns. They must not contain
/// NUL bytes and are limited to 24 bytes each by R1's native entry buffers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDeathMessage<'a> {
    pub killer: &'a [u8],
    pub victim: &'a [u8],
    pub killer_colour: u32,
    pub victim_colour: u32,
    pub weapon: u8,
}

impl LocalDeathMessage<'_> {
    const MAX_NAME_BYTES: usize = 24;

    fn is_valid(self) -> bool {
        !self.killer.contains(&0)
            && !self.victim.contains(&0)
            && self.killer.len() <= Self::MAX_NAME_BYTES
            && self.victim.len() <= Self::MAX_NAME_BYTES
    }
}

/// A three-dimensional value copied from the client snapshot.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// An owned, read-only local-player snapshot.
///
/// The host refreshes this on its verified game-thread packet pump. It is a
/// cache, so fetching it never waits for the game thread or exposes client
/// pointers.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalPlayer {
    pub id: u16,
    pub nickname: Vec<u8>,
    pub colour: u32,
    pub spawned: bool,
    pub health: f32,
    pub armour: f32,
    pub position: Vector3,
    pub velocity: Vector3,
    pub special_action: u8,
    pub animation_id: u16,
    pub vehicle_id: Option<u16>,
    pub score: i32,
    pub ping: u32,
}

/// An owned, read-only current-server snapshot.
///
/// The address and hostname remain bytes because SA-MP does not guarantee a
/// Unicode encoding. The host refreshes this on its verified R1 game-thread
/// packet pump, so retrieving it never waits for the game thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfo {
    pub address: Vec<u8>,
    pub hostname: Vec<u8>,
    pub port: u16,
}

/// C-compatible storage for [`LocalPlayer`].
///
/// This is output-only. `nickname_len` selects the initialized prefix of
/// `nickname`; the buffer has no required terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RakSampLocalPlayerV1 {
    pub id: u16,
    pub nickname_len: u16,
    pub nickname: [u8; 256],
    pub colour: u32,
    pub spawned: u8,
    pub special_action: u8,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
    pub position: Vector3,
    pub velocity: Vector3,
    pub has_vehicle: u8,
    pub _reserved: u8,
    pub vehicle_id: u16,
    pub score: i32,
    pub ping: u32,
}

impl Default for RakSampLocalPlayerV1 {
    fn default() -> Self {
        Self {
            id: 0,
            nickname_len: 0,
            nickname: [0; 256],
            colour: 0,
            spawned: 0,
            special_action: 0,
            animation_id: 0,
            health: 0.0,
            armour: 0.0,
            position: Vector3::default(),
            velocity: Vector3::default(),
            has_vehicle: 0,
            _reserved: 0,
            vehicle_id: 0,
            score: 0,
            ping: 0,
        }
    }
}

/// C-compatible storage for [`ServerInfo`].
///
/// This is output-only. Each length selects the initialized prefix of its
/// corresponding buffer; neither buffer requires a terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RakSampServerInfoV1 {
    pub address_len: u16,
    pub hostname_len: u16,
    pub address: [u8; 257],
    pub hostname: [u8; 257],
    pub port: u16,
}

impl Default for RakSampServerInfoV1 {
    fn default() -> Self {
        Self {
            address_len: 0,
            hostname_len: 0,
            address: [0; 257],
            hostname: [0; 257],
            port: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RakSampHostStatus {
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
pub enum RakSampClientVersion {
    R1 = 1,
    R2 = 2,
    R3_1 = 3,
    R4_2 = 4,
    R5_1 = 5,
    Dl = 6,
}

impl RakSampClientVersion {
    const fn from_raw(value: u32) -> Option<Self> {
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

/// The host-side ABI table exported by `rak_samp.asi`.
///
/// Fields are currently appended to preserve the v1 layout; during the ALPHA
/// stage the ABI may make an explicit compatibility break. Check `size` before
/// accessing fields added by a newer ABI version. Normal plugins use
/// [`HostApi`] instead of calling this table directly.
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
    /// Copies and queues a local R1 dialog request for the verified game-thread pump.
    pub show_local_dialog: unsafe extern "system" fn(
        u16,
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> RakSampResult,
    /// Copies the latest host-owned local-player snapshot into `output`.
    pub local_player: unsafe extern "system" fn(*mut RakSampLocalPlayerV1) -> RakSampResult,
    /// Copies the latest R1 `CNetGame` state scalar into `output`.
    pub samp_game_state: unsafe extern "system" fn(*mut i32) -> RakSampResult,
    /// Copies the detected SA-MP client version identity into `output`.
    pub samp_version: unsafe extern "system" fn(*mut u32) -> RakSampResult,
    /// Decodes an owned bit stream with SA-MP's native RakNet string compressor.
    ///
    /// `input_read_offset` is the initial cursor, and `output_read_offset`
    /// receives the cursor after a successful decode. The output buffer has no
    /// required terminator; `output_len` selects its initialized prefix.
    pub decode_string: unsafe extern "system" fn(
        *const u8,
        usize,
        usize,
        usize,
        *mut u8,
        usize,
        *mut usize,
        *mut usize,
    ) -> RakSampResult,
    /// Copies the latest host-owned R1 current-server snapshot into `output`.
    pub server_info: unsafe extern "system" fn(*mut RakSampServerInfoV1) -> RakSampResult,
    /// Copies and queues a local R1 chat entry for the verified game-thread pump.
    pub show_local_chat_message: unsafe extern "system" fn(
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
    ) -> RakSampResult,
    /// Copies and queues a local R1 death-window entry for the game-thread pump.
    pub show_local_death_message: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        u8,
    ) -> RakSampResult,
    /// Copies the latest game-thread-cached R1 chat display mode into `output`.
    pub local_chat_display_mode: unsafe extern "system" fn(*mut i32) -> RakSampResult,
    /// Copies the latest game-thread-cached R1 cursor mode into `output`.
    pub local_cursor_mode: unsafe extern "system" fn(*mut i32) -> RakSampResult,
    /// Copies the latest game-thread-cached R1 scoreboard-open flag into `output`.
    pub local_scoreboard_open: unsafe extern "system" fn(*mut u8) -> RakSampResult,
}

pub type RakSampGetApiV1 = unsafe extern "system" fn(u32) -> *const RakSampApiV1;

type EventHandler =
    dyn for<'event> Fn(&mut events::Event<'event>) -> RakSampHookAction + Send + Sync + 'static;

struct CallbackState {
    api: HostApi,
    handler: Box<EventHandler>,
}

type RegisterListener = unsafe extern "system" fn(
    RakSampDirection,
    Option<RakSampEventCallbackV1>,
    *mut c_void,
    *mut RakSampSubscription,
) -> RakSampResult;

/// A validated reference to the host API table.
#[derive(Clone, Copy)]
pub struct HostApi {
    raw: &'static RakSampApiV1,
}

/// An owned packet or RPC callback registration.
///
/// Call [`Self::unregister_and_wait`] from a worker thread before unloading the plugin ASI.
/// Dropping this value attempts a nonblocking listener removal and intentionally retains the
/// callback allocation, so it is memory-safe but does not prepare a plugin for `FreeLibrary`.
#[must_use = "a subscription must be synchronized before unloading the plugin ASI"]
pub struct Subscription {
    api: HostApi,
    raw: RakSampSubscription,
    callback: Option<Box<CallbackState>>,
}

impl Subscription {
    /// Returns this registration's host-assigned identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.raw.id
    }

    /// Removes this listener and waits until the host cannot invoke its callback anymore.
    ///
    /// Call this from a worker thread, never from `DllMain` or from this subscription's callback.
    /// On failure, the returned error retains the subscription so shutdown can be retried.
    pub fn unregister_and_wait(mut self) -> Result<(), SubscriptionShutdownError> {
        let result = unsafe { (self.api.raw.unregister_and_wait)(self.raw) };
        if matches!(
            result,
            RakSampResult::Ok | RakSampResult::SubscriptionNotFound
        ) {
            drop(self.callback.take());
            Ok(())
        } else {
            Err(SubscriptionShutdownError {
                result,
                subscription: self,
            })
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            // Do not wait here: Drop may run inside DllMain or a callback. The host listener is
            // detached, but the allocation must stay valid for any callback already in flight.
            let _ = unsafe { (self.api.raw.unregister)(self.raw) };
            let _ = Box::leak(callback);
        }
    }
}

impl fmt::Debug for Subscription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Subscription")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

/// A synchronized subscription removal that the host could not complete.
#[derive(Debug)]
pub struct SubscriptionShutdownError {
    result: RakSampResult,
    subscription: Subscription,
}

impl SubscriptionShutdownError {
    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(&self) -> RakSampResult {
        self.result
    }

    /// Returns the still-registered subscription so shutdown can be retried.
    pub fn into_subscription(self) -> Subscription {
        self.subscription
    }
}

impl fmt::Display for SubscriptionShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host could not synchronize subscription {}: {:?}",
            self.subscription.id(),
            self.result
        )
    }
}

impl std::error::Error for SubscriptionShutdownError {}

/// A group of callback subscriptions that should be stopped together.
///
/// Call [`Self::unregister_and_wait`] from a worker thread before unloading the plugin ASI.
#[must_use = "subscriptions must be synchronized before unloading the plugin ASI"]
#[derive(Debug, Default)]
pub struct SubscriptionSet {
    subscriptions: Vec<Subscription>,
}

impl SubscriptionSet {
    /// Creates an empty subscription group.
    pub const fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    /// Adds one successful registration to this group.
    pub fn push(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }

    /// Returns the number of owned subscriptions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    /// Returns whether this group has no subscriptions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }

    /// Adds a registration result while preserving earlier registrations if it failed.
    ///
    /// This is primarily useful to [`register_handlers!`] and other batch-registration helpers.
    pub fn try_add(
        mut self,
        registration: Result<Subscription, RakSampResult>,
    ) -> Result<Self, SubscriptionRegistrationError> {
        match registration {
            Ok(subscription) => {
                self.push(subscription);
                Ok(self)
            }
            Err(result) => Err(SubscriptionRegistrationError {
                result,
                subscriptions: self,
            }),
        }
    }

    /// Stops every callback and waits until the host cannot invoke any of them.
    ///
    /// Call this from a worker thread, never from `DllMain` or from one of the registered
    /// callbacks. Failures retain only the subscriptions that still need a retry.
    pub fn unregister_and_wait(self) -> Result<(), SubscriptionSetShutdownError> {
        let mut subscriptions = Vec::new();
        let mut failures = Vec::new();
        for subscription in self.subscriptions {
            if let Err(error) = subscription.unregister_and_wait() {
                let result = error.result();
                let subscription = error.into_subscription();
                failures.push(SubscriptionShutdownFailure {
                    id: subscription.id(),
                    result,
                });
                subscriptions.push(subscription);
            }
        }
        if subscriptions.is_empty() {
            Ok(())
        } else {
            Err(SubscriptionSetShutdownError {
                failures,
                subscriptions: Self { subscriptions },
            })
        }
    }
}

/// A callback registration that failed after earlier batch registrations succeeded.
#[derive(Debug)]
pub struct SubscriptionRegistrationError {
    result: RakSampResult,
    subscriptions: SubscriptionSet,
}

impl SubscriptionRegistrationError {
    /// Returns the host result from the failed registration.
    #[must_use]
    pub const fn result(&self) -> RakSampResult {
        self.result
    }

    /// Returns the earlier successful registrations for synchronized cleanup or retry.
    pub fn into_subscriptions(self) -> SubscriptionSet {
        self.subscriptions
    }
}

impl fmt::Display for SubscriptionRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host rejected a callback registration: {:?}",
            self.result
        )
    }
}

impl std::error::Error for SubscriptionRegistrationError {}

/// One subscription that the host could not synchronize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubscriptionShutdownFailure {
    id: u64,
    result: RakSampResult,
}

impl SubscriptionShutdownFailure {
    /// Returns the host-assigned subscription identifier.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.id
    }

    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(self) -> RakSampResult {
        self.result
    }
}

/// A batch shutdown that left one or more callbacks registered.
#[derive(Debug)]
pub struct SubscriptionSetShutdownError {
    failures: Vec<SubscriptionShutdownFailure>,
    subscriptions: SubscriptionSet,
}

impl SubscriptionSetShutdownError {
    /// Returns each callback that still needs synchronized removal.
    #[must_use]
    pub fn failures(&self) -> &[SubscriptionShutdownFailure] {
        &self.failures
    }

    /// Returns the remaining subscriptions so shutdown can be retried.
    pub fn into_subscriptions(self) -> SubscriptionSet {
        self.subscriptions
    }
}

impl fmt::Display for SubscriptionSetShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host could not synchronize {} subscriptions",
            self.failures.len()
        )
    }
}

impl std::error::Error for SubscriptionSetShutdownError {}

/// Registers a batch of packet and RPC handlers into one [`SubscriptionSet`].
///
/// The macro accepts `packet`, `rpc`, `packet_id`, `rpc_id`, `typed_packet`, and `typed_rpc`
/// entries. If one registration fails, the error retains every earlier successful subscription so
/// the caller can synchronize them before unloading the plugin.
#[macro_export]
macro_rules! register_handlers {
    ($api:expr; $($kind:ident($($argument:expr),*)),+ $(,)?) => {{
        (|| -> Result<$crate::SubscriptionSet, $crate::SubscriptionRegistrationError> {
            let api = $api;
            let subscriptions = $crate::SubscriptionSet::new();
            $(
                let subscriptions = $crate::register_handlers!(
                    @add subscriptions, api, $kind, $($argument),*
                )?;
            )+
            Ok(subscriptions)
        })()
    }};
    (@add $subscriptions:ident, $api:ident, packet, $direction:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_packet($direction, $handler))
    };
    (@add $subscriptions:ident, $api:ident, rpc, $direction:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_rpc($direction, $handler))
    };
    (@add $subscriptions:ident, $api:ident, packet_id, $direction:expr, $id:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_packet_id($direction, $id, $handler))
    };
    (@add $subscriptions:ident, $api:ident, rpc_id, $direction:expr, $id:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_rpc_id($direction, $id, $handler))
    };
    (@add $subscriptions:ident, $api:ident, typed_packet, $direction:expr, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_typed_packet($direction, $descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, typed_rpc, $direction:expr, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_typed_rpc($direction, $descriptor, $handler))
    };
}

impl HostApi {
    /// # Safety
    ///
    /// `raw` must point to a live API table exported by a compatible host.
    pub(crate) unsafe fn from_raw(raw: *const RakSampApiV1) -> Result<Self, ResolveError> {
        let raw = NonNull::new(raw.cast_mut()).ok_or(ResolveError::MissingApi)?;
        let raw = unsafe { raw.as_ref() };
        if raw.abi_version != ABI_VERSION_V1 || raw.size < mem::size_of::<RakSampApiV1>() as u32 {
            return Err(ResolveError::UnsupportedAbi);
        }
        Ok(Self { raw })
    }

    #[must_use]
    pub(crate) fn raw(self) -> &'static RakSampApiV1 {
        self.raw
    }

    #[must_use]
    pub fn status(self) -> RakSampHostStatus {
        (self.raw.host_status)()
    }

    /// Returns whether the host attached to a recognized SA-MP client and its
    /// RakClient hooks are ready.
    ///
    /// This is the safe host-level equivalent of SF.lua's
    /// `isSampAvailable`; it does not dereference `CNetGame` on the plugin
    /// thread.
    pub fn is_samp_available(self) -> bool {
        self.status() == RakSampHostStatus::Ready
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

    /// Registers a packet callback.
    ///
    /// The callback receives a view that is valid only for that invocation. Use typed packet
    /// descriptors from [`events::packet`] to decode, block, or replace a matching payload.
    pub fn on_packet<F>(
        self,
        direction: RakSampDirection,
        handler: F,
    ) -> Result<Subscription, RakSampResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> RakSampHookAction + Send + Sync + 'static,
    {
        self.register_listener(direction, handler, self.raw.register_packet)
    }

    /// Registers an RPC callback.
    ///
    /// The callback receives a view that is valid only for that invocation. Use typed RPC
    /// descriptors from [`events::rpc`] to decode, block, or replace a matching payload.
    pub fn on_rpc<F>(
        self,
        direction: RakSampDirection,
        handler: F,
    ) -> Result<Subscription, RakSampResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> RakSampHookAction + Send + Sync + 'static,
    {
        self.register_listener(direction, handler, self.raw.register_rpc)
    }

    /// Registers a packet callback that runs only for one packet ID.
    pub fn on_packet_id<F>(
        self,
        direction: RakSampDirection,
        packet_id: u8,
        handler: F,
    ) -> Result<Subscription, RakSampResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> RakSampHookAction + Send + Sync + 'static,
    {
        self.on_packet(direction, move |event| {
            if event.id() == packet_id {
                handler(event)
            } else {
                RakSampHookAction::Continue
            }
        })
    }

    /// Registers an RPC callback that runs only for one RPC ID.
    pub fn on_rpc_id<F>(
        self,
        direction: RakSampDirection,
        rpc_id: u8,
        handler: F,
    ) -> Result<Subscription, RakSampResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> RakSampHookAction + Send + Sync + 'static,
    {
        self.on_rpc(direction, move |event| {
            if event.id() == rpc_id {
                handler(event)
            } else {
                RakSampHookAction::Continue
            }
        })
    }

    /// Registers a packet callback that decodes one typed packet descriptor.
    ///
    /// Nonmatching packet IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_packet`] when decode failures need plugin-specific reporting.
    pub fn on_typed_packet<T, F>(
        self,
        direction: RakSampDirection,
        packet: events::Packet<T>,
        handler: F,
    ) -> Result<Subscription, RakSampResult>
    where
        T: 'static,
        F: Fn(T) -> events::RpcAction<T> + Send + Sync + 'static,
    {
        self.on_packet_id(direction, packet.id(), move |event| {
            packet
                .handle(event, &handler)
                .unwrap_or(RakSampHookAction::Continue)
        })
    }

    /// Registers an RPC callback that decodes one typed RPC descriptor.
    ///
    /// Nonmatching RPC IDs and decode errors continue without calling `handler`. Use
    /// [`Self::on_rpc`] when decode failures need plugin-specific reporting.
    pub fn on_typed_rpc<T, F>(
        self,
        direction: RakSampDirection,
        rpc: events::Rpc<T>,
        handler: F,
    ) -> Result<Subscription, RakSampResult>
    where
        T: 'static,
        F: Fn(T) -> events::RpcAction<T> + Send + Sync + 'static,
    {
        self.on_rpc_id(direction, rpc.id(), move |event| {
            rpc.handle(event, &handler)
                .unwrap_or(RakSampHookAction::Continue)
        })
    }

    fn register_listener<F>(
        self,
        direction: RakSampDirection,
        handler: F,
        register: RegisterListener,
    ) -> Result<Subscription, RakSampResult>
    where
        F: for<'event> Fn(&mut events::Event<'event>) -> RakSampHookAction + Send + Sync + 'static,
    {
        let mut callback = Box::new(CallbackState {
            api: self,
            handler: Box::new(handler),
        });
        let mut raw = RakSampSubscription::default();
        let result = unsafe {
            register(
                direction,
                Some(dispatch_callback),
                (&mut *callback as *mut CallbackState).cast(),
                &raw mut raw,
            )
        };
        if result == RakSampResult::Ok {
            Ok(Subscription {
                api: self,
                raw,
                callback: Some(callback),
            })
        } else {
            Err(result)
        }
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

    /// Sends one server-bound SA-MP chat message (RPC 101).
    ///
    /// This is the safe equivalent of SF.lua's `sampSendChat`. The message is
    /// serialized as the protocol's bounded `string8` payload; a
    /// slash-prefixed value instead uses the command RPC (50), matching the
    /// native helper. It is real network traffic, not a local chat display
    /// action.
    pub fn send_chat(self, text: &[u8]) -> RakSampResult {
        let descriptor = if text.first() == Some(&b'/') {
            events::rpc::outgoing::SEND_COMMAND
        } else {
            events::rpc::outgoing::SEND_CHAT
        };
        self.send_typed_rpc(descriptor, text.to_vec())
    }

    /// Sends SA-MP's empty request-spawn RPC (129).
    ///
    /// This is the protocol-level equivalent of SF.lua's
    /// `sampSendRequestSpawn`; it does not call native local-player methods or
    /// mutate client state.
    pub fn send_request_spawn(self) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_REQUEST_SPAWN, ())
    }

    /// Sends SA-MP's request-class RPC (128).
    ///
    /// This carries the same server-bound protocol value as SF.lua's
    /// `sampRequestClass`, but does not invoke the native local-player method
    /// or update any local class-selection state.
    pub fn send_request_class(self, class_id: i32) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_REQUEST_CLASS, class_id)
    }

    /// Sends SA-MP's interior-change RPC (118).
    ///
    /// This is protocol-only. It does not change the GTA interior or mutate
    /// SA-MP's native local-player state.
    pub fn send_interior_change(self, interior_id: u8) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_INTERIOR_CHANGE, interior_id)
    }

    /// Sends SA-MP's empty spawn RPC (52).
    ///
    /// This is protocol-only. It does not call the native local-player spawn
    /// method or change local spawn state.
    pub fn send_spawn(self) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_SPAWN, ())
    }

    /// Sends SA-MP's enter-vehicle RPC (26).
    ///
    /// This is protocol-only. It does not put the local GTA ped in a vehicle
    /// or otherwise alter native local-player state.
    pub fn send_enter_vehicle(self, vehicle_id: u16, passenger: bool) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_ENTER_VEHICLE,
            events::rpc::outgoing::EnterVehicle {
                vehicle_id,
                passenger,
            },
        )
    }

    /// Sends SA-MP's exit-vehicle RPC (154).
    ///
    /// This is protocol-only. It does not make the local GTA ped leave a
    /// vehicle or otherwise alter native local-player state.
    pub fn send_exit_vehicle(self, vehicle_id: u16) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_EXIT_VEHICLE, vehicle_id)
    }

    /// Sends a server-bound dialog response (RPC 62).
    pub fn send_dialog_response(
        self,
        dialog_id: u16,
        button: u8,
        list_item: u16,
        input: &[u8],
    ) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_DIALOG_RESPONSE,
            events::rpc::outgoing::DialogResponse {
                dialog_id,
                button,
                list_item,
                input: input.to_vec(),
            },
        )
    }

    /// Sends a server-bound player-click action (RPC 23).
    pub fn send_click_player(self, player_id: u16, source: u8) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_CLICK_PLAYER,
            events::rpc::outgoing::ClickPlayer { player_id, source },
        )
    }

    /// Sends a server-bound textdraw-click action (RPC 83).
    pub fn send_click_textdraw(self, textdraw_id: u16) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_CLICK_TEXT_DRAW, textdraw_id)
    }

    /// Sends a server-bound death notification naming another player (RPC 53).
    pub fn send_death_by_player(self, player_id: u16, reason: u8) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_DEATH_NOTIFICATION,
            events::rpc::outgoing::DeathNotification {
                reason,
                killer_id: player_id,
            },
        )
    }

    /// Sends the empty menu-quit RPC (140).
    pub fn send_menu_quit(self) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_QUIT_MENU, ())
    }

    /// Sends a server-bound menu-row selection (RPC 132).
    pub fn send_menu_select_row(self, row: u8) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_MENU_SELECT, row)
    }

    /// Sends a server-bound pickup notification (RPC 131).
    pub fn send_picked_up_pickup(self, pickup_id: i32) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_PICKED_UP_PICKUP, pickup_id)
    }

    /// Sends a server-bound vehicle-destroyed notification (RPC 136).
    pub fn send_vehicle_destroyed(self, vehicle_id: u16) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_VEHICLE_DESTROYED, vehicle_id)
    }

    /// Sends a server-bound vehicle-damage update (RPC 106).
    pub fn send_vehicle_damage(
        self,
        vehicle_id: u16,
        panel_damage: i32,
        door_damage: i32,
        lights: u8,
        tires: u8,
    ) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_VEHICLE_DAMAGED,
            events::rpc::outgoing::VehicleDamage {
                vehicle_id,
                panel_damage,
                door_damage,
                lights,
                tires,
            },
        )
    }

    /// Sends a server-bound SCM event (RPC 96).
    ///
    /// The values follow SA-MP's wire order: ID, first parameter, second
    /// parameter, then event ID.
    pub fn send_scm_event(self, event: i32, id: i32, param1: i32, param2: i32) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_VEHICLE_TUNING,
            events::rpc::outgoing::VehicleTuning {
                vehicle_id: id,
                param1,
                param2,
                event,
            },
        )
    }

    /// Sends a server-bound give-damage notification (RPC 115).
    pub fn send_give_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> RakSampResult {
        self.send_damage(player_id, damage, weapon, body_part, false)
    }

    /// Sends a server-bound take-damage notification (RPC 115).
    pub fn send_take_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> RakSampResult {
        self.send_damage(player_id, damage, weapon, body_part, true)
    }

    /// Sends a complete attached-object edit action (RPC 116).
    ///
    /// The typed value deliberately includes both colour fields. SF.lua's
    /// helper leaves them unspecified, so accepting its partial parameter list
    /// here could create malformed or accidentally lossy traffic.
    pub fn send_edit_attached_object(
        self,
        edit: events::rpc::outgoing::EditAttachedObject,
    ) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_EDIT_ATTACHED_OBJECT, edit)
    }

    /// Sends a complete global or player-object edit action (RPC 117).
    pub fn send_edit_object(self, edit: events::rpc::outgoing::EditObject) -> RakSampResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_EDIT_OBJECT, edit)
    }

    /// Sends a bounded server-bound RCON command packet (201).
    pub fn send_rcon_command(self, command: &[u8]) -> RakSampResult {
        self.send_typed_packet(
            events::packet::outgoing::SEND_RCON_COMMAND,
            command.to_vec(),
        )
    }

    /// Sends a complete local aim-sync packet (203).
    pub fn send_aim_sync(self, sync: events::packet::AimSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_AIM_SYNC, sync)
    }

    /// Sends a complete local bullet-sync packet (206).
    pub fn send_bullet_sync(self, sync: events::packet::BulletSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_BULLET_SYNC, sync)
    }

    /// Sends a complete local vehicle-sync packet (200).
    pub fn send_vehicle_sync(self, sync: events::packet::VehicleSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_VEHICLE_SYNC, sync)
    }

    /// Sends a complete local on-foot player-sync packet (207).
    pub fn send_player_sync(self, sync: events::packet::PlayerSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_PLAYER_SYNC, sync)
    }

    /// Sends a complete local spectator-sync packet (212).
    pub fn send_spectator_sync(self, sync: events::packet::SpectatorSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_SPECTATOR_SYNC, sync)
    }

    /// Sends a complete local trailer-sync packet (210).
    pub fn send_trailer_sync(self, sync: events::packet::TrailerSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_TRAILER_SYNC, sync)
    }

    /// Sends a complete local passenger-sync packet (211).
    pub fn send_passenger_sync(self, sync: events::packet::PassengerSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_PASSENGER_SYNC, sync)
    }

    /// Sends a complete local unoccupied-vehicle sync packet (209).
    pub fn send_unoccupied_sync(self, sync: events::packet::UnoccupiedSync) -> RakSampResult {
        self.send_typed_packet(events::packet::outgoing::SEND_UNOCCUPIED_SYNC, sync)
    }

    /// Sends a complete owned plugin-side bit stream as a packet payload.
    pub fn send_packet_stream(
        self,
        packet_id: u8,
        payload: &raknet::BitStream,
        options: RakSampSendOptions,
    ) -> RakSampResult {
        self.send_packet(packet_id, payload.as_bytes(), payload.len_bits(), options)
    }

    /// Sends a complete owned plugin-side bit stream as an RPC payload.
    pub fn send_rpc_stream(
        self,
        rpc_id: u8,
        payload: &raknet::BitStream,
        options: RakSampSendOptions,
    ) -> RakSampResult {
        self.send_rpc(rpc_id, payload.as_bytes(), payload.len_bits(), options)
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

    /// Decodes one native RakNet-compressed string from an owned bit stream.
    ///
    /// On success, advances `stream`'s read cursor by exactly the bits the
    /// client decoder consumed. The returned byte string has no terminating
    /// NUL and is bounded to [`MAX_RAKNET_DECODED_STRING_BYTES`]. On failure,
    /// the stream cursor is unchanged.
    pub fn decode_string(self, stream: &mut raknet::BitStream) -> Result<Vec<u8>, RakSampResult> {
        let mut output = vec![0_u8; MAX_RAKNET_DECODED_STRING_BYTES + 1];
        let mut output_len = 0_usize;
        let mut output_read_offset = 0_usize;
        let result = unsafe {
            (self.raw.decode_string)(
                stream.as_bytes().as_ptr(),
                stream.len_bytes(),
                stream.len_bits(),
                stream.read_offset_bits(),
                output.as_mut_ptr(),
                output.len(),
                &raw mut output_len,
                &raw mut output_read_offset,
            )
        };
        if result != RakSampResult::Ok {
            return Err(result);
        }
        if output_len > MAX_RAKNET_DECODED_STRING_BYTES || output_read_offset > stream.len_bits() {
            return Err(RakSampResult::NativeCallFailed);
        }
        stream
            .set_read_offset(output_read_offset)
            .map_err(|_| RakSampResult::NativeCallFailed)?;
        output.truncate(output_len);
        Ok(output)
    }

    /// Copies and queues a direct local dialog on the verified R1 game thread.
    ///
    /// [`RakSampResult::Ok`] confirms only that the host copied and queued the
    /// request; it does not mean the player has seen or dismissed the dialog.
    pub fn show_local_dialog(self, dialog: LocalDialog<'_>) -> RakSampResult {
        if !dialog.is_valid() {
            return RakSampResult::InvalidArgument;
        }
        unsafe {
            (self.raw.show_local_dialog)(
                dialog.id,
                dialog.style.as_raw(),
                dialog.title.as_ptr(),
                dialog.title.len(),
                dialog.text.as_ptr(),
                dialog.text.len(),
                dialog.button1.as_ptr(),
                dialog.button1.len(),
                dialog.button2.as_ptr(),
                dialog.button2.len(),
            )
        }
    }

    /// Copies and queues a direct local R1 chat entry on the game thread.
    ///
    /// [`RakSampResult::Ok`] confirms only that the host copied and queued the
    /// entry. It does not send a chat RPC or mean the player has seen it.
    pub fn show_local_chat_message(self, message: LocalChatMessage<'_>) -> RakSampResult {
        if !message.is_valid() {
            return RakSampResult::InvalidArgument;
        }
        unsafe {
            (self.raw.show_local_chat_message)(
                message.style.as_raw(),
                message.text.as_ptr(),
                message.text.len(),
                message.prefix.as_ptr(),
                message.prefix.len(),
                message.text_colour,
                message.prefix_colour,
            )
        }
    }

    /// Copies and queues a direct local R1 death-window entry on the game thread.
    ///
    /// [`RakSampResult::Ok`] confirms only that the host copied and queued the
    /// entry. It does not send any packet or RPC.
    pub fn show_local_death_message(self, message: LocalDeathMessage<'_>) -> RakSampResult {
        if !message.is_valid() {
            return RakSampResult::InvalidArgument;
        }
        unsafe {
            (self.raw.show_local_death_message)(
                message.killer.as_ptr(),
                message.killer.len(),
                message.victim.as_ptr(),
                message.victim.len(),
                message.killer_colour,
                message.victim_colour,
                message.weapon,
            )
        }
    }

    /// Returns the cached R1 local chat-window display mode.
    pub fn local_chat_display_mode(self) -> Result<LocalChatDisplayMode, RakSampResult> {
        let mut raw = 0;
        match unsafe { (self.raw.local_chat_display_mode)(&mut raw) } {
            RakSampResult::Ok => {
                LocalChatDisplayMode::from_raw(raw).ok_or(RakSampResult::NativeCallFailed)
            }
            result => Err(result),
        }
    }

    /// Returns whether the cached R1 local chat window is visible.
    pub fn is_local_chat_visible(self) -> Result<bool, RakSampResult> {
        self.local_chat_display_mode()
            .map(|mode| mode != LocalChatDisplayMode::Off)
    }

    /// Returns the cached R1 local cursor mode.
    pub fn local_cursor_mode(self) -> Result<LocalCursorMode, RakSampResult> {
        let mut raw = 0;
        match unsafe { (self.raw.local_cursor_mode)(&mut raw) } {
            RakSampResult::Ok => {
                LocalCursorMode::from_raw(raw).ok_or(RakSampResult::NativeCallFailed)
            }
            result => Err(result),
        }
    }

    /// Returns whether the cached R1 local cursor mode is active.
    pub fn is_local_cursor_active(self) -> Result<bool, RakSampResult> {
        self.local_cursor_mode()
            .map(|mode| mode != LocalCursorMode::None)
    }

    /// Returns whether the cached R1 local scoreboard is open.
    pub fn is_local_scoreboard_open(self) -> Result<bool, RakSampResult> {
        let mut raw = 0;
        match unsafe { (self.raw.local_scoreboard_open)(&mut raw) } {
            RakSampResult::Ok => match raw {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(RakSampResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }

    /// Returns a cloned, nonblocking local-player snapshot.
    ///
    /// This returns [`RakSampResult::NotReady`] until the verified R1 game
    /// thread has published its first complete, server-assigned snapshot.
    pub fn local_player(self) -> Result<LocalPlayer, RakSampResult> {
        let mut raw = RakSampLocalPlayerV1::default();
        match unsafe { (self.raw.local_player)(&mut raw) } {
            RakSampResult::Ok => {}
            result => return Err(result),
        }
        let nickname_len = usize::from(raw.nickname_len);
        if nickname_len > raw.nickname.len() {
            return Err(RakSampResult::NativeCallFailed);
        }
        Ok(LocalPlayer {
            id: raw.id,
            nickname: raw.nickname[..nickname_len].to_vec(),
            colour: raw.colour,
            spawned: raw.spawned != 0,
            health: raw.health,
            armour: raw.armour,
            position: raw.position,
            velocity: raw.velocity,
            special_action: raw.special_action,
            animation_id: raw.animation_id,
            vehicle_id: (raw.has_vehicle != 0).then_some(raw.vehicle_id),
            score: raw.score,
            ping: raw.ping,
        })
    }

    /// Returns the cached local-player ID.
    pub fn local_player_id(self) -> Result<u16, RakSampResult> {
        self.local_player().map(|player| player.id)
    }

    /// Returns owned local-player nickname bytes without assuming text encoding.
    pub fn local_player_nickname(self) -> Result<Vec<u8>, RakSampResult> {
        self.local_player().map(|player| player.nickname)
    }

    /// Returns the cached local-player ARGB colour.
    pub fn local_player_colour(self) -> Result<u32, RakSampResult> {
        self.local_player().map(|player| player.colour)
    }

    /// Returns whether the cached local player is spawned.
    pub fn is_local_player_spawned(self) -> Result<bool, RakSampResult> {
        self.local_player().map(|player| player.spawned)
    }

    /// Returns the cached local-player health.
    pub fn local_player_health(self) -> Result<f32, RakSampResult> {
        self.local_player().map(|player| player.health)
    }

    /// Returns the cached local-player armour.
    pub fn local_player_armour(self) -> Result<f32, RakSampResult> {
        self.local_player().map(|player| player.armour)
    }

    /// Returns the cached local-player special action.
    pub fn local_player_special_action(self) -> Result<u8, RakSampResult> {
        self.local_player().map(|player| player.special_action)
    }

    /// Returns the cached local-player animation ID.
    pub fn local_player_animation_id(self) -> Result<u16, RakSampResult> {
        self.local_player().map(|player| player.animation_id)
    }

    /// Returns the cached local-player score.
    pub fn local_player_score(self) -> Result<i32, RakSampResult> {
        self.local_player().map(|player| player.score)
    }

    /// Returns the cached local-player ping in milliseconds.
    pub fn local_player_ping(self) -> Result<u32, RakSampResult> {
        self.local_player().map(|player| player.ping)
    }

    /// Returns a cloned, nonblocking current-server snapshot.
    ///
    /// This returns [`RakSampResult::NotReady`] until the verified R1 game
    /// thread has published a valid address and port.
    pub fn server_info(self) -> Result<ServerInfo, RakSampResult> {
        let mut raw = RakSampServerInfoV1::default();
        match unsafe { (self.raw.server_info)(&mut raw) } {
            RakSampResult::Ok => {}
            result => return Err(result),
        }
        let address_len = usize::from(raw.address_len);
        let hostname_len = usize::from(raw.hostname_len);
        if address_len > raw.address.len() || hostname_len > raw.hostname.len() || raw.port == 0 {
            return Err(RakSampResult::NativeCallFailed);
        }
        Ok(ServerInfo {
            address: raw.address[..address_len].to_vec(),
            hostname: raw.hostname[..hostname_len].to_vec(),
            port: raw.port,
        })
    }

    /// Returns the cached native `CNetGame` state for the verified R1 client.
    ///
    /// The value is deliberately an opaque scalar: SA-MP has no stable public
    /// enum ABI for it. Like [`Self::local_player`], this never calls client
    /// code from the plugin thread and returns `NotReady` before publication.
    pub fn samp_game_state(self) -> Result<i32, RakSampResult> {
        let mut state = 0_i32;
        match unsafe { (self.raw.samp_game_state)(&mut state) } {
            RakSampResult::Ok => Ok(state),
            result => Err(result),
        }
    }

    /// Returns the version identity obtained when the host attached to `samp.dll`.
    ///
    /// This is a detection result, not a client-memory read, so it is available
    /// for every recognized client build once the host runtime is ready.
    pub fn samp_version(self) -> Result<RakSampClientVersion, RakSampResult> {
        let mut version = 0_u32;
        match unsafe { (self.raw.samp_version)(&mut version) } {
            RakSampResult::Ok => {
                RakSampClientVersion::from_raw(version).ok_or(RakSampResult::NativeCallFailed)
            }
            result => Err(result),
        }
    }

    fn send_typed_rpc<T>(self, descriptor: events::Rpc<T>, value: T) -> RakSampResult {
        let Ok(payload) = descriptor.encode(self, value) else {
            return RakSampResult::InvalidArgument;
        };
        self.send_rpc(
            descriptor.id(),
            payload.as_bytes(),
            payload.len_bits(),
            RakSampSendOptions::default(),
        )
    }

    fn send_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
        take: bool,
    ) -> RakSampResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_DAMAGE,
            events::rpc::outgoing::Damage {
                player_id,
                damage,
                weapon,
                body_part,
                take,
            },
        )
    }

    fn send_typed_packet<T>(self, descriptor: events::Packet<T>, value: T) -> RakSampResult {
        let Ok(payload) = descriptor.encode(self, value) else {
            return RakSampResult::InvalidArgument;
        };
        self.send_packet(
            descriptor.id(),
            payload.as_bytes(),
            payload.len_bits(),
            RakSampSendOptions::default(),
        )
    }
}

unsafe extern "system" fn dispatch_callback(
    user_data: *mut c_void,
    raw: *mut RakSampEventV1,
) -> RakSampHookAction {
    let Some(callback) = (unsafe { user_data.cast::<CallbackState>().as_ref() }) else {
        return RakSampHookAction::Continue;
    };
    let Ok(mut event) = (unsafe { events::Event::from_callback(callback.api, raw) }) else {
        return RakSampHookAction::Continue;
    };
    catch_unwind(AssertUnwindSafe(|| (callback.handler)(&mut event)))
        .unwrap_or(RakSampHookAction::Continue)
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
    use crate::events::{RpcAction, packet, rpc::incoming, test_support};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    static REGISTRATION_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct DropCounter(Arc<AtomicUsize>);

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Release);
        }
    }

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
    fn ready_fixture_host_reports_samp_available() {
        let api = test_support::test_api();
        assert!(api.is_samp_loaded());
        assert!(api.is_samp_available());
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
            mem::offset_of!(RakSampApiV1, show_local_dialog),
            mem::offset_of!(RakSampApiV1, event_read_encoded_string) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, local_player),
            mem::offset_of!(RakSampApiV1, show_local_dialog) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, samp_game_state),
            mem::offset_of!(RakSampApiV1, local_player) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, samp_version),
            mem::offset_of!(RakSampApiV1, samp_game_state) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, decode_string),
            mem::offset_of!(RakSampApiV1, samp_version) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, server_info),
            mem::offset_of!(RakSampApiV1, decode_string) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, show_local_chat_message),
            mem::offset_of!(RakSampApiV1, server_info) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, show_local_death_message),
            mem::offset_of!(RakSampApiV1, show_local_chat_message) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, local_chat_display_mode),
            mem::offset_of!(RakSampApiV1, show_local_death_message) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, local_cursor_mode),
            mem::offset_of!(RakSampApiV1, local_chat_display_mode) + function_size
        );
        assert_eq!(
            mem::offset_of!(RakSampApiV1, local_scoreboard_open),
            mem::offset_of!(RakSampApiV1, local_cursor_mode) + function_size
        );
        assert_eq!(
            mem::size_of::<RakSampApiV1>(),
            mem::offset_of!(RakSampApiV1, local_scoreboard_open) + function_size
        );
    }

    #[test]
    fn direct_dialog_rejects_nuls_and_oversized_fields_before_the_abi_call() {
        let api = test_support::test_api();
        let valid = LocalDialog {
            id: 7,
            style: LocalDialogStyle::MessageBox,
            title: b"title",
            text: b"text",
            button1: b"ok",
            button2: b"",
        };
        assert_eq!(api.show_local_dialog(valid), RakSampResult::Ok);

        let nul = LocalDialog {
            title: b"bad\0title",
            ..valid
        };
        assert_eq!(api.show_local_dialog(nul), RakSampResult::InvalidArgument);

        let too_long = [b'x'; 256];
        let long_title = LocalDialog {
            title: &too_long,
            ..valid
        };
        assert_eq!(
            api.show_local_dialog(long_title),
            RakSampResult::InvalidArgument
        );
    }

    #[test]
    fn direct_chat_rejects_nuls_and_native_entry_overflows_before_the_abi_call() {
        let api = test_support::test_api();
        let valid = LocalChatMessage {
            style: LocalChatMessageStyle::Debug,
            text: b"local message",
            prefix: b"[rak-samp]",
            text_colour: 0xFF_A9_C4_E4,
            prefix_colour: u32::MAX,
        };
        assert_eq!(api.show_local_chat_message(valid), RakSampResult::Ok);
        assert_eq!(
            api.show_local_chat_message(LocalChatMessage {
                text: b"bad\0text",
                ..valid
            }),
            RakSampResult::InvalidArgument
        );
        let too_long_text = [b'x'; 144];
        assert_eq!(
            api.show_local_chat_message(LocalChatMessage {
                text: &too_long_text,
                ..valid
            }),
            RakSampResult::InvalidArgument
        );
        let too_long_prefix = [b'x'; 28];
        assert_eq!(
            api.show_local_chat_message(LocalChatMessage {
                prefix: &too_long_prefix,
                ..valid
            }),
            RakSampResult::InvalidArgument
        );
    }

    #[test]
    fn direct_death_window_rejects_nuls_and_native_name_overflows_before_the_abi_call() {
        let api = test_support::test_api();
        let valid = LocalDeathMessage {
            killer: b"killer",
            victim: b"victim",
            killer_colour: 0xFFFF_0000,
            victim_colour: 0xFF00_FF00,
            weapon: 24,
        };
        assert_eq!(api.show_local_death_message(valid), RakSampResult::Ok);
        assert_eq!(
            api.show_local_death_message(LocalDeathMessage {
                killer: b"bad\0killer",
                ..valid
            }),
            RakSampResult::InvalidArgument
        );
        let too_long = [b'x'; 25];
        assert_eq!(
            api.show_local_death_message(LocalDeathMessage {
                victim: &too_long,
                ..valid
            }),
            RakSampResult::InvalidArgument
        );
    }

    #[test]
    fn local_player_snapshot_is_owned_and_converted_from_the_abi_buffer() {
        let snapshot = test_support::test_api()
            .local_player()
            .expect("test host publishes a snapshot");
        assert_eq!(snapshot.id, 42);
        assert_eq!(snapshot.nickname, b"fixture");
        assert_eq!(snapshot.vehicle_id, Some(19));
        assert_eq!(
            snapshot.position,
            Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
    }

    #[test]
    fn server_info_snapshot_is_owned_and_converted_from_the_abi_buffer() {
        let info = test_support::test_api()
            .server_info()
            .expect("test host publishes server metadata");
        assert_eq!(info.address, b"127.0.0.1");
        assert_eq!(info.hostname, b"fixture");
        assert_eq!(info.port, 7777);
    }

    #[test]
    fn samp_game_state_is_returned_from_the_scalar_abi_output() {
        assert_eq!(test_support::test_api().samp_game_state(), Ok(14));
    }

    #[test]
    fn local_chat_display_mode_is_converted_from_the_scalar_abi_output() {
        let api = test_support::test_api();
        assert_eq!(
            api.local_chat_display_mode(),
            Ok(LocalChatDisplayMode::Normal)
        );
        assert_eq!(api.is_local_chat_visible(), Ok(true));
        assert_eq!(LocalChatDisplayMode::from_raw(3), None);
    }

    #[test]
    fn local_cursor_and_scoreboard_state_are_converted_from_scalar_abi_outputs() {
        let api = test_support::test_api();
        assert_eq!(api.local_cursor_mode(), Ok(LocalCursorMode::LockCamera));
        assert_eq!(api.is_local_cursor_active(), Ok(true));
        assert_eq!(api.is_local_scoreboard_open(), Ok(false));
        assert_eq!(LocalCursorMode::from_raw(5), None);
    }

    #[test]
    fn samp_version_is_converted_from_the_scalar_abi_output() {
        assert_eq!(
            test_support::test_api().samp_version(),
            Ok(RakSampClientVersion::R1)
        );
    }

    #[test]
    fn decode_string_returns_owned_bytes_and_advances_the_owned_stream() {
        let api = test_support::test_api();
        let mut stream = raknet::BitStream::from_bits(vec![0b1010_0000], 3)
            .expect("fixture bit stream is valid");

        assert_eq!(api.decode_string(&mut stream), Ok(b"fixture".to_vec()));
        assert_eq!(stream.read_offset_bits(), 3);

        let mut rejected = raknet::BitStream::from_bits(vec![0b0100_0000], 3)
            .expect("fixture bit stream is valid");
        rejected.set_read_offset(1).expect("cursor is valid");
        assert_eq!(
            api.decode_string(&mut rejected),
            Err(RakSampResult::InvalidArgument)
        );
        assert_eq!(rejected.read_offset_bits(), 1);
    }

    #[test]
    fn local_player_query_conveniences_reuse_the_safe_snapshot() {
        let api = test_support::test_api();
        assert_eq!(api.local_player_id(), Ok(42));
        assert_eq!(api.local_player_nickname(), Ok(b"fixture".to_vec()));
        assert_eq!(api.local_player_colour(), Ok(0xFF00_00FF));
        assert_eq!(api.is_local_player_spawned(), Ok(true));
        assert_eq!(api.local_player_health(), Ok(99.0));
        assert_eq!(api.local_player_armour(), Ok(50.0));
        assert_eq!(api.local_player_special_action(), Ok(3));
        assert_eq!(api.local_player_animation_id(), Ok(12));
        assert_eq!(api.local_player_score(), Ok(123));
        assert_eq!(api.local_player_ping(), Ok(45));
    }

    #[test]
    fn owned_bit_stream_send_helpers_preserve_exact_partial_bit_lengths() {
        let mut stream = raknet::BitStream::new();
        stream.write_bits(&[0b0000_0101], 3).unwrap();

        let api = test_support::test_api();
        assert_eq!(
            api.send_packet_stream(200, &stream, RakSampSendOptions::default()),
            RakSampResult::NativeCallFailed
        );
        assert_eq!(
            api.send_rpc_stream(62, &stream, RakSampSendOptions::default()),
            RakSampResult::NativeCallFailed
        );
    }

    #[test]
    fn send_chat_uses_the_typed_bounded_rpc_101_payload() {
        let api = test_support::test_api();
        assert_eq!(api.send_chat(b"hi"), RakSampResult::Ok);
        assert_eq!(api.send_chat(b"/hi"), RakSampResult::Ok);
        assert_eq!(api.send_chat(&[b'x'; 256]), RakSampResult::InvalidArgument);
        assert_eq!(api.send_request_spawn(), RakSampResult::Ok);
    }

    #[test]
    fn local_player_protocol_actions_preserve_their_wire_vectors() {
        let api = test_support::test_api();
        assert_eq!(api.send_request_class(9), RakSampResult::Ok);
        assert_eq!(api.send_interior_change(7), RakSampResult::Ok);
        assert_eq!(api.send_spawn(), RakSampResult::Ok);
        assert_eq!(api.send_enter_vehicle(0x1234, true), RakSampResult::Ok);
        assert_eq!(api.send_exit_vehicle(0x1234), RakSampResult::Ok);
    }

    #[test]
    fn typed_protocol_action_conveniences_preserve_their_wire_vectors() {
        let api = test_support::test_api();
        assert_eq!(
            api.send_dialog_response(0x1234, 1, 0x3456, b"ok"),
            RakSampResult::Ok
        );
        assert_eq!(api.send_click_player(0x1234, 2), RakSampResult::Ok);
        assert_eq!(api.send_click_textdraw(0x1234), RakSampResult::Ok);
        assert_eq!(api.send_death_by_player(0x1234, 9), RakSampResult::Ok);
        assert_eq!(api.send_menu_quit(), RakSampResult::Ok);
        assert_eq!(api.send_menu_select_row(7), RakSampResult::Ok);
        assert_eq!(api.send_picked_up_pickup(9), RakSampResult::Ok);
        assert_eq!(api.send_vehicle_destroyed(0x1234), RakSampResult::Ok);
        assert_eq!(
            api.send_dialog_response(0, 0, 0, &[b'x'; 256]),
            RakSampResult::InvalidArgument
        );
    }

    #[test]
    fn additional_typed_protocol_actions_preserve_their_wire_vectors() {
        let api = test_support::test_api();
        assert_eq!(
            api.send_vehicle_damage(0x1234, 1, 2, 3, 4),
            RakSampResult::Ok
        );
        assert_eq!(api.send_scm_event(4, 1, 2, 3), RakSampResult::Ok);
        assert_eq!(api.send_give_damage(0x1234, 1.0, 24, 9), RakSampResult::Ok);
        assert_eq!(api.send_take_damage(0x1234, 1.0, 24, 9), RakSampResult::Ok);

        let zero = events::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let attached = events::rpc::outgoing::EditAttachedObject {
            response: 0,
            index: 0,
            model_id: 0,
            bone: 0,
            position: zero,
            rotation: zero,
            scale: zero,
            color1: 0,
            color2: 0,
        };
        let attached_payload = events::rpc::outgoing::SEND_EDIT_ATTACHED_OBJECT
            .encode(api, attached)
            .expect("zero attached-object edit must encode");
        assert_eq!(attached_payload.len_bits(), 480);
        assert_eq!(attached_payload.as_bytes(), &[0; 60]);
        assert_eq!(api.send_edit_attached_object(attached), RakSampResult::Ok);
        assert_eq!(
            api.send_edit_object(events::rpc::outgoing::EditObject {
                player_object: false,
                object_id: 0,
                response: 0,
                position: zero,
                rotation: zero,
            }),
            RakSampResult::Ok
        );
        assert_eq!(api.send_rcon_command(b"rcon"), RakSampResult::Ok);
        assert_eq!(
            api.send_rcon_command(&[b'x'; events::MAX_STRING32_BYTES + 1]),
            RakSampResult::InvalidArgument
        );
    }

    #[test]
    fn typed_sync_send_conveniences_preserve_their_fixed_wire_vectors() {
        let api = test_support::test_api();
        let zero = events::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            api.send_aim_sync(events::packet::AimSync {
                camera_mode: 0,
                camera_front: zero,
                camera_position: zero,
                aim_z: 0.0,
                zoom_and_weapon_state: 0,
                aspect_ratio: 0,
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_bullet_sync(events::packet::BulletSync {
                target_type: 0,
                target_id: 0,
                origin: zero,
                target: zero,
                center: zero,
                weapon_id: 0,
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_vehicle_sync(events::packet::VehicleSync {
                vehicle_id: 0,
                left_right_keys: 0,
                up_down_keys: 0,
                key_data: 0,
                quaternion: [0.0; 4],
                position: zero,
                move_speed: zero,
                vehicle_health: 0.0,
                player_health: 0,
                armour: 0,
                weapon_and_special_key: 0,
                siren: 0,
                landing_gear_state: 0,
                trailer_id: 0,
                vehicle_specific: [0; 4],
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_player_sync(events::packet::PlayerSync {
                left_right_keys: 0,
                up_down_keys: 0,
                key_data: 0,
                position: zero,
                quaternion: [0.0; 4],
                health: 0,
                armour: 0,
                weapon_and_special_key: 0,
                special_action: 0,
                move_speed: zero,
                surfing_offsets: zero,
                surfing_vehicle_id: 0,
                animation_id: 0,
                animation_flags: 0,
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_spectator_sync(events::packet::SpectatorSync {
                left_right_keys: 0,
                up_down_keys: 0,
                key_data: 0,
                position: zero,
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_trailer_sync(events::packet::TrailerSync {
                trailer_id: 0,
                position: zero,
                quaternion: [0.0; 4],
                move_speed: zero,
                turn_speed: zero,
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_passenger_sync(events::packet::PassengerSync {
                vehicle_id: 0,
                seat_driveby_cuffed: 0,
                weapon_and_special_key: 0,
                health: 0,
                armour: 0,
                left_right_keys: 0,
                up_down_keys: 0,
                key_data: 0,
                position: zero,
            }),
            RakSampResult::Ok
        );
        assert_eq!(
            api.send_unoccupied_sync(events::packet::UnoccupiedSync {
                vehicle_id: 0,
                seat_id: 0,
                roll: zero,
                direction: zero,
                position: zero,
                move_speed: zero,
                turn_speed: zero,
                vehicle_health: 0.0,
            }),
            RakSampResult::Ok
        );
    }

    #[test]
    fn safe_rpc_registration_dispatches_and_synchronizes() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let subscription = test_support::test_api()
            .on_rpc(RakSampDirection::Incoming, move |event| {
                assert_eq!(event.id(), 42);
                observed.fetch_add(1, Ordering::AcqRel);
                RakSampHookAction::Block
            })
            .expect("test registration must succeed");

        assert_eq!(subscription.id(), 1);
        assert_eq!(
            test_support::invoke_registered_callback(42),
            Some(RakSampHookAction::Block)
        );
        assert_eq!(calls.load(Ordering::Acquire), 1);

        subscription
            .unregister_and_wait()
            .expect("test shutdown must synchronize");
        assert_eq!(test_support::invoke_registered_callback(42), None);
        assert_eq!(
            test_support::registration_stats().unregister_and_wait_calls,
            1
        );
    }

    #[test]
    fn safe_callback_panic_fails_open() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let subscription = test_support::test_api()
            .on_packet(RakSampDirection::Outgoing, |_| {
                panic!("test callback panic")
            })
            .expect("test registration must succeed");

        assert_eq!(
            test_support::invoke_registered_callback(10),
            Some(RakSampHookAction::Continue)
        );
        subscription
            .unregister_and_wait()
            .expect("test shutdown must synchronize");
    }

    #[test]
    fn id_filtered_callback_ignores_unrelated_events() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let subscription = test_support::test_api()
            .on_rpc_id(RakSampDirection::Incoming, 42, move |_| {
                observed.fetch_add(1, Ordering::AcqRel);
                RakSampHookAction::Block
            })
            .expect("test registration must succeed");

        assert_eq!(
            test_support::invoke_registered_callback(41),
            Some(RakSampHookAction::Continue)
        );
        assert_eq!(
            test_support::invoke_registered_callback(42),
            Some(RakSampHookAction::Block)
        );
        assert_eq!(calls.load(Ordering::Acquire), 1);

        subscription
            .unregister_and_wait()
            .expect("test shutdown must synchronize");
    }

    #[test]
    fn typed_callback_decodes_matching_descriptor_and_fails_open() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let api = test_support::test_api();
        let subscription = api
            .on_typed_rpc(
                RakSampDirection::Incoming,
                incoming::ENABLE_STUNT_BONUS,
                move |enabled| {
                    assert!(enabled);
                    observed.fetch_add(1, Ordering::AcqRel);
                    RpcAction::Block
                },
            )
            .expect("test registration must succeed");

        assert_eq!(
            test_support::invoke_registered_callback(99),
            Some(RakSampHookAction::Continue)
        );
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(
                incoming::ENABLE_STUNT_BONUS.id(),
                incoming::ENABLE_STUNT_BONUS
                    .encode(api, true)
                    .expect("the typed test payload must encode"),
            ),
            Some(RakSampHookAction::Block)
        );
        assert_eq!(
            test_support::invoke_registered_callback(incoming::ENABLE_STUNT_BONUS.id()),
            Some(RakSampHookAction::Continue)
        );
        assert_eq!(calls.load(Ordering::Acquire), 1);

        subscription
            .unregister_and_wait()
            .expect("test shutdown must synchronize");
    }

    #[test]
    fn register_handlers_collects_every_supported_handler_form() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();

        let subscriptions = register_handlers!(test_support::test_api();
            packet(RakSampDirection::Incoming, |_| RakSampHookAction::Continue),
            rpc(RakSampDirection::Outgoing, |_| RakSampHookAction::Continue),
            packet_id(RakSampDirection::Incoming, 1, |_| RakSampHookAction::Continue),
            rpc_id(RakSampDirection::Outgoing, 2, |_| RakSampHookAction::Continue),
            typed_packet(
                RakSampDirection::Incoming,
                packet::incoming::CONNECTION_ACCEPTED,
                |_| RpcAction::Continue
            ),
            typed_rpc(
                RakSampDirection::Outgoing,
                incoming::ENABLE_STUNT_BONUS,
                |_| RpcAction::Continue
            ),
        )
        .expect("all test registrations must succeed");

        assert_eq!(subscriptions.len(), 6);
        assert_eq!(
            test_support::registration_stats().registered_callbacks,
            subscriptions.len()
        );
        subscriptions
            .unregister_and_wait()
            .expect("test shutdown must synchronize every callback");
        assert_eq!(test_support::registration_stats().registered_callbacks, 0);
    }

    #[test]
    fn subscription_set_retains_each_failed_shutdown_for_retry() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let api = test_support::test_api();
        let mut subscriptions = SubscriptionSet::new();
        subscriptions.push(
            api.on_packet(RakSampDirection::Incoming, |_| RakSampHookAction::Continue)
                .expect("test registration must succeed"),
        );
        subscriptions.push(
            api.on_rpc(RakSampDirection::Outgoing, |_| RakSampHookAction::Continue)
                .expect("test registration must succeed"),
        );
        test_support::set_unregister_and_wait_result(RakSampResult::CallbackInProgress);

        let error = subscriptions
            .unregister_and_wait()
            .expect_err("failed callbacks must remain available for retry");
        assert_eq!(error.failures().len(), 2);
        assert!(
            error
                .failures()
                .iter()
                .all(|failure| failure.result() == RakSampResult::CallbackInProgress)
        );
        assert_eq!(test_support::registration_stats().registered_callbacks, 2);

        test_support::set_unregister_and_wait_result(RakSampResult::Ok);
        error
            .into_subscriptions()
            .unregister_and_wait()
            .expect("retry must synchronize every callback");
        let stats = test_support::registration_stats();
        assert_eq!(stats.unregister_and_wait_calls, 4);
        assert_eq!(stats.registered_callbacks, 0);
    }

    #[test]
    fn subscription_set_preserves_earlier_registrations_after_a_registration_failure() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let subscription = test_support::test_api()
            .on_packet(RakSampDirection::Incoming, |_| RakSampHookAction::Continue)
            .expect("test registration must succeed");

        let error = SubscriptionSet::new()
            .try_add(Ok(subscription))
            .and_then(|subscriptions| subscriptions.try_add(Err(RakSampResult::NotReady)))
            .expect_err("the synthetic second registration must fail");
        assert_eq!(error.result(), RakSampResult::NotReady);
        let subscriptions = error.into_subscriptions();
        assert_eq!(subscriptions.len(), 1);
        subscriptions
            .unregister_and_wait()
            .expect("retained subscription must remain cleanly removable");
    }

    #[test]
    fn failed_synchronized_shutdown_keeps_the_subscription_for_retry() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let subscription = test_support::test_api()
            .on_rpc(RakSampDirection::Incoming, |_| RakSampHookAction::Continue)
            .expect("test registration must succeed");
        test_support::set_unregister_and_wait_result(RakSampResult::CallbackInProgress);

        let error = subscription
            .unregister_and_wait()
            .expect_err("callback-thread shutdown must remain retryable");
        assert_eq!(error.result(), RakSampResult::CallbackInProgress);
        let subscription = error.into_subscription();
        test_support::set_unregister_and_wait_result(RakSampResult::Ok);
        subscription
            .unregister_and_wait()
            .expect("retry must synchronize");
    }

    #[test]
    fn failed_registration_releases_the_handler() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        test_support::set_register_result(RakSampResult::NotReady);
        let drops = Arc::new(AtomicUsize::new(0));
        let counter = DropCounter(Arc::clone(&drops));

        let result = test_support::test_api().on_packet(RakSampDirection::Incoming, move |_| {
            let _ = &counter;
            RakSampHookAction::Continue
        });
        assert_eq!(result.unwrap_err(), RakSampResult::NotReady);
        assert_eq!(drops.load(Ordering::Acquire), 1);
    }

    #[test]
    fn dropping_a_subscription_detaches_without_freeing_callback_state() {
        let _serial = REGISTRATION_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        test_support::reset_registration();
        let drops = Arc::new(AtomicUsize::new(0));
        let counter = DropCounter(Arc::clone(&drops));
        let subscription = test_support::test_api()
            .on_packet(RakSampDirection::Incoming, move |_| {
                let _ = &counter;
                RakSampHookAction::Continue
            })
            .expect("test registration must succeed");

        drop(subscription);
        assert_eq!(drops.load(Ordering::Acquire), 0);
        assert_eq!(test_support::invoke_registered_callback(1), None);
        assert_eq!(test_support::registration_stats().unregister_calls, 1);
    }
}
