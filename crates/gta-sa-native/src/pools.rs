//! GTA `CPools` reference-conversion calls.
//!
//! The `CPools::GetPedRef` / `CPools::GetVehicleRef` targets are GTA-owned and
//! live in the GTA profile. SA-MP backends call through this wrapper instead of
//! holding their own GTA absolute addresses.

use crate::profile::AbsoluteAddress;
use modkit_win32::readable_range;
use std::{ffi::c_void, mem};

/// Calling convention of the selected `CPools` reference getter.
///
/// The R1 and classic SA-MP builds expose the same GTA function with different
/// calling conventions; the SA-MP backend selects the matching ABI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpoolRefAbi {
    R1,
    Classic,
}

type R1CpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;
type ClassicCpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;

/// Failure to invoke a verified `CPools` reference getter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpoolRefError;

/// Converts one guarded GTA game-object pointer to its GTA handle.
///
/// Returns `Ok(None)` when the getter reports a null handle (the entity is not
/// currently registered in the pool) and `Err` when the target is not readable.
///
/// # Safety
///
/// `target` must be a verified `CPools` reference getter for the active GTA
/// profile, `game_object` must be a valid, readable GTA game-object pointer for
/// the duration of the call, and `abi` must match the calling convention of the
/// selected getter.
pub unsafe fn cpool_ref(
    target: AbsoluteAddress,
    abi: CpoolRefAbi,
    game_object: *mut c_void,
) -> Result<Option<i32>, CpoolRefError> {
    let address = target.get();
    if !readable_range(address as *const u8, 1) {
        return Err(CpoolRefError);
    }
    let handle = unsafe {
        match abi {
            CpoolRefAbi::R1 => {
                let function: R1CpoolRefFn = mem::transmute(address);
                function(game_object)
            }
            CpoolRefAbi::Classic => {
                let function: ClassicCpoolRefFn = mem::transmute(address);
                function(game_object)
            }
        }
    };
    Ok((handle != 0).then_some(handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpool_ref_abi_variants_are_distinct() {
        assert_ne!(CpoolRefAbi::R1, CpoolRefAbi::Classic);
    }

    #[test]
    fn cpool_ref_rejects_an_unreadable_target() {
        let result = unsafe {
            cpool_ref(
                AbsoluteAddress::new(usize::MAX),
                CpoolRefAbi::R1,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(result, Err(CpoolRefError));
    }
}
