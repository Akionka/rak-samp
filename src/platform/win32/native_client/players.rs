//! Guarded player-pool reads shared by immutable client profiles.

use super::{
    memory::{read_unaligned, readable_range},
    profile::{LocalPlayerSource, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::DirectClientError;
use std::{ffi::c_void, mem};

type R1PlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type ClassicPlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type R1PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type ClassicPlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;

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

    fn player_function_target(self, rva: usize) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva)
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }
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
