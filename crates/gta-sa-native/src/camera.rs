//! Verified read-only `CCamera` state.

use crate::{GtaProfile, RawMatrix, RawVector3};
use gta_sa::{CameraSnapshot, Matrix, Vector3};
use modkit_win32::ReadableRegion;

/// Failure to copy the verified active-camera pose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraReadError {
    UnreadableState,
    InvalidState,
}

/// Copies the active camera position and world-pose matrix.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope after
/// `CGame::Process`. The selected profile must match the loaded image.
pub unsafe fn camera_snapshot(profile: GtaProfile) -> Result<CameraSnapshot, CameraReadError> {
    let camera = profile.spec.camera;
    let region = ReadableRegion::validate(camera.object.get(), camera.size.get())
        .ok_or(CameraReadError::UnreadableState)?;
    let game_position = unsafe { region.read_unaligned::<RawVector3>(camera.game_position.get()) }
        .ok_or(CameraReadError::UnreadableState)?;
    let transform = unsafe { region.read_unaligned::<RawMatrix>(camera.matrix.get()) }
        .ok_or(CameraReadError::UnreadableState)?;
    validate_snapshot(CameraSnapshot {
        game_position: vector_from_raw(game_position),
        transform: Matrix::new(
            vector_from_raw(transform.right),
            vector_from_raw(transform.forward),
            vector_from_raw(transform.up),
            vector_from_raw(transform.position),
        ),
    })
}

fn vector_from_raw(value: RawVector3) -> Vector3 {
    Vector3::new(value.x, value.y, value.z)
}

fn validate_snapshot(snapshot: CameraSnapshot) -> Result<CameraSnapshot, CameraReadError> {
    let vectors = [
        snapshot.game_position,
        snapshot.transform.right,
        snapshot.transform.forward,
        snapshot.transform.up,
        snapshot.transform.position,
    ];
    vectors
        .iter()
        .all(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .then_some(snapshot)
        .ok_or(CameraReadError::InvalidState)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_snapshot_rejects_non_finite_pose_values() {
        let snapshot = CameraSnapshot {
            game_position: Vector3::new(f32::NAN, 0.0, 0.0),
            ..CameraSnapshot::default()
        };
        assert_eq!(
            validate_snapshot(snapshot),
            Err(CameraReadError::InvalidState)
        );
    }
}
