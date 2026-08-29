//! Plugin-side GTA San Andreas value types and safe facade.
//!
//! This crate owns the typed GTA entity handles. It contains no fixed native
//! addresses and performs no direct memory dereferences; every operation that
//! uses a handle validates it against the current game state through the host
//! backend. SA-MP pool IDs and SA-MP-to-GTA mappings live in the `samp` facade,
//! not here.

use core::num::NonZeroI32;

/// A typed non-null GTA SA object handle (GTAREF).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectHandle(NonZeroI32);

/// A typed non-null GTA SA pickup handle (GTAREF).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PickupHandle(NonZeroI32);

/// A typed non-null GTA SA vehicle handle.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VehicleHandle(NonZeroI32);

/// A typed non-null GTA SA ped handle.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PedHandle(NonZeroI32);

macro_rules! gta_handle {
    ($name:ident) => {
        impl $name {
            /// Returns `None` for the null or negative GTA handle.
            ///
            /// GTA handles are positive signed tokens; zero and negative raw
            /// values are invalid and are rejected before a wrapper exists.
            #[must_use]
            pub const fn new(raw: i32) -> Option<Self> {
                match NonZeroI32::new(raw) {
                    Some(value) if value.get() > 0 => Some(Self(value)),
                    _ => None,
                }
            }

            /// Returns the raw positive non-null GTA handle.
            #[must_use]
            pub const fn get(self) -> i32 {
                self.0.get()
            }
        }
    };
}

gta_handle!(ObjectHandle);
gta_handle!(PickupHandle);
gta_handle!(VehicleHandle);
gta_handle!(PedHandle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_reject_zero_and_negative_raw_values() {
        for raw in [0, -1, i32::MIN] {
            assert_eq!(ObjectHandle::new(raw), None);
            assert_eq!(PickupHandle::new(raw), None);
            assert_eq!(VehicleHandle::new(raw), None);
            assert_eq!(PedHandle::new(raw), None);
        }
    }

    #[test]
    fn handles_accept_positive_raw_values() {
        for raw in [1, 42, i32::MAX] {
            assert_eq!(ObjectHandle::new(raw).map(ObjectHandle::get), Some(raw));
            assert_eq!(PickupHandle::new(raw).map(PickupHandle::get), Some(raw));
            assert_eq!(VehicleHandle::new(raw).map(VehicleHandle::get), Some(raw));
            assert_eq!(PedHandle::new(raw).map(PedHandle::get), Some(raw));
        }
    }

    #[test]
    fn pickup_handle_is_distinct_from_object_handle() {
        let pickup = PickupHandle::new(7).unwrap();
        let object = ObjectHandle::new(7).unwrap();
        assert_eq!(pickup.get(), object.get());
        assert_eq!(
            std::mem::size_of::<PickupHandle>(),
            std::mem::size_of::<i32>()
        );
    }
}
