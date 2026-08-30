//! PE/module helpers for host-internal module inspection.

use crate::memory::read_unaligned;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

/// Returns the base address of the loaded module with `name`, or `None` when it
/// is not loaded or the name contains an interior NUL byte.
pub fn loaded_module(name: &str) -> Option<usize> {
    if name.as_bytes().contains(&0) {
        return None;
    }
    let mut ansi_name = Vec::with_capacity(name.len() + 1);
    ansi_name.extend_from_slice(name.as_bytes());
    ansi_name.push(0);
    let handle = unsafe { GetModuleHandleA(ansi_name.as_ptr()) };
    if handle.is_null() {
        None
    } else {
        Some(handle as usize)
    }
}

/// Reads the PE entry-point RVA at `base`.
///
/// Returns `None` when `base` does not point at a valid PE image (DOS signature,
/// PE signature, or optional-header magic mismatch).
///
/// # Safety
///
/// `base` must reference a mapped image that stays resident for the call.
pub unsafe fn pe_entry_point(base: usize) -> Option<u32> {
    if unsafe { read_unaligned::<u16>(base) }? != 0x5A4D {
        return None;
    }
    let nt_offset_address = base.checked_add(0x3C)?;
    let nt_offset = unsafe { read_unaligned::<u32>(nt_offset_address) }? as usize;
    let nt_header = base.checked_add(nt_offset)?;
    if unsafe { read_unaligned::<u32>(nt_header) }? != 0x0000_4550 {
        return None;
    }
    if unsafe { read_unaligned::<u16>(nt_header.checked_add(24)?) }? != 0x10B {
        return None;
    }
    unsafe { read_unaligned::<u32>(nt_header.checked_add(40)?) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pe_entry_point_rejects_invalid_dos_and_pe_signatures() {
        let mut image = [0_u8; 0x100];
        // Not a DOS signature.
        assert_eq!(unsafe { pe_entry_point(image.as_ptr() as usize) }, None);

        // DOS signature but no PE signature.
        image[0x00] = 0x4D;
        image[0x01] = 0x5A;
        image[0x3C] = 0x80;
        assert_eq!(unsafe { pe_entry_point(image.as_ptr() as usize) }, None);
    }

    #[test]
    fn pe_entry_point_rejects_overflowing_nt_header_offset() {
        let mut image = [0_u8; 0x40];
        image[0x00..0x02].copy_from_slice(&0x5A4D_u16.to_le_bytes());
        image[0x3C..0x40].copy_from_slice(&u32::MAX.to_le_bytes());

        assert_eq!(unsafe { pe_entry_point(image.as_ptr() as usize) }, None);
    }

    #[test]
    fn loaded_module_rejects_interior_nul() {
        assert_eq!(loaded_module("samp\0.dll"), None);
    }
}
