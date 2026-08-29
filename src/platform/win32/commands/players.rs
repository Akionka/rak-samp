use super::*;

impl BackendState {
    pub(in crate::platform::win32) fn submit_local_player_spawn(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SpawnLocalPlayer)
    }

    pub(in crate::platform::win32) fn submit_local_player_special_action(
        &self,
        action: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(action, 0..=12 | 20..=25 | 68)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetLocalPlayerSpecialAction(action))
    }

    pub(in crate::platform::win32) fn submit_local_player_name(
        &self,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || name.len() > 255 || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetLocalPlayerName(name))
    }

    pub(in crate::platform::win32) fn submit_force_unoccupied_sync(
        &self,
        vehicle: u16,
        seat: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(vehicle) >= MAX_SAMP_VEHICLES
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceUnoccupiedSync { vehicle, seat })
    }

    pub(in crate::platform::win32) fn submit_force_aim_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceAimSync)
    }
    pub(in crate::platform::win32) fn submit_force_onfoot_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceOnfootSync)
    }
    pub(in crate::platform::win32) fn submit_force_stats_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceStatsSync)
    }

    pub(in crate::platform::win32) fn submit_force_trailer_sync(
        &self,
        trailer: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(trailer) >= MAX_SAMP_VEHICLES
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceTrailerSync { trailer })
    }

    pub(in crate::platform::win32) fn submit_force_vehicle_sync(
        &self,
        vehicle: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(vehicle) >= MAX_SAMP_VEHICLES
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceVehicleSync { vehicle })
    }

    pub(in crate::platform::win32) fn submit_force_passenger_sync(
        &self,
        vehicle: u16,
        seat: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(vehicle) >= MAX_SAMP_VEHICLES
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForcePassengerSync { vehicle, seat })
    }

    pub(in crate::platform::win32) fn submit_force_weapons_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceWeaponsSync)
    }

    pub(in crate::platform::win32) fn submit_player_colour(
        &self,
        id: u16,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetPlayerColour { id, colour })
    }
}
