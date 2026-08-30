//! Host-only direct GTA San Andreas native backend.
//!
//! This crate owns verified GTA executable profile data and the
//! `CGame::Process` tick runtime. SA-MP remains a participant of that runtime;
//! no SA-MP address or layout belongs here.

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("gta-sa-native supports only 32-bit Windows x86 targets");

mod call;
mod camera;
mod layout;
mod ped;
mod pools;
mod profile;
mod tick;
mod timer;
mod world;

pub use call::{NativeCallError, NativeCallTarget};
pub use camera::{CameraReadError, camera_snapshot};
pub use layout::{RawMatrix, RawVector2, RawVector3};
pub use ped::{PedReadError, local_ped_handle, local_ped_snapshot, teleport_local_ped};
pub use pools::{
    CpoolRefAbi, CpoolRefError, PoolKind, PoolReadError, cpool_ref, object_exists, ped_exists,
    vehicle_exists, vehicle_snapshot,
};
pub use profile::{
    AbsoluteAddress, CameraSpec, EntityLayoutSpec, EvidenceGrade, FieldOffset, GTA_SA_10_US_SHA256,
    GameSpec, GtaIdentity, GtaPoolSpec, GtaProfile, GtaProfileError, GtaProfileSpec,
    NativeEvidence, ObjectLayoutSpec, ObjectSize, PedLayoutSpec, PedVtableSpec, PlayerSpec,
    PoolLayoutSpec, RelativeVirtualAddress, TimerSpec, VehicleLayoutSpec, VtableSlot, WorldSpec,
};

pub use timer::{TimerReadError, timer_snapshot};

pub use tick::{
    GameProcessFn, GameTickInstallError, GameTickParticipant, GameTickRuntime,
    GameTickShutdownError,
};
pub use world::{WorldReadError, find_ground_z};
