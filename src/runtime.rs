use crate::{
    BitStream, Direction, ListenerHandle, PacketEvent, RpcEvent, SampVersion,
    command::{CommandError, CommandId},
    event::Registry,
    platform,
};
use core::fmt;
use std::{sync::Arc, time::Duration};

/// A copied dialog request that is safe to retain until the game-thread pump
/// can call the private native client backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalDialogRequest {
    pub(crate) id: u16,
    pub(crate) style: LocalDialogStyle,
    pub(crate) title: Vec<u8>,
    pub(crate) text: Vec<u8>,
    pub(crate) button1: Vec<u8>,
    pub(crate) button2: Vec<u8>,
}

/// A copied chat entry that is safe to retain until the game-thread pump can
/// call the private R1 chat backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalChatMessageRequest {
    pub(crate) style: LocalChatMessageStyle,
    pub(crate) text: Vec<u8>,
    pub(crate) prefix: Vec<u8>,
    pub(crate) text_colour: u32,
    pub(crate) prefix_colour: u32,
}

/// A copied death-window entry that is safe to retain until the game-thread
/// pump can call the private R1 death-window backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalDeathMessageRequest {
    pub(crate) killer: Vec<u8>,
    pub(crate) victim: Vec<u8>,
    pub(crate) killer_colour: u32,
    pub(crate) victim_colour: u32,
    pub(crate) weapon: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalDialogStyle {
    MessageBox,
    Input,
    List,
    Password,
    TabList,
    HeadersList,
}

impl LocalDialogStyle {
    pub(crate) const fn from_raw(value: u32) -> Option<Self> {
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

/// Host-owned data copied from the verified R1 game-thread client state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LocalPlayerSnapshot {
    pub(crate) id: u16,
    pub(crate) nickname: Vec<u8>,
    pub(crate) colour: u32,
    pub(crate) spawned: bool,
    pub(crate) health: f32,
    pub(crate) armour: f32,
    pub(crate) position: Vector3,
    pub(crate) velocity: Vector3,
    pub(crate) special_action: u8,
    pub(crate) animation_id: u16,
    pub(crate) vehicle_id: Option<u16>,
    pub(crate) score: i32,
    pub(crate) ping: u32,
}

/// Host-owned metadata copied from one active R1 dialog. The dynamic dialog
/// text, editbox contents, and listbox item strings are bounded copies made on
/// the verified game thread; no native or DXUT pointer crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalDialogSnapshot {
    pub(crate) id: i32,
    pub(crate) style: LocalDialogStyle,
    pub(crate) title: Vec<u8>,
    pub(crate) server_side: bool,
    pub(crate) selected_item: Option<i32>,
    pub(crate) list_item_count: Option<i32>,
    /// Bounded copy of the active dialog's dynamically allocated text.
    pub(crate) text: Vec<u8>,
    /// Bounded copy of the active dialog's editbox text. `None` marks dialogs
    /// without an editbox (for example a message box).
    pub(crate) editbox_text: Option<Vec<u8>>,
    /// Bounded copies of the active dialog's listbox item strings.
    pub(crate) listbox_items: Vec<Vec<u8>>,
}

/// Host-owned directory data copied for either the local or one remote R1
/// player. It deliberately omits every native and GTA pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerInfoSnapshot {
    pub(crate) id: u16,
    pub(crate) defined: bool,
    pub(crate) paused: bool,
    pub(crate) nickname: Vec<u8>,
    pub(crate) is_local: bool,
    pub(crate) is_npc: bool,
    pub(crate) colour: u32,
    pub(crate) score: i32,
    pub(crate) ping: u32,
}

/// Host-owned volatile state copied from one remote R1 player record on the
/// verified game thread. It deliberately contains no player, ped, or GTA
/// pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RemotePlayerStateSnapshot {
    pub(crate) id: u16,
    pub(crate) health: f32,
    pub(crate) armour: f32,
    pub(crate) special_action: u8,
    pub(crate) animation_id: u16,
}

/// Host-owned gangzone data copied from the verified R1 game thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GangzoneSnapshot {
    pub(crate) id: u16,
    pub(crate) left: f32,
    pub(crate) bottom: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) colour: u32,
    pub(crate) alternate_colour: u32,
}

/// Host-owned data copied from one R1 3D text-label record on the verified
/// game thread. The dynamically allocated native text is copied within its
/// protocol-bounded lifetime; no native pointer crosses this boundary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextLabelSnapshot {
    pub(crate) id: u16,
    pub(crate) text: Vec<u8>,
    pub(crate) colour: u32,
    pub(crate) position: Vector3,
    pub(crate) draw_distance: f32,
    pub(crate) behind_walls: bool,
    pub(crate) attached_player_id: Option<u16>,
    pub(crate) attached_vehicle_id: Option<u16>,
}

