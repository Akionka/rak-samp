//! Generic inline-hook wrapper around MinHook for host-internal detours.

use std::ffi::c_void;
use windows_sys::Win32::System::Memory::{PAGE_READWRITE, VirtualProtect};

/// Failure to create or enable an inline hook.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InlineHookError;

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
    pub fn create(
        name: &'static str,
        target: usize,
        detour: usize,
    ) -> Result<(Self, usize), InlineHookError> {
        let trampoline = unsafe {
            minhook::MinHook::create_hook(target as *mut c_void, detour as *mut c_void)
                .map_err(|_| InlineHookError)?
        };
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
            .map_err(|_| InlineHookError)?;
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
        self.remove();
    }

    fn remove(&mut self) {
        if self.target == 0 {
            return;
        }
        let target = self.target as *mut c_void;
        if self.enabled {
            let _ = unsafe { minhook::MinHook::disable_hook(target) };
        }
        let _ = unsafe { minhook::MinHook::remove_hook(target) };
        self.target = 0;
        self.enabled = false;
    }
}

impl Drop for InlineHook {
    fn drop(&mut self) {
        self.remove();
    }
}

/// Writes `value` to `address`, changing page protection to writable and back.
///
/// # Safety
///
/// `address` must reference writable-capable committed memory that stays
/// resident for the call.
pub unsafe fn write_protected<T>(address: *mut T, value: T) -> Result<(), InlineHookError> {
    let mut old_protection = 0;
    if unsafe {
        VirtualProtect(
            address.cast(),
            std::mem::size_of::<T>(),
            PAGE_READWRITE,
            &mut old_protection,
        )
    } == 0
    {
        return Err(InlineHookError);
    }
    unsafe { address.write(value) };
    let mut ignored = 0;
    let _ = unsafe {
        VirtualProtect(
            address.cast(),
            std::mem::size_of::<T>(),
            old_protection,
            &mut ignored,
        )
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
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

        let (mut hook, trampoline) = InlineHook::create("test target", target, detour).unwrap();

        assert_eq!(unsafe { self::target(7) }, 8);
        TEST_TRAMPOLINE.store(trampoline, Ordering::Release);
        hook.enable().unwrap();
        assert_eq!(unsafe { self::target(7) }, 18);

        hook.disable();
        assert_eq!(unsafe { self::target(7) }, 8);

        let (recreated, recreated_trampoline) =
            InlineHook::create("test target", target, detour).unwrap();
        assert_ne!(recreated_trampoline, 0);
        recreated.disable();
        TEST_TRAMPOLINE.store(0, Ordering::Release);
    }
}
