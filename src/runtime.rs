use crate::{
    BitStream, Direction, ListenerHandle, PacketEvent, RpcEvent, event::Registry, platform,
};
use core::fmt;
use std::sync::Arc;

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
                formatter.write_str("rak_rs requires a 32-bit Windows process")
            }
            Self::SampNotLoaded => formatter.write_str("samp.dll is not loaded"),
            Self::UnsupportedClient { entry_point } => {
                write!(
                    formatter,
                    "unsupported samp.dll entry point RVA: 0x{entry_point:X}"
                )
            }
            Self::ClientNotReady => formatter.write_str("the SA-MP RakClient is not ready yet"),
            Self::AlreadyAttached => formatter.write_str("a rak_rs runtime is already attached"),
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
        PacketPriority, PacketReliability, SendError, SendOptions, validate_packet_options,
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
}
