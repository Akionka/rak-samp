//! Generic inline-hook wrapper around MinHook for host-internal detours.

use std::{ffi::c_void, mem};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE,
    VirtualProtect, VirtualQuery,
};

/// Failure to create a hook or change native page protection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineHookError {
    InvalidAddress,
    CreateFailed,
    EnableFailed,
    DisableFailed,
    RemoveFailed,
    ProtectFailed,
    RestoreProtectionFailed,
}

/// A created-but-not-yet-enabled MinHook inline hook.
///
/// The hook is disabled on creation so the caller can publish the trampoline
/// before enabling it. Dropping an enabled hook disables and removes it.
#[derive(Debug)]
pub struct InlineHook {
    name: &'static str,
    target: usize,
    detour: usize,
    trampoline: usize,
    enabled: bool,
}

impl InlineHook {
    /// Creates the hook for `target` with `detour` without enabling it.
    ///
    /// # Safety
    ///
    /// Both addresses must remain resident executable functions with compatible
    /// calling conventions and signatures for the lifetime of the hook.
    pub unsafe fn create(
        name: &'static str,
        target: usize,
        detour: usize,
    ) -> Result<(Self, usize), InlineHookError> {
        if target == 0 || detour == 0 {
            return Err(InlineHookError::InvalidAddress);
        }
        let trampoline = unsafe {
            minhook::MinHook::create_hook(target as *mut c_void, detour as *mut c_void)
                .map_err(|_| InlineHookError::CreateFailed)?
        };
        if trampoline.is_null() {
            if let Err(error) = unsafe { minhook::MinHook::remove_hook(target as *mut c_void) } {
                log::warn!("failed to remove MinHook entry after a null trampoline: {error:?}");
            }
            return Err(InlineHookError::CreateFailed);
        }
        Ok((
            Self {
                name,
                target,
                detour,
                trampoline: trampoline as usize,
                enabled: false,
            },
            trampoline as usize,
        ))
    }

    /// Enables the hook.
    pub fn enable(&mut self) -> Result<(), InlineHookError> {
        unsafe { minhook::MinHook::enable_hook(self.target as *mut c_void) }
            .map_err(|_| InlineHookError::EnableFailed)?;
        self.enabled = true;
        log::debug!(
            "enabled MinHook inline hook {name}: target=0x{target:08X}, detour=0x{detour:08X}, trampoline=0x{trampoline:08X}",
            name = self.name,
            target = self.target,
            detour = self.detour,
            trampoline = self.trampoline,
        );
        Ok(())
    }

    /// Disables and removes the hook.
    pub fn disable(mut self) {
        self.remove_best_effort();
    }

    /// Disables the hook while retaining its MinHook entry for a later removal.
    pub fn try_disable(&mut self) -> Result<(), InlineHookError> {
        if self.target == 0 || !self.enabled {
            return Ok(());
        }
        unsafe { minhook::MinHook::disable_hook(self.target as *mut c_void) }
            .map_err(|_| InlineHookError::DisableFailed)?;
        self.enabled = false;
        Ok(())
    }

    /// Disables and removes the hook, retaining ownership on failure.
    pub fn try_remove(&mut self) -> Result<(), InlineHookError> {
        if self.target == 0 {
            return Ok(());
        }
        self.try_disable()?;
        unsafe { minhook::MinHook::remove_hook(self.target as *mut c_void) }
            .map_err(|_| InlineHookError::RemoveFailed)?;
        self.target = 0;
        self.trampoline = 0;
        Ok(())
    }

    fn remove_best_effort(&mut self) {
        if let Err(error) = self.try_remove() {
            log::warn!(
                "failed to disable or remove MinHook inline hook {}: {error:?}",
                self.name,
            );
        }
    }
}

impl Drop for InlineHook {
    fn drop(&mut self) {
        self.remove_best_effort();
    }
}

