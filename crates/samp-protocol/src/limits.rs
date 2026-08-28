//! Shared bounds for neutral Protocol wire values.

/// Maximum bytes accepted by a 32-bit length-prefixed Protocol byte field.
pub const MAX_STRING32_BYTES: usize = 4096;
/// Maximum logical bytes accepted by a Native compressed-string field.
pub const MAX_ENCODED_STRING_BYTES: usize = 4_095;
