//! Fixed-width host-issued identity types.
//!
//! `0` is invalid/reserved for every host-issued ID. Hosts must not reuse an ID
//! during the lifetime of a process in a way that could make a stale plugin ID
//! refer to a different live object.

use core::fmt;

macro_rules! id_type {
    ($name:ident, $ty:ty, $docs:literal) => {
        #[doc = $docs]
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub $ty);

        impl $name {
            /// Returns whether this is the reserved invalid ID `0`.
            #[must_use]
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_fmt(format_args!("{}({})", stringify!($name), self.0))
            }
        }
    };
}

id_type!(ServiceId, u32, "A stable service identifier.");
id_type!(
    SubscriptionId,
    u64,
    "A host-issued subscription identifier."
);
id_type!(
    CommandReceiptId,
    u64,
    "A host-issued command receipt identifier."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_the_reserved_invalid_id() {
        assert!(ServiceId(0).is_zero());
        assert!(SubscriptionId(0).is_zero());
        assert!(CommandReceiptId(0).is_zero());
        assert!(!ServiceId(1).is_zero());
        assert!(!SubscriptionId(1).is_zero());
        assert!(!CommandReceiptId(1).is_zero());
    }

    #[test]
    fn ids_are_transparent_fixed_width() {
        assert_eq!(core::mem::size_of::<ServiceId>(), 4);
        assert_eq!(core::mem::size_of::<SubscriptionId>(), 8);
        assert_eq!(core::mem::size_of::<CommandReceiptId>(), 8);
    }
}
