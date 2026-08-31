//! Exact-version SA-MP native string codec service ABI.

use crate::{ModResult, ServiceHeader};

pub const SAMP_CODEC_SERVICE_VERSION_V1: u32 = 1;

/// Native compressed-string decoding for plugin-owned bitstreams.
#[repr(C)]
pub struct SampCodecServiceV1 {
    pub header: ServiceHeader,
    pub decode_string: unsafe extern "system" fn(
        input: *const u8,
        input_byte_len: u32,
        input_bit_len: u32,
        input_read_offset: u32,
        output: *mut u8,
        output_capacity: u32,
        output_len: *mut u32,
        output_read_offset: *mut u32,
    ) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_layout_is_header_plus_one_function() {
        assert_eq!(core::mem::offset_of!(SampCodecServiceV1, decode_string), 16);
        assert_eq!(
            core::mem::size_of::<SampCodecServiceV1>(),
            16 + core::mem::size_of::<usize>()
        );
    }
}
