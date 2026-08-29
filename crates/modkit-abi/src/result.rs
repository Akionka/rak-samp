//! Extensible integer-compatible result codes.
//!
//! A transparent newtype is used instead of a Rust `enum` so a future host can
//! return a code an older plugin does not know without creating an invalid Rust
//! enum discriminant. `0` means success; non-zero means non-success.

use core::fmt;

/// A host/service result code. `0` is success; non-zero is non-success.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModResult(pub i32);

pub const MOD_OK: ModResult = ModResult(0);
pub const MOD_NOT_READY: ModResult = ModResult(1);
pub const MOD_INVALID_ARGUMENT: ModResult = ModResult(2);
pub const MOD_UNSUPPORTED_VERSION: ModResult = ModResult(3);
pub const MOD_NOT_FOUND: ModResult = ModResult(4);
pub const MOD_OUT_OF_BOUNDS: ModResult = ModResult(5);
pub const MOD_PAYLOAD_TOO_LARGE: ModResult = ModResult(6);
pub const MOD_NATIVE_CALL_FAILED: ModResult = ModResult(7);
pub const MOD_CALLBACK_IN_PROGRESS: ModResult = ModResult(8);
pub const MOD_QUEUE_FULL: ModResult = ModResult(9);
pub const MOD_PENDING: ModResult = ModResult(10);
pub const MOD_TIMED_OUT: ModResult = ModResult(11);
pub const MOD_WAIT_REJECTED: ModResult = ModResult(12);
pub const MOD_SHUTTING_DOWN: ModResult = ModResult(13);
pub const MOD_BUSY: ModResult = ModResult(14);
pub const MOD_UNSUPPORTED: ModResult = ModResult(15);
pub const MOD_BUFFER_TOO_SMALL: ModResult = ModResult(16);

impl ModResult {
    /// Returns whether this is the success code.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }

    /// Returns whether this is a non-success code.
    #[must_use]
    pub const fn is_err(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Debug for ModResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.0 {
            0 => "MOD_OK",
            1 => "MOD_NOT_READY",
            2 => "MOD_INVALID_ARGUMENT",
            3 => "MOD_UNSUPPORTED_VERSION",
            4 => "MOD_NOT_FOUND",
            5 => "MOD_OUT_OF_BOUNDS",
            6 => "MOD_PAYLOAD_TOO_LARGE",
            7 => "MOD_NATIVE_CALL_FAILED",
            8 => "MOD_CALLBACK_IN_PROGRESS",
            9 => "MOD_QUEUE_FULL",
            10 => "MOD_PENDING",
            11 => "MOD_TIMED_OUT",
            12 => "MOD_WAIT_REJECTED",
            13 => "MOD_SHUTTING_DOWN",
            14 => "MOD_BUSY",
            15 => "MOD_UNSUPPORTED",
            16 => "MOD_BUFFER_TOO_SMALL",
            _ => "MOD_UNKNOWN",
        };
        if name == "MOD_UNKNOWN" {
            formatter.write_fmt(format_args!("ModResult({})", self.0))
        } else {
            formatter.write_str(name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_success_and_nonzero_is_error() {
        assert!(MOD_OK.is_ok());
        assert!(!MOD_OK.is_err());
        for code in [
            MOD_NOT_READY,
            MOD_INVALID_ARGUMENT,
            MOD_UNSUPPORTED_VERSION,
            MOD_NOT_FOUND,
            MOD_OUT_OF_BOUNDS,
            MOD_PAYLOAD_TOO_LARGE,
            MOD_NATIVE_CALL_FAILED,
            MOD_CALLBACK_IN_PROGRESS,
            MOD_QUEUE_FULL,
            MOD_PENDING,
            MOD_TIMED_OUT,
            MOD_WAIT_REJECTED,
            MOD_SHUTTING_DOWN,
            MOD_BUSY,
            MOD_UNSUPPORTED,
            MOD_BUFFER_TOO_SMALL,
        ] {
            assert!(code.is_err());
            assert!(!code.is_ok());
        }
    }

    #[test]
    fn numeric_assignments_are_immutable() {
        assert_eq!(MOD_OK.0, 0);
        assert_eq!(MOD_NOT_READY.0, 1);
        assert_eq!(MOD_INVALID_ARGUMENT.0, 2);
        assert_eq!(MOD_UNSUPPORTED_VERSION.0, 3);
        assert_eq!(MOD_NOT_FOUND.0, 4);
        assert_eq!(MOD_OUT_OF_BOUNDS.0, 5);
        assert_eq!(MOD_PAYLOAD_TOO_LARGE.0, 6);
        assert_eq!(MOD_NATIVE_CALL_FAILED.0, 7);
        assert_eq!(MOD_CALLBACK_IN_PROGRESS.0, 8);
        assert_eq!(MOD_QUEUE_FULL.0, 9);
        assert_eq!(MOD_PENDING.0, 10);
        assert_eq!(MOD_TIMED_OUT.0, 11);
        assert_eq!(MOD_WAIT_REJECTED.0, 12);
        assert_eq!(MOD_SHUTTING_DOWN.0, 13);
        assert_eq!(MOD_BUSY.0, 14);
        assert_eq!(MOD_UNSUPPORTED.0, 15);
        assert_eq!(MOD_BUFFER_TOO_SMALL.0, 16);
    }

    #[test]
    fn unknown_codes_are_preserved_for_diagnostics() {
        let unknown = ModResult(999);
        assert!(unknown.is_err());
        assert_eq!(format!("{unknown:?}"), "ModResult(999)");
    }

    #[test]
    fn result_is_a_transparent_i32() {
        assert_eq!(
            core::mem::size_of::<ModResult>(),
            core::mem::size_of::<i32>()
        );
        assert_eq!(
            core::mem::align_of::<ModResult>(),
            core::mem::align_of::<i32>()
        );
    }
}
