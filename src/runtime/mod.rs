use crate::{event::Registry, platform};
use std::sync::Arc;

mod commands;
mod errors;
mod network;
mod options;
mod reads;
mod requests;
mod snapshots;

pub use errors::{AttachError, SendError};
pub(crate) use errors::{CodecError, DirectClientError};
use options::validate_packet_options;
pub use options::{PacketPriority, PacketReliability, SendOptions};
pub(crate) use requests::{
    LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest, LocalDialogRequest,
    LocalDialogStyle,
};
pub(crate) use snapshots::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, GangzoneSnapshot, InCarSyncSnapshot,
    LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot,
    PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot,
    TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
};
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
