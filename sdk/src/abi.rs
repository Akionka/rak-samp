//! Stable C ABI declarations and table layout.

use super::*;

pub const ABI_VERSION_V1: u32 = 1;
pub const DEFAULT_HOST_MODULE: &[u8] = b"samp_client_sdk.asi\0";

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

/// C-compatible storage for [`LocalPlayer`].
///
/// This is output-only. `nickname_len` selects the initialized prefix of
/// `nickname`; the buffer has no required terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkLocalPlayerV1 {
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

/// C-compatible storage for an active R1 dialog core snapshot.
///
/// `active` is zero when no dialog is active. When it is one, `title_len`
/// selects the initialized prefix of `title`; the buffer has no required
/// terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkActiveDialogV1 {
    pub active: u8,
    pub style: u8,
    pub server_side: u8,
    pub _reserved: u8,
    pub id: i32,
    pub title_len: u8,
    pub title: [u8; 65],
}

/// Fixed ABI storage for the cached R1 chat-input text. `len` selects the
/// initialized byte prefix; the buffer has no required terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkChatInputTextV1 {
    pub len: u8,
    pub bytes: [u8; 128],
}

/// Fixed ABI storage for one cached R1 dialog listbox item text.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkDialogListItemV1 {
    pub len: u8,
    pub bytes: [u8; MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES],
}

/// Fixed ABI storage for one coherent active-dialog cache publication.
///
/// `active` is zero when no dialog is active. Otherwise, the length fields
/// select initialized byte prefixes with no required terminators.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkDialogSnapshotV1 {
    pub active: u8,
    pub style: u8,
    pub server_side: u8,
    pub has_editbox: u8,
    pub id: i32,
    pub title_len: u8,
    pub editbox_text_len: u8,
    pub listbox_item_count: u8,
    pub _reserved: u8,
    pub text_len: u16,
    pub _reserved2: [u8; 2],
    pub title: [u8; 65],
    pub editbox_text: [u8; MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES],
    pub text: [u8; MAX_SAMP_DIALOG_TEXT_BYTES],
    pub listbox_items: [SampClientSdkDialogListItemV1; MAX_SAMP_DIALOG_LISTBOX_ITEMS],
}

/// C-compatible storage for an owned [`PlayerInfo`] result.
///
/// `exists` is zero for a cached disconnected ID and one for a copied entry.
/// The host always initializes the whole structure; `nickname_len` selects the
/// initialized prefix of `nickname` when `exists` is one.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkPlayerInfoV1 {
    pub exists: u8,
    pub is_local: u8,
    pub is_npc: u8,
    pub _reserved: u8,
    pub id: u16,
    pub nickname_len: u16,
    pub nickname: [u8; 256],
    pub colour: u32,
    pub score: i32,
    pub ping: u32,
}

/// C-compatible storage for an owned remote-player state result.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkRemotePlayerStateV1 {
    pub exists: u8,
    pub special_action: u8,
    pub _reserved: u16,
    pub id: u16,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
}

/// C-compatible storage for an owned [`Gangzone`] result.
///
/// `exists` is zero when the latest completed query found no gangzone. The
/// host initializes all fields in that case; when it is one, the scalar fields
/// hold an R1 game-thread copy.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkGangzoneV1 {
    pub exists: u8,
    pub _reserved: [u8; 3],
    pub id: u16,
    pub _reserved2: u16,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub colour: u32,
    pub alternate_colour: u32,
}

/// C-compatible storage for an owned [`TextLabel`] result.
///
/// `exists` is zero when the latest completed query found no label. When it
/// is one, `text_len` selects the initialized prefix of `text`; the buffer has
/// no required terminator. `0xFFFF` in either attachment field means `None`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkTextLabelV1 {
    pub exists: u8,
    pub behind_walls: u8,
    pub _reserved: [u8; 2],
    pub id: u16,
    pub attached_player_id: u16,
    pub attached_vehicle_id: u16,
    pub _reserved2: u16,
    pub colour: u32,
    pub position: Vector3,
    pub draw_distance: f32,
    pub text_len: u16,
    pub _reserved3: [u8; 2],
    pub text: [u8; MAX_SAMP_TEXT_LABEL_TEXT_BYTES],
}

