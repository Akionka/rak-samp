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
