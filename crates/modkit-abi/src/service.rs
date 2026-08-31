//! Service table header and stable service identifiers.

use crate::ServiceId;

/// Stable service ID ranges by subsystem.
pub const SERVICE_ID_CORE: ServiceId = ServiceId(0x0000_0001);
pub const SERVICE_ID_GTA_SA: ServiceId = ServiceId(0x0000_1000);
pub const SERVICE_ID_SAMP: ServiceId = ServiceId(0x0000_2000);
pub const SERVICE_ID_SAMP_NETWORK: ServiceId = ServiceId(0x0000_2001);
pub const SERVICE_ID_SAMP_TEXT_LABEL: ServiceId = ServiceId(0x0000_2002);
pub const SERVICE_ID_SAMP_CONTROL: ServiceId = ServiceId(0x0000_2003);
pub const SERVICE_ID_SAMP_UI: ServiceId = ServiceId(0x0000_2004);
pub const SERVICE_ID_SAMP_PLAYER: ServiceId = ServiceId(0x0000_2005);
pub const SERVICE_ID_SAMP_POOL: ServiceId = ServiceId(0x0000_2006);
pub const SERVICE_ID_SAMP_TEXTDRAW: ServiceId = ServiceId(0x0000_2007);
pub const SERVICE_ID_SAMP_CODEC: ServiceId = ServiceId(0x0000_2008);
pub const SERVICE_ID_RENDER: ServiceId = ServiceId(0x0000_3000);
pub const SERVICE_ID_INPUT: ServiceId = ServiceId(0x0000_4000);
pub const SERVICE_ID_LEGACY_SAMP_ABI: ServiceId = ServiceId(0x0000_F000);

/// The exact-version prefix of every published service table.
///
/// `service_id`, `version`, and `size` must match the returned table. All
/// reserved fields must be zero when produced and ignored when consumed.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceHeader {
    pub service_id: ServiceId,
    pub version: u32,
    pub size: u32,
    pub reserved: u32,
}

impl ServiceHeader {
    /// Returns whether the header matches an exact service ID, version, and
    /// immutable V1 table size.
    ///
    /// Consumers intentionally ignore `reserved`. Producers must write zero.
    #[must_use]
    pub const fn matches(self, service_id: ServiceId, version: u32, expected_size: u32) -> bool {
        self.service_id.0 == service_id.0 && self.version == version && self.size == expected_size
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
        assert!(!header.matches(SERVICE_ID_CORE, 1, 32));
    }

    #[test]
    fn header_ignores_reserved_when_consumed() {
        let header = ServiceHeader {
            service_id: SERVICE_ID_CORE,
            version: 1,
            size: 64,
            reserved: 1,
        };
        assert!(header.matches(SERVICE_ID_CORE, 1, 64));
    }

    #[test]
    fn service_ids_are_stable() {
        assert_eq!(SERVICE_ID_CORE.0, 0x0000_0001);
        assert_eq!(SERVICE_ID_GTA_SA.0, 0x0000_1000);
        assert_eq!(SERVICE_ID_SAMP.0, 0x0000_2000);
        assert_eq!(SERVICE_ID_SAMP_NETWORK.0, 0x0000_2001);
        assert_eq!(SERVICE_ID_SAMP_TEXT_LABEL.0, 0x0000_2002);
        assert_eq!(SERVICE_ID_SAMP_CONTROL.0, 0x0000_2003);
        assert_eq!(SERVICE_ID_SAMP_UI.0, 0x0000_2004);
        assert_eq!(SERVICE_ID_SAMP_PLAYER.0, 0x0000_2005);
        assert_eq!(SERVICE_ID_SAMP_POOL.0, 0x0000_2006);
        assert_eq!(SERVICE_ID_SAMP_TEXTDRAW.0, 0x0000_2007);
        assert_eq!(SERVICE_ID_SAMP_CODEC.0, 0x0000_2008);
        assert_eq!(SERVICE_ID_RENDER.0, 0x0000_3000);
        assert_eq!(SERVICE_ID_INPUT.0, 0x0000_4000);
        assert_eq!(SERVICE_ID_LEGACY_SAMP_ABI.0, 0x0000_F000);
    }

    #[test]
    fn service_header_layout_is_fixed() {
        assert_eq!(core::mem::size_of::<ServiceHeader>(), 16);
        assert_eq!(core::mem::align_of::<ServiceHeader>(), 4);
    }
}
