use crate::{
    BitStream, Direction, ListenerHandle, PacketEvent, RpcEvent, SampVersion, event::Registry,
    platform,
};
use core::fmt;
use std::sync::Arc;

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

/// Host-owned directory data copied for either the local or one remote R1
/// player. It deliberately omits every native and GTA pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerInfoSnapshot {
    pub(crate) id: u16,
    pub(crate) nickname: Vec<u8>,
    pub(crate) is_local: bool,
    pub(crate) is_npc: bool,
    pub(crate) colour: u32,
    pub(crate) score: i32,
    pub(crate) ping: u32,
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
                formatter.write_str("rak_samp requires a 32-bit Windows process")
            }
            Self::SampNotLoaded => formatter.write_str("samp.dll is not loaded"),
            Self::UnsupportedClient { entry_point } => {
                write!(
                    formatter,
                    "unsupported samp.dll entry point RVA: 0x{entry_point:X}"
                )
            }
            Self::ClientNotReady => formatter.write_str("the SA-MP RakClient is not ready yet"),
            Self::AlreadyAttached => formatter.write_str("a rak_samp runtime is already attached"),
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

    /// Sends a packet through the original SA-MP RakClient method.
    ///
    /// This bypasses outgoing listeners to prevent recursive hook dispatch.
    pub fn send_packet(&self, packet_id: u8, payload: &BitStream) -> Result<bool, SendError> {
        self.backend
            .send_packet(packet_id, payload, SendOptions::default())
    }

    /// Sends a packet with explicit RakNet delivery settings.
    pub fn send_packet_with_options(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        validate_packet_options(options)?;
        self.backend.send_packet(packet_id, payload, options)
    }

    /// Sends an RPC through the original SA-MP RakClient method.
    ///
    /// This bypasses outgoing listeners to prevent recursive hook dispatch.
    pub fn send_rpc(&self, rpc_id: u8, payload: &BitStream) -> Result<bool, SendError> {
        self.backend
            .send_rpc(rpc_id, payload, SendOptions::default())
    }

    /// Sends an RPC with explicit RakNet delivery settings.
    pub fn send_rpc_with_options(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.backend.send_rpc(rpc_id, payload, options)
    }

    /// Queues a packet for the client; incoming listeners run once when it is dequeued.
    pub fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.backend.emulate_incoming_packet(packet_id, payload)
    }

    /// Delivers an RPC to the client after incoming listeners run.
    pub fn emulate_incoming_rpc(&self, rpc_id: u8, payload: BitStream) -> Result<bool, SendError> {
        self.backend.emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn client_hook_status(&self) -> ClientHookStatus {
        self.backend.client_hook_status()
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

    pub(crate) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        self.backend.local_player()
    }

    pub(crate) fn player_info(
        &self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        self.backend.player_info(id)
    }

    pub(crate) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        self.backend.player_count(include_npcs)
    }

    pub(crate) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        self.backend.player_max_id()
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

    pub(crate) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_chat_input_active()
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
