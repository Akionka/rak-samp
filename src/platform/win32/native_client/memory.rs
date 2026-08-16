//! Guarded native-memory primitives shared by all direct client profiles.
//!
//! The helpers return copied values or owned byte buffers. Native pointers
//! remain inside the game-thread operation that resolved them.

use crate::runtime::{DirectClientError, Vector3};
use std::{ffi::c_void, mem, ptr};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
    PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE, PAGE_WRITECOPY, VirtualQuery,
};

pub(crate) unsafe fn read_pointer(address: usize) -> Option<*mut u8> {
    unsafe { read_unaligned::<usize>(address) }.map(|value| value as *mut u8)
}

pub(crate) unsafe fn read_unaligned<T: Copy>(address: usize) -> Option<T> {
    readable_range(address as *const u8, mem::size_of::<T>())
        .then(|| unsafe { (address as *const T).read_unaligned() })
}

pub(crate) unsafe fn read_vector3(address: usize) -> Option<Vector3> {
    Some(Vector3 {
        x: unsafe { read_unaligned::<f32>(address) }?,
        y: unsafe { read_unaligned::<f32>(address.checked_add(4)?) }?,
        z: unsafe { read_unaligned::<f32>(address.checked_add(8)?) }?,
    })
}

pub(crate) fn read_i32_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<i32>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

pub(crate) fn read_u8_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<u8>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

pub(crate) unsafe fn write_unaligned<T: Copy>(address: usize, value: T) -> bool {
    if !writable_range(address as *const u8, mem::size_of::<T>()) {
        return false;
    }
    unsafe { (address as *mut T).write_unaligned(value) };
    true
}

pub(crate) unsafe fn copy_bytes(destination: *mut u8, source: &[u8]) -> bool {
    if !writable_range(destination.cast_const(), source.len()) {
        return false;
    }
    unsafe { ptr::copy_nonoverlapping(source.as_ptr(), destination, source.len()) };
    true
}

pub(crate) unsafe fn zero_bytes(destination: *mut u8, length: usize) -> bool {
    if !writable_range(destination.cast_const(), length) {
        return false;
    }
    unsafe { ptr::write_bytes(destination, 0, length) };
    true
}

pub(crate) unsafe fn bounded_c_string(pointer: *const u8, maximum: usize) -> Option<Vec<u8>> {
    if pointer.is_null() {
        return None;
    }
    let mut output = Vec::new();
    for index in 0..maximum {
        let byte = unsafe { read_unaligned::<u8>((pointer as usize).checked_add(index)?) }?;
        if byte == 0 {
            return Some(output);
        }
        output.push(byte);
    }
    None
}

pub(crate) fn readable_range(address: *const u8, length: usize) -> bool {
    guarded_range(address, length, |protection| {
        protection & (PAGE_GUARD | PAGE_NOACCESS) == 0
    })
}

pub(crate) fn writable_range(address: *const u8, length: usize) -> bool {
    guarded_range(address, length, |protection| {
        matches!(
            protection & 0xFF,
            PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY
        )
    })
}

fn guarded_range(address: *const u8, length: usize, permits: impl Fn(u32) -> bool) -> bool {
    if address.is_null() || length == 0 {
        return length == 0;
    }
    let Some(end) = (address as usize).checked_add(length) else {
        return false;
    };
    let mut current = address as usize;
    while current < end {
        let mut info = mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::zeroed();
        let queried = unsafe {
            VirtualQuery(
                current as *const c_void,
                info.as_mut_ptr(),
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if queried == 0 {
            return false;
        }
        let info = unsafe { info.assume_init() };
        let Some(region_end) = (info.BaseAddress as usize).checked_add(info.RegionSize) else {
            return false;
        };
        if info.State != MEM_COMMIT || !permits(info.Protect) || region_end <= current {
            return false;
        }
        current = region_end.min(end);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guarded_reads_and_writes_accept_owned_memory() {
        let mut values = [0_u8; 8];
        let address = values.as_mut_ptr() as usize;

        assert!(readable_range(values.as_ptr(), values.len()));
        assert!(writable_range(values.as_ptr(), values.len()));
        assert!(unsafe { write_unaligned(address, 0x1122_3344_u32) });
        assert_eq!(unsafe { read_unaligned::<u32>(address) }, Some(0x1122_3344));
        assert!(unsafe { copy_bytes(values.as_mut_ptr().add(4), &[5, 6, 7, 8]) });
        assert!(unsafe { zero_bytes(values.as_mut_ptr().add(6), 2) });
        assert_eq!(values, [0x44, 0x33, 0x22, 0x11, 5, 6, 0, 0]);
    }

    #[test]
    fn guarded_ranges_reject_null_invalid_and_overflow_addresses() {
        assert!(readable_range(ptr::null(), 0));
        assert!(!readable_range(ptr::null(), 1));
        assert!(!readable_range(usize::MAX as *const u8, 1));
        assert!(!writable_range(usize::MAX as *const u8, 1));
        assert_eq!(unsafe { read_unaligned::<u32>(usize::MAX) }, None);
    }

    #[test]
    fn bounded_strings_copy_only_terminated_native_data() {
        assert_eq!(unsafe { bounded_c_string(ptr::null(), 1) }, None);
        assert_eq!(
            unsafe { bounded_c_string(b"abc\0".as_ptr(), 4) },
            Some(b"abc".to_vec())
        );
        assert_eq!(
            unsafe { bounded_c_string(b"ab\0c".as_ptr(), 4) },
            Some(b"ab".to_vec())
        );
        assert_eq!(unsafe { bounded_c_string(b"abc".as_ptr(), 3) }, None);
        assert_eq!(unsafe { bounded_c_string(b"\0".as_ptr(), 0) }, None);
    }

    #[test]
    fn strict_boolean_readers_reject_non_boolean_values() {
        let i32_false = 0_i32;
        let i32_true = 1_i32;
        let i32_invalid = 2_i32;
        let u8_false = 0_u8;
        let u8_true = 1_u8;
        let u8_invalid = 2_u8;

        assert_eq!(
            read_i32_bool((&i32_false as *const i32) as usize),
            Ok(false)
        );
        assert_eq!(read_i32_bool((&i32_true as *const i32) as usize), Ok(true));
        assert_eq!(
            read_i32_bool((&i32_invalid as *const i32) as usize),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(read_u8_bool((&u8_false as *const u8) as usize), Ok(false));
        assert_eq!(read_u8_bool((&u8_true as *const u8) as usize), Ok(true));
        assert_eq!(
            read_u8_bool((&u8_invalid as *const u8) as usize),
            Err(DirectClientError::NotReady)
        );
    }
}
