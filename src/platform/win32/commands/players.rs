use super::*;

#[derive(Debug)]
pub(in crate::platform::win32) enum PlayerCommand {
    SpawnLocalPlayer,
    SetLocalPlayerSpecialAction(u8),
    SetLocalPlayerName(Vec<u8>),
    ForceUnoccupiedSync { vehicle: u16, seat: u8 },
    ForceAimSync,
    ForceOnfootSync,
    ForceStatsSync,
    ForceTrailerSync { trailer: u16 },
    ForcePassengerSync { vehicle: u16, seat: u8 },
    ForceWeaponsSync,
    ForceVehicleSync { vehicle: u16 },
    SetPlayerColour { id: u16, colour: u32 },
}

impl BackendState {
    fn queue_player_command(&self, command: PlayerCommand) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::Player(command))
    }
    pub(in crate::platform::win32) fn submit_local_player_spawn(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_player_command(PlayerCommand::SpawnLocalPlayer)
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
        self.queue_player_command(PlayerCommand::SetLocalPlayerSpecialAction(action))
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
        self.queue_player_command(PlayerCommand::SetLocalPlayerName(name))
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
        self.queue_player_command(PlayerCommand::ForceUnoccupiedSync { vehicle, seat })
    }

    pub(in crate::platform::win32) fn submit_force_aim_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_player_command(PlayerCommand::ForceAimSync)
    }
    pub(in crate::platform::win32) fn submit_force_onfoot_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_player_command(PlayerCommand::ForceOnfootSync)
    }
    pub(in crate::platform::win32) fn submit_force_stats_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_player_command(PlayerCommand::ForceStatsSync)
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
        self.queue_player_command(PlayerCommand::ForceTrailerSync { trailer })
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
        self.queue_player_command(PlayerCommand::ForceVehicleSync { vehicle })
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
        self.queue_player_command(PlayerCommand::ForcePassengerSync { vehicle, seat })
    }

    pub(in crate::platform::win32) fn submit_force_weapons_sync(
        &self,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() || self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_player_command(PlayerCommand::ForceWeaponsSync)
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
        self.queue_player_command(PlayerCommand::SetPlayerColour { id, colour })
    }

    pub(super) fn execute_player_command(
        &self,
        command: PlayerCommand,
    ) -> Result<(), CommandError> {
        match command {
            PlayerCommand::SpawnLocalPlayer => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .spawn_local_player()
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::SetLocalPlayerSpecialAction(action) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_local_player_special_action(action)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::SetLocalPlayerName(name) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_local_player_name(&name)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceUnoccupiedSync { vehicle, seat } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_unoccupied_sync(vehicle, seat)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceAimSync => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_aim_sync()
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceOnfootSync => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_onfoot_sync()
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceStatsSync => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_stats_sync()
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceTrailerSync { trailer } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_trailer_sync(trailer)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceVehicleSync { vehicle } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_vehicle_sync(vehicle)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForcePassengerSync { vehicle, seat } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_passenger_sync(vehicle, seat)
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::ForceWeaponsSync => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .force_weapons_sync()
                        .map_err(|_| CommandError::NativeFailure)
                }),
            PlayerCommand::SetPlayerColour { id, colour } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_player_colour(id, colour)
                        .map_err(|_| CommandError::NativeFailure)
                }),
        }
    }
}
