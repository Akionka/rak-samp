use super::{Net, PedHandle, PlayerId, Samp, VehicleId};
use crate::{
    AimSync, CommandReceipt, HostApi, InCarSync, LocalAnimation, LocalPlayer, OnFootSync,
    PassengerSync, PlayerInfo, RemotePlayerState, SampClientSdkResult, SpecialAction, TrailerSync,
};

#[derive(Clone, Copy)]
pub struct Local {
    api: HostApi,
}

impl Local {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn player(self) -> Result<LocalPlayer, SampClientSdkResult> {
        self.api.local_player()
    }

    /// Queues the R1 local-player spawn path.
    pub fn spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_player_spawn()
    }

    /// Queues one established R1 local-player special action.
    pub fn set_special_action(
        self,
        action: SpecialAction,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_player_special_action(action)
    }

    /// Queues a documented R1 local-player nickname update.
    pub fn set_nickname(self, name: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_player_name(name)
    }

    /// Queues the documented R1 unoccupied-vehicle synchronization send.
    pub fn force_unoccupied_sync(
        self,
        vehicle: VehicleId,
        seat: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_force_unoccupied_sync(vehicle.get(), seat)
    }
    /// Queues the documented R1 aim synchronization send.
    pub fn force_aim_sync(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_force_aim_sync()
    }
    /// Queues the documented R1 on-foot synchronization send.
    pub fn force_onfoot_sync(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_force_onfoot_sync()
    }
    /// Queues the documented R1 stats synchronization send.
    pub fn force_stats_sync(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_force_stats_sync()
    }

    /// Queues one documented R1 trailer synchronization send.
    pub fn force_trailer_sync(
        self,
        trailer: VehicleId,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_force_trailer_sync(trailer.get())
    }

    /// Queues the protocol-level class request without changing local class state.
    pub fn request_class(self, class_id: i32) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net::from_api(self.api).send_request_class(class_id)
    }

    /// Queues the protocol-level interior-change RPC without changing GTA state.
    pub fn send_interior_change(
        self,
        interior_id: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net::from_api(self.api).send_interior_change(interior_id)
    }

    /// Queues the protocol-level spawn RPC without invoking native spawn code.
    pub fn send_spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net::from_api(self.api).send_spawn()
    }

    /// Queues the protocol-level enter-vehicle RPC without changing the local ped.
    pub fn send_enter_vehicle(
        self,
        vehicle: VehicleId,
        passenger: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net::from_api(self.api).send_enter_vehicle(vehicle.get(), passenger)
    }

    /// Queues the protocol-level exit-vehicle RPC without changing the local ped.
    pub fn send_exit_vehicle(
        self,
        vehicle: VehicleId,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net::from_api(self.api).send_exit_vehicle(vehicle.get())
    }
}

#[derive(Clone, Copy)]
pub struct Players {
    api: HostApi,
}

impl Players {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    #[must_use]
    pub fn player(self, id: PlayerId) -> Player {
        Player { api: self.api, id }
    }

    pub fn get(self, id: PlayerId) -> Result<Option<PlayerInfo>, SampClientSdkResult> {
        self.api.player_info(id.get())
    }

    pub fn remote_state(
        self,
        id: PlayerId,
    ) -> Result<Option<RemotePlayerState>, SampClientSdkResult> {
        self.api.remote_player_state(id.get())
    }

    pub fn is_defined(self, id: PlayerId) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_defined(id.get())
    }

    pub fn is_paused(self, id: PlayerId) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_paused(id.get())
    }

    pub fn count(self, include_npcs: bool) -> Result<u16, SampClientSdkResult> {
        self.api.player_count(include_npcs)
    }

    pub fn max_id(self) -> Result<Option<PlayerId>, SampClientSdkResult> {
        self.api.player_max_id().map(PlayerId::new)
    }
}

/// Safe, nonblocking view of one checked SA-MP player-pool entry.
#[derive(Clone, Copy)]
pub struct Player {
    api: HostApi,
    id: PlayerId,
}

impl Player {
    #[must_use]
    pub const fn id(self) -> PlayerId {
        self.id
    }

