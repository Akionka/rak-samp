//! Verified read-only `CWorld` queries.

use crate::{GtaProfile, NativeCallTarget};

/// Failure to execute a verified `CWorld` query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReadError {
    InvalidCoordinate,
    NativeCall,
    InvalidResult,
}

/// Finds the world ground height for one finite XY coordinate.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope. The selected
/// profile must match the loaded image.
pub unsafe fn find_ground_z(profile: GtaProfile, x: f32, y: f32) -> Result<f32, WorldReadError> {
    if !x.is_finite() || !y.is_finite() {
        return Err(WorldReadError::InvalidCoordinate);
    }
    let target = NativeCallTarget::resolve(profile.spec.world.find_ground_z)
        .map_err(|_| WorldReadError::NativeCall)?;
    let z = unsafe { target.call_cdecl_f32_f32_to_f32(x, y) };
    z.is_finite()
        .then_some(z)
        .ok_or(WorldReadError::InvalidResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_query_rejects_non_finite_coordinates_before_native_access() {
        let profile = GtaProfile::select(0x0040_0000, crate::GTA_SA_10_US_SHA256).unwrap();
        assert_eq!(
            unsafe { find_ground_z(profile, f32::NAN, 0.0) },
            Err(WorldReadError::InvalidCoordinate)
        );
        assert_eq!(
            unsafe { find_ground_z(profile, 0.0, f32::INFINITY) },
            Err(WorldReadError::InvalidCoordinate)
        );
    }
}
