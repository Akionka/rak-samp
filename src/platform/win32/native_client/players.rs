//! Guarded player-pool reads shared by immutable client profiles.

use super::{
    memory::{bounded_c_string, read_pointer, read_unaligned, read_vector3, readable_range},
    profile::{LocalPlayerSource, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{DirectClientError, LocalPlayerSnapshot};
use std::{ffi::c_void, mem};

type R1PlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type ClassicPlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type R1PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type ClassicPlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type R1PlayerPoolGetNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type ClassicPlayerPoolGetNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type R1PlayerPoolGetLocalStatFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicPlayerPoolGetLocalStatFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type R1LocalPlayerGetPedFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type R1LocalPlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type ClassicLocalPlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type R1PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
type ClassicPedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;

impl NativeClientProfile {
    /// Copies the count pair from the guarded player pool.
    pub(crate) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        let pool = self.player_pool()?;
        let target = self.player_function_target(self.spec.players.pool_rvas.get_count.get())?;
        let (including_npcs, excluding_npcs) = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let get_count: R1PlayerPoolGetCountFn = mem::transmute(target);
                    (get_count(pool, 1), get_count(pool, 0))
                }
                PoolGetterAbi::Classic => {
                    let get_count: ClassicPlayerPoolGetCountFn = mem::transmute(target);
                    (get_count(pool, 1), get_count(pool, 0))
                }
            }
        };
        validate_player_counts(
            including_npcs,
            excluding_npcs,
            self.spec.pools.limits.players.get(),
        )
    }

    /// Copies and validates the largest assigned player ID from the pool.
    pub(crate) fn player_max_id(self) -> Result<u16, DirectClientError> {
        let pool = self.player_pool()?;
        let offset = self.spec.pools.player.largest_id_offset.get();
        let id = unsafe {
            read_unaligned::<i32>(
                (pool as usize)
                    .checked_add(offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        validate_player_max_id(id, self.spec.pools.limits.players.get())
    }

    /// Resolves the selected profile's local-player object as an opaque address.
    pub(crate) fn local_player_address(self) -> Result<*mut c_void, DirectClientError> {
        match self.spec.strategies.local_player_source {
            LocalPlayerSource::PlayerPoolGetter => {
                let pool = self.player_pool()?;
                let target = self
                    .player_function_target(self.spec.players.pool_rvas.get_local_player.get())?;
                let local = unsafe {
                    match self.spec.strategies.pool_getter_abi {
                        PoolGetterAbi::R1 => {
                            let get_local: R1PlayerPoolGetLocalPlayerFn = mem::transmute(target);
                            get_local(pool)
                        }
                        PoolGetterAbi::Classic => {
                            let get_local: ClassicPlayerPoolGetLocalPlayerFn =
                                mem::transmute(target);
                            get_local(pool)
                        }
                    }
                };
                let minimum_size = self
                    .spec
                    .players
                    .local
                    .readable_size
                    .map_or_else(
                        || {
                            self.spec
                                .players
                                .local
                                .last_any_update_offset
                                .get()
                                .checked_add(mem::size_of::<u32>())
                        },
                        |size| Some(size.get()),
                    )
                    .ok_or(DirectClientError::NotReady)?;
                (!local.is_null() && readable_range(local.cast(), minimum_size))
                    .then_some(local)
                    .ok_or(DirectClientError::NotReady)
            }
            LocalPlayerSource::NetGameField => Err(DirectClientError::NotReady),
        }
    }

    /// Copies the selected profile's local-player snapshot on the game thread.
    pub(crate) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        let pool = self.player_pool()?;
        let id = unsafe {
            read_unaligned::<u16>(
                (pool as usize)
                    .checked_add(self.spec.pools.player.local_id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|id| usize::from(*id) < self.spec.pools.limits.players.get())
        .ok_or(DirectClientError::NotReady)?;
        let local = self.local_player_address()?;
        let ped = self.local_player_ped(local)?;
        let game_ped = unsafe {
            read_pointer(
                (ped as usize)
                    .checked_add(self.spec.players.local.game_ped_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|game_ped| !game_ped.is_null() && readable_range(game_ped.cast(), 1))
        .ok_or(DirectClientError::NotReady)?;
        let _ = game_ped;

        let targets = LocalPlayerTargets {
            name: self.player_function_target(self.spec.players.pool_rvas.get_name.get())?,
            score: self
                .player_function_target(self.spec.players.pool_rvas.get_local_score.get())?,
            ping: self.player_function_target(self.spec.players.pool_rvas.get_local_ping.get())?,
            colour: self
                .player_function_target(self.spec.players.local_rvas.get_colour_argb.get())?,
            health: self.player_function_target(self.spec.players.ped_rvas.get_health.get())?,
            armour: self.player_function_target(self.spec.players.ped_rvas.get_armour.get())?,
        };
        let (name_pointer, score, ping, colour, health, armour) = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let name: R1PlayerPoolGetNameFn = mem::transmute(targets.name);
                    let score: R1PlayerPoolGetLocalStatFn = mem::transmute(targets.score);
                    let ping: R1PlayerPoolGetLocalStatFn = mem::transmute(targets.ping);
                    let colour: R1LocalPlayerGetColourFn = mem::transmute(targets.colour);
                    let health: R1PedGetStatFn = mem::transmute(targets.health);
                    let armour: R1PedGetStatFn = mem::transmute(targets.armour);
                    (
                        name(pool, id),
                        score(pool),
                        ping(pool),
                        colour(local),
                        health(ped),
                        armour(ped),
                    )
                }
                PoolGetterAbi::Classic => {
                    let name: ClassicPlayerPoolGetNameFn = mem::transmute(targets.name);
                    let score: ClassicPlayerPoolGetLocalStatFn = mem::transmute(targets.score);
                    let ping: ClassicPlayerPoolGetLocalStatFn = mem::transmute(targets.ping);
                    let colour: ClassicLocalPlayerGetColourFn = mem::transmute(targets.colour);
                    let health: ClassicPedGetStatFn = mem::transmute(targets.health);
                    let armour: ClassicPedGetStatFn = mem::transmute(targets.armour);
                    (
                        name(pool, id),
                        score(pool),
                        ping(pool),
                        colour(local),
                        health(ped),
                        armour(ped),
                    )
                }
            }
        };
        let nickname = unsafe {
            bounded_c_string(
                name_pointer,
                self.spec
                    .players
                    .local_player_name_capacity
                    .get()
                    .checked_add(1)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|name| !name.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let local_address = local as usize;
        let local_layout = self.spec.players.local;
        let current_vehicle = unsafe {
            read_unaligned::<u16>(
                local_address
                    .checked_add(local_layout.current_vehicle_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let vehicle_id = (current_vehicle != u16::MAX).then_some(current_vehicle);
        let state_offset = if vehicle_id.is_some() {
            local_layout.incar_offset
        } else {
            local_layout.onfoot_offset
        };
        let state_address = local_address
            .checked_add(state_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let (position_offset, speed_offset) = if vehicle_id.is_some() {
            (
                local_layout.incar.position_offset,
                local_layout.incar.speed_offset,
            )
        } else {
            (
                local_layout.onfoot.position_offset,
                local_layout.onfoot.speed_offset,
            )
        };
        let position = unsafe {
            read_vector3(
                state_address
                    .checked_add(position_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let velocity = unsafe {
            read_vector3(
                state_address
                    .checked_add(speed_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let spawned = unsafe {
            read_unaligned::<u32>(
                local_address
                    .checked_add(local_layout.active_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?
            != 0;
        let onfoot_address = local_address
            .checked_add(local_layout.onfoot_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let special_action = unsafe {
            read_unaligned::<u8>(
                onfoot_address
                    .checked_add(local_layout.onfoot.special_action_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let animation_id = unsafe {
            read_unaligned::<u32>(
                onfoot_address
                    .checked_add(local_layout.onfoot.animation_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)? as u16;
        Ok(LocalPlayerSnapshot {
            id,
            nickname,
            colour,
            spawned,
            health,
            armour,
            position,
            velocity,
            special_action,
            animation_id,
            vehicle_id,
            score,
            ping: ping.max(0) as u32,
        })
    }

    fn local_player_ped(self, local: *mut c_void) -> Result<*mut c_void, DirectClientError> {
        let ped = match self.spec.players.local_rvas.get_ped {
            Some(rva) => {
                let target = self.player_function_target(rva.get())?;
                unsafe {
                    match self.spec.strategies.pool_getter_abi {
                        PoolGetterAbi::R1 => {
                            let get_ped: R1LocalPlayerGetPedFn = mem::transmute(target);
                            get_ped(local)
                        }
                        PoolGetterAbi::Classic => return Err(DirectClientError::NotReady),
                    }
                }
            }
            None => unsafe {
                read_pointer(
                    (local as usize)
                        .checked_add(
                            self.spec
                                .players
                                .local
                                .ped_offset
                                .ok_or(DirectClientError::NotReady)?
                                .get(),
                        )
                        .ok_or(DirectClientError::NotReady)?,
                )
                .map_or(std::ptr::null_mut(), |pointer| pointer.cast())
            },
        };
        (!ped.is_null()
            && readable_range(
                ped.cast(),
                self.spec
                    .players
                    .local
                    .game_ped_offset
                    .get()
                    .checked_add(mem::size_of::<usize>())
                    .ok_or(DirectClientError::NotReady)?,
            ))
        .then_some(ped)
        .ok_or(DirectClientError::NotReady)
    }

    fn player_function_target(self, rva: usize) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva)
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }
}

struct LocalPlayerTargets {
    name: usize,
    score: usize,
    ping: usize,
    colour: usize,
    health: usize,
    armour: usize,
}

fn validate_player_counts(
    including_npcs: i32,
    excluding_npcs: i32,
    player_limit: usize,
) -> Result<(u16, u16), DirectClientError> {
    let including_npcs = u16::try_from(including_npcs)
        .ok()
        .filter(|count| usize::from(*count) <= player_limit)
        .ok_or(DirectClientError::NotReady)?;
    let excluding_npcs = u16::try_from(excluding_npcs)
        .ok()
        .filter(|count| *count <= including_npcs)
        .ok_or(DirectClientError::NotReady)?;
    Ok((including_npcs, excluding_npcs))
}

fn validate_player_max_id(id: i32, player_limit: usize) -> Result<u16, DirectClientError> {
    u16::try_from(id)
        .ok()
        .filter(|id| usize::from(*id) < player_limit)
        .ok_or(DirectClientError::NotReady)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    #[test]
    fn player_count_validation_preserves_the_public_bounds() {
        assert_eq!(validate_player_counts(1004, 1000, 1004), Ok((1004, 1000)));
        assert_eq!(
            validate_player_counts(-1, 0, 1004),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            validate_player_counts(1005, 1000, 1004),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            validate_player_counts(4, 5, 1004),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn player_id_validation_preserves_the_public_bounds() {
        assert_eq!(validate_player_max_id(1003, 1004), Ok(1003));
        assert_eq!(
            validate_player_max_id(-1, 1004),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            validate_player_max_id(1004, 1004),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn every_supported_profile_uses_a_verified_player_pool_source() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.spec.strategies.local_player_source,
                LocalPlayerSource::PlayerPoolGetter
            );
            assert_eq!(profile.spec.pools.limits.players.get(), 1004);
            assert!(profile.spec.players.local.last_any_update_offset.get() > 0);
        }
    }
}
