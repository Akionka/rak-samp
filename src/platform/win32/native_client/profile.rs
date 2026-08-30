use crate::SampVersion;

/// Transitional host binding for one `samp-native` profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeClientProfile(samp_native::NativeProfile);

impl NativeClientProfile {
    /// Selects a data-only native profile for an exact supported executable identity.
    pub(crate) fn select(
        module_base: usize,
        version: SampVersion,
        entry_point: u32,
    ) -> Option<Self> {
        let version = match version {
            SampVersion::R1 => samp_native::SampVersion::R1,
            SampVersion::R2 => samp_native::SampVersion::R2,
            SampVersion::R3_1 => samp_native::SampVersion::R3_1,
            SampVersion::R4_2 => samp_native::SampVersion::R4_2,
            SampVersion::R5_1 => samp_native::SampVersion::R5_1,
            SampVersion::Dl => samp_native::SampVersion::Dl,
        };
        samp_native::NativeProfile::select(module_base, version, entry_point).map(Self)
    }
}

impl core::ops::Deref for NativeClientProfile {
    type Target = samp_native::NativeProfile;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_rejects_a_zero_module_base() {
        assert!(NativeClientProfile::select(0, SampVersion::R1, 0x31DF13).is_none());
    }
}
