//! Owned GTA entity snapshots.

use crate::{PedHandle, Vector3};

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
}
