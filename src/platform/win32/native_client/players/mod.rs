//! Guarded player-pool reads shared by immutable client profiles.

mod animation;
mod control;
mod pool;
mod sync;

#[cfg(test)]
use pool::{validate_player_counts, validate_player_max_id};

use super::{
    memory::{
        bounded_c_string, read_pointer, read_unaligned, read_vector3, readable_range,
        write_unaligned,
    },
    profile::{ForceSyncReset, LocalPlayerSource, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{
    AimSyncSnapshot, AnimationSnapshot, DirectClientError, InCarSyncSnapshot, LocalPlayerSnapshot,
    OnFootSyncSnapshot, PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot,
    TrailerSyncSnapshot,
};
use gta_sa_native::{CpoolRefAbi, GtaProfile, cpool_ref};
use std::{ffi::c_void, mem};

impl NativeClientProfile {
    fn player_function_target(self, rva: usize) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva)
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }
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

    #[test]
    fn sync_specs_cover_every_supported_profile() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.spec.strategies.force_sync_reset,
                ForceSyncReset::ClearLastAnyUpdate
            );
            assert_eq!(profile.spec.sync.onfoot.size.get(), 68);
            assert_eq!(profile.spec.sync.incar.size.get(), 63);
            assert_eq!(profile.spec.sync.passenger.size.get(), 24);
            assert_eq!(profile.spec.sync.trailer.size.get(), 54);
            assert_eq!(profile.spec.sync.aim.size.get(), 31);
            assert!(profile.spec.players.local_rvas.send_unoccupied_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_aim_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_onfoot_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_stats.get() > 0);
            assert!(profile.spec.players.local_rvas.send_trailer_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_passenger_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_incar_data.get() > 0);
            assert!(profile.spec.players.local_rvas.update_weapons.get() > 0);
        }
    }

    #[test]
    fn animation_and_send_rate_specs_cover_every_supported_profile() {
        let expected = [
            (SampVersion::R1, 0xF15B0, [0xEC0A8, 0xEC0AC, 0xEC0B0]),
            (SampVersion::R3_1, 0x1039D0, [0xFE0A8, 0xFE0AC, 0xFE0B0]),
            (SampVersion::R5_1, 0x1039E8, [0xFE0A8, 0xFE0AC, 0xFE0B0]),
            (SampVersion::Dl, 0x1419D0, [0x13C0A8, 0x13C0AC, 0x13C0B0]),
        ];
        for (version, table_rva, send_rates) in expected {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            let animation = profile.spec.players.animation;
            assert_eq!(animation.rva.get(), table_rva);
            assert_eq!(animation.entry_count.get(), 1812);
            assert_eq!(animation.entry_size.get(), 36);
            assert_eq!(profile.spec.sync.send_rates.onfoot.get(), send_rates[0]);
            assert_eq!(profile.spec.sync.send_rates.incar.get(), send_rates[1]);
            assert_eq!(profile.spec.sync.send_rates.aim.get(), send_rates[2]);
        }
    }
}
