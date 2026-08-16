//! DL-R1 full live validator using the shared profile probe engine.

#[path = "../../network_probe_common/src/lib.rs"]
mod probe;

use windows_sys::core::BOOL;

pub use probe::{FULL_SUCCESS_STATUS, MAIN_SUCCESS_STATUS};

#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkDlNetworkProbe_Shutdown() -> BOOL {
    probe::SampClientSdkR5NetworkProbe_Shutdown()
}

#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkDlNetworkProbe_Status() -> u32 {
    probe::SampClientSdkR5NetworkProbe_Status()
}

#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkDlNetworkProbe_Failure() -> u32 {
    probe::SampClientSdkR5NetworkProbe_Failure()
}