/// Host-owned data copied from one R1 textdraw record on the verified game
/// thread. The fixed native display-string buffer is copied before this value
/// crosses the private profile boundary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextdrawSnapshot {
    pub(crate) pool_index: u16,
    pub(crate) text: Vec<u8>,
    pub(crate) letter_width: f32,
    pub(crate) letter_height: f32,
    pub(crate) letter_colour: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) shadow: u8,
    pub(crate) outline: u8,
    pub(crate) background_colour: u32,
    pub(crate) style: i32,
    pub(crate) proportional: bool,
    pub(crate) align_left: bool,
    pub(crate) align_center: bool,
    pub(crate) align_right: bool,
    pub(crate) box_enabled: bool,
    pub(crate) box_width: f32,
    pub(crate) box_height: f32,
    pub(crate) box_colour: u32,
    pub(crate) model_id: u16,
    pub(crate) rotation: Vector3,
    pub(crate) zoom: f32,
    pub(crate) model_colour1: u16,
    pub(crate) model_colour2: u16,
}

/// Host-owned data copied from one R1 fixed chat-history entry on the
/// verified game thread. No native chat or UI pointer crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChatEntrySnapshot {
    pub(crate) id: u16,
    pub(crate) text: Vec<u8>,
    pub(crate) prefix: Vec<u8>,
    pub(crate) text_colour: u32,
    pub(crate) prefix_colour: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalChatMessageStyle {
    Chat,
    Info,
    Debug,
}

impl LocalChatMessageStyle {
    pub(crate) const fn from_raw(value: u32) -> Option<Self> {
        match value {
            2 => Some(Self::Chat),
            4 => Some(Self::Info),
            8 => Some(Self::Debug),
            _ => None,
        }
    }

    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::Chat => 2,
            Self::Info => 4,
            Self::Debug => 8,
        }
    }
}

/// Host-owned current-server metadata copied from the verified R1 game thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServerInfoSnapshot {
    pub(crate) address: Vec<u8>,
    pub(crate) hostname: Vec<u8>,
    pub(crate) port: u16,
}

/// One owned entry from R1's fixed animation-name table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AnimationSnapshot {
    pub(crate) name: Vec<u8>,
    pub(crate) file: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vector3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

/// Failures specific to the direct, profile-gated client helpers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DirectClientError {
    NotReady,
    UnsupportedVersion,
    QueueFull,
}

/// Failure to attach the SDK to a compatible SA-MP client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachError {
    UnsupportedPlatform,
    SampNotLoaded,
    UnsupportedClient { entry_point: u32 },
    ClientNotReady,
    AlreadyAttached,
    HookInstallFailed(&'static str),
}

impl fmt::Display for AttachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("samp_client_sdk requires a 32-bit Windows process")
            }
            Self::SampNotLoaded => formatter.write_str("samp.dll is not loaded"),
            Self::UnsupportedClient { entry_point } => {
                write!(
                    formatter,
                    "unsupported samp.dll entry point RVA: 0x{entry_point:X}"
                )
            }
            Self::ClientNotReady => formatter.write_str("the SA-MP RakClient is not ready yet"),
            Self::AlreadyAttached => {
                formatter.write_str("a samp_client_sdk runtime is already attached")
            }
            Self::HookInstallFailed(detail) => {
                write!(formatter, "failed to install SA-MP hook: {detail}")
            }
        }
    }
}

impl std::error::Error for AttachError {}

/// Reliability priority used by [`SendOptions`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketPriority {
    System,
    High,
    Medium,
    Low,
}

/// Delivery behavior used by [`SendOptions`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketReliability {
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,
}

/// RakNet delivery options for raw packet and RPC sends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendOptions {
    pub priority: PacketPriority,
    pub reliability: PacketReliability,
    pub ordering_channel: u8,
    pub timestamp: bool,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            priority: PacketPriority::High,
            reliability: PacketReliability::ReliableOrdered,
            ordering_channel: 0,
            timestamp: false,
        }
    }
}

/// Failure to send or locally emulate network traffic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SendError {
    ClientNotReady,
    QueueFull,
    PayloadTooLarge,
    NativeCallFailed,
    TimestampedPacketUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CodecError {
    ClientNotReady,
    InvalidArgument,
    PayloadTooLarge,
    NativeCallFailed,
}

