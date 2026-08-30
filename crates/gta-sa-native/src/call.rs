//! Guarded x86 native call and vtable-target helpers.

use crate::{
    layout::RawVector3,
    profile::{AbsoluteAddress, VtableSlot},
};
use modkit_win32::{read_pointer, readable_range};
use std::{ffi::c_void, mem};

/// Failure to resolve a readable native function target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeCallError;

/// A target whose first byte is currently readable.
///
/// Construction validates memory only. The caller must still obtain the
/// address and signature from the selected verified profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeCallTarget(AbsoluteAddress);

impl NativeCallTarget {
    pub fn resolve(address: AbsoluteAddress) -> Result<Self, NativeCallError> {
        readable_range(address.get() as *const u8, 1)
            .then_some(Self(address))
            .ok_or(NativeCallError)
    }

    #[must_use]
    pub const fn address(self) -> AbsoluteAddress {
        self.0
    }

    /// Resolves one guarded function pointer from an object's vtable.
    ///
    /// # Safety
    ///
    /// `object` must identify a live native object for the duration of this
    /// call. `slot` must come from verified profile evidence for that object.
    pub unsafe fn from_vtable(
        object: *mut c_void,
        slot: VtableSlot,
    ) -> Result<Self, NativeCallError> {
        let vtable = unsafe { read_pointer(object as usize) }.ok_or(NativeCallError)?;
        let byte_offset = slot
            .get()
            .checked_mul(mem::size_of::<usize>())
            .ok_or(NativeCallError)?;
        let entry = (vtable as usize)
            .checked_add(byte_offset)
            .ok_or(NativeCallError)?;
        let target = unsafe { read_pointer(entry) }.ok_or(NativeCallError)?;
        Self::resolve(AbsoluteAddress::new(target as usize))
    }

    /// Calls a verified cdecl target taking an `i32` and returning a pointer.
    ///
    /// # Safety
    ///
    /// The selected profile must prove this exact signature and ABI.
    pub unsafe fn call_cdecl_i32_to_ptr(self, value: i32) -> *mut c_void {
        type Function = unsafe extern "cdecl" fn(i32) -> *mut c_void;
        let function: Function = unsafe { mem::transmute(self.0.get()) };
        unsafe { function(value) }
    }

    /// Calls a verified cdecl target taking a pointer and returning an `i32`.
    ///
    /// # Safety
    ///
    /// The selected profile must prove this exact signature and ABI, and
    /// `value` must satisfy the native function's object contract.
    pub unsafe fn call_cdecl_ptr_to_i32(self, value: *mut c_void) -> i32 {
        type Function = unsafe extern "cdecl" fn(*mut c_void) -> i32;
        let function: Function = unsafe { mem::transmute(self.0.get()) };
        unsafe { function(value) }
    }

    /// Calls a verified stdcall target with no arguments and a `u32` result.
    ///
    /// # Safety
    ///
    /// The selected profile must prove this exact signature and ABI.
    pub unsafe fn call_stdcall0_to_u32(self) -> u32 {
        type Function = unsafe extern "stdcall" fn() -> u32;
        let function: Function = unsafe { mem::transmute(self.0.get()) };
        unsafe { function() }
    }

    /// Calls a verified thiscall target with a vector and byte-sized boolean.
    ///
    /// # Safety
    ///
    /// The selected profile must prove this exact signature, ABI, vtable slot,
    /// and execution constraint. `object` must remain live for the call.
    pub unsafe fn call_thiscall_vector3_bool(
        self,
        object: *mut c_void,
        vector: RawVector3,
        flag: u8,
    ) {
        type Function = unsafe extern "thiscall" fn(*mut c_void, RawVector3, u8);
        let function: Function = unsafe { mem::transmute(self.0.get()) };
        unsafe { function(object, vector, flag) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "cdecl" fn echo_pointer(value: *mut c_void) -> i32 {
        value as usize as i32
    }

    unsafe extern "stdcall" fn fixed_value() -> u32 {
        0xA5A5_5A5A
    }

    #[test]
    fn target_rejects_unreadable_addresses() {
        assert_eq!(
            NativeCallTarget::resolve(AbsoluteAddress::new(usize::MAX)),
            Err(NativeCallError)
        );
    }

    #[test]
    fn typed_cdecl_and_stdcall_helpers_preserve_their_abis() {
        let cdecl =
            NativeCallTarget::resolve(AbsoluteAddress::new(echo_pointer as *const () as usize))
                .unwrap();
        let stdcall =
            NativeCallTarget::resolve(AbsoluteAddress::new(fixed_value as *const () as usize))
                .unwrap();
        let value = 0x1234usize as *mut c_void;
        assert_eq!(unsafe { cdecl.call_cdecl_ptr_to_i32(value) }, 0x1234);
        assert_eq!(unsafe { stdcall.call_stdcall0_to_u32() }, 0xA5A5_5A5A);
    }

    #[test]
    fn vtable_resolution_validates_object_slot_and_target() {
        let vtable = [echo_pointer as *const () as usize];
        let object = [vtable.as_ptr() as usize];
        let target = unsafe {
            NativeCallTarget::from_vtable(object.as_ptr() as *mut c_void, VtableSlot::new(0))
        }
        .unwrap();
        assert_eq!(
            unsafe { target.call_cdecl_ptr_to_i32(7usize as *mut c_void) },
            7
        );
        assert_eq!(
            unsafe { NativeCallTarget::from_vtable(std::ptr::null_mut(), VtableSlot::new(0)) },
            Err(NativeCallError)
        );
    }
}