    pub fn is_connected(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_connected(self.id.get())
    }

    pub fn nickname(self) -> Result<Option<Vec<u8>>, SampClientSdkResult> {
        self.api.player_nickname(self.id.get())
    }

    pub fn is_npc(self) -> Result<Option<bool>, SampClientSdkResult> {
        self.api.is_player_npc(self.id.get())
    }

    pub fn score(self) -> Result<Option<i32>, SampClientSdkResult> {
        self.api.player_score(self.id.get())
    }

    pub fn ping(self) -> Result<Option<u32>, SampClientSdkResult> {
        self.api.player_ping(self.id.get())
    }

    pub fn armour(self) -> Result<Option<f32>, SampClientSdkResult> {
        self.api.player_armour(self.id.get())
    }

    pub fn health(self) -> Result<Option<f32>, SampClientSdkResult> {
        self.api.player_health(self.id.get())
    }

    pub fn is_paused(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_paused(self.id.get())
    }

    pub fn special_action(self) -> Result<Option<u8>, SampClientSdkResult> {
        self.api.player_special_action(self.id.get())
    }

    pub fn animation_id(self) -> Result<Option<u16>, SampClientSdkResult> {
        self.api.player_animation_id(self.id.get())
    }

    /// Returns an owned on-foot synchronization snapshot for this local or
    /// remote player after the host has completed a game-thread refresh.
    pub fn onfoot_sync(self) -> Result<Option<OnFootSync>, SampClientSdkResult> {
        self.api.onfoot_sync(self.id.get())
    }

    /// Returns an owned in-car synchronization snapshot for this local or
    /// remote player after the host has completed a game-thread refresh.
    pub fn vehicle_sync(self) -> Result<Option<InCarSync>, SampClientSdkResult> {
        self.api.vehicle_sync(self.id.get())
    }

    /// Returns an owned passenger synchronization snapshot for this local or
    /// remote player after the host has completed a game-thread refresh.
    pub fn passenger_sync(self) -> Result<Option<PassengerSync>, SampClientSdkResult> {
        self.api.passenger_sync(self.id.get())
    }

    /// Returns an owned trailer synchronization snapshot for this local or
    /// remote player after the host has completed a game-thread refresh.
    pub fn trailer_sync(self) -> Result<Option<TrailerSync>, SampClientSdkResult> {
        self.api.trailer_sync(self.id.get())
    }

    /// Returns an owned aim synchronization snapshot after a game-thread refresh.
    pub fn aim_sync(self) -> Result<Option<AimSync>, SampClientSdkResult> {
        self.api.aim_sync(self.id.get())
    }

    pub fn is_defined(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_defined(self.id.get())
    }

    pub fn colour(self) -> Result<Option<u32>, SampClientSdkResult> {
        self.api.player_colour(self.id.get())
    }

    /// Queues a documented R1 local- or remote-player colour change.
    pub fn set_colour(self, colour: u32) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_player_colour(self.id.get(), colour)
    }

    /// Returns the cached GTA SA ped handle for this player.
    pub fn ped_handle(self) -> Result<Option<PedHandle>, SampClientSdkResult> {
        self.api
            .player_ped_handle(self.id.get())
            .map(|handle| handle.and_then(|handle| PedHandle::new(handle as u32)))
    }
}

impl PedHandle {
    /// Resolves this GTA SA ped handle back to a checked player-pool ID.
    pub fn to_id(self, samp: Samp) -> Result<Option<PlayerId>, SampClientSdkResult> {
        samp.api()
            .player_id_by_ped_handle(self.get() as i32)
            .map(|id| id.and_then(PlayerId::new))
    }
}

#[derive(Clone, Copy)]
pub struct Anim {
    api: HostApi,
}

/// Compatibility spelling for the `samp.anim()` facade.
pub type Animations = Anim;

impl Anim {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn get(self, id: u16) -> Result<LocalAnimation, SampClientSdkResult> {
        self.api.local_animation(id)
    }

    pub fn find(self, name: &[u8], file: &[u8]) -> Result<Option<u16>, SampClientSdkResult> {
        self.api.local_animation_id(name, file)
    }
}
