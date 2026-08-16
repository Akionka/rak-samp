//! R1 full live validator using the shared profile probe implementation.

#[path = "../../network_probe_common/src/lib.rs"]
mod probe;

use windows_sys::core::BOOL;

pub use probe::{FULL_SUCCESS_STATUS, MAIN_SUCCESS_STATUS};

/// Stops the callbacks before a hot-unload manager calls `FreeLibrary`.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR1NetworkProbe_Shutdown() -> BOOL {
    probe::SampClientSdkR5NetworkProbe_Shutdown()
}

/// Returns the R1 validator stage bitset.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR1NetworkProbe_Status() -> u32 {
    probe::SampClientSdkR5NetworkProbe_Status()
}

/// Returns the first R1 validator failure, if any.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR1NetworkProbe_Failure() -> u32 {
    probe::SampClientSdkR5NetworkProbe_Failure()
}
