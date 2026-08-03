//! In-game validation plugin for the rak-samp native packet and RPC paths.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_validation_plugin supports only 32-bit Windows x86 targets");

mod callbacks;
mod lifecycle;
mod logging;
mod reporting;
mod self_test;
mod state;

use std::{ffi::c_void, sync::atomic::Ordering};
use windows_sys::{
    Win32::{
        Foundation::{HINSTANCE, TRUE},
        System::{
            LibraryLoader::DisableThreadLibraryCalls,
            SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        },
    },
    core::BOOL,
};

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(instance) };
            lifecycle::start(instance);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

/// Stops workers and callbacks before an unload manager calls `FreeLibrary`.
///
/// This must be called from a worker thread, not from `DllMain` or a rak-samp callback.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampPlugin_Shutdown() -> BOOL {
    lifecycle::shutdown()
}

/// Returns the number of incoming packet callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampValidation_IncomingPacketCount() -> u32 {
    state::METRICS.incoming_packets.load(Ordering::Relaxed)
}

/// Returns the number of outgoing packet callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampValidation_OutgoingPacketCount() -> u32 {
    state::METRICS.outgoing_packets.load(Ordering::Relaxed)
}

/// Returns the number of incoming RPC callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampValidation_IncomingRpcCount() -> u32 {
    state::METRICS.incoming_rpcs.load(Ordering::Relaxed)
}

/// Returns the number of outgoing RPC callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampValidation_OutgoingRpcCount() -> u32 {
    state::METRICS.outgoing_rpcs.load(Ordering::Relaxed)
}

/// Reports whether all enabled local, send, and emulation self-tests finished.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampValidation_SelfTestsComplete() -> BOOL {
    let statuses = [
        state::SELF_TESTS.packet.load(Ordering::Acquire),
        state::SELF_TESTS.rpc.load(Ordering::Acquire),
        state::SELF_TESTS.dialog.load(Ordering::Acquire),
        state::SELF_TESTS.direct_client.load(Ordering::Acquire),
        state::SELF_TESTS
            .direct_snapshot_state
            .load(Ordering::Acquire),
        state::SELF_TESTS.player_directory.load(Ordering::Acquire),
        state::SELF_TESTS
            .remote_player_state
            .load(Ordering::Acquire),
        state::SELF_TESTS.vehicle_exists.load(Ordering::Acquire),
        state::SELF_TESTS.text_label_exists.load(Ordering::Acquire),
        state::SELF_TESTS.text_label.load(Ordering::Acquire),
        state::SELF_TESTS.textdraw_exists.load(Ordering::Acquire),
        state::SELF_TESTS.textdraw.load(Ordering::Acquire),
        state::SELF_TESTS.object_exists.load(Ordering::Acquire),
        state::SELF_TESTS.gangzone.load(Ordering::Acquire),
        state::SELF_TESTS.send_packet.load(Ordering::Acquire),
        state::SELF_TESTS.send_rpc.load(Ordering::Acquire),
    ];
    BOOL::from(statuses.into_iter().all(state::self_test_finished))
}
