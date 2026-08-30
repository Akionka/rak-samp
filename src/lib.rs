#![doc = include_str!("../CORE.md")]
#![deny(unsafe_op_in_unsafe_fn)]

//! `samp_client_sdk` is a Windows x86 SDK for observing and controlling SA-MP client
//! packet and RPC traffic from a Rust ASI/DLL.
//!
//! The safe API deliberately exposes no client pointers. Native hooks and
//! SA-MP version-specific offsets remain private implementation details.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp-client-sdk supports only 32-bit Windows x86 targets");

mod bitstream;
mod client;
mod command;
mod event;
mod host_api;
mod logging;
mod platform;
mod runtime;

#[cfg(all(windows, target_pointer_width = "32"))]
use core::ffi::c_void;
#[cfg(all(windows, target_pointer_width = "32"))]
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

pub use bitstream::{BitStream, BitStreamError};
pub use client::{AddressSet, SampVersion};
pub use event::{
    Direction, HookAction, ListenerHandle, ListenerId, ListenerRegistrationError, PacketEvent,
    PacketHandler, RpcEvent, RpcHandler,
};
pub use runtime::{
    AttachError, PacketPriority, PacketReliability, Runtime, SendError, SendOptions,
};

#[cfg(all(windows, target_pointer_width = "32"))]
#[unsafe(no_mangle)]
/// Windows loader entry point for the `samp_client_sdk.asi` host.
///
/// # Safety
///
/// Windows calls this function with valid loader-owned arguments. External
/// callers must not invoke it directly.
pub unsafe extern "system" fn DllMain(
    _module: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        host_api::begin_bootstrap();
    }
    1
}
