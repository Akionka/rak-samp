//! Owned GTA entity snapshots.

use crate::{PedHandle, Vector3, VehicleHandle};

/// Minimal verified state common to the first entity slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntitySnapshot {
    pub position: Vector3,
}

/// Verified copied state for one live ped observation.
///
/// The handle is ephemeral. Every later native operation must validate it
/// again; this snapshot does not keep the native ped alive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PedSnapshot {
    pub handle: PedHandle,
    pub entity: EntitySnapshot,
    pub health: f32,
    pub armour: f32,
}

impl PedSnapshot {
    #[must_use]
    pub const fn position(&self) -> Vector3 {
        self.entity.position
    }
}

/// Verified copied state for one live vehicle observation.
///
/// The handle is ephemeral. Every later native operation must validate it
/// again; this snapshot does not keep the native vehicle alive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleSnapshot {
    pub handle: VehicleHandle,
    pub entity: EntitySnapshot,
    pub health: f32,
}

impl VehicleSnapshot {
    #[must_use]
    pub const fn position(&self) -> Vector3 {
        self.entity.position
    }
}
/// Verified copied state from `CTimer` after one game-process step.
///
/// Time-step values use the engine's native units. Game time wraps according
/// to the native 32-bit counter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimerSnapshot {
    pub frame_counter: u32,
    pub game_time_ms: u32,
    pub time_step: f32,
    pub time_step_non_clipped: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ped_snapshot_owns_its_values() {
        let snapshot = PedSnapshot {
            handle: PedHandle::new(7).unwrap(),
            entity: EntitySnapshot {
                position: Vector3::new(1.0, 2.0, 3.0),
            },
            health: 95.0,
            armour: 25.0,
        };
        assert_eq!(snapshot.position(), Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(snapshot.handle.get(), 7);
    }

    #[test]
    fn vehicle_snapshot_owns_its_values() {
        let snapshot = VehicleSnapshot {
            handle: VehicleHandle::new(19).unwrap(),
            entity: EntitySnapshot {
                position: Vector3::new(-3.0, 4.0, 8.0),
            },
            health: 750.0,
        };
        assert_eq!(snapshot.position(), Vector3::new(-3.0, 4.0, 8.0));
        assert_eq!(snapshot.handle.get(), 19);
        assert_eq!(snapshot.health, 750.0);
    }

    #[test]
    fn timer_snapshot_owns_native_counter_values() {
        let snapshot = TimerSnapshot {
            frame_counter: 42,
            game_time_ms: 1_250,
            time_step: 1.0,
            time_step_non_clipped: 1.25,
        };
        assert_eq!(snapshot.frame_counter, 42);
        assert_eq!(snapshot.game_time_ms, 1_250);
        assert_eq!(snapshot.time_step, 1.0);
        assert_eq!(snapshot.time_step_non_clipped, 1.25);
    }
}
