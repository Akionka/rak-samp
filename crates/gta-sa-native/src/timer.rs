//! Verified read-only `CTimer` state.

use crate::{AbsoluteAddress, GtaProfile};
use gta_sa::TimerSnapshot;
use modkit_win32::ReadableRegion;

/// Failure to copy the verified `CTimer` state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerReadError {
    UnreadableState,
    InvalidState,
}

/// Copies the verified frame and time-step globals.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope after
/// `CGame::Process`. The selected profile must match the loaded image.
pub unsafe fn timer_snapshot(profile: GtaProfile) -> Result<TimerSnapshot, TimerReadError> {
    let timer = profile.spec.timer;
    validate_snapshot(TimerSnapshot {
        frame_counter: unsafe { read_scalar(timer.frame_counter) }?,
        game_time_ms: unsafe { read_scalar(timer.game_time_ms) }?,
        time_step: unsafe { read_scalar(timer.time_step) }?,
        time_step_non_clipped: unsafe { read_scalar(timer.time_step_non_clipped) }?,
    })
}

unsafe fn read_scalar<T: Copy>(address: AbsoluteAddress) -> Result<T, TimerReadError> {
    ReadableRegion::validate(address.get(), core::mem::size_of::<T>())
        .and_then(|region| unsafe { region.read_unaligned::<T>(0) })
        .ok_or(TimerReadError::UnreadableState)
}

fn validate_snapshot(snapshot: TimerSnapshot) -> Result<TimerSnapshot, TimerReadError> {
    if !snapshot.time_step.is_finite() || !snapshot.time_step_non_clipped.is_finite() {
        return Err(TimerReadError::InvalidState);
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_snapshot_rejects_non_finite_steps() {
        let snapshot = TimerSnapshot {
            time_step: f32::NAN,
            ..TimerSnapshot::default()
        };
        assert_eq!(
            validate_snapshot(snapshot),
            Err(TimerReadError::InvalidState)
        );

        let snapshot = TimerSnapshot {
            time_step_non_clipped: f32::INFINITY,
            ..TimerSnapshot::default()
        };
        assert_eq!(
            validate_snapshot(snapshot),
            Err(TimerReadError::InvalidState)
        );
    }
}
