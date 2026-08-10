//! Stable C ABI definitions and safe host-discovery helpers for `samp-client-sdk` plugins.
//!
//! Depend on this crate from an independently loaded ASI plugin. Do **not**
//! depend on the `samp_client_sdk` host crate: that would embed a second hook engine in
//! the process instead of communicating with `samp_client_sdk.asi`. Register callbacks with
//! [`Samp::net`] to register callbacks or send owned traffic. Use the ID-filtered and typed
//! variants when one handler owns one protocol message, and [`register_handlers!`] to keep a
//! group in one [`SubscriptionSet`]. Synchronize subscriptions before unloading the plugin.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk supports only 32-bit Windows x86 targets");

pub mod events;
mod facade;
mod host_api;
pub mod limits;
pub mod raknet;
pub mod raw;

pub use facade::*;
use limits::{
    MAX_RAKNET_DECODED_STRING_BYTES, MAX_SAMP_CHAT_ENTRIES, MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES,
    MAX_SAMP_CHAT_ENTRY_TEXT_BYTES, MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES,
    MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES, MAX_SAMP_DIALOG_LISTBOX_ITEMS, MAX_SAMP_DIALOG_TEXT_BYTES,
    MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABEL_TEXT_BYTES,
    MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAW_STRING_BYTES, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES,
};

use core::{ffi::c_void, fmt, mem, ptr::NonNull};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    time::{Duration, Instant},
};

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
    /// Converts the six native R1 dialog-style values to a typed style.
    pub const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::MessageBox),
            1 => Some(Self::Input),
            2 => Some(Self::List),
            3 => Some(Self::Password),
            4 => Some(Self::TabList),
            5 => Some(Self::HeadersList),
            _ => None,
        }
    }

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

/// Owned, copied state of the active R1 dialog. All text is a game-thread
/// snapshot; no native pointer crosses the plugin boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDialogState {
    pub id: i32,
    pub style: LocalDialogStyle,
    pub title: Vec<u8>,
    pub server_side: bool,
    /// Owned copy of the active dialog body text.
    pub text: Vec<u8>,
    /// Owned copy of the active dialog editbox text, when the dialog has one.
    pub editbox_text: Option<Vec<u8>>,
    /// Owned copies of the active dialog listbox item strings.
    pub items: Vec<Vec<u8>>,
}

impl LocalDialogState {
    /// Returns the copied active dialog ID.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Returns the copied active dialog style.
    #[must_use]
    pub const fn style(&self) -> LocalDialogStyle {
        self.style
    }

    /// Returns the copied active dialog caption bytes.
    #[must_use]
    pub fn caption(&self) -> &[u8] {
        &self.title
    }

    /// Returns whether the copied active dialog is local rather than server-side.
    #[must_use]
    pub const fn is_client_side(&self) -> bool {
        !self.server_side
    }

    /// Returns the copied active dialog body text bytes.
    #[must_use]
    pub fn text(&self) -> &[u8] {
        &self.text
    }

    /// Returns the copied active dialog editbox text bytes, when present.
    #[must_use]
    pub fn editbox_text(&self) -> Option<&[u8]> {
        self.editbox_text.as_deref()
    }

    /// Returns the copied active dialog listbox item strings.
    #[must_use]
    pub fn items(&self) -> &[Vec<u8>] {
        &self.items
    }
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

    #[must_use]
    pub const fn raw(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::NoShadow => 1,
            Self::Normal => 2,
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
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::None => 0,
            Self::LockKeysNoCursor => 1,
            Self::LockCameraAndControl => 2,
            Self::LockCamera => 3,
            Self::LockCameraNoCursor => 4,
        }
    }

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

impl LocalPlayer {
    /// Returns the copied local SA-MP player ID.
    #[must_use]
    pub const fn id(&self) -> u16 {
        self.id
    }

    /// Returns the copied local nickname bytes without assuming an encoding.
    #[must_use]
    pub fn nickname(&self) -> &[u8] {
        &self.nickname
    }

    /// Returns the copied local ARGB colour.
    #[must_use]
    pub const fn colour(&self) -> u32 {
        self.colour
    }

    /// Returns whether the copied local-player snapshot is spawned.
    #[must_use]
    pub const fn is_spawned(&self) -> bool {
        self.spawned
    }
}

/// An owned player-directory entry copied from the verified R1 game thread.
///
/// [`HostApi::player_info`] returns a local entry immediately once the local
/// snapshot exists. Remote IDs are demand-refreshed through the host's
/// game-thread pump, so the first query can return [`SampClientSdkResult::NotReady`]
/// while the copy is pending. No client, ped, or GTA handle crosses this API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerInfo {
    pub id: u16,
    pub nickname: Vec<u8>,
    pub is_local: bool,
    pub is_npc: bool,
    pub colour: u32,
    pub score: i32,
    pub ping: u32,
}

/// Volatile read-only state copied from one defined remote R1 player.
///
/// Remote state is demand-refreshed by the verified game-thread pump. The
/// first lookup can return [`SampClientSdkResult::NotReady`]; no client or GTA
/// pointer crosses the ABI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemotePlayerState {
    pub id: u16,
    pub health: f32,
    pub armour: f32,
    pub special_action: u8,
    pub animation_id: u16,
}

/// An owned R1 gangzone record copied from the game-thread cache.
///
/// The four coordinates retain the native pool's left, bottom, right, top
/// order. Both colours are the native Direct3D ARGB values used by the R1
/// draw path; no gangzone or GTA pointer crosses this API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gangzone {
    pub id: u16,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub colour: u32,
    pub alternate_colour: u32,
}

/// An owned R1 3D text-label record copied from the game-thread cache.
///
/// Text remains bytes because SA-MP does not guarantee a Unicode encoding.
/// `attached_player_id` and `attached_vehicle_id` are `None` for R1's native
/// `0xFFFF` sentinel. No label, pool, player, vehicle, or GTA pointer crosses
/// this API.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLabel {
    pub id: u16,
    pub text: Vec<u8>,
    pub colour: u32,
    pub position: Vector3,
    pub draw_distance: f32,
    pub behind_walls: bool,
    pub attached_player_id: Option<u16>,
    pub attached_vehicle_id: Option<u16>,
}

/// An owned R1 textdraw numeric record copied from the game-thread cache.
///
/// `pool_index` uses R1's raw order of 2,048 global slots followed by 256
/// local slots. Colours retain the native Direct3D values from the R1 draw
/// record. The display-string storage is intentionally not included until its
/// separate native lifecycle and semantic role are independently proven.
#[derive(Clone, Debug, PartialEq)]
pub struct TextDraw {
    pub pool_index: u16,
    pub text: Vec<u8>,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_colour: u32,
    pub x: f32,
    pub y: f32,
    pub shadow: u8,
    pub outline: u8,
    pub background_colour: u32,
    pub style: i32,
    pub proportional: bool,
    pub align_left: bool,
    pub align_center: bool,
    pub align_right: bool,
    pub box_enabled: bool,
    pub box_width: f32,
    pub box_height: f32,
    pub box_colour: u32,
    pub model_id: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub model_colour1: u16,
    pub model_colour2: u16,
}

/// An owned R1 fixed chat-history entry copied from the game-thread cache.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatEntry {
    pub id: u16,
    pub text: Vec<u8>,
    pub prefix: Vec<u8>,
    pub text_colour: u32,
    pub prefix_colour: u32,
}

/// Compatibility spelling for a copied R1 textdraw record.
pub type Textdraw = TextDraw;

impl TextDraw {
    /// Returns copied width, height, and ARGB letter colour.
    #[must_use]
    pub const fn letter_style(&self) -> (f32, f32, u32) {
        (self.letter_width, self.letter_height, self.letter_colour)
    }

    /// Returns copied X/Y screen coordinates.
    #[must_use]
    pub const fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// Returns the copied native shadow value.
    #[must_use]
    pub const fn shadow(&self) -> u8 {
        self.shadow
    }

    /// Returns the copied native outline value.
    #[must_use]
    pub const fn outline(&self) -> u8 {
        self.outline
    }

    /// Returns the copied native textdraw style.
    #[must_use]
    pub const fn style(&self) -> i32 {
        self.style
    }

    /// Returns whether the copied textdraw uses proportional spacing.
    #[must_use]
    pub const fn is_proportional(&self) -> bool {
        self.proportional
    }

    /// Returns copied left, centre, and right alignment flags.
    #[must_use]
    pub const fn alignment(&self) -> (bool, bool, bool) {
        (self.align_left, self.align_center, self.align_right)
    }

    /// Returns copied box enabled state, dimensions, and ARGB colour.
    #[must_use]
    pub const fn box_style(&self) -> (bool, f32, f32, u32) {
        (
            self.box_enabled,
            self.box_width,
            self.box_height,
            self.box_colour,
        )
    }

