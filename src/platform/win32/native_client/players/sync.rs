//! Player sync reads and force-sync operations.

use super::super::profile;
use super::*;

type R1LocalPlayerSendUnoccupiedFn = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type ClassicLocalPlayerSendUnoccupiedFn = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type R1LocalPlayerNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type ClassicLocalPlayerNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type R1LocalPlayerSendTrailerFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type ClassicLocalPlayerSendTrailerFn = unsafe extern "thiscall" fn(*mut c_void, u16);

impl NativeClientProfile {
    /// Copies one on-foot record from the selected local or remote player.
    pub(crate) fn onfoot_sync(
        self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.onfoot_offset,
            self.spec.players.remote.onfoot_offset,
        )?
        else {
            return Ok(None);
        };
        copy_onfoot_sync(id, address, self.spec.sync.onfoot).map(Some)
    }

    /// Copies one in-car record from the selected local or remote player.
    pub(crate) fn incar_sync(
        self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.incar_offset,
            self.spec.players.remote.incar_offset,
        )?
        else {
            return Ok(None);
        };
        copy_incar_sync(id, address, self.spec.sync.incar).map(Some)
    }

    /// Copies one passenger record from the selected local or remote player.
    pub(crate) fn passenger_sync(
        self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.passenger_offset,
            self.spec.players.remote.passenger_offset,
        )?
        else {
            return Ok(None);
        };
        copy_passenger_sync(id, address, self.spec.sync.passenger).map(Some)
    }

    /// Copies one trailer record from the selected local or remote player.
    pub(crate) fn trailer_sync(
        self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.trailer_offset,
            self.spec.players.remote.trailer_offset,
        )?
        else {
            return Ok(None);
        };
        copy_trailer_sync(id, address, self.spec.sync.trailer).map(Some)
    }

    /// Copies one aim record from the selected local or remote player.
    pub(crate) fn aim_sync(self, id: u16) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.aim_offset,
            self.spec.players.remote.aim_offset,
        )?
        else {
            return Ok(None);
        };
        copy_aim_sync(id, address, self.spec.sync.aim).map(Some)
    }

    /// Sends unoccupied sync with the unified unsigned seat contract.
    pub(crate) fn force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<(), DirectClientError> {
        if usize::from(vehicle) >= self.spec.pools.limits.vehicles.get() {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let target =
            self.player_function_target(self.spec.players.local_rvas.send_unoccupied_data.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let send: R1LocalPlayerSendUnoccupiedFn = mem::transmute(target);
                    send(local, vehicle, i32::from(seat));
                }
                PoolGetterAbi::Classic => {
                    let send: ClassicLocalPlayerSendUnoccupiedFn = mem::transmute(target);
                    send(local, vehicle, i32::from(seat));
                }
            }
        }
        Ok(())
    }

    /// Invokes the local aim sync send after clearing the profile's send gate.
    pub(crate) fn force_aim_sync(self) -> Result<(), DirectClientError> {
        self.reset_force_sync_gate()?;
        self.call_local_no_arg(self.spec.players.local_rvas.send_aim_data)
    }

    /// Invokes the local on-foot sync send after clearing the profile's send gate.
    pub(crate) fn force_onfoot_sync(self) -> Result<(), DirectClientError> {
        self.reset_force_sync_gate()?;
        self.call_local_no_arg(self.spec.players.local_rvas.send_onfoot_data)
    }

    /// Invokes the local stats sync send after clearing the profile's send gate.
    pub(crate) fn force_stats_sync(self) -> Result<(), DirectClientError> {
        self.reset_force_sync_gate()?;
        self.call_local_no_arg(self.spec.players.local_rvas.send_stats)
    }

    /// Updates and sends the local trailer sync record.
    pub(crate) fn force_trailer_sync(self, trailer: u16) -> Result<(), DirectClientError> {
        self.validate_vehicle_id(trailer)?;
        self.reset_force_sync_gate()?;
        self.call_local_trailer(self.spec.players.local_rvas.send_trailer_data, trailer)
    }

    /// Updates and sends the local in-car sync record.
    pub(crate) fn force_vehicle_sync(self, vehicle: u16) -> Result<(), DirectClientError> {
        self.validate_vehicle_id(vehicle)?;
        let local = self.local_player_address()?;
        self.write_local_sync_field(
            local,
            self.spec.players.local.incar_offset.get(),
            self.spec.sync.incar.vehicle_id.get(),
            vehicle,
        )?;
        self.reset_force_sync_gate_for(local)?;
        self.call_local_no_arg_for(local, self.spec.players.local_rvas.send_incar_data)
    }

    /// Updates and sends the local passenger sync record.
    pub(crate) fn force_passenger_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<(), DirectClientError> {
        self.validate_vehicle_id(vehicle)?;
        let local = self.local_player_address()?;
        let parent = self.spec.players.local.passenger_offset.get();
        self.write_local_sync_field(
            local,
            parent,
            self.spec.sync.passenger.vehicle_id.get(),
            vehicle,
        )?;
        self.write_local_sync_field(local, parent, self.spec.sync.passenger.seat_id.get(), seat)?;
        self.reset_force_sync_gate_for(local)?;
        self.call_local_no_arg_for(local, self.spec.players.local_rvas.send_passenger_data)
    }

    /// Invokes the local weapons update without resetting the send gate.
    pub(crate) fn force_weapons_sync(self) -> Result<(), DirectClientError> {
        self.call_local_no_arg(self.spec.players.local_rvas.update_weapons)
    }

    fn validate_vehicle_id(self, vehicle: u16) -> Result<(), DirectClientError> {
        (usize::from(vehicle) < self.spec.pools.limits.vehicles.get())
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    fn sync_record_address(
        self,
        id: u16,
        local_offset: profile::FieldOffset,
        remote_offset: profile::FieldOffset,
    ) -> Result<Option<usize>, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.players.get() {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id = unsafe {
            read_unaligned::<u16>(
                (pool as usize)
                    .checked_add(self.spec.pools.player.local_id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|value| usize::from(*value) < self.spec.pools.limits.players.get());
        if local_id == Some(id) {
            return (self.local_player_address()? as usize)
                .checked_add(local_offset.get())
                .map(Some)
                .ok_or(DirectClientError::NotReady);
        }
        let targets = self.remote_player_targets()?;
        let Some(remote) = self.connected_remote_player(pool, id, targets)? else {
            return Ok(None);
        };
        if !self.remote_player_defined(remote, targets)? {
            return Ok(None);
        }
        (remote as usize)
            .checked_add(remote_offset.get())
            .map(Some)
            .ok_or(DirectClientError::NotReady)
    }

    fn reset_force_sync_gate(self) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        self.reset_force_sync_gate_for(local)
    }

    fn reset_force_sync_gate_for(self, local: *mut c_void) -> Result<(), DirectClientError> {
        match self.spec.strategies.force_sync_reset {
            ForceSyncReset::ClearLastAnyUpdate => {
                let address = (local as usize)
                    .checked_add(self.spec.players.local.last_any_update_offset.get())
                    .ok_or(DirectClientError::NotReady)?;
                unsafe { write_unaligned(address, 0_u32) }
                    .then_some(())
                    .ok_or(DirectClientError::NotReady)
            }
            ForceSyncReset::ProfileSpecific => Err(DirectClientError::NotReady),
        }
    }

    fn write_local_sync_field<T: Copy>(
        self,
        local: *mut c_void,
        parent_offset: usize,
        field_offset: usize,
        value: T,
    ) -> Result<(), DirectClientError> {
        let address = (local as usize)
            .checked_add(parent_offset)
            .and_then(|address| address.checked_add(field_offset))
            .ok_or(DirectClientError::NotReady)?;
        unsafe { write_unaligned(address, value) }
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    fn call_local_no_arg(self, rva: profile::NativeRva) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        self.call_local_no_arg_for(local, rva)
    }

    fn call_local_no_arg_for(
        self,
        local: *mut c_void,
        rva: profile::NativeRva,
    ) -> Result<(), DirectClientError> {
        let target = self.player_function_target(rva.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let send: R1LocalPlayerNoArgFn = mem::transmute(target);
                    send(local);
                }
                PoolGetterAbi::Classic => {
                    let send: ClassicLocalPlayerNoArgFn = mem::transmute(target);
                    send(local);
                }
            }
        }
        Ok(())
    }

    fn call_local_trailer(
        self,
        rva: profile::NativeRva,
        trailer: u16,
    ) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        let target = self.player_function_target(rva.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let send: R1LocalPlayerSendTrailerFn = mem::transmute(target);
                    send(local, trailer);
                }
                PoolGetterAbi::Classic => {
                    let send: ClassicLocalPlayerSendTrailerFn = mem::transmute(target);
                    send(local, trailer);
                }
            }
        }
        Ok(())
    }
}

