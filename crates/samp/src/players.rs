use crate::{CommandReceipt, PedHandle, PlayerId, Vector3, VehicleId};
use modkit_abi::{
    MOD_NATIVE_CALL_FAILED, ModResult, SAMP_MAX_ANIMATION_NAME_BYTES, SampAimSyncV1,
    SampInCarSyncV1, SampOnFootSyncV1, SampPassengerSyncV1, SampRemotePlayerStateV1,
    SampTrailerSyncV1, SampVector3V1,
};
use modkit_sdk::{Core, SampPlayerService, SampPoolService};

#[derive(Clone, Copy)]
pub struct Local {
    core: Core,
    service: SampPlayerService,
}

#[derive(Clone, Copy)]
pub struct Players {
    core: Core,
    service: SampPlayerService,
    pools: SampPoolService,
}

#[derive(Clone, Copy)]
pub struct Player {
    core: Core,
    service: SampPlayerService,
    pools: SampPoolService,
    id: PlayerId,
}

#[derive(Clone, Copy)]
pub struct Animations {
    service: SampPlayerService,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecialAction {
    None = 0,
    Duck = 1,
    Jetpack = 2,
    EnterVehicle = 3,
    ExitVehicle = 4,
    Dance1 = 5,
    Dance2 = 6,
    Dance3 = 7,
    Dance4 = 8,
    HandsUp = 9,
    UseCellphone = 10,
    Sitting = 11,
    StopUseCellphone = 12,
    DrinkBeer = 20,
    SmokeCigarette = 21,
    DrinkWine = 22,
    DrinkSprunk = 23,
    Cuffed = 24,
    Carry = 25,
    Urinating = 68,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemotePlayerState {
    pub id: PlayerId,
    pub special_action: u8,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnFootSync {
    pub id: PlayerId,
    pub health: u8,
    pub armour: u8,
    pub weapon: u8,
    pub special_action: u8,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub speed: Vector3,
    pub surfing_offset: Vector3,
    pub surfing_vehicle_id: u16,
    pub animation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InCarSync {
    pub id: PlayerId,
    pub driver_health: u8,
    pub driver_armour: u8,
    pub weapon: u8,
    pub siren: bool,
    pub landing_gear: u8,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub quaternion: [f32; 4],
    pub position: Vector3,
    pub speed: Vector3,
    pub vehicle_health: f32,
    pub trailer_id: u16,
    pub vehicle_specific: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PassengerSync {
    pub id: PlayerId,
    pub seat_id: u8,
    pub weapon: u8,
    pub health: u8,
    pub armour: u8,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub position: Vector3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailerSync {
    pub id: PlayerId,
    pub trailer_id: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub speed: Vector3,
    pub turn_speed: Vector3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimSync {
    pub id: PlayerId,
    pub camera_mode: u8,
    pub zoom_and_weapon_state: u8,
    pub aspect_ratio: u8,
    pub aim_first: Vector3,
    pub aim_position: Vector3,
    pub aim_z: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Animation {
    pub name: Vec<u8>,
    pub file: Vec<u8>,
}

impl Local {
    pub(crate) const fn new(core: Core, service: SampPlayerService) -> Self {
        Self { core, service }
    }

    pub fn spawn(self) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_spawn()?)
    }

    pub fn set_special_action(self, action: SpecialAction) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_special_action(action as u8)?)
    }

    pub fn set_nickname(self, name: &[u8]) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_name(name)?)
    }

    pub fn force_unoccupied_sync(
        self,
        vehicle: VehicleId,
        seat: u8,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service
                .submit_force_unoccupied_sync(vehicle.get(), seat)?,
        )
    }

    pub fn force_aim_sync(self) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_force_aim_sync()?)
    }

    pub fn force_onfoot_sync(self) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_force_onfoot_sync()?)
    }

    pub fn force_stats_sync(self) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_force_stats_sync()?)
    }

    pub fn force_trailer_sync(self, trailer: VehicleId) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_force_trailer_sync(trailer.get())?,
        )
    }

    pub fn force_vehicle_sync(self, vehicle: VehicleId) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_force_vehicle_sync(vehicle.get())?,
        )
    }

    pub fn force_passenger_sync(
        self,
        vehicle: VehicleId,
        seat: u8,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service
                .submit_force_passenger_sync(vehicle.get(), seat)?,
        )
    }

    pub fn force_weapons_sync(self) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_force_weapons_sync()?)
    }
}

