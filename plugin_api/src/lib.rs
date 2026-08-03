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
        assert!(test_support::test_api().is_samp_available());
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
            mem::size_of::<RakSampApiV1>(),
            mem::offset_of!(RakSampApiV1, samp_version) + function_size
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
    fn samp_game_state_is_returned_from_the_scalar_abi_output() {
        assert_eq!(test_support::test_api().samp_game_state(), Ok(14));
    }

    #[test]
    fn samp_version_is_converted_from_the_scalar_abi_output() {
        assert_eq!(
            test_support::test_api().samp_version(),
            Ok(RakSampClientVersion::R1)
        );
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
