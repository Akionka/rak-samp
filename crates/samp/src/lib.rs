//! Safe service-backed SA-MP facade for independently loaded Rust plugins.

#![deny(unsafe_op_in_unsafe_fn)]

mod chat;
mod error;
mod network;
mod receipt;
mod types;

pub use chat::{Chat, ChatCommandRegistration};
pub use error::ConnectError;
pub use network::{Action, Direction, Event, Net, SendOptions};
pub use receipt::{CommandReceipt, Subscription};
pub use types::{ClientVersion, LocalPlayer, PlayerId, PlayerInfo, ServerInfo, Vector3};

use modkit_sdk::{Core, Host, SampNetService, SampService};
use std::time::Duration;

/// Connected access to the versioned SA-MP services.
#[derive(Clone, Copy)]
pub struct Samp {
    core: Core,
    service: SampService,
    net: SampNetService,
}

impl Samp {
    /// Connects to the default host and resolves every Phase 7 SA-MP service.
    ///
    /// This may block while the host initializes. Do not call it from `DllMain`.
    pub fn connect(timeout: Duration) -> Result<Self, ConnectError> {
        let host = Host::connect(timeout).map_err(ConnectError::Host)?;
        Self::from_host(host)
    }

    /// Resolves the SA-MP services from an existing host connection.
    pub fn from_host(host: Host) -> Result<Self, ConnectError> {
        Ok(Self {
            core: host.core().map_err(ConnectError::Service)?,
            service: host.samp().map_err(ConnectError::Service)?,
            net: host.samp_net().map_err(ConnectError::Service)?,
        })
    }

    pub fn version(self) -> Result<ClientVersion, modkit_abi::ModResult> {
        ClientVersion::from_raw(self.service.version()?).ok_or(modkit_abi::MOD_NATIVE_CALL_FAILED)
    }

    pub fn game_state(self) -> Result<i32, modkit_abi::ModResult> {
        self.service.game_state()
    }

    pub fn server_info(self) -> Result<ServerInfo, modkit_abi::ModResult> {
        ServerInfo::from_abi(self.service.server_info()?)
    }

    pub fn local_player(self) -> Result<LocalPlayer, modkit_abi::ModResult> {
        LocalPlayer::from_abi(self.service.local_player()?)
    }

    pub fn player(self, id: PlayerId) -> Result<Option<PlayerInfo>, modkit_abi::ModResult> {
        PlayerInfo::from_abi(self.service.player_info(id.get())?)
    }

    #[must_use]
    pub const fn chat(self) -> Chat {
        Chat::new(self.core, self.service)
    }

    #[must_use]
    pub const fn net(self) -> Net {
        Net::new(self.core, self.net)
    }
}
