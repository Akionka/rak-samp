//! Version-selected direct client profiles.
//!
//! Network hooks use [`crate::AddressSet`] and are independently supported for
//! several SA-MP builds. Direct object layouts and native calls require a
//! separately verified profile, so this boundary deliberately selects no
//! profile for a recognized build until its layout gates are complete.

use super::{r1_client::R1ClientProfile, r3_client::R3ClientProfile};
use crate::{
    SampVersion,
    runtime::{DirectClientError, ServerInfoSnapshot},
};

/// A verified direct-native profile selected for the loaded SA-MP build.
///
/// More variants are added only with their own fixture-backed layout and live
/// validation. The R3-1 variant is deliberately limited to CNetGame scalar
/// cache reads; all other R3 direct helpers remain unavailable.
#[derive(Clone, Copy, Debug)]
pub(super) enum NativeProfile {
    R1(R1ClientProfile),
    R3Scalars(R3ClientProfile),
}

impl NativeProfile {
    /// Selects a direct-native profile independently of the network
    /// [`crate::AddressSet`].
    pub(super) fn select(
        module_base: usize,
        version: SampVersion,
        entry_point: u32,
    ) -> Option<Self> {
        match version {
            SampVersion::R1 => R1ClientProfile::verify(module_base, entry_point).map(Self::R1),
            SampVersion::R3_1 => {
                R3ClientProfile::verify(module_base, entry_point).map(Self::R3Scalars)
            }
            SampVersion::R2 | SampVersion::R4_2 | SampVersion::R5_1 | SampVersion::Dl => None,
        }
    }

    /// Returns the R1 fixed-layout implementation when it is the selected
    /// profile. Existing direct helpers remain R1-specific until their build
    /// alternatives have been independently proven.
    pub(super) const fn as_r1(self) -> Option<R1ClientProfile> {
        match self {
            Self::R1(profile) => Some(profile),
            Self::R3Scalars(_) => None,
        }
    }

    /// Reads the narrow scalar cache surface available on the selected build.
    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        match self {
            Self::R1(profile) => profile.game_state(),
            Self::R3Scalars(profile) => profile.game_state(),
        }
    }

    /// Reads copied server metadata from the selected scalar profile.
    pub(super) fn server_info(self) -> Result<ServerInfoSnapshot, DirectClientError> {
        match self {
            Self::R1(profile) => profile.server_info(),
            Self::R3Scalars(profile) => profile.server_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_limits_r3_to_the_verified_scalar_profile() {
        assert!(NativeProfile::select(0x10000, SampVersion::R1, 0x31DF13).is_some());
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R3_1, SampVersion::R3_1.entry_point()),
            Some(NativeProfile::R3Scalars(_))
        ));
        for version in [
            SampVersion::R2,
            SampVersion::R4_2,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            assert!(NativeProfile::select(0x10000, version, version.entry_point()).is_none());
        }
    }
}
