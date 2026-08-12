//! Version-selected direct client profiles.
//!
//! Network hooks use [`crate::AddressSet`] and are independently supported for
//! several SA-MP builds. Direct object layouts and native calls require a
//! separately verified profile, so this boundary deliberately selects no
//! profile for a recognized build until its layout gates are complete.

use super::r1_client::R1ClientProfile;
use crate::SampVersion;

/// A verified direct-native profile selected for the loaded SA-MP build.
///
/// More variants are added only with their own fixture-backed layout and live
/// validation. Until then, non-R1 builds remain network-only.
#[derive(Clone, Copy, Debug)]
pub(super) enum NativeProfile {
    R1(R1ClientProfile),
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
            SampVersion::R2
            | SampVersion::R3_1
            | SampVersion::R4_2
            | SampVersion::R5_1
            | SampVersion::Dl => None,
        }
    }

    /// Returns the R1 fixed-layout implementation when it is the selected
    /// profile. Existing direct helpers remain R1-specific until their build
    /// alternatives have been independently proven.
    pub(super) const fn as_r1(self) -> Option<R1ClientProfile> {
        match self {
            Self::R1(profile) => Some(profile),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_keeps_unverified_builds_network_only() {
        assert!(NativeProfile::select(0x10000, SampVersion::R1, 0x31DF13).is_some());
        for version in [
            SampVersion::R2,
            SampVersion::R3_1,
            SampVersion::R4_2,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            assert!(NativeProfile::select(0x10000, version, version.entry_point()).is_none());
        }
    }
}
