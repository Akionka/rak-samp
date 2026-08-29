//! Host-only direct GTA San Andreas native backend.
//!
//! This crate owns verified GTA executable profile data and the
//! `CGame::Process` tick runtime. SA-MP remains a participant of that runtime;
//! no SA-MP address or layout belongs here.

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("gta-sa-native supports only 32-bit Windows x86 targets");

mod profile;
mod tick;

pub use profile::{AbsoluteAddress, GameSpec, GtaIdentity, GtaProfile, GtaProfileSpec};
pub use tick::{GameProcessFn, GameTickInstallError, GameTickParticipant, GameTickRuntime};
