//! Guarded native-memory primitives for host-internal access to process memory.
//!
//! Every read/write path validates the target range against the OS page
//! protection before touching memory. Helpers return copied values or owned
//! byte buffers; native pointers remain inside the operation that resolved
//! them.

use std::{ffi::c_void, mem, ptr};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS, PAGE_READONLY, PAGE_READWRITE,
    PAGE_WRITECOPY, VirtualQuery,
};

/// A validated readable range of process memory.
///
/// Validation is performed once for the whole region; subsequent `read`
/// operations are checked offset reads within that validated range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadableRegion {
    start: usize,
    len: usize,
}

/// A validated writable range of process memory.
///
/// Validation is performed once for the whole region; subsequent `write`
/// operations are checked offset writes within that validated range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WritableRegion {
    start: usize,
    len: usize,
}

/// Returns whether the byte range is readable (committed and not guarded/no-access).
pub fn readable_range(address: *const u8, length: usize) -> bool {
    guarded_range(address, length, |protection| {
        protection & (PAGE_GUARD | PAGE_NOACCESS) == 0
            && matches!(
                protection & 0xFF,
                PAGE_READONLY
                    | PAGE_READWRITE
                    | PAGE_WRITECOPY
                    | PAGE_EXECUTE_READ
                    | PAGE_EXECUTE_READWRITE
                    | PAGE_EXECUTE_WRITECOPY
            )
    })
}