impl fmt::Display for SendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientNotReady => formatter.write_str("the SA-MP client hook is not ready"),
            Self::QueueFull => formatter.write_str("the game-thread command queue is full"),
            Self::PayloadTooLarge => {
                formatter.write_str("the payload does not fit into the native bit stream")
            }
            Self::NativeCallFailed => {
                formatter.write_str("the SA-MP client rejected the network operation")
            }
            Self::TimestampedPacketUnsupported => {
                formatter.write_str("timestamped packet sends are not supported")
            }
        }
    }
}

impl std::error::Error for SendError {}

/// A live SA-MP hook runtime.
///
/// Only one runtime may be attached in a process. Drop it before unloading the
/// containing ASI/DLL so native detours and vtable changes are restored.
pub struct Runtime {
    registry: Arc<Registry>,
    backend: platform::Backend,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientHookStatus {
    Pending,
    Ready,
    Failed,
}

impl Runtime {
    /// Installs the startup hook for a supported SA-MP client.
    ///
    /// Call this after `samp.dll` loads but before RakClient construction. A
    /// host ASI/DLL should wait for `samp.dll`, then attach immediately rather
    /// than waiting for the normal game-loop callback.
    pub fn attach() -> Result<Self, AttachError> {
        let registry = Registry::new();
        let backend = platform::attach(Arc::clone(&registry))?;
        Ok(Self { registry, backend })
    }

    /// Registers a synchronous packet listener.
    pub fn on_packet(
        &self,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut PacketEvent<'event>) -> crate::HookAction + Send + 'static,
    ) -> ListenerHandle {
        self.registry.register_packet(direction, callback)
    }

    /// Registers a synchronous RPC listener.
    pub fn on_rpc(
        &self,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut RpcEvent<'event>) -> crate::HookAction + Send + 'static,
    ) -> ListenerHandle {
        self.registry.register_rpc(direction, callback)
    }

    /// Queues a packet for the original SA-MP RakClient method on the game thread.
    ///
    /// This bypasses outgoing listeners to prevent recursive hook dispatch.
    pub fn send_packet(&self, packet_id: u8, payload: &BitStream) -> Result<bool, SendError> {
        self.backend
            .send_packet(packet_id, payload, SendOptions::default())
    }

    /// Queues a packet with explicit RakNet delivery settings for the game thread.
    pub fn send_packet_with_options(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        validate_packet_options(options)?;
        self.backend.send_packet(packet_id, payload, options)
    }

    /// Queues an RPC for the original SA-MP RakClient method on the game thread.
    ///
    /// This bypasses outgoing listeners to prevent recursive hook dispatch.
    pub fn send_rpc(&self, rpc_id: u8, payload: &BitStream) -> Result<bool, SendError> {
        self.backend
            .send_rpc(rpc_id, payload, SendOptions::default())
    }

    /// Queues an RPC with explicit RakNet delivery settings for the game thread.
    pub fn send_rpc_with_options(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.backend.send_rpc(rpc_id, payload, options)
    }

    pub(crate) fn submit_packet_with_options(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        validate_packet_options(options)?;
        self.backend.submit_packet(packet_id, payload, options)
    }

    pub(crate) fn submit_rpc_with_options(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.backend.submit_rpc(rpc_id, payload, options)
    }

    /// Queues a packet for game-thread emulation; incoming listeners run once then.
    pub fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.backend.emulate_incoming_packet(packet_id, payload)
    }

    /// Queues an RPC for game-thread delivery after incoming listeners run.
    pub fn emulate_incoming_rpc(&self, rpc_id: u8, payload: BitStream) -> Result<bool, SendError> {
        self.backend.emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn submit_emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.backend
            .submit_emulate_incoming_packet(packet_id, payload)
    }

    pub(crate) fn submit_emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.backend.submit_emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn client_hook_status(&self) -> ClientHookStatus {
        self.backend.client_hook_status()
    }

    pub(crate) fn raw_rakclient(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_rakclient()
    }

    pub(crate) fn raw_rakpeer(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_rakpeer()
    }

    pub(crate) fn raw_player_pool(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_player_pool()
    }

    pub(crate) fn raw_vehicle_pool(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_vehicle_pool()
    }

    pub(crate) fn raw_local_player(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_local_player()
    }

    pub(crate) fn samp_version(&self) -> SampVersion {
        self.backend.samp_version()
    }

    pub(crate) fn encode_string(&self, value: &[u8]) -> Result<BitStream, CodecError> {
        self.backend.encode_string(value)
    }

