//! Independently loaded plugin used by the ASI ABI end-to-end fixture.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_e2e_plugin supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    LocalDialog, LocalDialogStyle, RakSampDirection, RakSampHookAction, Subscription,
    raknet::BitStream, wait_for_default_host,
};
use std::{
    ffi::c_void,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{LibraryLoader::DisableThreadLibraryCalls, SystemServices::DLL_PROCESS_ATTACH},
};
use windows_sys::core::BOOL;

const TEST_RPC_ID: u8 = 42;

static SUBSCRIPTION: Mutex<Option<Subscription>> = Mutex::new(None);
static READY: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);
static CALLBACKS: AtomicU32 = AtomicU32::new(0);
static DIALOG_RESULT: AtomicU32 = AtomicU32::new(u32::MAX);
static LOCAL_PLAYER_ID: AtomicU32 = AtomicU32::new(u32::MAX);
static SAMP_GAME_STATE: AtomicI32 = AtomicI32::new(i32::MIN);
static SAMP_VERSION: AtomicU32 = AtomicU32::new(u32::MAX);
static DECODE_RESULT: AtomicU32 = AtomicU32::new(u32::MAX);

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
    let dialog_result = api.show_local_dialog(LocalDialog {
        id: 0x7000,
        style: LocalDialogStyle::MessageBox,
        title: b"e2e",
        text: b"direct local dialog",
        button1: b"ok",
        button2: b"",
    });
    DIALOG_RESULT.store(dialog_result as u32, Ordering::Release);
    if let Ok(local_player) = api.local_player() {
        LOCAL_PLAYER_ID.store(u32::from(local_player.id), Ordering::Release);
    }
    if let Ok(game_state) = api.samp_game_state() {
        SAMP_GAME_STATE.store(game_state, Ordering::Release);
    }
    if let Ok(version) = api.samp_version() {
        SAMP_VERSION.store(version as u32, Ordering::Release);
    }
    let mut compressed = match BitStream::from_bits(vec![0b1010_0000], 3) {
        Ok(value) => value,
        Err(_) => return,
    };
    if let Ok(decoded) = api.decode_string(&mut compressed)
        && decoded == b"e2e"
        && compressed.read_offset_bits() == 3
    {
        DECODE_RESULT.store(1, Ordering::Release);
    }
    let subscription = api.on_rpc_id(RakSampDirection::Incoming, TEST_RPC_ID, |_| {
        CALLBACKS.fetch_add(1, Ordering::AcqRel);
        RakSampHookAction::Continue
    });
    let Ok(subscription) = subscription else {
        return;
    };
    if STOP.load(Ordering::Acquire) {
        let _ = subscription.unregister_and_wait();
        return;
    }
    let mut slot = SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if slot.is_some() {
        drop(slot);
        let _ = subscription.unregister_and_wait();
        return;
    }
    *slot = Some(subscription);
    READY.store(true, Ordering::Release);
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
pub extern "system" fn RakSampE2ePlugin_DialogResult() -> u32 {
    DIALOG_RESULT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalPlayerId() -> u32 {
    LOCAL_PLAYER_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_SampGameState() -> i32 {
    SAMP_GAME_STATE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_SampVersion() -> u32 {
    SAMP_VERSION.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_DecodeResult() -> u32 {
    DECODE_RESULT.load(Ordering::Acquire)
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
    i32::from(subscription.unregister_and_wait().is_ok())
}
