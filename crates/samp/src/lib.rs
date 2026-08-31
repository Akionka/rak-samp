//! Safe service-backed SA-MP facade for independently loaded Rust plugins.

#![deny(unsafe_op_in_unsafe_fn)]

mod chat;
mod error;
pub mod events;
mod network;
mod players;
mod pools;
pub mod raw;
mod receipt;
mod text_labels;
mod textdraws;
mod types;
mod ui;

pub use chat::{Chat, ChatCommandRegistration, ChatEntry, ChatStyle, DeathMessage};
pub use error::{ConnectError, ProtocolSendError};
pub use network::{Action, Direction, Event, Net, SendOptions};
pub use players::{
    AimSync, Animation, Animations, InCarSync, Local, OnFootSync, PassengerSync, Player, Players,
    RemotePlayerState, SpecialAction, TrailerSync,
};
pub use pools::{
    Gangzone, GangzoneId, Gangzones, ObjectHandle, ObjectId, Objects, PedHandle, PickupHandle,
    PickupId, Pickups, Pools, VehicleHandle, Vehicles,
};
pub use receipt::{CommandReceipt, Subscription, TextLabelCreateReceipt};
pub use text_labels::Labels;
pub use textdraws::{Textdraw, TextdrawId, Textdraws};
pub use types::{
    ClientVersion, GameState, LocalPlayer, PlayerId, PlayerInfo, SendRateKind, ServerInfo,
    TextLabel, TextLabelId, Vector3, VehicleId,
};
pub use ui::{
    ChatDisplayMode, ChatInput, Cursor, CursorMode, DialogRequest, DialogResponse, DialogState,
    DialogStyle, Dialogs, Scoreboard, Ui,
};

use modkit_sdk::{Core, Host, SampNetService, SampService};
use std::time::Duration;

/// Connected access to the versioned SA-MP services.
#[derive(Clone, Copy)]
pub struct Samp {
    core: Core,
    service: SampService,
    net: SampNetService,
    labels: modkit_sdk::SampTextLabelService,
    control: modkit_sdk::SampControlService,
    ui: modkit_sdk::SampUiService,
    players: modkit_sdk::SampPlayerService,
    pools: modkit_sdk::SampPoolService,
    textdraws: modkit_sdk::SampTextdrawService,
    codec: modkit_sdk::SampCodecService,
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
            labels: host.samp_text_labels().map_err(ConnectError::Service)?,
            control: host.samp_control().map_err(ConnectError::Service)?,
            ui: host.samp_ui().map_err(ConnectError::Service)?,
            players: host.samp_players().map_err(ConnectError::Service)?,
            pools: host.samp_pools().map_err(ConnectError::Service)?,
            textdraws: host.samp_textdraws().map_err(ConnectError::Service)?,
            codec: host.samp_codec().map_err(ConnectError::Service)?,
        })
    }

    pub fn version(self) -> Result<ClientVersion, modkit_abi::ModResult> {
        ClientVersion::from_raw(self.service.version()?).ok_or(modkit_abi::MOD_NATIVE_CALL_FAILED)
    }

    pub fn game_state(self) -> Result<i32, modkit_abi::ModResult> {
        self.service.game_state()
    }

    pub fn set_game_state(self, state: GameState) -> Result<CommandReceipt, modkit_abi::ModResult> {
        CommandReceipt::new(self.core, self.control.submit_game_state(state.raw())?)
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
        Chat::new(self.core, self.service, self.ui)
    }

    #[must_use]
    pub const fn net(self) -> Net {
        Net::new(self.core, self.net, self.control, self.codec)
    }

    #[must_use]
    pub const fn labels(self) -> Labels {
        Labels::new(self.core, self.labels)
    }
    #[must_use]
    pub const fn ui(self) -> Ui {
        Ui::new(self.core, self.ui)
    }

    #[must_use]
    pub const fn local(self) -> Local {
        Local::new(self.core, self.players)
    }

    #[must_use]
    pub const fn players(self) -> Players {
        Players::new(self.core, self.players, self.pools)
    }

    #[must_use]
    pub const fn animations(self) -> Animations {
        Animations::new(self.players)
    }

    #[must_use]
    pub const fn pools(self) -> Pools {
        Pools::new(self.pools)
    }

    #[must_use]
    pub const fn objects(self) -> Objects {
        self.pools().objects()
    }

    #[must_use]
    pub const fn pickups(self) -> Pickups {
        self.pools().pickups()
    }

    #[must_use]
    pub const fn vehicles(self) -> Vehicles {
        self.pools().vehicles()
    }

    #[must_use]
    pub const fn gangzones(self) -> Gangzones {
        self.pools().gangzones()
    }

    #[must_use]
    pub const fn textdraws(self) -> Textdraws {
        Textdraws::new(self.core, self.textdraws)
    }
}
