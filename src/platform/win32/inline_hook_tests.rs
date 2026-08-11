use super::*;
use std::mem;
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
    let original: unsafe extern "C" fn(i32) -> i32 = unsafe { mem::transmute(trampoline) };
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

    let (mut hook, trampoline) = InlineHook::create(target, detour).unwrap();

    // Creation must leave the target disabled until the caller publishes
    // the trampoline used by the detour.
    assert_eq!(unsafe { self::target(7) }, 8);
    TEST_TRAMPOLINE.store(trampoline, Ordering::Release);
    hook.enable().unwrap();
    assert_eq!(unsafe { self::target(7) }, 18);

    hook.disable();
    assert_eq!(unsafe { self::target(7) }, 8);

    let (recreated, recreated_trampoline) = InlineHook::create(target, detour).unwrap();
    assert_ne!(recreated_trampoline, 0);
    recreated.disable();
    TEST_TRAMPOLINE.store(0, Ordering::Release);
}