    pub(crate) fn decode_string(
        &self,
        payload: &mut BitStream,
        output: &mut [u8],
    ) -> Result<usize, CodecError> {
        self.backend.decode_string(payload, output)
    }

    pub(crate) fn show_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        self.backend.show_local_dialog(request)
    }

    pub(crate) fn show_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        self.backend.show_local_chat_message(request)
    }

    pub(crate) fn show_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        self.backend.show_local_death_message(request)
    }

    pub(crate) fn submit_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog(request)
    }

    pub(crate) fn submit_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_message(request)
    }

    pub(crate) fn submit_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_death_message(request)
    }

    pub(crate) fn submit_local_cursor_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_cursor_mode(mode)
    }

    pub(crate) fn submit_local_chat_display_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_display_mode(mode)
    }

    pub(crate) fn submit_local_chat_entry(
        &self,
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_local_chat_entry(id, text, prefix, text_colour, prefix_colour)
    }

    pub(crate) fn submit_local_dialog_close(
        &self,
        button: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_close(button)
    }

    pub(crate) fn submit_local_chat_input_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_input_text(text)
    }

    pub(crate) fn submit_local_chat_input_enabled(
        &self,
        enabled: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_input_enabled(enabled)
    }

    pub(crate) fn submit_local_chat_input_process(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_input_process(text)
    }

    pub(crate) fn submit_local_cursor_toggle(
        &self,
        show: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_cursor_toggle(show)
    }

    pub(crate) fn submit_local_scoreboard_open(
        &self,
        open: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_scoreboard_open(open)
    }

    pub(crate) fn submit_local_dialog_client_side(
        &self,
        client_side: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_client_side(client_side)
    }

    pub(crate) fn submit_local_dialog_selected_item(
        &self,
        selected: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_selected_item(selected)
    }

    pub(crate) fn submit_local_dialog_editbox_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_editbox_text(text)
    }

    pub(crate) fn submit_samp_game_state(
        &self,
        state: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_samp_game_state(state)
    }

    pub(crate) fn submit_connect_to_server(
        &self,
        address: Vec<u8>,
        port: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_connect_to_server(address, port)
    }

    pub(crate) fn submit_disconnect_with_reason(
        &self,
        block_duration: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_disconnect_with_reason(block_duration)
    }

    pub(crate) fn submit_delete_textdraw(&self, id: u16) -> Result<CommandId, DirectClientError> {
        self.backend.submit_delete_textdraw(id)
    }

    pub(crate) fn submit_delete_text_label(&self, id: u16) -> Result<CommandId, DirectClientError> {
        self.backend.submit_delete_text_label(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit_create_text_label(
        &self,
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_create_text_label(
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        )
    }

    pub(crate) fn submit_set_textdraw_position(
        &self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_position(id, x, y)
    }

    pub(crate) fn submit_set_textdraw_letter_style(
        &self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_letter_style(id, width, height, colour)
    }

    pub(crate) fn submit_set_textdraw_proportional(
        &self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_proportional(id, proportional)
    }

    pub(crate) fn submit_set_textdraw_shadow(
        &self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_shadow(id, shadow, colour)
    }

    pub(crate) fn submit_set_textdraw_outline(
        &self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_outline(id, outline, colour)
    }

    pub(crate) fn submit_set_textdraw_box(
        &self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_box(id, enabled, colour, width, height)
    }

    pub(crate) fn submit_set_textdraw_alignment(
        &self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_alignment(id, alignment)
    }

    pub(crate) fn submit_set_textdraw_string(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_string(id, text)
    }

    pub(crate) fn submit_set_textdraw_model_style(
        &self,
        id: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
    }

    pub(crate) fn submit_local_player_spawn(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_player_spawn()
    }

    pub(crate) fn submit_local_player_special_action(
        &self,
        action: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_player_special_action(action)
    }

    pub(crate) fn submit_local_player_name(
        &self,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_player_name(name)
    }

    pub(crate) fn submit_force_unoccupied_sync(
        &self,
        vehicle: u16,
        seat: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_unoccupied_sync(vehicle, seat)
    }

    pub(crate) fn submit_send_rate(
        &self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_send_rate(kind, milliseconds)
    }

    pub(crate) fn submit_player_colour(
        &self,
        id: u16,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_player_colour(id, colour)
    }

    pub(crate) fn try_take_command(
        &self,
        id: CommandId,
    ) -> Result<Option<Result<(), CommandError>>, CommandError> {
        self.backend.try_take_command(id)
    }

    pub(crate) fn wait_for_command(
        &self,
        id: CommandId,
        timeout: Duration,
    ) -> Result<Result<(), CommandError>, CommandError> {
        self.backend.wait_for_command(id, timeout)
    }

    pub(crate) fn release_command(&self, id: CommandId) -> Result<(), CommandError> {
        self.backend.release_command(id)
    }

    pub(crate) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        self.backend.local_player()
    }

    pub(crate) fn player_info(
        &self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        self.backend.player_info(id)
    }

    pub(crate) fn remote_player_state(
        &self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        self.backend.remote_player_state(id)
    }

    pub(crate) fn player_defined(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.player_defined(id)
    }

    pub(crate) fn player_paused(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.player_paused(id)
    }

    pub(crate) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        self.backend.player_count(include_npcs)
    }

    pub(crate) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        self.backend.player_max_id()
    }

    pub(crate) fn vehicle_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.vehicle_exists(id)
    }

    pub(crate) fn text_label_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.text_label_exists(id)
    }

    pub(crate) fn textdraw_exists(&self, pool_index: u16) -> Result<bool, DirectClientError> {
        self.backend.textdraw_exists(pool_index)
    }

    pub(crate) fn object_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.object_exists(id)
    }

    pub(crate) fn gangzone(&self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        self.backend.gangzone(id)
    }

    pub(crate) fn text_label(
        &self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        self.backend.text_label(id)
    }

    pub(crate) fn textdraw(
        &self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        self.backend.textdraw(pool_index)
    }

    pub(crate) fn chat_entry(&self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        self.backend.chat_entry(id)
    }

    pub(crate) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        self.backend.server_info()
    }

    pub(crate) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        self.backend.samp_game_state()
    }

    pub(crate) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        self.backend.local_chat_display_mode()
    }

    pub(crate) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        self.backend.local_cursor_mode()
    }

    pub(crate) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        self.backend.local_scoreboard_open()
    }

    pub(crate) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_dialog_active()
    }

    pub(crate) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        self.backend.local_dialog_state()
    }

    pub(crate) fn local_dialog_selected_item(&self) -> Result<i32, DirectClientError> {
        self.backend.local_dialog_selected_item()
    }

    pub(crate) fn local_dialog_list_item_count(&self) -> Result<i32, DirectClientError> {
        self.backend.local_dialog_list_item_count()
    }

    pub(crate) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_chat_input_active()
    }

    pub(crate) fn local_chat_input_text(&self) -> Result<Vec<u8>, DirectClientError> {
        self.backend.local_chat_input_text()
    }

    pub(crate) fn local_animation(&self, id: u16) -> Result<AnimationSnapshot, DirectClientError> {
        self.backend.local_animation(id)
    }

    pub(crate) fn local_animation_id(
        &self,
        name: &[u8],
        file: &[u8],
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.local_animation_id(name, file)
    }
}