/// Returns whether the byte range is writable (committed with a write-capable protection).
pub fn writable_range(address: *const u8, length: usize) -> bool {
    guarded_range(address, length, |protection| {
        protection & (PAGE_GUARD | PAGE_NOACCESS) == 0
            && matches!(
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

impl ReadableRegion {
    /// Validates that `[address, address + len)` is one readable range.
    pub fn validate(address: usize, len: usize) -> Option<Self> {
        (len != 0 && readable_range(address as *const u8, len)).then_some(Self {
            start: address,
            len,
        })
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reads a `Copy` value at `offset` within the validated region.
    ///
    /// # Safety
    ///
    /// The region must still describe valid committed memory at the time of the
    /// call; the caller must not race an unmap, free, or protection change.
    pub unsafe fn read_unaligned<T: Copy>(&self, offset: usize) -> Option<T> {
        let length = mem::size_of::<T>();
        (length != 0 && self.contains(offset, length))
            .then(|| unsafe { ((self.start + offset) as *const T).read_unaligned() })
    }

    /// Returns a sub-region at `offset` of `len` bytes within this region.
    pub fn subregion(&self, offset: usize, len: usize) -> Option<Self> {
        self.contains(offset, len).then_some(Self {
            start: self.start + offset,
            len,
        })
    }

    fn contains(&self, offset: usize, len: usize) -> bool {
        offset.checked_add(len).is_some_and(|end| end <= self.len)
    }
}

impl WritableRegion {
    /// Validates that `[address, address + len)` is one writable range.
    pub fn validate(address: usize, len: usize) -> Option<Self> {
        (len != 0 && writable_range(address as *const u8, len)).then_some(Self {
            start: address,
            len,
        })
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Writes a `Copy` value at `offset` within the validated region.
    ///
    /// # Safety
    ///
    /// The region must still describe valid committed memory at the time of the
    /// call; the caller must not race an unmap, free, or protection change.
    pub unsafe fn write_unaligned<T: Copy>(&self, offset: usize, value: T) -> bool {
        let length = mem::size_of::<T>();
        if length == 0 || !self.contains(offset, length) {
            return false;
        }
        unsafe { ((self.start + offset) as *mut T).write_unaligned(value) };
        true
    }

    /// Copies `source` bytes at `offset` within the validated region.
    ///
    /// # Safety
    ///
    /// The region must still describe valid committed memory at the time of the
    /// call; the caller must not race an unmap, free, or protection change, and
    /// `source` must not overlap the destination range.
    pub unsafe fn copy_bytes(&self, offset: usize, source: &[u8]) -> bool {
        if !self.contains(offset, source.len()) {
            return false;
        }
        if source.is_empty() {
            return true;
        }
        unsafe {
            ptr::copy_nonoverlapping(
                source.as_ptr(),
                (self.start + offset) as *mut u8,
                source.len(),
            )
        };
        true
    }

    /// Zeroes `length` bytes at `offset` within the validated region.
    ///
    /// # Safety
    ///
    /// The region must still describe valid committed memory at the time of the
    /// call; the caller must not race an unmap, free, or protection change.
    pub unsafe fn zero_bytes(&self, offset: usize, length: usize) -> bool {
        if !self.contains(offset, length) {
            return false;
        }
        if length == 0 {
            return true;
        }
        unsafe { ptr::write_bytes((self.start + offset) as *mut u8, 0, length) };
        true
    }

    /// Returns a sub-region at `offset` of `len` bytes within this region.
    pub fn subregion(&self, offset: usize, len: usize) -> Option<Self> {
        self.contains(offset, len).then_some(Self {
            start: self.start + offset,
            len,
        })
    }

    fn contains(&self, offset: usize, len: usize) -> bool {
        offset.checked_add(len).is_some_and(|end| end <= self.len)
    }
}

/// Reads a native pointer-sized value as an opaque byte pointer.
///
/// # Safety
///
/// `address` must not race an unmap, free, or protection change.
pub unsafe fn read_pointer(address: usize) -> Option<*mut u8> {
    unsafe { read_unaligned::<usize>(address) }.map(|value| value as *mut u8)
}

/// Reads a `Copy` value at `address`, validating the range first.
///
/// # Safety
///
/// `address` must not race an unmap, free, or protection change.
pub unsafe fn read_unaligned<T: Copy>(address: usize) -> Option<T> {
    let length = mem::size_of::<T>();
    (length != 0 && readable_range(address as *const u8, length))
        .then(|| unsafe { (address as *const T).read_unaligned() })
}

/// Writes a `Copy` value at `address`, validating the range first.
///
/// # Safety
///
/// `address` must not race an unmap, free, or protection change.
pub unsafe fn write_unaligned<T: Copy>(address: usize, value: T) -> bool {
    let length = mem::size_of::<T>();
    if length == 0 || !writable_range(address as *const u8, length) {
        return false;
    }
    unsafe { (address as *mut T).write_unaligned(value) };
    true
}

/// Copies `source` bytes to `destination`, validating the range first.
///
/// # Safety
///
/// `destination` must not race an unmap, free, or protection change. `source`
/// must remain valid and must not overlap `destination`.
pub unsafe fn copy_bytes(destination: *mut u8, source: &[u8]) -> bool {
    if source.is_empty() {
        return true;
    }
    if !writable_range(destination.cast_const(), source.len()) {
        return false;
    }
    unsafe { ptr::copy_nonoverlapping(source.as_ptr(), destination, source.len()) };
    true
}

/// Zeroes `length` bytes at `destination`, validating the range first.
///
/// # Safety
///
/// `destination` must not race an unmap, free, or protection change.
pub unsafe fn zero_bytes(destination: *mut u8, length: usize) -> bool {
    if length == 0 {
        return true;
    }
    if !writable_range(destination.cast_const(), length) {
        return false;
    }
    unsafe { ptr::write_bytes(destination, 0, length) };
    true
}

/// Copies a bounded NUL-terminated byte string, returning the copied bytes.
///
/// Returns `None` when the string is null, exceeds `maximum` bytes, or crosses
/// an unreadable page before the terminator.
///
/// # Safety
///
/// `pointer` must not race an unmap, free, or protection change.
pub unsafe fn bounded_c_string(pointer: *const u8, maximum: usize) -> Option<Vec<u8>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Memory::{
        MEM_RELEASE, MEM_RESERVE, VirtualAlloc, VirtualFree, VirtualProtect,
    };

    const PAGE_SIZE: usize = 0x1000;

    struct VirtualAllocation(*mut c_void);

    impl VirtualAllocation {
        fn new(length: usize) -> Self {
            let address = unsafe {
                VirtualAlloc(
                    ptr::null(),
                    length,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                )
            };
            assert!(!address.is_null(), "VirtualAlloc failed");
            Self(address)
        }

        fn bytes(&self) -> *mut u8 {
            self.0.cast()
        }

        fn protect(&self, offset: usize, length: usize, protection: u32) {
            let mut previous = 0;
            let changed = unsafe {
                VirtualProtect(
                    self.bytes().add(offset).cast(),
                    length,
                    protection,
                    &mut previous,
                )
            };
            assert_ne!(changed, 0, "VirtualProtect failed");
        }
    }

    impl Drop for VirtualAllocation {
        fn drop(&mut self) {
            let released = unsafe { VirtualFree(self.0, 0, MEM_RELEASE) };
            assert_ne!(released, 0, "VirtualFree failed");
        }
    }

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
        assert_eq!(unsafe { read_unaligned::<()>(0) }, None);
        assert!(!unsafe { write_unaligned(0, ()) });
    }

    #[test]
    fn guarded_ranges_follow_page_protection_and_modifiers() {
        let allocation = VirtualAllocation::new(PAGE_SIZE * 2);
        let address = allocation.bytes();

        assert!(readable_range(address, PAGE_SIZE * 2));
        assert!(writable_range(address, PAGE_SIZE * 2));

        allocation.protect(0, PAGE_SIZE, PAGE_READONLY);
        assert!(readable_range(address, PAGE_SIZE));
        assert!(!writable_range(address, PAGE_SIZE));

        allocation.protect(0, PAGE_SIZE, PAGE_NOACCESS);
        assert!(!readable_range(address, PAGE_SIZE));
        assert!(!writable_range(address, PAGE_SIZE));

        allocation.protect(0, PAGE_SIZE, PAGE_READWRITE | PAGE_GUARD);
        assert!(!readable_range(address, PAGE_SIZE));
        assert!(!writable_range(address, PAGE_SIZE));
    }

    #[test]
    fn guarded_ranges_validate_each_page_in_a_cross_page_range() {
        let allocation = VirtualAllocation::new(PAGE_SIZE * 2);
        let address = allocation.bytes();
        allocation.protect(PAGE_SIZE, PAGE_SIZE, PAGE_READONLY);

        assert!(readable_range(address, PAGE_SIZE * 2));
        assert!(!writable_range(address, PAGE_SIZE * 2));
    }

    #[test]
    fn empty_writes_do_not_touch_null_destinations() {
        assert!(unsafe { copy_bytes(ptr::null_mut(), &[]) });
        assert!(unsafe { zero_bytes(ptr::null_mut(), 0) });
    }

    #[test]
    fn bounded_strings_copy_only_terminated_native_data() {
        assert_eq!(unsafe { bounded_c_string(ptr::null(), 1) }, None);
        assert_eq!(
            unsafe { bounded_c_string(c"abc".as_ptr().cast(), 4) },
            Some(b"abc".to_vec())
        );
        assert_eq!(
            unsafe { bounded_c_string(b"ab\0c".as_ptr(), 4) },
            Some(b"ab".to_vec())
        );
        assert_eq!(unsafe { bounded_c_string(b"abc".as_ptr(), 3) }, None);
        assert_eq!(unsafe { bounded_c_string(c"".as_ptr().cast(), 0) }, None);
    }

    #[test]
    fn regions_validate_owned_memory_and_bounds_offsets() {
        let mut values = [0_u8; 16];
        let base = values.as_mut_ptr() as usize;

        let readable = ReadableRegion::validate(base, 16).expect("readable region");
        let writable = WritableRegion::validate(base, 16).expect("writable region");

        assert_eq!(unsafe { readable.read_unaligned::<u32>(0) }, Some(0));
        assert_eq!(unsafe { readable.read_unaligned::<u32>(13) }, None);
        assert_eq!(readable.subregion(4, 8).map(|r| r.len()), Some(8));
        assert_eq!(readable.subregion(12, 8), None);

        assert!(unsafe { writable.write_unaligned::<u32>(0, 0xDEAD_BEEF) });
        assert_eq!(
            unsafe { readable.read_unaligned::<u32>(0) },
            Some(0xDEAD_BEEF)
        );
        assert!(!unsafe { writable.write_unaligned::<u32>(15, 1) });
        assert_eq!(unsafe { readable.read_unaligned::<()>(0) }, None);
        assert!(!unsafe { writable.write_unaligned(0, ()) });
        assert!(unsafe { writable.copy_bytes(8, &[1, 2, 3]) });
        assert!(unsafe { writable.zero_bytes(11, 2) });
        assert_eq!(values[8..13], [1, 2, 3, 0, 0]);
    }

    #[test]
    fn regions_reject_overflow_and_invalid_addresses() {
        assert_eq!(ReadableRegion::validate(usize::MAX, 1), None);
        assert_eq!(WritableRegion::validate(usize::MAX, 1), None);
        assert_eq!(ReadableRegion::validate(0, 0), None);
        assert_eq!(WritableRegion::validate(0, 0), None);
        assert_eq!(ReadableRegion::validate(0x1000, 0), None);
    }
}