    /// Returns copied model ID, rotation, zoom, and model colours.
    #[must_use]
    pub const fn model_style(&self) -> (u16, Vector3, f32, u16, u16) {
        (
            self.model_id,
            self.rotation,
            self.zoom,
            self.model_colour1,
            self.model_colour2,
        )
    }
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

/// An owned R1 animation-table entry.
///
/// The bytes before the `:` separator are `name`; the bytes after it are
/// `file`. They remain bytes because the client does not guarantee Unicode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalAnimation {
    pub name: Vec<u8>,
    pub file: Vec<u8>,
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

impl Default for SampClientSdkChatInputTextV1 {
    fn default() -> Self {
        Self {
            len: 0,
            bytes: [0; 128],
        }
    }
}

/// Fixed ABI storage for one cached R1 dialog listbox item text.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkDialogListItemV1 {
    pub len: u8,
    pub bytes: [u8; MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES],
}

impl Default for SampClientSdkDialogListItemV1 {
    fn default() -> Self {
        Self {
            len: 0,
            bytes: [0; MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES],
        }
    }
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

impl Default for SampClientSdkDialogSnapshotV1 {
    fn default() -> Self {
        Self {
            active: 0,
            style: 0,
            server_side: 0,
            has_editbox: 0,
            id: 0,
            title_len: 0,
            editbox_text_len: 0,
            listbox_item_count: 0,
            _reserved: 0,
            text_len: 0,
            _reserved2: [0; 2],
            title: [0; 65],
            editbox_text: [0; MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES],
            text: [0; MAX_SAMP_DIALOG_TEXT_BYTES],
            listbox_items: [SampClientSdkDialogListItemV1::default();
                MAX_SAMP_DIALOG_LISTBOX_ITEMS],
        }
    }
}

impl Default for SampClientSdkActiveDialogV1 {
    fn default() -> Self {
        Self {
            active: 0,
            style: 0,
            server_side: 0,
            _reserved: 0,
            id: 0,
            title_len: 0,
            title: [0; 65],
        }
    }
}

impl Default for SampClientSdkLocalPlayerV1 {
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

impl Default for SampClientSdkPlayerInfoV1 {
    fn default() -> Self {
        Self {
            exists: 0,
            is_local: 0,
            is_npc: 0,
            _reserved: 0,
            id: 0,
            nickname_len: 0,
            nickname: [0; 256],
            colour: 0,
            score: 0,
            ping: 0,
        }
    }
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

impl Default for SampClientSdkChatEntryV1 {
    fn default() -> Self {
        Self {
            id: 0,
            text_len: 0,
            prefix_len: 0,
            text_colour: 0,
            prefix_colour: 0,
            text: [0; MAX_SAMP_CHAT_ENTRY_TEXT_BYTES],
            prefix: [0; MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES],
        }
    }
}

impl Default for SampClientSdkTextDrawV1 {
    fn default() -> Self {
        Self {
            exists: 0,
            proportional: 0,
            align_left: 0,
            align_center: 0,
            align_right: 0,
            box_enabled: 0,
            _reserved: [0; 2],
            pool_index: 0,
            shadow: 0,
            outline: 0,
            letter_width: 0.0,
            letter_height: 0.0,
            letter_colour: 0,
            x: 0.0,
            y: 0.0,
            background_colour: 0,
            style: 0,
            box_width: 0.0,
            box_height: 0.0,
            box_colour: 0,
            model_id: 0,
            _reserved2: 0,
            rotation: Vector3::default(),
            zoom: 0.0,
            model_colour1: 0,
            model_colour2: 0,
            text_len: 0,
            _reserved3: [0; 2],
            text: [0; MAX_SAMP_TEXTDRAW_STRING_BYTES],
        }
    }
}

impl Default for SampClientSdkTextLabelV1 {
    fn default() -> Self {
        Self {
            exists: 0,
            behind_walls: 0,
            _reserved: [0; 2],
            id: 0,
            attached_player_id: 0,
            attached_vehicle_id: 0,
            _reserved2: 0,
            colour: 0,
            position: Vector3::default(),
            draw_distance: 0.0,
            text_len: 0,
            _reserved3: [0; 2],
            text: [0; MAX_SAMP_TEXT_LABEL_TEXT_BYTES],
        }
    }
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

impl Default for SampClientSdkServerInfoV1 {
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

/// C-compatible storage for [`LocalAnimation`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkAnimationV1 {
    pub name_len: u8,
    pub file_len: u8,
    pub name: [u8; 36],
    pub file: [u8; 36],
}

impl Default for SampClientSdkAnimationV1 {
    fn default() -> Self {
        Self {
            name_len: 0,
            file_len: 0,
            name: [0; 36],
            file: [0; 36],
        }
    }
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

/// The established R1 `CNetGame` state values accepted by
/// [`crate::Samp::set_game_state`].
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampGameState {
    None = 0,
    WaitingForConnection = 9,
    Connecting = 13,
    Connected = 14,
    AwaitingJoin = 15,
    Restarting = 18,
}

/// The R1 replication stream whose send interval is configured by
/// [`crate::Net::set_send_rate`].
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendRateKind {
    OnFoot = 0,
    InVehicle = 1,
    Aim = 2,
}

impl SendRateKind {
    #[must_use]
    pub const fn raw(self) -> u8 {
        self as u8
    }
}

/// The established R1 special actions accepted by
/// [`crate::Local::set_special_action`].
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecialAction {
    None = 0,
    Duck = 1,
    Jetpack = 2,
    EnterVehicle = 3,
    ExitVehicle = 4,
    Dance1 = 5,
    Dance2 = 6,
    Dance3 = 7,
    Dance4 = 8,
    HandsUp = 9,
    UseCellphone = 10,
    Sitting = 11,
    StopUseCellphone = 12,
    DrinkBeer = 20,
    SmokeCigarette = 21,
    DrinkWine = 22,
    DrinkSprunk = 23,
    Cuffed = 24,
    Carry = 25,
    Urinating = 68,
}

impl SpecialAction {
    #[must_use]
    pub const fn raw(self) -> u8 {
        self as u8
    }
}

impl SampGameState {
    /// Returns the R1 scalar value written to `CNetGame`.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self as i32
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
    bytes: Vec<u8>,
    bit_len: usize,
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
}

pub type SampClientSdkGetApiV1 = unsafe extern "system" fn(u32) -> *const SampClientSdkApiV1;

type EventHandler = dyn for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
    + Send
    + Sync
    + 'static;

struct CallbackState {
    api: HostApi,
    handler: Box<EventHandler>,
}

type RegisterListener = unsafe extern "system" fn(
    SampClientSdkDirection,
    Option<SampClientSdkEventCallbackV1>,
    *mut c_void,
    *mut SampClientSdkSubscription,
) -> SampClientSdkResult;

/// A validated reference to the host API table.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct HostApi {
    raw: &'static SampClientSdkApiV1,
}

/// One owned completion receipt for a game-thread command.
///
/// Polling and waiting consume the receipt once the command completes. Dropping
/// a pending receipt releases only the waiter; the host still owns and executes
/// the copied command on a later game tick.
pub struct CommandReceipt<T> {
    api: HostApi,
    raw: SampClientSdkCommandReceipt,
    decode: fn(SampClientSdkCommandResultV1) -> Result<T, SampClientSdkResult>,
    active: bool,
}

impl<T> CommandReceipt<T> {
    fn new(
        api: HostApi,
        raw: SampClientSdkCommandReceipt,
        decode: fn(SampClientSdkCommandResultV1) -> Result<T, SampClientSdkResult>,
    ) -> Self {
        Self {
            api,
            raw,
            decode,
            active: true,
        }
    }

    /// Returns the host-owned opaque command identity.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.raw.id
    }

    /// Consumes and returns a ready completion, or returns `Ok(None)` while it
    /// remains pending. Completion failures are returned as SDK result codes.
    pub fn try_take(&mut self) -> Result<Option<T>, SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut output = SampClientSdkCommandResultV1::default();
        match unsafe { (self.api.raw.command_try_take)(self.raw, &mut output) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                (self.decode)(output).map(Some)
            }
            SampClientSdkResult::CommandPending => Ok(None),
            error => Err(error),
        }
    }

    /// Waits for and consumes the completion.
    ///
    /// `TimedOut` leaves this receipt usable for another poll or wait. The host
    /// rejects waits from a listener callback and, once enabled, the game thread.
    pub fn wait(&mut self, timeout: Duration) -> Result<T, SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
        let mut output = SampClientSdkCommandResultV1::default();
        match unsafe { (self.api.raw.command_wait)(self.raw, timeout_ms, &mut output) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                (self.decode)(output)
            }
            error => Err(error),
        }
    }

    /// Detaches this waiter without cancelling the copied native command.
    pub fn release(mut self) -> Result<(), SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        match unsafe { (self.api.raw.command_release)(self.raw) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                Ok(())
            }
            error => Err(error),
        }
    }
}