fn validate_packet_options(options: SendOptions) -> Result<(), SendError> {
    if options.timestamp {
        Err(SendError::TimestampedPacketUnsupported)
    } else {
        Ok(())
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.backend.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LocalChatMessageStyle, LocalDialogStyle, PacketPriority, PacketReliability, SendError,
        SendOptions, validate_packet_options,
    };

    #[test]
    fn timestamped_packet_options_are_explicitly_unsupported() {
        let options = SendOptions {
            priority: PacketPriority::High,
            reliability: PacketReliability::ReliableOrdered,
            ordering_channel: 0,
            timestamp: true,
        };

        assert_eq!(
            validate_packet_options(options),
            Err(SendError::TimestampedPacketUnsupported)
        );
    }

    #[test]
    fn direct_dialog_style_accepts_only_the_six_native_values() {
        assert_eq!(
            LocalDialogStyle::from_raw(0),
            Some(LocalDialogStyle::MessageBox)
        );
        assert_eq!(
            LocalDialogStyle::from_raw(5),
            Some(LocalDialogStyle::HeadersList)
        );
        assert_eq!(LocalDialogStyle::from_raw(6), None);
    }

    #[test]
    fn direct_chat_style_accepts_only_the_three_native_values() {
        assert_eq!(
            LocalChatMessageStyle::from_raw(2),
            Some(LocalChatMessageStyle::Chat)
        );
        assert_eq!(
            LocalChatMessageStyle::from_raw(8),
            Some(LocalChatMessageStyle::Debug)
        );
        assert_eq!(LocalChatMessageStyle::from_raw(1), None);
    }
}