impl Players {
    pub(crate) const fn new(
        core: Core,
        service: SampPlayerService,
        pools: SampPoolService,
    ) -> Self {
        Self {
            core,
            service,
            pools,
        }
    }

    #[must_use]
    pub const fn player(self, id: PlayerId) -> Player {
        Player {
            core: self.core,
            service: self.service,
            pools: self.pools,
            id,
        }
    }

    pub fn remote_state(self, id: PlayerId) -> Result<Option<RemotePlayerState>, ModResult> {
        remote_state(self.service.remote_state(id.get())?)
    }

    pub fn is_defined(self, id: PlayerId) -> Result<bool, ModResult> {
        self.service.player_defined(id.get())
    }

    pub fn is_paused(self, id: PlayerId) -> Result<bool, ModResult> {
        self.service.player_paused(id.get())
    }

    pub fn count(self, include_npcs: bool) -> Result<u16, ModResult> {
        self.service.player_count(include_npcs)
    }

    pub fn max_id(self) -> Result<Option<PlayerId>, ModResult> {
        match self.service.player_max_id()? {
            None => Ok(None),
            Some(raw) => PlayerId::new(raw).map(Some).ok_or(MOD_NATIVE_CALL_FAILED),
        }
    }

    pub fn id_by_ped_handle(self, handle: PedHandle) -> Result<Option<PlayerId>, ModResult> {
        crate::pools::player_id_by_ped_handle(self.pools, handle)
    }
}

impl Player {
    #[must_use]
    pub const fn id(self) -> PlayerId {
        self.id
    }

    pub fn is_defined(self) -> Result<bool, ModResult> {
        self.service.player_defined(self.id.get())
    }

    pub fn is_paused(self) -> Result<bool, ModResult> {
        self.service.player_paused(self.id.get())
    }

    pub fn remote_state(self) -> Result<Option<RemotePlayerState>, ModResult> {
        remote_state(self.service.remote_state(self.id.get())?)
    }

    pub fn streamed_out_position(self) -> Result<Option<Vector3>, ModResult> {
        let raw = self.service.streamed_out_position(self.id.get())?;
        Ok((raw.exists != 0).then(|| vector(raw.position)))
    }

    pub fn ped_handle(self) -> Result<Option<PedHandle>, ModResult> {
        crate::pools::player_ped_handle(self.pools, self.id)
    }

    pub fn onfoot_sync(self) -> Result<Option<OnFootSync>, ModResult> {
        onfoot(self.service.onfoot_sync(self.id.get())?)
    }

    pub fn vehicle_sync(self) -> Result<Option<InCarSync>, ModResult> {
        in_car(self.service.vehicle_sync(self.id.get())?)
    }

    pub fn passenger_sync(self) -> Result<Option<PassengerSync>, ModResult> {
        passenger(self.service.passenger_sync(self.id.get())?)
    }

    pub fn trailer_sync(self) -> Result<Option<TrailerSync>, ModResult> {
        trailer(self.service.trailer_sync(self.id.get())?)
    }

    pub fn aim_sync(self) -> Result<Option<AimSync>, ModResult> {
        aim(self.service.aim_sync(self.id.get())?)
    }

    pub fn set_colour(self, colour: u32) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_colour(self.id.get(), colour)?,
        )
    }
}

impl Animations {
    pub(crate) const fn new(service: SampPlayerService) -> Self {
        Self { service }
    }

    pub fn get(self, id: u16) -> Result<Animation, ModResult> {
        let raw = self.service.animation(id)?;
        let name_len = usize::from(raw.name_len);
        let file_len = usize::from(raw.file_len);
        if name_len > SAMP_MAX_ANIMATION_NAME_BYTES || file_len > SAMP_MAX_ANIMATION_NAME_BYTES {
            return Err(MOD_NATIVE_CALL_FAILED);
        }
        Ok(Animation {
            name: raw.name[..name_len].to_vec(),
            file: raw.file[..file_len].to_vec(),
        })
    }

    pub fn find(self, name: &[u8], file: &[u8]) -> Result<Option<u16>, ModResult> {
        self.service.animation_id(name, file)
    }
}