impl<T> Drop for CommandReceipt<T> {
    fn drop(&mut self) {
        if self.active {
            let _ = unsafe { (self.api.raw.command_release)(self.raw) };
            self.active = false;
        }
    }
}

impl HostApi {
    fn command_receipt(
        self,
        result: SampClientSdkResult,
        receipt: SampClientSdkCommandReceipt,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        match result {
            SampClientSdkResult::Ok if receipt.id != 0 => {
                Ok(CommandReceipt::new(self, receipt, unit_command_result))
            }
            SampClientSdkResult::Ok => Err(SampClientSdkResult::NativeCallFailed),
            error => Err(error),
        }
    }
}

fn unit_command_result(result: SampClientSdkCommandResultV1) -> Result<(), SampClientSdkResult> {
    match result.status {
        SampClientSdkResult::Ok => Ok(()),
        error => Err(error),
    }
}

/// An owned packet or RPC callback registration.
///
/// Call [`Self::unregister_and_wait`] from a worker thread before unloading the plugin ASI.
/// Dropping this value attempts a nonblocking listener removal and intentionally retains the
/// callback allocation, so it is memory-safe but does not prepare a plugin for `FreeLibrary`.
#[must_use = "a subscription must be synchronized before unloading the plugin ASI"]
pub struct Subscription {
    api: HostApi,
    raw: SampClientSdkSubscription,
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
            SampClientSdkResult::Ok | SampClientSdkResult::SubscriptionNotFound
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
    result: SampClientSdkResult,
    subscription: Subscription,
}

impl SubscriptionShutdownError {
    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(&self) -> SampClientSdkResult {
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
        registration: Result<Subscription, SampClientSdkResult>,
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
    result: SampClientSdkResult,
    subscriptions: SubscriptionSet,
}

impl SubscriptionRegistrationError {
    /// Returns the host result from the failed registration.
    #[must_use]
    pub const fn result(&self) -> SampClientSdkResult {
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
    result: SampClientSdkResult,
}

impl SubscriptionShutdownFailure {
    /// Returns the host-assigned subscription identifier.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.id
    }

    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(self) -> SampClientSdkResult {
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
    pub(crate) unsafe fn from_raw(raw: *const SampClientSdkApiV1) -> Result<Self, ResolveError> {
        let raw = NonNull::new(raw.cast_mut()).ok_or(ResolveError::MissingApi)?;
        let raw = unsafe { raw.as_ref() };
        if raw.abi_version != ABI_VERSION_V1
            || raw.size < mem::size_of::<SampClientSdkApiV1>() as u32
        {
            return Err(ResolveError::UnsupportedAbi);
        }
        Ok(Self { raw })
    }

    #[must_use]
    pub(crate) fn raw(self) -> &'static SampClientSdkApiV1 {
        self.raw
    }

    /// Sends one server-bound SA-MP chat message (RPC 101).
    ///
    /// This is the safe equivalent of SF.lua's `sampSendChat`. The message is
    /// serialized as the protocol's bounded `string8` payload; a
    /// slash-prefixed value instead uses the command RPC (50), matching the
    /// native helper. It is real network traffic, not a local chat display
    /// action.
    pub fn send_chat(self, text: &[u8]) -> SampClientSdkResult {
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
    pub fn send_request_spawn(self) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_REQUEST_SPAWN, ())
    }

