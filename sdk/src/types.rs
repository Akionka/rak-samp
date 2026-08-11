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

    pub(crate) const fn as_raw(self) -> u32 {
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

    pub(crate) fn is_valid(self) -> bool {
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
    pub(crate) const fn as_raw(self) -> u32 {
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
    pub(crate) const fn from_raw(value: i32) -> Option<Self> {
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

    pub(crate) const fn from_raw(value: i32) -> Option<Self> {
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

    pub(crate) fn is_valid(self) -> bool {
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

    pub(crate) fn is_valid(self) -> bool {
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

/// An owned R1 on-foot synchronization record copied on the verified game
/// thread for either the local or a defined remote player.
///
/// The controller and animation fields preserve their native raw bit patterns;
/// `surfing_vehicle_id` preserves the native `0xFFFF` sentinel. The first
/// lookup can return [`SampClientSdkResult::NotReady`] while the game-thread
/// cache refresh is pending. No client, player, ped, or GTA pointer crosses
/// this API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnFootSync {
    pub id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub position: crate::Vector3,
    pub quaternion: [f32; 4],
    pub health: u8,
    pub armour: u8,
    pub weapon: u8,
    pub special_action: u8,
    pub speed: crate::Vector3,
    pub surfing_offset: crate::Vector3,
    pub surfing_vehicle_id: u16,
    pub animation: u32,
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

impl SampGameState {
    /// Returns the R1 scalar value written to `CNetGame`.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self as i32
    }
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
