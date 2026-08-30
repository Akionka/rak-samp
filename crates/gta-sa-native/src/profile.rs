//! Data-only GTA executable profiles.

use sha2::{Digest, Sha256};
use std::{env, fs::File, io::Read, ptr};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

/// SHA-256 of the verified GTA SA 1.0 US executable.
pub const GTA_SA_10_US_SHA256: [u8; 32] = [
    0xA5, 0x59, 0xAA, 0x77, 0x2F, 0xD1, 0x36, 0x37, 0x91, 0x55, 0xEF, 0xA7, 0x1F, 0x00, 0xC4, 0x7A,
    0xAD, 0x34, 0xBB, 0xFE, 0xAE, 0x61, 0x96, 0xB0, 0xFE, 0x10, 0x47, 0xD0, 0x64, 0x5C, 0xBD, 0x26,
];

/// An absolute native address, distinct from an image-relative RVA.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbsoluteAddress(usize);

impl AbsoluteAddress {
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// An image-relative virtual address, resolved only against a selected profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelativeVirtualAddress(u32);

impl RelativeVirtualAddress {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    #[must_use]
    pub fn resolve(self, module_base: usize) -> Option<AbsoluteAddress> {
        module_base
            .checked_add(self.0 as usize)
            .map(AbsoluteAddress::new)
    }
}

/// A byte offset within one verified native object layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldOffset(usize);

impl FieldOffset {
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A verified native object size in bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectSize(usize);

impl ObjectSize {
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A zero-based native vtable slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtableSlot(usize);

impl VtableSlot {
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Evidence grade attached to one profile fact group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceGrade {
    A,
    B,
}

/// Reproducible provenance for one group of production profile facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeEvidence {
    pub grade: EvidenceGrade,
    pub source: &'static str,
    pub verified_at: &'static str,
}

/// Identity of one verified GTA executable target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GtaIdentity {
    pub name: &'static str,
    pub image_base: AbsoluteAddress,
    pub sha256: [u8; 32],
}

/// Verified game-loop symbols for one GTA executable target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameSpec {
    pub process: AbsoluteAddress,
    pub evidence: NativeEvidence,
}

/// Verified GTA `CPools` reference-conversion targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GtaPoolSpec {
    pub get_ped_ref: AbsoluteAddress,
    pub get_vehicle_ref: AbsoluteAddress,
    pub evidence: NativeEvidence,
}

/// Verified local-player symbols.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerSpec {
    pub find_player_ped: AbsoluteAddress,
    pub evidence: NativeEvidence,
}

/// Fixture-backed `CPlaceable` and `CEntity` layout facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityLayoutSpec {
    pub placeable_position: FieldOffset,
    pub matrix_pointer: FieldOffset,
    pub size: ObjectSize,
    pub evidence: NativeEvidence,
}

/// Fixture-backed `CPed` layout facts used by the first snapshot slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PedLayoutSpec {
    pub size: ObjectSize,
    pub health: FieldOffset,
    pub armour: FieldOffset,
    pub evidence: NativeEvidence,
}

/// Exact ped vtables and virtual teleport target for the selected executable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PedVtableSpec {
    pub ped: AbsoluteAddress,
    pub player_ped: AbsoluteAddress,
    pub teleport_slot: VtableSlot,
    pub teleport_target: AbsoluteAddress,
    pub evidence: NativeEvidence,
}

/// Data-only profile specification for one GTA executable target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GtaProfileSpec {
    pub identity: GtaIdentity,
    pub game: GameSpec,
    pub pools: GtaPoolSpec,
    pub player: PlayerSpec,
    pub entity: EntityLayoutSpec,
    pub ped: PedLayoutSpec,
    pub ped_vtable: PedVtableSpec,
}

/// Selected GTA executable profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GtaProfile {
    pub module_base: usize,
    pub spec: &'static GtaProfileSpec,
}

/// Failure to identify the current process as a supported GTA executable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GtaProfileError {
    ExecutablePathUnavailable,
    ExecutableReadFailed,
    ModuleUnavailable,
    Unsupported {
        module_base: usize,
        sha256: [u8; 32],
    },
}