    /// Sends SA-MP's request-class RPC (128).
    ///
    /// This carries the same server-bound protocol value as SF.lua's
    /// `sampRequestClass`, but does not invoke the native local-player method
    /// or update any local class-selection state.
    pub fn send_request_class(self, class_id: i32) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_REQUEST_CLASS, class_id)
    }

    /// Sends SA-MP's interior-change RPC (118).
    ///
    /// This is protocol-only. It does not change the GTA interior or mutate
    /// SA-MP's native local-player state.
    pub fn send_interior_change(self, interior_id: u8) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_INTERIOR_CHANGE, interior_id)
    }

    /// Sends SA-MP's empty spawn RPC (52).
    ///
    /// This is protocol-only. It does not call the native local-player spawn
    /// method or change local spawn state.
    pub fn send_spawn(self) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_SPAWN, ())
    }

    /// Sends SA-MP's enter-vehicle RPC (26).
    ///
    /// This is protocol-only. It does not put the local GTA ped in a vehicle
    /// or otherwise alter native local-player state.
    pub fn send_enter_vehicle(self, vehicle_id: u16, passenger: bool) -> SampClientSdkResult {
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
    pub fn send_exit_vehicle(self, vehicle_id: u16) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_EXIT_VEHICLE, vehicle_id)
    }

    /// Sends a server-bound dialog response (RPC 62).
    pub fn send_dialog_response(
        self,
        dialog_id: u16,
        button: u8,
        list_item: u16,
        input: &[u8],
    ) -> SampClientSdkResult {
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
    pub fn send_click_player(self, player_id: u16, source: u8) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_CLICK_PLAYER,
            events::rpc::outgoing::ClickPlayer { player_id, source },
        )
    }

    /// Sends a server-bound textdraw-click action (RPC 83).
    pub fn send_click_textdraw(self, textdraw_id: u16) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_CLICK_TEXT_DRAW, textdraw_id)
    }

    /// Sends a server-bound death notification naming another player (RPC 53).
    pub fn send_death_by_player(self, player_id: u16, reason: u8) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_DEATH_NOTIFICATION,
            events::rpc::outgoing::DeathNotification {
                reason,
                killer_id: player_id,
            },
        )
    }

    /// Sends the empty menu-quit RPC (140).
    pub fn send_menu_quit(self) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_QUIT_MENU, ())
    }

    /// Sends a server-bound menu-row selection (RPC 132).
    pub fn send_menu_select_row(self, row: u8) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_MENU_SELECT, row)
    }

    /// Sends a server-bound pickup notification (RPC 131).
    pub fn send_picked_up_pickup(self, pickup_id: i32) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_PICKED_UP_PICKUP, pickup_id)
    }

    /// Sends a server-bound vehicle-destroyed notification (RPC 136).
    pub fn send_vehicle_destroyed(self, vehicle_id: u16) -> SampClientSdkResult {
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
    ) -> SampClientSdkResult {
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
    pub fn send_scm_event(
        self,
        event: i32,
        id: i32,
        param1: i32,
        param2: i32,
    ) -> SampClientSdkResult {
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
    ) -> SampClientSdkResult {
        self.send_damage(player_id, damage, weapon, body_part, false)
    }

    /// Sends a server-bound take-damage notification (RPC 115).
    pub fn send_take_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> SampClientSdkResult {
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
    ) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_EDIT_ATTACHED_OBJECT, edit)
    }

    /// Sends a complete global or player-object edit action (RPC 117).
    pub fn send_edit_object(self, edit: events::rpc::outgoing::EditObject) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_EDIT_OBJECT, edit)
    }

    /// Sends a bounded server-bound RCON command packet (201).
    pub fn send_rcon_command(self, command: &[u8]) -> SampClientSdkResult {
        self.send_typed_packet(
            events::packet::outgoing::SEND_RCON_COMMAND,
            command.to_vec(),
        )
    }

    /// Sends a complete local aim-sync packet (203).
    pub fn send_aim_sync(self, sync: events::packet::AimSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_AIM_SYNC, sync)
    }

    /// Sends a complete local bullet-sync packet (206).
    pub fn send_bullet_sync(self, sync: events::packet::BulletSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_BULLET_SYNC, sync)
    }

    /// Sends a complete local vehicle-sync packet (200).
    pub fn send_vehicle_sync(self, sync: events::packet::VehicleSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_VEHICLE_SYNC, sync)
    }

    /// Sends a complete local on-foot player-sync packet (207).
    pub fn send_player_sync(self, sync: events::packet::PlayerSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_PLAYER_SYNC, sync)
    }

    /// Sends a complete local spectator-sync packet (212).
    pub fn send_spectator_sync(self, sync: events::packet::SpectatorSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_SPECTATOR_SYNC, sync)
    }

    /// Sends a complete local trailer-sync packet (210).
    pub fn send_trailer_sync(self, sync: events::packet::TrailerSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_TRAILER_SYNC, sync)
    }

    /// Sends a complete local passenger-sync packet (211).
    pub fn send_passenger_sync(self, sync: events::packet::PassengerSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_PASSENGER_SYNC, sync)
    }

    /// Sends a complete local unoccupied-vehicle sync packet (209).
    pub fn send_unoccupied_sync(self, sync: events::packet::UnoccupiedSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_UNOCCUPIED_SYNC, sync)
    }

    /// Queues an incoming packet for SA-MP after incoming plugin listeners run.
    ///
    /// `payload` excludes the packet ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`SampClientSdkResult::Ok`].
    pub fn emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> SampClientSdkResult {
        unsafe {
            (self.raw.emulate_incoming_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len)
        }
    }

    /// Dispatches an incoming RPC to plugin listeners and then SA-MP unless blocked.
    ///
    /// `payload` excludes the RPC ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`SampClientSdkResult::Ok`].
    pub fn emulate_incoming_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> SampClientSdkResult {
        unsafe { (self.raw.emulate_incoming_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len) }
    }

    /// Copies and queues a locally generated incoming packet, returning its completion receipt.
    pub fn submit_emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_emulate_incoming_packet)(
                packet_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a locally generated incoming RPC, returning its completion receipt.
    pub fn submit_emulate_incoming_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_emulate_incoming_rpc)(
                rpc_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Queues one validated R1 CNetGame-state write and returns its completion receipt.
    pub fn submit_samp_game_state(
        self,
        state: SampGameState,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_samp_game_state)(state.raw(), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one R1 replication send-rate write in milliseconds.
    pub fn submit_send_rate(
        self,
        kind: SendRateKind,
        milliseconds: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_send_rate)(kind.raw(), milliseconds, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one documented R1 unoccupied-vehicle synchronization send.
    pub fn submit_force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_force_unoccupied_sync)(vehicle, seat, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one bounded R1 chat-history entry replacement.
    pub fn submit_local_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_local_chat_entry)(
                id,
                text.as_ptr(),
                text.len(),
                prefix.as_ptr(),
                prefix.len(),
                text_colour,
                prefix_colour,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    pub(crate) fn submit_typed_rpc<T>(
        self,
        descriptor: events::Rpc<T>,
        value: T,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let Ok(payload) = descriptor.encode(self, value) else {
            return Err(SampClientSdkResult::InvalidArgument);
        };
        self.submit_rpc(
            descriptor.id(),
            payload.as_bytes(),
            payload.len_bits(),
            SampClientSdkSendOptions::default(),
        )
    }

    fn send_typed_rpc<T>(self, descriptor: events::Rpc<T>, value: T) -> SampClientSdkResult {
        self.submit_typed_rpc(descriptor, value)
            .map_or_else(|error| error, |_| SampClientSdkResult::Ok)
    }

    fn send_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
        take: bool,
    ) -> SampClientSdkResult {
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

    pub(crate) fn submit_typed_packet<T>(
        self,
        descriptor: events::Packet<T>,
        value: T,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let Ok(payload) = descriptor.encode(self, value) else {
            return Err(SampClientSdkResult::InvalidArgument);
        };
        self.submit_packet(
            descriptor.id(),
            payload.as_bytes(),
            payload.len_bits(),
            SampClientSdkSendOptions::default(),
        )
    }

    fn send_typed_packet<T>(self, descriptor: events::Packet<T>, value: T) -> SampClientSdkResult {
        self.submit_typed_packet(descriptor, value)
            .map_or_else(|error| error, |_| SampClientSdkResult::Ok)
    }
}

fn valid_bounded_bytes(value: &[u8], maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.contains(&0)
}

fn local_dialog_state_from_abi(
    raw: SampClientSdkDialogSnapshotV1,
) -> Result<Option<LocalDialogState>, SampClientSdkResult> {
    match raw.active {
        0 if raw == SampClientSdkDialogSnapshotV1::default() => Ok(None),
        1 => {
            let Some(style) = LocalDialogStyle::from_raw(raw.style) else {
                return Err(SampClientSdkResult::NativeCallFailed);
            };
            let server_side = match raw.server_side {
                0 => false,
                1 => true,
                _ => return Err(SampClientSdkResult::NativeCallFailed),
            };
            let title_len = usize::from(raw.title_len);
            let text_len = usize::from(raw.text_len);
            let editbox_text_len = usize::from(raw.editbox_text_len);
            let listbox_item_count = usize::from(raw.listbox_item_count);
            if title_len > raw.title.len()
                || text_len > raw.text.len()
                || editbox_text_len > raw.editbox_text.len()
                || listbox_item_count > raw.listbox_items.len()
            {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            let editbox_text = match raw.has_editbox {
                0 if editbox_text_len == 0 => None,
                1 => Some(raw.editbox_text[..editbox_text_len].to_vec()),
                _ => return Err(SampClientSdkResult::NativeCallFailed),
            };
            let items = raw.listbox_items[..listbox_item_count]
                .iter()
                .map(|item| {
                    let len = usize::from(item.len);
                    (len <= item.bytes.len())
                        .then(|| item.bytes[..len].to_vec())
                        .ok_or(SampClientSdkResult::NativeCallFailed)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(LocalDialogState {
                id: raw.id,
                style,
                title: raw.title[..title_len].to_vec(),
                server_side,
                text: raw.text[..text_len].to_vec(),
                editbox_text,
                items,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

fn local_animation_from_abi(raw: SampClientSdkAnimationV1) -> Option<LocalAnimation> {
    let name_len = usize::from(raw.name_len);
    let file_len = usize::from(raw.file_len);
    if name_len == 0
        || file_len == 0
        || name_len > raw.name.len()
        || file_len > raw.file.len()
        || raw.name[..name_len].contains(&0)
        || raw.file[..file_len].contains(&0)
    {
        return None;
    }
    Some(LocalAnimation {
        name: raw.name[..name_len].to_vec(),
        file: raw.file[..file_len].to_vec(),
    })
}

fn player_info_from_abi(
    raw: SampClientSdkPlayerInfoV1,
) -> Result<Option<PlayerInfo>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkPlayerInfoV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 => {
            let nickname_len = usize::from(raw.nickname_len);
            if nickname_len == 0
                || nickname_len > raw.nickname.len()
                || !matches!(raw.is_local, 0 | 1)
                || !matches!(raw.is_npc, 0 | 1)
                || raw._reserved != 0
                || (raw.is_local != 0 && raw.is_npc != 0)
            {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(Some(PlayerInfo {
                id: raw.id,
                nickname: raw.nickname[..nickname_len].to_vec(),
                is_local: raw.is_local != 0,
                is_npc: raw.is_npc != 0,
                colour: raw.colour,
                score: raw.score,
                ping: raw.ping,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

fn remote_player_state_from_abi(
    raw: SampClientSdkRemotePlayerStateV1,
) -> Result<Option<RemotePlayerState>, SampClientSdkResult> {
    match raw.exists {
        0 if raw == SampClientSdkRemotePlayerStateV1::default() => Ok(None),
        1 if raw._reserved == 0 && raw.health.is_finite() && raw.armour.is_finite() => {
            Ok(Some(RemotePlayerState {
                id: raw.id,
                health: raw.health,
                armour: raw.armour,
                special_action: raw.special_action,
                animation_id: raw.animation_id,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

fn gangzone_from_abi(
    raw: SampClientSdkGangzoneV1,
) -> Result<Option<Gangzone>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkGangzoneV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 if raw._reserved == [0; 3]
            && raw._reserved2 == 0
            && raw.left.is_finite()
            && raw.bottom.is_finite()
            && raw.right.is_finite()
            && raw.top.is_finite() =>
        {
            Ok(Some(Gangzone {
                id: raw.id,
                left: raw.left,
                bottom: raw.bottom,
                right: raw.right,
                top: raw.top,
                colour: raw.colour,
                alternate_colour: raw.alternate_colour,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

fn chat_entry_from_abi(raw: SampClientSdkChatEntryV1) -> Result<ChatEntry, SampClientSdkResult> {
    if raw.id >= MAX_SAMP_CHAT_ENTRIES
        || usize::from(raw.text_len) > MAX_SAMP_CHAT_ENTRY_TEXT_BYTES
        || usize::from(raw.prefix_len) > MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES
    {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    Ok(ChatEntry {
        id: raw.id,
        text: raw.text[..usize::from(raw.text_len)].to_vec(),
        prefix: raw.prefix[..usize::from(raw.prefix_len)].to_vec(),
        text_colour: raw.text_colour,
        prefix_colour: raw.prefix_colour,
    })
}

fn text_label_from_abi(
    raw: SampClientSdkTextLabelV1,
) -> Result<Option<TextLabel>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkTextLabelV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 if matches!(raw.behind_walls, 0 | 1)
            && raw._reserved == [0; 2]
            && raw._reserved2 == 0
            && raw._reserved3 == [0; 2]
            && usize::from(raw.text_len) <= raw.text.len()
            && !raw.text[..usize::from(raw.text_len)].contains(&0)
            && raw._reserved3 == [0; 2]
            && raw.position.x.is_finite()
            && raw.position.y.is_finite()
            && raw.position.z.is_finite()
            && raw.draw_distance.is_finite() =>
        {
            let text_len = usize::from(raw.text_len);
            if text_len > raw.text.len() || raw.text[..text_len].contains(&0) {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(Some(TextLabel {
                id: raw.id,
                text: raw.text[..text_len].to_vec(),
                colour: raw.colour,
                position: raw.position,
                draw_distance: raw.draw_distance,
                behind_walls: raw.behind_walls != 0,
                attached_player_id: (raw.attached_player_id != u16::MAX)
                    .then_some(raw.attached_player_id),
                attached_vehicle_id: (raw.attached_vehicle_id != u16::MAX)
                    .then_some(raw.attached_vehicle_id),
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

fn textdraw_from_abi(
    raw: SampClientSdkTextDrawV1,
) -> Result<Option<TextDraw>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkTextDrawV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 if matches!(raw.proportional, 0 | 1)
            && matches!(raw.align_left, 0 | 1)
            && matches!(raw.align_center, 0 | 1)
            && matches!(raw.align_right, 0 | 1)
            && matches!(raw.box_enabled, 0 | 1)
            && raw._reserved == [0; 2]
            && raw._reserved2 == 0
            && usize::from(raw.text_len) <= MAX_SAMP_TEXTDRAW_STRING_BYTES
            && raw.letter_width.is_finite()
            && raw.letter_height.is_finite()
            && raw.x.is_finite()
            && raw.y.is_finite()
            && raw.box_width.is_finite()
            && raw.box_height.is_finite()
            && raw.rotation.x.is_finite()
            && raw.rotation.y.is_finite()
            && raw.rotation.z.is_finite()
            && raw.zoom.is_finite() =>
        {
            Ok(Some(TextDraw {
                pool_index: raw.pool_index,
                text: raw.text[..usize::from(raw.text_len)].to_vec(),
                letter_width: raw.letter_width,
                letter_height: raw.letter_height,
                letter_colour: raw.letter_colour,
                x: raw.x,
                y: raw.y,
                shadow: raw.shadow,
                outline: raw.outline,
                background_colour: raw.background_colour,
                style: raw.style,
                proportional: raw.proportional != 0,
                align_left: raw.align_left != 0,
                align_center: raw.align_center != 0,
                align_right: raw.align_right != 0,
                box_enabled: raw.box_enabled != 0,
                box_width: raw.box_width,
                box_height: raw.box_height,
                box_colour: raw.box_colour,
                model_id: raw.model_id,
                rotation: raw.rotation,
                zoom: raw.zoom,
                model_colour1: raw.model_colour1,
                model_colour2: raw.model_colour2,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

unsafe extern "system" fn dispatch_callback(
    user_data: *mut c_void,
    raw: *mut SampClientSdkEventV1,
) -> SampClientSdkHookAction {
    let Some(callback) = (unsafe { user_data.cast::<CallbackState>().as_ref() }) else {
        return SampClientSdkHookAction::Continue;
    };
    let Ok(mut event) = (unsafe { events::Event::from_callback(callback.api, raw) }) else {
        return SampClientSdkHookAction::Continue;
    };
    catch_unwind(AssertUnwindSafe(|| (callback.handler)(&mut event)))
        .unwrap_or(SampClientSdkHookAction::Continue)
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
            Self::UnsupportedPlatform => {
                formatter.write_str("samp-client-sdk plugins require Windows")
            }
            Self::HostNotLoaded => formatter.write_str("samp-client-sdk host module is not loaded"),
            Self::MissingApi => {
                formatter.write_str("samp-client-sdk host does not export SampClientSdk_GetApiV1")
            }
            Self::UnsupportedAbi => {
                formatter.write_str("samp-client-sdk host ABI v1 is unavailable")
            }
            Self::HostFailed => formatter.write_str("samp-client-sdk host failed to initialize"),
            Self::TimedOut => {
                formatter.write_str("timed out waiting for samp-client-sdk host readiness")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Waits for the default `samp_client_sdk.asi` host to expose a ready v1 API.
///
/// Call this from a plugin worker thread, never from `DllMain`.
pub fn wait_for_default_host(timeout: Duration) -> Result<HostApi, ResolveError> {
    wait_for_host(DEFAULT_HOST_MODULE, timeout)
}

/// Waits for a named host module to expose a ready v1 API.
///
/// `module_name` must be NUL-terminated, for example `b"samp_client_sdk.asi\\0"`.
pub fn wait_for_host(module_name: &[u8], timeout: Duration) -> Result<HostApi, ResolveError> {
    if module_name.last() != Some(&0) {
        return Err(ResolveError::HostNotLoaded);
    }
    let started = Instant::now();
    loop {
        match resolve_host(module_name) {
            Ok(api) => match api.status() {
                SampClientSdkHostStatus::Ready => return Ok(api),
                SampClientSdkHostStatus::Failed => return Err(ResolveError::HostFailed),
                SampClientSdkHostStatus::WaitingForSamp => {}
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
    let symbol = unsafe { GetProcAddress(module, c"SampClientSdk_GetApiV1".as_ptr().cast()) };
    let Some(symbol) = symbol else {
        return Err(ResolveError::MissingApi);
    };
    let get_api: SampClientSdkGetApiV1 = unsafe { mem::transmute(symbol) };
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
        assert_eq!(SampClientSdkSendOptions::default().priority, 1);
        assert_eq!(SampClientSdkSendOptions::default().reliability, 9);
    }

    #[test]
    fn default_host_module_matches_the_deploy_artifact() {
        assert_eq!(DEFAULT_HOST_MODULE, b"samp_client_sdk.asi\0");
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
            mem::offset_of!(SampClientSdkApiV1, emulate_incoming_packet),
            mem::offset_of!(SampClientSdkApiV1, unregister_and_wait) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, emulate_incoming_rpc),
            mem::offset_of!(SampClientSdkApiV1, emulate_incoming_packet) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, event_remaining_bits),
            mem::offset_of!(SampClientSdkApiV1, emulate_incoming_rpc) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, event_read_bits),
            mem::offset_of!(SampClientSdkApiV1, event_remaining_bits) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, event_replace_bits),
            mem::offset_of!(SampClientSdkApiV1, event_read_bits) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, encode_string),
            mem::offset_of!(SampClientSdkApiV1, event_replace_bits) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, event_read_encoded_string),
            mem::offset_of!(SampClientSdkApiV1, encode_string) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, show_local_dialog),
            mem::offset_of!(SampClientSdkApiV1, event_read_encoded_string) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_player),
            mem::offset_of!(SampClientSdkApiV1, show_local_dialog) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, samp_game_state),
            mem::offset_of!(SampClientSdkApiV1, local_player) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, samp_version),
            mem::offset_of!(SampClientSdkApiV1, samp_game_state) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, decode_string),
            mem::offset_of!(SampClientSdkApiV1, samp_version) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, server_info),
            mem::offset_of!(SampClientSdkApiV1, decode_string) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, show_local_chat_message),
            mem::offset_of!(SampClientSdkApiV1, server_info) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, show_local_death_message),
            mem::offset_of!(SampClientSdkApiV1, show_local_chat_message) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_chat_display_mode),
            mem::offset_of!(SampClientSdkApiV1, show_local_death_message) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_cursor_mode),
            mem::offset_of!(SampClientSdkApiV1, local_chat_display_mode) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_scoreboard_open),
            mem::offset_of!(SampClientSdkApiV1, local_cursor_mode) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_dialog_active),
            mem::offset_of!(SampClientSdkApiV1, local_scoreboard_open) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_chat_input_active),
            mem::offset_of!(SampClientSdkApiV1, local_dialog_active) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_animation),
            mem::offset_of!(SampClientSdkApiV1, local_chat_input_active) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_animation_id),
            mem::offset_of!(SampClientSdkApiV1, local_animation) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, player_info),
            mem::offset_of!(SampClientSdkApiV1, local_animation_id) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, player_count),
            mem::offset_of!(SampClientSdkApiV1, player_info) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, player_max_id),
            mem::offset_of!(SampClientSdkApiV1, player_count) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, vehicle_exists),
            mem::offset_of!(SampClientSdkApiV1, player_max_id) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, active_local_dialog),
            mem::offset_of!(SampClientSdkApiV1, vehicle_exists) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, text_label_exists),
            mem::offset_of!(SampClientSdkApiV1, active_local_dialog) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, textdraw_exists),
            mem::offset_of!(SampClientSdkApiV1, text_label_exists) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, object_exists),
            mem::offset_of!(SampClientSdkApiV1, textdraw_exists) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, gangzone_info),
            mem::offset_of!(SampClientSdkApiV1, object_exists) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, text_label_info),
            mem::offset_of!(SampClientSdkApiV1, gangzone_info) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, textdraw_info),
            mem::offset_of!(SampClientSdkApiV1, text_label_info) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, player_defined),
            mem::offset_of!(SampClientSdkApiV1, textdraw_info) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, player_paused),
            mem::offset_of!(SampClientSdkApiV1, player_defined) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, remote_player_state),
            mem::offset_of!(SampClientSdkApiV1, player_paused) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog),
            mem::offset_of!(SampClientSdkApiV1, remote_player_state) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_message),
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_death_message),
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_message) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, command_try_take),
            mem::offset_of!(SampClientSdkApiV1, submit_local_death_message) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, command_wait),
            mem::offset_of!(SampClientSdkApiV1, command_try_take) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, command_release),
            mem::offset_of!(SampClientSdkApiV1, command_wait) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_packet),
            mem::offset_of!(SampClientSdkApiV1, command_release) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_rpc),
            mem::offset_of!(SampClientSdkApiV1, submit_packet) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_packet),
            mem::offset_of!(SampClientSdkApiV1, submit_rpc) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_rpc),
            mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_packet) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, raw_player_pool),
            mem::offset_of!(SampClientSdkApiV1, raw_rakclient) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, raw_vehicle_pool),
            mem::offset_of!(SampClientSdkApiV1, raw_player_pool) + function_size
        );
        assert_eq!(
            mem::size_of::<SampClientSdkApiV1>(),
            mem::offset_of!(SampClientSdkApiV1, local_player_id_by_ped_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_dialog_snapshot),
            mem::offset_of!(SampClientSdkApiV1, submit_create_text_label) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_editbox_text),
            mem::offset_of!(SampClientSdkApiV1, local_dialog_snapshot) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_object_handle),
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_editbox_text) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_object_id_by_handle),
            mem::offset_of!(SampClientSdkApiV1, local_object_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_pickup_handle),
            mem::offset_of!(SampClientSdkApiV1, local_object_id_by_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_pickup_id_by_handle),
            mem::offset_of!(SampClientSdkApiV1, local_pickup_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_vehicle_handle),
            mem::offset_of!(SampClientSdkApiV1, local_pickup_id_by_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_vehicle_id_by_handle),
            mem::offset_of!(SampClientSdkApiV1, local_vehicle_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_player_ped_handle),
            mem::offset_of!(SampClientSdkApiV1, local_vehicle_id_by_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_player_id_by_ped_handle),
            mem::offset_of!(SampClientSdkApiV1, local_player_ped_handle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, raw_rakclient),
            mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_rpc) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_mode),
            mem::offset_of!(SampClientSdkApiV1, raw_vehicle_pool) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_scoreboard_open),
            mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_mode) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_client_side),
            mem::offset_of!(SampClientSdkApiV1, submit_local_scoreboard_open) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_samp_game_state),
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_client_side) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, raw_local_player),
            mem::offset_of!(SampClientSdkApiV1, submit_samp_game_state) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_player_spawn),
            mem::offset_of!(SampClientSdkApiV1, raw_local_player) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_player_special_action),
            mem::offset_of!(SampClientSdkApiV1, submit_local_player_spawn) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_send_rate),
            mem::offset_of!(SampClientSdkApiV1, submit_local_player_special_action) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_toggle),
            mem::offset_of!(SampClientSdkApiV1, submit_send_rate) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_display_mode),
            mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_toggle) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, raw_rakpeer),
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_display_mode) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_close),
            mem::offset_of!(SampClientSdkApiV1, raw_rakpeer) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_text),
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_close) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_enabled),
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_text) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_process),
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_enabled) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_chat_input_text),
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_process) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_player_colour),
            mem::offset_of!(SampClientSdkApiV1, local_chat_input_text) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_player_name),
            mem::offset_of!(SampClientSdkApiV1, submit_player_colour) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_force_unoccupied_sync),
            mem::offset_of!(SampClientSdkApiV1, submit_local_player_name) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_connect_to_server),
            mem::offset_of!(SampClientSdkApiV1, submit_force_unoccupied_sync) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_disconnect_with_reason),
            mem::offset_of!(SampClientSdkApiV1, submit_connect_to_server) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_delete_textdraw),
            mem::offset_of!(SampClientSdkApiV1, submit_disconnect_with_reason) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_position),
            mem::offset_of!(SampClientSdkApiV1, submit_delete_textdraw) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_letter_style),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_position) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_proportional),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_letter_style) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_shadow),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_proportional) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_outline),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_shadow) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_box),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_outline) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_alignment),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_box) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_string),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_alignment) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_dialog_selected_item),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_string) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_selected_item),
            mem::offset_of!(SampClientSdkApiV1, local_dialog_selected_item) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_delete_text_label),
            mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_selected_item) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, local_dialog_list_item_count),
            mem::offset_of!(SampClientSdkApiV1, submit_delete_text_label) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_model_style),
            mem::offset_of!(SampClientSdkApiV1, local_dialog_list_item_count) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_entry),
            mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_model_style) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, chat_entry_info),
            mem::offset_of!(SampClientSdkApiV1, submit_local_chat_entry) + function_size
        );
        assert_eq!(
            mem::offset_of!(SampClientSdkApiV1, submit_create_text_label),
            mem::offset_of!(SampClientSdkApiV1, chat_entry_info) + function_size
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
        assert_eq!(api.show_local_dialog(valid), SampClientSdkResult::Ok);

        let nul = LocalDialog {
            title: b"bad\0title",
            ..valid
        };
        assert_eq!(
            api.show_local_dialog(nul),
            SampClientSdkResult::InvalidArgument
        );

        let too_long = [b'x'; 256];
        let long_title = LocalDialog {
            title: &too_long,
            ..valid
        };
        assert_eq!(
            api.show_local_dialog(long_title),
            SampClientSdkResult::InvalidArgument
        );
    }

    #[test]
    fn direct_chat_rejects_nuls_and_native_entry_overflows_before_the_abi_call() {
        let api = test_support::test_api();
        let valid = LocalChatMessage {
            style: LocalChatMessageStyle::Debug,
            text: b"local message",
            prefix: b"[samp-client-sdk]",
            text_colour: 0xFF_A9_C4_E4,
            prefix_colour: u32::MAX,
        };
        assert_eq!(api.show_local_chat_message(valid), SampClientSdkResult::Ok);
        assert_eq!(
            api.show_local_chat_message(LocalChatMessage {
                text: b"bad\0text",
                ..valid
            }),
            SampClientSdkResult::InvalidArgument
        );
        let too_long_text = [b'x'; 144];
        assert_eq!(
            api.show_local_chat_message(LocalChatMessage {
                text: &too_long_text,
                ..valid
            }),
            SampClientSdkResult::InvalidArgument
        );
        let too_long_prefix = [b'x'; 28];
        assert_eq!(
            api.show_local_chat_message(LocalChatMessage {
                prefix: &too_long_prefix,
                ..valid
            }),
            SampClientSdkResult::InvalidArgument
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
        assert_eq!(api.show_local_death_message(valid), SampClientSdkResult::Ok);
        assert_eq!(
            api.show_local_death_message(LocalDeathMessage {
                killer: b"bad\0killer",
                ..valid
            }),
            SampClientSdkResult::InvalidArgument
        );
        let too_long = [b'x'; 25];
        assert_eq!(
            api.show_local_death_message(LocalDeathMessage {
                victim: &too_long,
                ..valid
            }),
            SampClientSdkResult::InvalidArgument
        );
    }

    #[test]
    fn direct_commands_return_owned_receipts_that_poll_wait_and_release() {
        let api = test_support::test_api();
        let mut dialog = api
            .submit_local_dialog(LocalDialog {
                id: 7,
                style: LocalDialogStyle::MessageBox,
                title: b"title",
                text: b"text",
                button1: b"ok",
                button2: b"",
            })
            .expect("fixture accepts dialog submissions");
        assert_eq!(dialog.id(), 1);
        assert_eq!(dialog.try_take(), Ok(Some(())));

        let mut chat = api
            .submit_local_chat_message(LocalChatMessage {
                style: LocalChatMessageStyle::Debug,
                text: b"local message",
                prefix: b"[samp-client-sdk]",
                text_colour: 0xFF_A9_C4_E4,
                prefix_colour: u32::MAX,
            })
            .expect("fixture accepts chat submissions");
        assert_eq!(chat.id(), 2);
        assert_eq!(chat.wait(Duration::ZERO), Ok(()));

        let death = api
            .submit_local_death_message(LocalDeathMessage {
                killer: b"killer",
                victim: b"victim",
                killer_colour: 0xFFFF_0000,
                victim_colour: 0xFF00_FF00,
                weapon: 24,
            })
            .expect("fixture accepts death-window submissions");
        assert_eq!(death.id(), 3);
        assert_eq!(death.release(), Ok(()));
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
    fn player_directory_entry_is_owned_and_handles_a_cached_disconnect() {
        let api = test_support::test_api();
        assert_eq!(
            api.player_info(7),
            Ok(Some(PlayerInfo {
                id: 7,
                nickname: b"remote".to_vec(),
                is_local: false,
                is_npc: true,
                colour: 0xFF22_4466,
                score: -10,
                ping: 55,
            }))
        );
        assert_eq!(api.is_player_connected(7), Ok(true));
        assert_eq!(api.is_player_defined(7), Ok(true));
        assert_eq!(api.is_player_paused(7), Ok(false));
        assert_eq!(api.player_nickname(7), Ok(Some(b"remote".to_vec())));
        assert_eq!(api.is_player_npc(7), Ok(Some(true)));
        assert_eq!(api.player_colour(7), Ok(Some(0xFF22_4466)));
        assert_eq!(api.player_score(7), Ok(Some(-10)));
        assert_eq!(api.player_ping(7), Ok(Some(55)));
        assert_eq!(
            api.remote_player_state(7),
            Ok(Some(RemotePlayerState {
                id: 7,
                health: 75.0,
                armour: 25.0,
                special_action: 3,
                animation_id: 123,
            }))
        );
        assert_eq!(api.player_health(7), Ok(Some(75.0)));
        assert_eq!(api.player_armour(7), Ok(Some(25.0)));
        assert_eq!(api.player_special_action(7), Ok(Some(3)));
        assert_eq!(api.player_animation_id(7), Ok(Some(123)));
        assert_eq!(api.remote_player_state(8), Ok(None));
        assert_eq!(api.player_info(8), Ok(None));
        assert_eq!(api.is_player_connected(8), Ok(false));
        assert_eq!(api.is_player_defined(8), Ok(false));
        assert_eq!(api.is_player_paused(8), Ok(false));
        assert_eq!(api.is_player_paused(9), Ok(true));
        assert_eq!(api.player_count(true), Ok(3));
        assert_eq!(api.player_count(false), Ok(2));
        assert_eq!(api.player_max_id(), Ok(42));
        assert_eq!(api.is_vehicle_defined(7), Ok(true));
        assert_eq!(api.is_vehicle_defined(8), Ok(false));
        assert_eq!(
            api.is_vehicle_defined(MAX_SAMP_VEHICLES),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(api.is_text_label_defined(7), Ok(true));
        assert_eq!(api.is_text_label_defined(8), Ok(false));
        assert_eq!(
            api.is_text_label_defined(MAX_SAMP_TEXT_LABELS),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(api.is_textdraw_defined(7), Ok(true));
        assert_eq!(api.is_textdraw_defined(8), Ok(false));
        assert_eq!(
            api.is_textdraw_defined(MAX_SAMP_TEXTDRAWS),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(api.is_object_defined(7), Ok(true));
        assert_eq!(api.is_object_defined(8), Ok(false));
        assert_eq!(
            api.is_object_defined(MAX_SAMP_OBJECTS),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(
            api.gangzone(7),
            Ok(Some(Gangzone {
                id: 7,
                left: -1.0,
                bottom: -2.0,
                right: 3.0,
                top: 4.0,
                colour: 0xFF11_2233,
                alternate_colour: 0xFF44_5566,
            }))
        );
        assert_eq!(api.gangzone(8), Ok(None));
        assert_eq!(
            api.gangzone(MAX_SAMP_GANGZONES),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(
            api.text_label(7),
            Ok(Some(TextLabel {
                id: 7,
                text: b"fixture".to_vec(),
                colour: 0xFF11_2233,
                position: Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                draw_distance: 25.0,
                behind_walls: true,
                attached_player_id: Some(8),
                attached_vehicle_id: None,
            }))
        );
        assert_eq!(api.text_label(8), Ok(None));
        assert_eq!(
            api.text_label(MAX_SAMP_TEXT_LABELS),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(
            api.textdraw(7),
            Ok(Some(TextDraw {
                pool_index: 7,
                text: b"fixture".to_vec(),
                letter_width: 1.0,
                letter_height: 2.0,
                letter_colour: 0xFF11_2233,
                x: 3.0,
                y: 4.0,
                shadow: 2,
                outline: 3,
                background_colour: 0xFF44_5566,
                style: 5,
                proportional: true,
                align_left: false,
                align_center: true,
                align_right: false,
                box_enabled: true,
                box_width: 6.0,
                box_height: 7.0,
                box_colour: 0xFF77_8899,
                model_id: 10,
                rotation: Vector3 {
                    x: 8.0,
                    y: 9.0,
                    z: 10.0,
                },
                zoom: 11.0,
                model_colour1: 12,
                model_colour2: 13,
            }))
        );
        assert_eq!(api.textdraw(8), Ok(None));
        assert_eq!(
            api.textdraw(MAX_SAMP_TEXTDRAWS),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(
            api.chat_entry(7),
            Ok(ChatEntry {
                id: 7,
                text: b"fixture".to_vec(),
                prefix: b"prefix".to_vec(),
                text_colour: 0xFF11_2233,
                prefix_colour: 0xFF44_5566,
            })
        );
        assert_eq!(
            api.chat_entry(MAX_SAMP_CHAT_ENTRIES),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(
            api.active_local_dialog(),
            Ok(Some(LocalDialogState {
                id: 7,
                style: LocalDialogStyle::Input,
                title: b"fixture".to_vec(),
                server_side: false,
                text: b"fixture".to_vec(),
                editbox_text: Some(b"fixture".to_vec()),
                items: vec![b"fixture".to_vec(); 3],
            }))
        );
        assert_eq!(
            api.player_info(MAX_SAMP_PLAYERS),
            Err(SampClientSdkResult::InvalidArgument)
        );
    }

    #[test]
    fn dialog_snapshot_preserves_an_absent_editbox() {
        let mut raw = SampClientSdkDialogSnapshotV1::default();
        raw.active = 1;
        raw.style = 0;
        raw.id = 7;
        let dialog = local_dialog_state_from_abi(raw)
            .expect("canonical dialog snapshot")
            .expect("active dialog");

        assert_eq!(dialog.style, LocalDialogStyle::MessageBox);
        assert_eq!(dialog.editbox_text(), None);
    }

    #[test]
    fn dialog_list_item_abi_length_covers_its_entire_payload() {
        assert_eq!(MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES, usize::from(u8::MAX));
        assert_eq!(mem::size_of::<SampClientSdkDialogListItemV1>(), 256);
    }

    #[test]
    fn dialog_snapshot_abi_layout_is_stable() {
        assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, id), 4);
        assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, text_len), 12);
        assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, title), 16);
        assert_eq!(
            mem::offset_of!(SampClientSdkDialogSnapshotV1, editbox_text),
            81
        );
        assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, text), 209);
        assert_eq!(
            mem::offset_of!(SampClientSdkDialogSnapshotV1, listbox_items),
            4_305
        );
        assert_eq!(mem::size_of::<SampClientSdkDialogSnapshotV1>(), 29_908);
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
        assert_eq!(api.is_local_dialog_active(), Ok(false));
        assert_eq!(api.is_local_chat_input_active(), Ok(false));
        assert_eq!(LocalCursorMode::from_raw(5), None);
    }

    #[test]
    fn local_animation_table_uses_owned_bounded_abi_storage() {
        let api = test_support::test_api();
        assert_eq!(
            api.local_animation(0),
            Ok(LocalAnimation {
                name: b"AIRPORT".to_vec(),
                file: b"THRW_BARL_THRW".to_vec(),
            })
        );
        assert_eq!(
            api.local_animation_id(b"AIRPORT", b"THRW_BARL_THRW"),
            Ok(Some(0))
        );
        assert_eq!(api.local_animation_id(b"missing", b"entry"), Ok(None));
        assert_eq!(
            api.local_animation_id(b"", b"entry"),
            Err(SampClientSdkResult::InvalidArgument)
        );
        assert_eq!(
            api.local_animation_id(&[b'x'; 36], b"entry"),
            Err(SampClientSdkResult::InvalidArgument)
        );
    }

    #[test]
    fn samp_version_is_converted_from_the_scalar_abi_output() {
        assert_eq!(
            test_support::test_api().samp_version(),
            Ok(SampClientSdkClientVersion::R1)
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
            Err(SampClientSdkResult::InvalidArgument)
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
            api.send_packet_stream(200, &stream, SampClientSdkSendOptions::default()),
            SampClientSdkResult::NativeCallFailed
        );
        assert_eq!(
            api.send_rpc_stream(62, &stream, SampClientSdkSendOptions::default()),
            SampClientSdkResult::NativeCallFailed
        );
    }

    #[test]
    fn send_chat_uses_the_typed_bounded_rpc_101_payload() {
        let api = test_support::test_api();
        assert_eq!(api.send_chat(b"hi"), SampClientSdkResult::Ok);
        assert_eq!(api.send_chat(b"/hi"), SampClientSdkResult::Ok);
        assert_eq!(
            api.send_chat(&[b'x'; 256]),
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(api.send_request_spawn(), SampClientSdkResult::Ok);
    }

    #[test]
    fn local_player_protocol_actions_preserve_their_wire_vectors() {
        let api = test_support::test_api();
        assert_eq!(api.send_request_class(9), SampClientSdkResult::Ok);
        assert_eq!(api.send_interior_change(7), SampClientSdkResult::Ok);
        assert_eq!(api.send_spawn(), SampClientSdkResult::Ok);
        assert_eq!(
            api.send_enter_vehicle(0x1234, true),
            SampClientSdkResult::Ok
        );
        assert_eq!(api.send_exit_vehicle(0x1234), SampClientSdkResult::Ok);
    }

    #[test]
    fn typed_protocol_action_conveniences_preserve_their_wire_vectors() {
        let api = test_support::test_api();
        assert_eq!(
            api.send_dialog_response(0x1234, 1, 0x3456, b"ok"),
            SampClientSdkResult::Ok
        );
        assert_eq!(api.send_click_player(0x1234, 2), SampClientSdkResult::Ok);
        assert_eq!(api.send_click_textdraw(0x1234), SampClientSdkResult::Ok);
        assert_eq!(api.send_death_by_player(0x1234, 9), SampClientSdkResult::Ok);
        assert_eq!(api.send_menu_quit(), SampClientSdkResult::Ok);
        assert_eq!(api.send_menu_select_row(7), SampClientSdkResult::Ok);
        assert_eq!(api.send_picked_up_pickup(9), SampClientSdkResult::Ok);
        assert_eq!(api.send_vehicle_destroyed(0x1234), SampClientSdkResult::Ok);
        assert_eq!(
            api.send_dialog_response(0, 0, 0, &[b'x'; 256]),
            SampClientSdkResult::InvalidArgument
        );
    }

    #[test]
    fn additional_typed_protocol_actions_preserve_their_wire_vectors() {
        let api = test_support::test_api();
        assert_eq!(
            api.send_vehicle_damage(0x1234, 1, 2, 3, 4),
            SampClientSdkResult::Ok
        );
        assert_eq!(api.send_scm_event(4, 1, 2, 3), SampClientSdkResult::Ok);
        assert_eq!(
            api.send_give_damage(0x1234, 1.0, 24, 9),
            SampClientSdkResult::Ok
        );
        assert_eq!(
            api.send_take_damage(0x1234, 1.0, 24, 9),
            SampClientSdkResult::Ok
        );

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
        assert_eq!(
            api.send_edit_attached_object(attached),
            SampClientSdkResult::Ok
        );
        assert_eq!(
            api.send_edit_object(events::rpc::outgoing::EditObject {
                player_object: false,
                object_id: 0,
                response: 0,
                position: zero,
                rotation: zero,
            }),
            SampClientSdkResult::Ok
        );
        assert_eq!(api.send_rcon_command(b"rcon"), SampClientSdkResult::Ok);
        assert_eq!(
            api.send_rcon_command(&[b'x'; events::MAX_STRING32_BYTES + 1]),
            SampClientSdkResult::InvalidArgument
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
            SampClientSdkResult::Ok
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
            SampClientSdkResult::Ok
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
            SampClientSdkResult::Ok
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
            SampClientSdkResult::Ok
        );
        assert_eq!(
            api.send_spectator_sync(events::packet::SpectatorSync {
                left_right_keys: 0,
                up_down_keys: 0,
                key_data: 0,
                position: zero,
            }),
            SampClientSdkResult::Ok
        );
        assert_eq!(
            api.send_trailer_sync(events::packet::TrailerSync {
                trailer_id: 0,
                position: zero,
                quaternion: [0.0; 4],
                move_speed: zero,
                turn_speed: zero,
            }),
            SampClientSdkResult::Ok
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
            SampClientSdkResult::Ok
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
            SampClientSdkResult::Ok
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
            .on_rpc(SampClientSdkDirection::Incoming, move |event| {
                assert_eq!(event.id(), 42);
                observed.fetch_add(1, Ordering::AcqRel);
                SampClientSdkHookAction::Block
            })
            .expect("test registration must succeed");

        assert_eq!(subscription.id(), 1);
        assert_eq!(
            test_support::invoke_registered_callback(42),
            Some(SampClientSdkHookAction::Block)
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
            .on_packet(SampClientSdkDirection::Outgoing, |_| {
                panic!("test callback panic")
            })
            .expect("test registration must succeed");

        assert_eq!(
            test_support::invoke_registered_callback(10),
            Some(SampClientSdkHookAction::Continue)
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
            .on_rpc_id(SampClientSdkDirection::Incoming, 42, move |_| {
                observed.fetch_add(1, Ordering::AcqRel);
                SampClientSdkHookAction::Block
            })
            .expect("test registration must succeed");

        assert_eq!(
            test_support::invoke_registered_callback(41),
            Some(SampClientSdkHookAction::Continue)
        );
        assert_eq!(
            test_support::invoke_registered_callback(42),
            Some(SampClientSdkHookAction::Block)
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
                SampClientSdkDirection::Incoming,
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
            Some(SampClientSdkHookAction::Continue)
        );
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(
                incoming::ENABLE_STUNT_BONUS.id(),
                incoming::ENABLE_STUNT_BONUS
                    .encode(api, true)
                    .expect("the typed test payload must encode"),
            ),
            Some(SampClientSdkHookAction::Block)
        );
        assert_eq!(
            test_support::invoke_registered_callback(incoming::ENABLE_STUNT_BONUS.id()),
            Some(SampClientSdkHookAction::Continue)
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
            packet(SampClientSdkDirection::Incoming, |_| SampClientSdkHookAction::Continue),
            rpc(SampClientSdkDirection::Outgoing, |_| SampClientSdkHookAction::Continue),
            packet_id(SampClientSdkDirection::Incoming, 1, |_| SampClientSdkHookAction::Continue),
            rpc_id(SampClientSdkDirection::Outgoing, 2, |_| SampClientSdkHookAction::Continue),
            typed_packet(
                SampClientSdkDirection::Incoming,
                packet::incoming::CONNECTION_ACCEPTED,
                |_| RpcAction::Continue
            ),
            typed_rpc(
                SampClientSdkDirection::Outgoing,
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
            api.on_packet(SampClientSdkDirection::Incoming, |_| {
                SampClientSdkHookAction::Continue
            })
            .expect("test registration must succeed"),
        );
        subscriptions.push(
            api.on_rpc(SampClientSdkDirection::Outgoing, |_| {
                SampClientSdkHookAction::Continue
            })
            .expect("test registration must succeed"),
        );
        test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

        let error = subscriptions
            .unregister_and_wait()
            .expect_err("failed callbacks must remain available for retry");
        assert_eq!(error.failures().len(), 2);
        assert!(
            error
                .failures()
                .iter()
                .all(|failure| failure.result() == SampClientSdkResult::CallbackInProgress)
        );
        assert_eq!(test_support::registration_stats().registered_callbacks, 2);

        test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
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
            .on_packet(SampClientSdkDirection::Incoming, |_| {
                SampClientSdkHookAction::Continue
            })
            .expect("test registration must succeed");

        let error = SubscriptionSet::new()
            .try_add(Ok(subscription))
            .and_then(|subscriptions| subscriptions.try_add(Err(SampClientSdkResult::NotReady)))
            .expect_err("the synthetic second registration must fail");
        assert_eq!(error.result(), SampClientSdkResult::NotReady);
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
            .on_rpc(SampClientSdkDirection::Incoming, |_| {
                SampClientSdkHookAction::Continue
            })
            .expect("test registration must succeed");
        test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

        let error = subscription
            .unregister_and_wait()
            .expect_err("callback-thread shutdown must remain retryable");
        assert_eq!(error.result(), SampClientSdkResult::CallbackInProgress);
        let subscription = error.into_subscription();
        test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
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
        test_support::set_register_result(SampClientSdkResult::NotReady);
        let drops = Arc::new(AtomicUsize::new(0));
        let counter = DropCounter(Arc::clone(&drops));

        let result =
            test_support::test_api().on_packet(SampClientSdkDirection::Incoming, move |_| {
                let _ = &counter;
                SampClientSdkHookAction::Continue
            });
        assert_eq!(result.unwrap_err(), SampClientSdkResult::NotReady);
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
            .on_packet(SampClientSdkDirection::Incoming, move |_| {
                let _ = &counter;
                SampClientSdkHookAction::Continue
            })
            .expect("test registration must succeed");

        drop(subscription);
        assert_eq!(drops.load(Ordering::Acquire), 0);
        assert_eq!(test_support::invoke_registered_callback(1), None);
        assert_eq!(test_support::registration_stats().unregister_calls, 1);
    }
}
