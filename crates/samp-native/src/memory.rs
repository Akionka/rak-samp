//! Guarded native-memory primitives shared by all direct client profiles.
//!
//! The generic guarded read/write and bounded-string helpers now live in
//! `modkit-win32` and are re-exported here for SA-MP backend use. This module
//! retains the SA-MP-specific strict boolean readers and the profile-facing
//! `Vector3` read helper.

pub use modkit_win32::{
    bounded_c_string, copy_bytes, read_pointer, read_unaligned, readable_range, writable_range,
    write_unaligned, zero_bytes,
};

use crate::{DirectClientError, Vector3};

/// Reads a copied vector from one previously selected native address.
///
/// # Safety
///
/// `address` must identify native process memory. Guarded scalar reads reject
/// unreadable fields; the caller must still ensure the address belongs to the
/// active SA-MP profile and is not concurrently invalidated.
pub unsafe fn read_vector3(address: usize) -> Option<Vector3> {
    Some(Vector3 {
        x: unsafe { read_unaligned::<f32>(address) }?,
        y: unsafe { read_unaligned::<f32>(address.checked_add(4)?) }?,
        z: unsafe { read_unaligned::<f32>(address.checked_add(8)?) }?,
    })
}

pub fn read_i32_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<i32>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

pub fn read_u8_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<u8>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_boolean_readers_reject_non_boolean_values() {
        let i32_false = 0_i32;
        let i32_true = 1_i32;
        let i32_invalid = 2_i32;
        let u8_false = 0_u8;
        let u8_true = 1_u8;
        let u8_invalid = 2_u8;

        assert_eq!(
            read_i32_bool((&i32_false as *const i32) as usize),
            Ok(false)
        );
        assert_eq!(read_i32_bool((&i32_true as *const i32) as usize), Ok(true));
        assert_eq!(
            read_i32_bool((&i32_invalid as *const i32) as usize),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(read_u8_bool((&u8_false as *const u8) as usize), Ok(false));
        assert_eq!(read_u8_bool((&u8_true as *const u8) as usize), Ok(true));
        assert_eq!(
            read_u8_bool((&u8_invalid as *const u8) as usize),
            Err(DirectClientError::NotReady)
        );
    }
}
