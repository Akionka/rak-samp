//! Explicit unsafe access to plugin-owned storage only.
//!
//! Native SA-MP pointers remain Host-internal. This module does not expose
//! client singletons, pools, vtables, RPC nodes, or callback addresses.

/// Returns the byte-storage address of an owned Protocol bit stream.
///
/// # Safety
///
/// The pointer is valid only while `stream` remains alive and no mutation can
/// reallocate its storage. It may be dangling when the stream has no bytes and
/// must never be read beyond [`samp_protocol::BitStream::len_bytes`].
#[must_use]
pub unsafe fn bitstream_data(stream: &samp_protocol::BitStream) -> *const u8 {
    stream.as_bytes().as_ptr()
}

#[cfg(test)]
mod tests {
    #[test]
    fn bitstream_data_points_to_owned_storage() {
        let stream = samp_protocol::BitStream::from_bytes([1, 2, 3]).unwrap();
        assert_eq!(
            unsafe { super::bitstream_data(&stream) },
            stream.as_bytes().as_ptr()
        );
    }
}