fn remote_state(raw: SampRemotePlayerStateV1) -> Result<Option<RemotePlayerState>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    Ok(Some(RemotePlayerState {
        id: player_id(raw.id)?,
        special_action: raw.special_action,
        animation_id: raw.animation_id,
        health: raw.health,
        armour: raw.armour,
    }))
}

fn onfoot(raw: SampOnFootSyncV1) -> Result<Option<OnFootSync>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    Ok(Some(OnFootSync {
        id: player_id(raw.id)?,
        health: raw.health,
        armour: raw.armour,
        weapon: raw.weapon,
        special_action: raw.special_action,
        controller_left_stick_x: raw.controller_left_stick_x,
        controller_left_stick_y: raw.controller_left_stick_y,
        controller_buttons: raw.controller_buttons,
        position: vector(raw.position),
        quaternion: raw.quaternion,
        speed: vector(raw.speed),
        surfing_offset: vector(raw.surfing_offset),
        surfing_vehicle_id: raw.surfing_vehicle_id,
        animation: raw.animation,
    }))
}

fn in_car(raw: SampInCarSyncV1) -> Result<Option<InCarSync>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    Ok(Some(InCarSync {
        id: player_id(raw.id)?,
        driver_health: raw.driver_health,
        driver_armour: raw.driver_armour,
        weapon: raw.weapon,
        siren: raw.siren != 0,
        landing_gear: raw.landing_gear,
        vehicle_id: raw.vehicle_id,
        controller_left_stick_x: raw.controller_left_stick_x,
        controller_left_stick_y: raw.controller_left_stick_y,
        controller_buttons: raw.controller_buttons,
        quaternion: raw.quaternion,
        position: vector(raw.position),
        speed: vector(raw.speed),
        vehicle_health: raw.vehicle_health,
        trailer_id: raw.trailer_id,
        vehicle_specific: raw.vehicle_specific,
    }))
}

fn passenger(raw: SampPassengerSyncV1) -> Result<Option<PassengerSync>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    Ok(Some(PassengerSync {
        id: player_id(raw.id)?,
        seat_id: raw.seat_id,
        weapon: raw.weapon,
        health: raw.health,
        armour: raw.armour,
        vehicle_id: raw.vehicle_id,
        controller_left_stick_x: raw.controller_left_stick_x,
        controller_left_stick_y: raw.controller_left_stick_y,
        controller_buttons: raw.controller_buttons,
        position: vector(raw.position),
    }))
}

fn trailer(raw: SampTrailerSyncV1) -> Result<Option<TrailerSync>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    Ok(Some(TrailerSync {
        id: player_id(raw.id)?,
        trailer_id: raw.trailer_id,
        position: vector(raw.position),
        quaternion: raw.quaternion,
        speed: vector(raw.speed),
        turn_speed: vector(raw.turn_speed),
    }))
}

fn aim(raw: SampAimSyncV1) -> Result<Option<AimSync>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    Ok(Some(AimSync {
        id: player_id(raw.id)?,
        camera_mode: raw.camera_mode,
        zoom_and_weapon_state: raw.zoom_and_weapon_state,
        aspect_ratio: raw.aspect_ratio,
        aim_first: vector(raw.aim_first),
        aim_position: vector(raw.aim_position),
        aim_z: raw.aim_z,
    }))
}

fn player_id(raw: u16) -> Result<PlayerId, ModResult> {
    PlayerId::new(raw).ok_or(MOD_NATIVE_CALL_FAILED)
}

fn vector(raw: SampVector3V1) -> Vector3 {
    Vector3 {
        x: raw.x,
        y: raw.y,
        z: raw.z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sync_ignores_invalid_id() {
        let raw = SampAimSyncV1 {
            exists: 0,
            id: u16::MAX,
            ..SampAimSyncV1::default()
        };
        assert_eq!(aim(raw), Ok(None));
    }

    #[test]
    fn present_sync_rejects_invalid_player_id() {
        let raw = SampAimSyncV1 {
            exists: 1,
            id: u16::MAX,
            ..SampAimSyncV1::default()
        };
        assert_eq!(aim(raw), Err(MOD_NATIVE_CALL_FAILED));
    }
}