fn copy_onfoot_sync(
    id: u16,
    address: usize,
    layout: profile::OnFootSyncLayout,
) -> Result<OnFootSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    let scalar = |offset| unsafe {
        read_unaligned::<i16>(
            address
                .checked_add(offset)
                .ok_or(DirectClientError::NotReady)?,
        )
        .ok_or(DirectClientError::NotReady)
    };
    let vector = |offset| unsafe {
        read_vector3(
            address
                .checked_add(offset)
                .ok_or(DirectClientError::NotReady)?,
        )
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)
    };
    let quaternion = [0, 4, 8, 12].map(|delta| unsafe {
        read_unaligned::<f32>(
            address
                .checked_add(layout.quaternion.get() + delta)
                .ok_or(DirectClientError::NotReady)?,
        )
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)
    });
    let [q0, q1, q2, q3] = quaternion;
    Ok(OnFootSyncSnapshot {
        id,
        controller_left_stick_x: scalar(layout.controller_left_stick_x.get())?,
        controller_left_stick_y: scalar(layout.controller_left_stick_y.get())?,
        controller_buttons: scalar(layout.controller_buttons.get())?,
        position: vector(layout.position.get())?,
        quaternion: [q0?, q1?, q2?, q3?],
        health: unsafe { read_unaligned(address + layout.health.get()) }
            .ok_or(DirectClientError::NotReady)?,
        armour: unsafe { read_unaligned(address + layout.armour.get()) }
            .ok_or(DirectClientError::NotReady)?,
        weapon: unsafe { read_unaligned(address + layout.weapon.get()) }
            .ok_or(DirectClientError::NotReady)?,
        special_action: unsafe { read_unaligned(address + layout.special_action.get()) }
            .ok_or(DirectClientError::NotReady)?,
        speed: vector(layout.speed.get())?,
        surfing_offset: vector(layout.surfing_offset.get())?,
        surfing_vehicle_id: unsafe { read_unaligned(address + layout.surfing_vehicle_id.get()) }
            .ok_or(DirectClientError::NotReady)?,
        animation: unsafe { read_unaligned(address + layout.animation.get()) }
            .ok_or(DirectClientError::NotReady)?,
    })
}

