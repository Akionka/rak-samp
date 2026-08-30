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
}

/// Data-only profile specification for one GTA executable target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GtaProfileSpec {
    pub identity: GtaIdentity,
    pub game: GameSpec,
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
}
