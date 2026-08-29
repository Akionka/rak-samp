//! Service table header and stable service identifiers.

/// Stable service ID ranges by subsystem.
pub const SERVICE_ID_CORE: u32 = 0x0000_0001;
pub const SERVICE_ID_GTA_SA: u32 = 0x0000_1000;
pub const SERVICE_ID_SAMP: u32 = 0x0000_2000;
pub const SERVICE_ID_SAMP_NETWORK: u32 = 0x0000_2001;
pub const SERVICE_ID_RENDER: u32 = 0x0000_3000;
pub const SERVICE_ID_INPUT: u32 = 0x0000_4000;
pub const SERVICE_ID_LEGACY_SAMP_ABI: u32 = 0x0000_F000;

/// The exact-version prefix of every published service table.
///
/// `service_id`, `version`, and `size` must match the returned table. All
/// reserved fields must be zero when produced and ignored when consumed.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceHeader {
    pub service_id: u32,
    pub version: u32,
    pub size: u32,
    pub reserved: u32,
}

impl ServiceHeader {
    /// Returns whether the header matches an exact service ID + version pair
    /// and reports a size that is at least the expected table size.
    #[must_use]
    pub const fn matches(self, service_id: u32, version: u32, expected_size: u32) -> bool {
        self.service_id == service_id
            && self.version == version
            && self.size >= expected_size
            && self.reserved == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_matches_exact_pair_and_rejects_mismatches() {
        let header = ServiceHeader {
            service_id: SERVICE_ID_CORE,
            version: 1,
            size: 64,
            reserved: 0,
        };
        assert!(header.matches(SERVICE_ID_CORE, 1, 64));
        assert!(!header.matches(SERVICE_ID_CORE, 2, 64));
        assert!(!header.matches(SERVICE_ID_SAMP, 1, 64));
        assert!(!header.matches(SERVICE_ID_CORE, 1, 128));
        assert!(header.matches(SERVICE_ID_CORE, 1, 32));
    }

    #[test]
    fn header_rejects_nonzero_reserved() {
        let header = ServiceHeader {
            service_id: SERVICE_ID_CORE,
            version: 1,
            size: 64,
            reserved: 1,
        };
        assert!(!header.matches(SERVICE_ID_CORE, 1, 64));
    }

    #[test]
    fn service_ids_are_stable() {
        assert_eq!(SERVICE_ID_CORE, 0x0000_0001);
        assert_eq!(SERVICE_ID_GTA_SA, 0x0000_1000);
        assert_eq!(SERVICE_ID_SAMP, 0x0000_2000);
        assert_eq!(SERVICE_ID_SAMP_NETWORK, 0x0000_2001);
        assert_eq!(SERVICE_ID_RENDER, 0x0000_3000);
        assert_eq!(SERVICE_ID_INPUT, 0x0000_4000);
        assert_eq!(SERVICE_ID_LEGACY_SAMP_ABI, 0x0000_F000);
    }

    #[test]
    fn service_header_layout_is_fixed() {
        assert_eq!(core::mem::size_of::<ServiceHeader>(), 16);
        assert_eq!(core::mem::align_of::<ServiceHeader>(), 4);
    }
}
