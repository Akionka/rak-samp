//! Windows x86 implementation primitives for the modkit host.
//!
//! This crate owns generic Windows/x86 concerns reused by the host backends:
//! guarded native-memory primitives and validated regions, PE/module helpers,
//! and a generic inline-hook wrapper around MinHook. It deliberately contains
//! no GTA or SA-MP addresses and no profile-specific constants.

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("modkit-win32 supports only 32-bit Windows x86 targets");

mod hooks;
mod memory;
mod module;

pub use hooks::{InlineHook, InlineHookError, write_protected};
pub use memory::{
    ReadableRegion, WritableRegion, bounded_c_string, copy_bytes, read_pointer, read_unaligned,
    readable_range, writable_range, write_unaligned, zero_bytes,
};
pub use module::{loaded_module, pe_entry_point};
