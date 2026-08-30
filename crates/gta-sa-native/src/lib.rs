//! Host-only direct GTA San Andreas native backend.
//!
//! This crate owns verified GTA executable profile data and the
//! `CGame::Process` tick runtime. SA-MP remains a participant of that runtime;
//! no SA-MP address or layout belongs here.

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("gta-sa-native supports only 32-bit Windows x86 targets");

mod call;
mod layout;
mod ped;
mod pools;
mod profile;
mod tick;

pub use call::{NativeCallError, NativeCallTarget};
pub use layout::{RawMatrix, RawVector2, RawVector3};
pub use ped::{PedReadError, local_ped_handle, local_ped_snapshot, teleport_local_ped};
pub use pools::{CpoolRefAbi, CpoolRefError, cpool_ref};
pub use profile::{
    AbsoluteAddress, EntityLayoutSpec, EvidenceGrade, FieldOffset, GTA_SA_10_US_SHA256, GameSpec,
    GtaIdentity, GtaPoolSpec, GtaProfile, GtaProfileError, GtaProfileSpec, NativeEvidence,
    ObjectSize, PedLayoutSpec, PedVtableSpec, PlayerSpec, RelativeVirtualAddress, VtableSlot,
};
pub use tick::{
    GameProcessFn, GameTickInstallError, GameTickParticipant, GameTickRuntime,
    GameTickShutdownError,
};
