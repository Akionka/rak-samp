//! Version-selected direct client profiles.
//!
//! Network hooks use [`crate::AddressSet`] and are independently supported for
//! several SA-MP builds. Direct object layouts and native calls require a
//! separately verified profile, so this boundary deliberately selects no
//! profile for a recognized build until its layout gates are complete.

use super::{
    native_client::profile::NativeClientProfile, r1_client::R1ClientProfile,
    r3_client::ClassicClientProfile,
};
use crate::{SampVersion, runtime::DirectClientError};

/// A verified direct-native profile selected for the loaded SA-MP build.
///
/// More variants are added only with their own fixture-backed layout and live
/// validation.
#[derive(Clone, Copy, Debug)]
pub(super) enum NativeProfile {
    R1(R1ClientProfile),
    R3(ClassicClientProfile),
    R5(ClassicClientProfile),
    Dl(ClassicClientProfile),
}

impl NativeProfile {
    /// Selects a direct-native profile independently of the network
    /// [`crate::AddressSet`].
    #[cfg(test)]
    pub(super) fn select(
        module_base: usize,
        version: SampVersion,
        entry_point: u32,
    ) -> Option<Self> {
        NativeClientProfile::select(module_base, version, entry_point)
            .map(Self::from_native_client_profile)
    }

    /// Converts the selected immutable specification into the temporary
    /// legacy operation dispatch until each operation reads the specification.
    pub(super) fn from_native_client_profile(profile: NativeClientProfile) -> Self {
        match profile.spec.identity.version {
            SampVersion::R1 => Self::R1(R1ClientProfile::from_selected(profile.module_base)),
            SampVersion::R3_1 => {
                Self::R3(ClassicClientProfile::from_selected_r3(profile.module_base))
            }
            SampVersion::R5_1 => {
                Self::R5(ClassicClientProfile::from_selected_r5(profile.module_base))
            }
            SampVersion::Dl => {
                Self::Dl(ClassicClientProfile::from_selected_dl(profile.module_base))
            }
            SampVersion::R2 | SampVersion::R4_2 => unreachable!("unsupported profile identity"),
        }
    }

    pub(super) fn animation_catalog(
        self,
    ) -> Result<Vec<crate::runtime::AnimationSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.animation_catalog(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.animation_catalog()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_enables_only_verified_direct_profiles() {
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R1, 0x31DF13),
            Some(NativeProfile::R1(_))
        ));
        assert!(NativeProfile::select(0x10000, SampVersion::R1, 0x31DF14).is_none());
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R3_1, SampVersion::R3_1.entry_point()),
            Some(NativeProfile::R3(_))
        ));
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R5_1, SampVersion::R5_1.entry_point()),
            Some(NativeProfile::R5(_))
        ));
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point()),
            Some(NativeProfile::Dl(_))
        ));
        for version in [SampVersion::R2, SampVersion::R4_2] {
            assert!(NativeProfile::select(0x10000, version, version.entry_point()).is_none());
        }
    }

    #[test]
    fn r1_player_and_sync_reads_reach_the_verified_profile() {
        let native_client = NativeClientProfile::select(0x7000_0000, SampVersion::R1, 0x31DF13)
            .expect("the exact R1 entry point must select an immutable profile");

        assert!(matches!(
            native_client.player_info(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.remote_player_state(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.remote_player_is_streamed_out(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.onfoot_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.incar_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.passenger_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.trailer_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.aim_sync(0),
            Err(DirectClientError::NotReady)
        ));
    }
}