const GTA_SA_10_US_SPEC: GtaProfileSpec = GtaProfileSpec {
    identity: GtaIdentity {
        name: "GTA SA 1.0 US",
        image_base: AbsoluteAddress::new(0x0040_0000),
        sha256: GTA_SA_10_US_SHA256,
    },
    game: GameSpec {
        process: AbsoluteAddress::new(0x53BEE0),
        evidence: NativeEvidence {
            grade: EvidenceGrade::A,
            source: "docs/evidence/phase-5-gta-native-runtime.md",
            verified_at: "2026-08-26",
        },
    },
    pools: GtaPoolSpec {
        get_ped_ref: AbsoluteAddress::new(0x54FF60),
        get_vehicle_ref: AbsoluteAddress::new(0x54FFC0),
        evidence: NativeEvidence {
            grade: EvidenceGrade::A,
            source: "docs/evidence/phase-6-gta-handles.md",
            verified_at: "2026-08-26",
        },
    },
    player: PlayerSpec {
        find_player_ped: AbsoluteAddress::new(0x56E210),
        evidence: NativeEvidence {
            grade: EvidenceGrade::A,
            source: "docs/evidence/phase-9-gta-native-foundations.md",
            verified_at: "2026-08-30",
        },
    },
    entity: EntityLayoutSpec {
        placeable_position: FieldOffset::new(0x04),
        matrix_pointer: FieldOffset::new(0x14),
        size: ObjectSize::new(0x38),
        evidence: NativeEvidence {
            grade: EvidenceGrade::A,
            source: "tests/fixtures/gta_sa_layout.cpp",
            verified_at: "2026-08-30",
        },
    },
    ped: PedLayoutSpec {
        size: ObjectSize::new(0x79C),
        health: FieldOffset::new(0x540),
        armour: FieldOffset::new(0x548),
        evidence: NativeEvidence {
            grade: EvidenceGrade::A,
            source: "tests/fixtures/gta_sa_layout.cpp",
            verified_at: "2026-08-30",
        },
    },
    ped_vtable: PedVtableSpec {
        ped: AbsoluteAddress::new(0x86C358),
        player_ped: AbsoluteAddress::new(0x86D168),
        teleport_slot: VtableSlot::new(14),
        teleport_target: AbsoluteAddress::new(0x5E4110),
        evidence: NativeEvidence {
            grade: EvidenceGrade::A,
            source: "docs/evidence/phase-9-gta-teleport-static.md",
            verified_at: "2026-08-30",
        },
    },
};

impl GtaProfile {
    /// Selects a profile only when the mapped base and full executable hash match.
    #[must_use]
    pub fn select(module_base: usize, sha256: [u8; 32]) -> Option<Self> {
        let spec = &GTA_SA_10_US_SPEC;
        (module_base == spec.identity.image_base.get() && sha256 == spec.identity.sha256)
            .then_some(Self { module_base, spec })
    }

    /// Hashes and identifies the executable hosting the current process.
    pub fn detect_current() -> Result<Self, GtaProfileError> {
        let path = env::current_exe().map_err(|_| GtaProfileError::ExecutablePathUnavailable)?;
        let mut file = File::open(path).map_err(|_| GtaProfileError::ExecutableReadFailed)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|_| GtaProfileError::ExecutableReadFailed)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let sha256: [u8; 32] = hasher.finalize().into();
        let module = unsafe { GetModuleHandleW(ptr::null()) };
        if module.is_null() {
            return Err(GtaProfileError::ModuleUnavailable);
        }
        let module_base = module as usize;
        Self::select(module_base, sha256).ok_or(GtaProfileError::Unsupported {
            module_base,
            sha256,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gta_sa_10_us_requires_the_exact_image_base_and_hash() {
        assert_eq!(GtaProfile::select(0x0040_0000, [0; 32]), None);
        assert_eq!(GtaProfile::select(0x0050_0000, GTA_SA_10_US_SHA256), None);

        let profile = GtaProfile::select(0x0040_0000, GTA_SA_10_US_SHA256).unwrap();
        assert_eq!(profile.spec.identity.name, "GTA SA 1.0 US");
        assert_eq!(profile.module_base, 0x0040_0000);
        assert_eq!(profile.spec.game.process.get(), 0x53BEE0);
    }

    #[test]
    fn gta_sa_10_us_owns_the_verified_cpools_ref_targets() {
        let profile = GtaProfile::select(0x0040_0000, GTA_SA_10_US_SHA256).unwrap();
        assert_eq!(profile.spec.pools.get_ped_ref.get(), 0x54FF60);
        assert_eq!(profile.spec.pools.get_vehicle_ref.get(), 0x54FFC0);
    }

    #[test]
    fn gta_sa_10_us_owns_verified_first_ped_slice_facts() {
        let profile = GtaProfile::select(0x0040_0000, GTA_SA_10_US_SHA256).unwrap();
        assert_eq!(profile.spec.player.find_player_ped.get(), 0x56E210);
        assert_eq!(profile.spec.entity.placeable_position.get(), 0x04);
        assert_eq!(profile.spec.entity.matrix_pointer.get(), 0x14);
        assert_eq!(profile.spec.entity.size.get(), 0x38);
        assert_eq!(profile.spec.ped.size.get(), 0x79C);
        assert_eq!(profile.spec.ped.health.get(), 0x540);
        assert_eq!(profile.spec.ped.armour.get(), 0x548);
        assert_eq!(profile.spec.ped.evidence.grade, EvidenceGrade::A);
        assert_eq!(profile.spec.ped_vtable.ped.get(), 0x86C358);
        assert_eq!(profile.spec.ped_vtable.player_ped.get(), 0x86D168);
        assert_eq!(profile.spec.ped_vtable.teleport_slot.get(), 14);
        assert_eq!(profile.spec.ped_vtable.teleport_target.get(), 0x5E4110);
    }

    #[test]
    fn relative_addresses_resolve_without_wrapping() {
        assert_eq!(
            RelativeVirtualAddress::new(0x1000)
                .resolve(0x0040_0000)
                .map(AbsoluteAddress::get),
            Some(0x0040_1000)
        );
        assert_eq!(
            RelativeVirtualAddress::new(u32::MAX).resolve(usize::MAX),
            None
        );
    }
}