/// C-compatible storage for an owned [`TextDraw`] result.
///
/// `exists` is zero when the latest completed query found no textdraw. When
/// it is one, all scalar fields are initialized from one R1 game-thread copy.
/// Flags use canonical zero or one values; colours retain their native R1
/// Direct3D representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkTextDrawV1 {
    pub exists: u8,
    pub proportional: u8,
    pub align_left: u8,
    pub align_center: u8,
    pub align_right: u8,
    pub box_enabled: u8,
    pub _reserved: [u8; 2],
    pub pool_index: u16,
    pub shadow: u8,
    pub outline: u8,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_colour: u32,
    pub x: f32,
    pub y: f32,
    pub background_colour: u32,
    pub style: i32,
    pub box_width: f32,
    pub box_height: f32,
    pub box_colour: u32,
    pub model_id: u16,
    pub _reserved2: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub model_colour1: u16,
    pub model_colour2: u16,
    pub text_len: u16,
    pub _reserved3: [u8; 2],
    pub text: [u8; MAX_SAMP_TEXTDRAW_STRING_BYTES],
}

/// C-compatible storage for an owned [`ChatEntry`] result.
///
/// `text_len` and `prefix_len` select initialized, non-NUL byte prefixes;
/// neither buffer requires a terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkChatEntryV1 {
    pub id: u16,
    pub text_len: u8,
    pub prefix_len: u8,
    pub text_colour: u32,
    pub prefix_colour: u32,
    pub text: [u8; MAX_SAMP_CHAT_ENTRY_TEXT_BYTES],
    pub prefix: [u8; MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES],
}

/// C-compatible storage for [`ServerInfo`].
///
/// This is output-only. Each length selects the initialized prefix of its
/// corresponding buffer; neither buffer requires a terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkServerInfoV1 {
    pub address_len: u16,
    pub hostname_len: u16,
    pub address: [u8; 257],
    pub hostname: [u8; 257],
    pub port: u16,
}

/// C-compatible storage for [`LocalAnimation`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkAnimationV1 {
    pub name_len: u8,
    pub file_len: u8,
    pub name: [u8; 36],
    pub file: [u8; 36],
}

macro_rules! impl_zeroed_abi_default {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Default for $type {
                fn default() -> Self {
                    // Every listed ABI storage type contains only fields for
                    // which an all-zero bit pattern is valid.
                    unsafe { core::mem::zeroed() }
                }
            }
        )+
    };
}

impl_zeroed_abi_default!(
    SampClientSdkChatInputTextV1,
    SampClientSdkDialogListItemV1,
    SampClientSdkDialogSnapshotV1,
    SampClientSdkActiveDialogV1,
    SampClientSdkLocalPlayerV1,
    SampClientSdkPlayerInfoV1,
    SampClientSdkChatEntryV1,
    SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1,
    SampClientSdkServerInfoV1,
    SampClientSdkAnimationV1,
);

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

