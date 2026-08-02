/// A supported SA-MP 0.3.7 client build.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SampVersion {
    R1,
    R2,
    R3_1,
    R4_2,
    R5_1,
    Dl,
}

impl SampVersion {
    /// Identifies a client build from the PE optional header entry-point RVA.
    #[must_use]
    pub const fn from_entry_point(entry_point: u32) -> Option<Self> {
        match entry_point {
            0x31DF13 => Some(Self::R1),
            0x3195DD => Some(Self::R2),
            0x0CC4D0 => Some(Self::R3_1),
            0x0CBCB0 => Some(Self::R4_2),
            0x0CBC90 => Some(Self::R5_1),
            0x0FDB60 => Some(Self::Dl),
            _ => None,
        }
    }

    /// Returns the PE optional-header entry-point RVA used to identify this build.
    #[must_use]
    pub const fn entry_point(self) -> u32 {
        match self {
            Self::R1 => 0x31DF13,
            Self::R2 => 0x3195DD,
            Self::R3_1 => 0x0CC4D0,
            Self::R4_2 => 0x0CBCB0,
            Self::R5_1 => 0x0CBC90,
            Self::Dl => 0x0FDB60,
        }
    }
}

/// Version-relative addresses required by the low-level hook backend.
///
/// These are RVAs, not absolute pointers. The backend adds the loaded
/// `samp.dll` image base only after it has positively identified the build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressSet {
    pub incoming_rpc_handler: u32,
    pub rak_client_constructor: u32,
    pub allocate_packet: u32,
    pub write_lock: u32,
    pub write_unlock: u32,
    pub string_write_encoder: u32,
    pub string_read_decoder: u32,
    pub compressor_ptr: u32,
}

impl AddressSet {
    /// Returns the address set for a verified client version.
    #[must_use]
    pub const fn for_version(version: SampVersion) -> Self {
        match version {
            SampVersion::R1 => Self {
                incoming_rpc_handler: 0x372F0,
                rak_client_constructor: 0x33DC0,
                allocate_packet: 0x347E0,
                write_lock: 0x35B10,
                write_unlock: 0x35B50,
                string_write_encoder: 0x506B0,
                string_read_decoder: 0x507E0,
                compressor_ptr: 0x10D894,
            },
            SampVersion::R2 => Self {
                incoming_rpc_handler: 0x373D0,
                rak_client_constructor: 0x33F10,
                allocate_packet: 0x348C0,
                write_lock: 0x35BF0,
                write_unlock: 0x35C30,
                string_write_encoder: 0x50790,
                string_read_decoder: 0x508C0,
                compressor_ptr: 0x10D894,
            },
            SampVersion::R3_1 => Self {
                incoming_rpc_handler: 0x3A6A0,
                rak_client_constructor: 0x37170,
                allocate_packet: 0x37B90,
                write_lock: 0x38EC0,
                write_unlock: 0x38F00,
                string_write_encoder: 0x53A60,
                string_read_decoder: 0x53B90,
                compressor_ptr: 0x121914,
            },
            SampVersion::R4_2 => Self {
                incoming_rpc_handler: 0x3ADE0,
                rak_client_constructor: 0x378B0,
                allocate_packet: 0x382D0,
                write_lock: 0x39600,
                write_unlock: 0x39640,
                string_write_encoder: 0x541A0,
                string_read_decoder: 0x542D0,
                compressor_ptr: 0x121A3C,
            },
            SampVersion::R5_1 => Self {
                incoming_rpc_handler: 0x3ADE0,
                rak_client_constructor: 0x378B0,
                allocate_packet: 0x382D0,
                write_lock: 0x39600,
                write_unlock: 0x39640,
                string_write_encoder: 0x541A0,
                string_read_decoder: 0x542D0,
                compressor_ptr: 0x121A3C,
            },
            SampVersion::Dl => Self {
                incoming_rpc_handler: 0x3A8A0,
                rak_client_constructor: 0x37370,
                allocate_packet: 0x37D90,
                write_lock: 0x390C0,
                write_unlock: 0x39100,
                string_write_encoder: 0x53C60,
                string_read_decoder: 0x53D90,
                compressor_ptr: 0x15FA54,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_supported_entry_point() {
        for version in [
            SampVersion::R1,
            SampVersion::R2,
            SampVersion::R3_1,
            SampVersion::R4_2,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            assert_eq!(
                SampVersion::from_entry_point(version.entry_point()),
                Some(version)
            );
            let addresses = AddressSet::for_version(version);
            assert_ne!(addresses.incoming_rpc_handler, 0);
            assert_ne!(addresses.rak_client_constructor, 0);
            assert_ne!(addresses.allocate_packet, 0);
            assert_ne!(addresses.write_lock, 0);
            assert_ne!(addresses.write_unlock, 0);
            assert_ne!(addresses.string_write_encoder, 0);
            assert_ne!(addresses.string_read_decoder, 0);
            assert_ne!(addresses.compressor_ptr, 0);
        }
    }
}