/// Writes `value` to `address`, changing page protection to writable and back.
///
/// # Safety
///
/// `address` must reference initialized committed memory that stays resident
/// for the call and does not race another protection change.
pub unsafe fn write_protected<T: Copy>(address: *mut T, value: T) -> Result<(), InlineHookError> {
    let length = mem::size_of::<T>();
    if address.is_null() || length == 0 {
        return Err(InlineHookError::InvalidAddress);
    }
    let start = address as usize;
    let end = start
        .checked_add(length)
        .ok_or(InlineHookError::InvalidAddress)?;
    let mut info = mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::zeroed();
    let queried = unsafe {
        VirtualQuery(
            address.cast(),
            info.as_mut_ptr(),
            mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if queried == 0 {
        return Err(InlineHookError::InvalidAddress);
    }
    let info = unsafe { info.assume_init() };
    let region_start = info.BaseAddress as usize;
    let region_end = region_start
        .checked_add(info.RegionSize)
        .ok_or(InlineHookError::InvalidAddress)?;
    if info.State != MEM_COMMIT
        || info.Protect & (PAGE_GUARD | PAGE_NOACCESS) != 0
        || start < region_start
        || end > region_end
    {
        return Err(InlineHookError::InvalidAddress);
    }

    let mut old_protection = 0;
    if unsafe { VirtualProtect(address.cast(), length, PAGE_READWRITE, &mut old_protection) } == 0 {
        return Err(InlineHookError::ProtectFailed);
    }
    unsafe { address.write_unaligned(value) };
    let mut ignored = 0;
    if unsafe { VirtualProtect(address.cast(), length, old_protection, &mut ignored) } == 0 {
        return Err(InlineHookError::RestoreProtectionFailed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    };
    use windows_sys::Win32::System::Memory::{
        MEM_RELEASE, MEM_RESERVE, PAGE_READONLY, VirtualAlloc, VirtualFree,
    };

    static TEST_TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[inline(never)]
    unsafe extern "C" fn target(value: i32) -> i32 {
        value + 1
    }

    #[inline(never)]
    unsafe extern "C" fn detour(value: i32) -> i32 {
        let trampoline = TEST_TRAMPOLINE.load(Ordering::Acquire);
        if trampoline == 0 {
            return i32::MIN;
        }
        let original: unsafe extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(trampoline) };
        unsafe { original(value) + 10 }
    }

    #[test]
    fn publishes_trampoline_before_enabling_and_can_recreate_inline_hook() {
        let _serial = TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let target = target as *const () as usize;
        let detour = detour as *const () as usize;

        let (mut hook, trampoline) =
            unsafe { InlineHook::create("test target", target, detour) }.unwrap();

        assert_eq!(unsafe { self::target(7) }, 8);
        TEST_TRAMPOLINE.store(trampoline, Ordering::Release);
        hook.enable().unwrap();
        assert_eq!(unsafe { self::target(7) }, 18);

        hook.try_disable().unwrap();
        assert_eq!(unsafe { self::target(7) }, 8);
        hook.try_remove().unwrap();

        let (recreated, recreated_trampoline) =
            unsafe { InlineHook::create("test target", target, detour) }.unwrap();
        assert_ne!(recreated_trampoline, 0);
        recreated.disable();
        TEST_TRAMPOLINE.store(0, Ordering::Release);
    }

    #[test]
    fn rejects_invalid_hook_and_protected_write_addresses() {
        assert!(matches!(
            unsafe { InlineHook::create("invalid", 0, detour as *const () as usize) },
            Err(InlineHookError::InvalidAddress)
        ));
        assert_eq!(
            unsafe { write_protected(std::ptr::null_mut(), 1_u32) },
            Err(InlineHookError::InvalidAddress)
        );
    }

    #[test]
    fn protected_write_supports_unaligned_values_and_restores_protection() {
        const PAGE_SIZE: usize = 0x1000;
        let allocation = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                PAGE_SIZE,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };
        assert!(!allocation.is_null(), "VirtualAlloc failed");

        let mut previous = 0;
        assert_ne!(
            unsafe { VirtualProtect(allocation, PAGE_SIZE, PAGE_READONLY, &mut previous) },
            0,
            "VirtualProtect failed"
        );

        let target = unsafe { allocation.cast::<u8>().add(1).cast::<u32>() };
        assert_eq!(unsafe { write_protected(target, 0x1122_3344) }, Ok(()));
        assert_eq!(unsafe { target.read_unaligned() }, 0x1122_3344);

        let mut info = mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::zeroed();
        assert_ne!(
            unsafe {
                VirtualQuery(
                    allocation,
                    info.as_mut_ptr(),
                    mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            },
            0
        );
        assert_eq!(unsafe { info.assume_init() }.Protect, PAGE_READONLY);
        assert_ne!(unsafe { VirtualFree(allocation, 0, MEM_RELEASE) }, 0);
    }
}