fn sync_scalar<T: Copy>(address: usize, offset: usize) -> Result<T, DirectClientError> {
    let address = address
        .checked_add(offset)
        .ok_or(DirectClientError::NotReady)?;
    unsafe { read_unaligned(address) }.ok_or(DirectClientError::NotReady)
}

fn sync_vector(
    address: usize,
    offset: usize,
) -> Result<crate::runtime::Vector3, DirectClientError> {
    let address = address
        .checked_add(offset)
        .ok_or(DirectClientError::NotReady)?;
    unsafe { read_vector3(address) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)
}

fn sync_quaternion(address: usize, offset: usize) -> Result<[f32; 4], DirectClientError> {
    let values = [
        sync_scalar::<f32>(address, offset)?,
        sync_scalar::<f32>(address, offset + 4)?,
        sync_scalar::<f32>(address, offset + 8)?,
        sync_scalar::<f32>(address, offset + 12)?,
    ];
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(values)
        .ok_or(DirectClientError::NotReady)
}

fn copy_incar_sync(
    id: u16,
    address: usize,
    layout: profile::InCarSyncLayout,
) -> Result<InCarSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    let boolean = |offset| match sync_scalar::<u8>(address, offset)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DirectClientError::NotReady),
    };
    Ok(InCarSyncSnapshot {
        id,
        vehicle_id: sync_scalar(address, layout.vehicle_id.get())?,
        controller_left_stick_x: sync_scalar(address, layout.controller_left_stick_x.get())?,
        controller_left_stick_y: sync_scalar(address, layout.controller_left_stick_y.get())?,
        controller_buttons: sync_scalar(address, layout.controller_buttons.get())?,
        quaternion: sync_quaternion(address, layout.quaternion.get())?,
        position: sync_vector(address, layout.position.get())?,
        speed: sync_vector(address, layout.speed.get())?,
        vehicle_health: finite_scalar(address, layout.vehicle_health.get())?,
        driver_health: sync_scalar(address, layout.driver_health.get())?,
        driver_armour: sync_scalar(address, layout.driver_armour.get())?,
        weapon: sync_scalar(address, layout.weapon.get())?,
        siren: boolean(layout.siren.get())?,
        landing_gear: boolean(layout.landing_gear.get())?,
        trailer_id: sync_scalar(address, layout.trailer_id.get())?,
        vehicle_specific: sync_scalar(address, layout.vehicle_specific.get())?,
    })
}

