//! PE/module helpers for host-internal module inspection.

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

/// Returns the base address of the loaded module with `name` (no extension), or
/// `None` when it is not loaded.
pub fn loaded_module(name: &str) -> Option<usize> {
    let mut wide = Vec::with_capacity(name.len() + 1);
    for byte in name.bytes() {
        if byte == 0 {
            break;
        }
        wide.push(byte);
    }
    wide.push(0);
    let handle = unsafe { GetModuleHandleA(wide.as_ptr()) };
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
    let image = base as *const u8;
    if unsafe { image.cast::<u16>().read_unaligned() } != 0x5A4D {
        return None;
    }
    let nt_offset = unsafe { image.add(0x3C).cast::<u32>().read_unaligned() } as usize;
    let nt_header = unsafe { image.add(nt_offset) };
    if unsafe { nt_header.cast::<u32>().read_unaligned() } != 0x0000_4550 {
        return None;
    }
    if unsafe { nt_header.add(24).cast::<u16>().read_unaligned() } != 0x10B {
        return None;
    }
    Some(unsafe { nt_header.add(40).cast::<u32>().read_unaligned() })
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
}