/// The host-side ABI table exported by `samp_client_sdk.asi`.
///
/// Fields are currently appended to preserve the v1 layout; during the ALPHA
/// stage the ABI may make an explicit compatibility break. Check `size` before
/// accessing fields added by a newer ABI version. Normal plugins use
/// [`HostApi`] instead of calling this table directly.
#[repr(C)]
pub struct SampClientSdkApiV1 {
    pub abi_version: u32,
    pub size: u32,
    pub host_status: extern "system" fn() -> SampClientSdkHostStatus,
    pub register_packet: unsafe extern "system" fn(
        SampClientSdkDirection,
        Option<SampClientSdkEventCallbackV1>,
        *mut c_void,
        *mut SampClientSdkSubscription,
    ) -> SampClientSdkResult,
    pub register_rpc: unsafe extern "system" fn(
        SampClientSdkDirection,
        Option<SampClientSdkEventCallbackV1>,
        *mut c_void,
        *mut SampClientSdkSubscription,
    ) -> SampClientSdkResult,
    pub unregister: unsafe extern "system" fn(SampClientSdkSubscription) -> SampClientSdkResult,
    pub event_id: unsafe extern "system" fn(*const SampClientSdkEventV1) -> u8,
    pub event_reset_read:
        unsafe extern "system" fn(*mut SampClientSdkEventV1) -> SampClientSdkResult,
    pub event_clear: unsafe extern "system" fn(*mut SampClientSdkEventV1) -> SampClientSdkResult,
    pub event_read_u8:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u8) -> SampClientSdkResult,
    pub event_read_u16:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u16) -> SampClientSdkResult,
    pub event_read_u32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u32) -> SampClientSdkResult,
    pub event_read_f32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut f32) -> SampClientSdkResult,
    pub event_read_bytes:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u8, usize) -> SampClientSdkResult,
    pub event_write_u8:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, u8) -> SampClientSdkResult,
    pub event_write_u16:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, u16) -> SampClientSdkResult,
    pub event_write_u32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, u32) -> SampClientSdkResult,
    pub event_write_f32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, f32) -> SampClientSdkResult,
    pub event_write_bytes: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *const u8,
        usize,
    ) -> SampClientSdkResult,
    pub send_packet: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
    ) -> SampClientSdkResult,
    pub send_rpc: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
    ) -> SampClientSdkResult,
    /// Atomically replaces a byte-aligned callback payload. This field was appended to ABI v1.
    pub event_replace_bytes: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *const u8,
        usize,
    ) -> SampClientSdkResult,
    /// Removes a listener and waits for callbacks already running on other threads.
    pub unregister_and_wait:
        unsafe extern "system" fn(SampClientSdkSubscription) -> SampClientSdkResult,
    /// Queues a locally generated incoming packet. `data` excludes the packet ID.
    pub emulate_incoming_packet:
        unsafe extern "system" fn(u8, *const u8, usize, usize) -> SampClientSdkResult,
    /// Dispatches a locally generated incoming RPC. `data` excludes the RPC ID.
    pub emulate_incoming_rpc:
        unsafe extern "system" fn(u8, *const u8, usize, usize) -> SampClientSdkResult,
    /// Returns unread bits in a callback-local event. This field was appended to ABI v1.
    pub event_remaining_bits: unsafe extern "system" fn(*mut SampClientSdkEventV1) -> usize,
    /// Reads exact bits into a left-aligned byte buffer. This field was appended to ABI v1.
    pub event_read_bits:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u8, usize) -> SampClientSdkResult,
    /// Atomically replaces a callback payload with an exact bit length.
    pub event_replace_bits: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *const u8,
        usize,
        usize,
    ) -> SampClientSdkResult,
    /// Encodes one string with SA-MP's native RakNet compressor.
    pub encode_string: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut u8,
        usize,
        *mut usize,
    ) -> SampClientSdkResult,
    /// Decodes one string from a callback event and advances its read cursor.
    pub event_read_encoded_string: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *mut u8,
        usize,
        *mut usize,
    ) -> SampClientSdkResult,
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
    ) -> SampClientSdkResult,
    /// Copies the latest host-owned local-player snapshot into `output`.
    pub local_player:
        unsafe extern "system" fn(*mut SampClientSdkLocalPlayerV1) -> SampClientSdkResult,
    /// Copies the latest R1 `CNetGame` state scalar into `output`.
    pub samp_game_state: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Copies the detected SA-MP client version identity into `output`.
    pub samp_version: unsafe extern "system" fn(*mut u32) -> SampClientSdkResult,
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
    ) -> SampClientSdkResult,
    /// Copies the latest host-owned R1 current-server snapshot into `output`.
    pub server_info:
        unsafe extern "system" fn(*mut SampClientSdkServerInfoV1) -> SampClientSdkResult,
    /// Copies and queues a local R1 chat entry for the verified game-thread pump.
    pub show_local_chat_message: unsafe extern "system" fn(
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
    ) -> SampClientSdkResult,
    /// Copies and queues a local R1 death-window entry for the game-thread pump.
    pub show_local_death_message: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        u8,
    ) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 chat display mode into `output`.
    pub local_chat_display_mode: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 cursor mode into `output`.
    pub local_cursor_mode: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 scoreboard-open flag into `output`.
    pub local_scoreboard_open: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 dialog-active flag into `output`.
    pub local_dialog_active: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 chat-input-active flag into `output`.
    pub local_chat_input_active: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    /// Copies one entry from the cached R1 animation table into `output`.
    pub local_animation:
        unsafe extern "system" fn(u16, *mut SampClientSdkAnimationV1) -> SampClientSdkResult,
    /// Finds an R1 animation-table entry by copied name and file bytes.
    pub local_animation_id: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        *mut i32,
    ) -> SampClientSdkResult,
    /// Copies a cached local or demand-refreshed remote R1 player directory entry.
    pub player_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkPlayerInfoV1) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 player-pool count into `output`.
    pub player_count: unsafe extern "system" fn(u8, *mut u16) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 non-streamed player maximum ID into `output`.
    pub player_max_id: unsafe extern "system" fn(*mut u16) -> SampClientSdkResult,
    /// Copies a cached R1 vehicle-pool existence flag into `output`.
    pub vehicle_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached active R1 dialog core into `output`.
    pub active_local_dialog:
        unsafe extern "system" fn(*mut SampClientSdkActiveDialogV1) -> SampClientSdkResult,
    /// Copies a cached R1 3D text-label-pool existence flag into `output`.
    pub text_label_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 textdraw-pool existence flag into `output`.
    pub textdraw_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 object-pool existence flag into `output`.
    pub object_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 gangzone record into `output`.
    pub gangzone_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkGangzoneV1) -> SampClientSdkResult,
    /// Copies a cached R1 3D text-label record into `output`.
    pub text_label_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkTextLabelV1) -> SampClientSdkResult,
    /// Copies a cached R1 numeric textdraw record into `output`.
    pub textdraw_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkTextDrawV1) -> SampClientSdkResult,
    /// Copies a cached R1 player-world-defined flag into `output`.
    pub player_defined: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 player-paused flag into `output`.
    pub player_paused: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 remote-player volatile state record into `output`.
    pub remote_player_state: unsafe extern "system" fn(
        u16,
        *mut SampClientSdkRemotePlayerStateV1,
    ) -> SampClientSdkResult,
    /// Copies and submits a local R1 dialog request, returning a completion receipt.
    pub submit_local_dialog: unsafe extern "system" fn(
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
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and submits a local R1 chat entry, returning a completion receipt.
    pub submit_local_chat_message: unsafe extern "system" fn(
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and submits a local R1 death-window entry, returning a completion receipt.
    pub submit_local_death_message: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        u8,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Consumes an available completion, or returns `CommandPending` without consuming it.
    pub command_try_take: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        *mut SampClientSdkCommandResultV1,
    ) -> SampClientSdkResult,
    /// Waits for and consumes a completion. A timeout leaves the receipt valid for retry.
    pub command_wait: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        u32,
        *mut SampClientSdkCommandResultV1,
    ) -> SampClientSdkResult,
    /// Detaches a pending receipt without cancelling its owned command.
    pub command_release:
        unsafe extern "system" fn(SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies and queues a server-bound packet, returning its game-thread completion receipt.
    pub submit_packet: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a server-bound RPC, returning its game-thread completion receipt.
    pub submit_rpc: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a locally generated incoming packet, returning its completion receipt.
    pub submit_emulate_incoming_packet: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a locally generated incoming RPC, returning its completion receipt.
    pub submit_emulate_incoming_rpc: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies the host-captured RakClient address into `output` as an opaque pointer.
    pub raw_rakclient: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Copies the latest game-thread-captured player-pool address into `output`.
    pub raw_player_pool: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Copies the latest game-thread-captured vehicle-pool address into `output`.
    pub raw_vehicle_pool: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Queues one validated R1 cursor-mode write and returns its completion receipt.
    pub submit_local_cursor_mode:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 scoreboard-enabled write and returns its completion receipt.
    pub submit_local_scoreboard_open:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 dialog client-side write and returns its completion receipt.
    pub submit_local_dialog_client_side:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one validated R1 CNetGame-state write and returns its completion receipt.
    pub submit_samp_game_state:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies the latest game-thread-captured local-player address into `output`.
    pub raw_local_player: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Queues the R1 local-player spawn path and returns its completion receipt.
    pub submit_local_player_spawn:
        unsafe extern "system" fn(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one established R1 local-player special action and returns its completion receipt.
    pub submit_local_player_special_action:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 replication send-rate write and returns its completion receipt.
    pub submit_send_rate:
        unsafe extern "system" fn(u8, u32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues the R1 cursor toggle transition and returns its completion receipt.
    pub submit_local_cursor_toggle:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 chat display-mode write and returns its completion receipt.
    pub submit_local_chat_display_mode:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies the validated R1 RakPeer base into `output` as an opaque pointer.
    pub raw_rakpeer: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Queues one R1 dialog close with the selected response button.
    pub submit_local_dialog_close:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies and queues a R1 chat-input text update.
    pub submit_local_chat_input_text: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues the native R1 chat-input open or close transition.
    pub submit_local_chat_input_enabled:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies text and queues R1 chat-input command processing.
    pub submit_local_chat_input_process: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies the game-thread-cached R1 chat-input text into `output`.
    pub local_chat_input_text:
        unsafe extern "system" fn(*mut SampClientSdkChatInputTextV1) -> SampClientSdkResult,
    /// Queues a documented R1 local- or remote-player colour change.
    pub submit_player_colour: unsafe extern "system" fn(
        u16,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a documented R1 local-player nickname update.
    pub submit_local_player_name: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues one documented R1 unoccupied-vehicle synchronization send.
    pub submit_force_unoccupied_sync: unsafe extern "system" fn(
        u16,
        i32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues the documented R1 reconnect sequence.
    pub submit_connect_to_server: unsafe extern "system" fn(
        *const u8,
        usize,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues the documented R1 RakClient disconnect and restart sequence.
    pub submit_disconnect_with_reason:
        unsafe extern "system" fn(u32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a documented R1 textdraw-pool deletion.
    pub submit_delete_textdraw:
        unsafe extern "system" fn(u16, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a finite R1 textdraw screen-position update.
    pub submit_set_textdraw_position: unsafe extern "system" fn(
        u16,
        f32,
        f32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues finite R1 textdraw letter dimensions and a native colour value.
    pub submit_set_textdraw_letter_style: unsafe extern "system" fn(
        u16,
        f32,
        f32,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues an R1 textdraw proportional-flag update.
    pub submit_set_textdraw_proportional:
        unsafe extern "system" fn(u16, u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues an R1 textdraw shadow and background-colour update.
    pub submit_set_textdraw_shadow: unsafe extern "system" fn(
        u16,
        u8,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues an R1 textdraw outline and background-colour update.
    pub submit_set_textdraw_outline: unsafe extern "system" fn(
        u16,
        u8,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues a finite R1 textdraw box update.
    pub submit_set_textdraw_box: unsafe extern "system" fn(
        u16,
        u8,
        u32,
        f32,
        f32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues a validated R1 textdraw alignment update.
    pub submit_set_textdraw_alignment:
        unsafe extern "system" fn(u16, u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a bounded R1 textdraw display-string update.
    pub submit_set_textdraw_string: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies the game-thread-cached R1 dialog list selection.
    pub local_dialog_selected_item: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Queues an R1 dialog list-selection write.
    pub submit_local_dialog_selected_item:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a documented R1 3D text-label-pool deletion.
    pub submit_delete_text_label:
        unsafe extern "system" fn(u16, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies the game-thread-cached count of items in the active R1 dialog list.
    pub local_dialog_list_item_count: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Queues a finite R1 textdraw model rotation, zoom, and vehicle-colour update.
    pub submit_set_textdraw_model_style: unsafe extern "system" fn(
        u16,
        f32,
        f32,
        f32,
        f32,
        u16,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues one bounded R1 chat-history entry replacement.
    pub submit_local_chat_entry: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies one cached fixed R1 chat-history entry into `output`.
    pub chat_entry_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkChatEntryV1) -> SampClientSdkResult,
    /// Queues a documented R1 3D text-label-pool creation at a caller-selected ID.
    pub submit_create_text_label: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        u32,
        Vector3,
        f32,
        u8,
        u16,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies one coherent game-thread-cached R1 dialog snapshot.
    pub local_dialog_snapshot:
        unsafe extern "system" fn(*mut SampClientSdkDialogSnapshotV1) -> SampClientSdkResult,
    /// Queues a bounded R1 dialog editbox text write.
    pub submit_local_dialog_editbox_text: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies one cached R1 object GTAREF for an object-pool ID.
    pub local_object_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 object-pool ID from its GTAREF.
    pub local_object_id_by_handle: unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Copies one cached R1 pickup GTAREF for a pickup-pool ID.
    pub local_pickup_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 pickup-pool ID from its GTAREF.
    pub local_pickup_id_by_handle: unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Copies one cached R1 vehicle GTA handle for a vehicle-pool ID.
    pub local_vehicle_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 vehicle-pool ID from its GTA handle.
    pub local_vehicle_id_by_handle: unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Copies one cached R1 player GTA ped handle for a player-pool ID.
    pub local_player_ped_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 player-pool ID from its GTA ped handle.
    pub local_player_id_by_ped_handle:
        unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Queues one bounded native R1 chat-command registration.
    pub submit_register_chat_command: unsafe extern "system" fn(
        *const u8,
        usize,
        Option<SampClientSdkChatCommandCallbackV1>,
        *mut c_void,
        *mut SampClientSdkSubscription,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Reports whether an exact bounded name is present in the game-thread-cached R1 command table.
    pub local_chat_command_defined:
        unsafe extern "system" fn(*const u8, usize, *mut u8) -> SampClientSdkResult,
    /// Queues R1 3D text-label creation at the first native free pool slot.
    pub submit_create_text_label_auto: unsafe extern "system" fn(
        *const u8,
        usize,
        u32,
        Vector3,
        f32,
        u8,
        u16,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Polls a text-label creation receipt and copies its typed completion.
    pub text_label_create_try_take: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        *mut SampClientSdkTextLabelCreateResultV1,
    ) -> SampClientSdkResult,
    /// Waits for a text-label creation receipt and copies its typed completion.
    pub text_label_create_wait: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        u32,
        *mut SampClientSdkTextLabelCreateResultV1,
    ) -> SampClientSdkResult,
}

pub type SampClientSdkGetApiV1 = unsafe extern "system" fn(u32) -> *const SampClientSdkApiV1;
