//! Independently loaded plugin used by the ASI ABI end-to-end fixture.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_e2e_plugin supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    HostApi, RakSampDirection, RakSampEventV1, RakSampHookAction, RakSampResult,
    RakSampSubscription, wait_for_default_host,
};
use std::{
    ffi::c_void,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{LibraryLoader::DisableThreadLibraryCalls, SystemServices::DLL_PROCESS_ATTACH},
};
use windows_sys::core::BOOL;

const TEST_RPC_ID: u8 = 42;

static API: OnceLock<HostApi> = OnceLock::new();
static SUBSCRIPTION: Mutex<Option<RakSampSubscription>> = Mutex::new(None);
static READY: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);
static CALLBACKS: AtomicU32 = AtomicU32::new(0);

#[unsafe(no_mangle)]
/// Windows invokes this with loader-owned arguments while the plugin module is loaded.
///
/// # Safety
///
/// It must only be called by the Windows loader with a valid module handle and reason code.
pub unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(instance) };
        let _ = std::thread::Builder::new()
            .name("rak-samp-e2e-plugin".into())
            .spawn(initialize);
    }
    TRUE
}

fn initialize() {
    let Ok(api) = wait_for_default_host(Duration::from_secs(5)) else {
        return;
    };
    if STOP.load(Ordering::Acquire) {
        return;
    }
    let mut subscription = RakSampSubscription::default();
    let result = unsafe {
        (api.raw().register_rpc)(
            RakSampDirection::Incoming,
            Some(on_incoming_rpc),
            std::ptr::null_mut(),
            &raw mut subscription,
        )
    };
    if result != RakSampResult::Ok || STOP.load(Ordering::Acquire) {
        if result == RakSampResult::Ok {
            let _ = api.unregister_and_wait(subscription);
        }
        return;
    }
    if API.set(api).is_err() {
        let _ = api.unregister_and_wait(subscription);
        return;
    }
    *SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(subscription);
    READY.store(true, Ordering::Release);
}

unsafe extern "system" fn on_incoming_rpc(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    let Some(api) = API.get().copied() else {
        return RakSampHookAction::Continue;
    };
    let id = unsafe { (api.raw().event_id)(event) };
    if id == TEST_RPC_ID {
        CALLBACKS.fetch_add(1, Ordering::AcqRel);
    }
    RakSampHookAction::Continue
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_Ready() -> i32 {
    i32::from(READY.load(Ordering::Acquire))
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_CallbackCount() -> u32 {
    CALLBACKS.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_Shutdown() -> i32 {
    STOP.store(true, Ordering::Release);
    READY.store(false, Ordering::Release);
    let subscription = SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take();
    let Some(subscription) = subscription else {
        return 1;
    };
    let Some(api) = API.get().copied() else {
        return 0;
    };
    i32::from(matches!(
        api.unregister_and_wait(subscription),
        RakSampResult::Ok | RakSampResult::SubscriptionNotFound
    ))
}