fn finite_scalar(address: usize, offset: usize) -> Result<f32, DirectClientError> {
    let value = sync_scalar::<f32>(address, offset)?;
    value
        .is_finite()
        .then_some(value)
        .ok_or(DirectClientError::NotReady)
}

fn copy_passenger_sync(
    id: u16,
    address: usize,
    layout: profile::PassengerSyncLayout,
) -> Result<PassengerSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    Ok(PassengerSyncSnapshot {
        id,
        vehicle_id: sync_scalar(address, layout.vehicle_id.get())?,
        seat_id: sync_scalar(address, layout.seat_id.get())?,
        weapon: sync_scalar(address, layout.weapon.get())?,
        health: sync_scalar(address, layout.health.get())?,
        armour: sync_scalar(address, layout.armour.get())?,
        controller_left_stick_x: sync_scalar(address, layout.controller_left_stick_x.get())?,
        controller_left_stick_y: sync_scalar(address, layout.controller_left_stick_y.get())?,
        controller_buttons: sync_scalar(address, layout.controller_buttons.get())?,
        position: sync_vector(address, layout.position.get())?,
    })
}

fn copy_trailer_sync(
    id: u16,
    address: usize,
    layout: profile::TrailerSyncLayout,
) -> Result<TrailerSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    Ok(TrailerSyncSnapshot {
        id,
        trailer_id: sync_scalar(address, layout.id.get())?,
        position: sync_vector(address, layout.position.get())?,
        quaternion: sync_quaternion(address, layout.quaternion.get())?,
        speed: sync_vector(address, layout.speed.get())?,
        turn_speed: sync_vector(address, layout.turn_speed.get())?,
    })
}

fn copy_aim_sync(
    id: u16,
    address: usize,
    layout: profile::AimSyncLayout,
) -> Result<AimSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    let aim_z = sync_scalar::<f32>(address, layout.z.get())?;
    aim_z
        .is_finite()
        .then_some(())
        .ok_or(DirectClientError::NotReady)?;
    Ok(AimSyncSnapshot {
        id,
        camera_mode: sync_scalar(address, layout.camera_mode.get())?,
        aim_first: sync_vector(address, layout.first.get())?,
        aim_position: sync_vector(address, layout.position.get())?,
        aim_z,
        zoom_and_weapon_state: sync_scalar(address, layout.zoom_weapon_state.get())?,
        aspect_ratio: sync_scalar(address, layout.aspect_ratio.get())?,
    })
}
