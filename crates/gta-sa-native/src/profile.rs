//! Data-only GTA executable profiles.

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
    pub spec: &'static GtaProfileSpec,
}

const GTA_SA_10_US_SPEC: GtaProfileSpec = GtaProfileSpec {
    identity: GtaIdentity {
        name: "GTA SA 1.0 US",
    },
    game: GameSpec {
        process: AbsoluteAddress::new(0x53BEE0),
    },
};

impl GtaProfile {
    /// Returns the exact GTA SA 1.0 US profile currently verified by the host.
    #[must_use]
    pub const fn gta_sa_10_us() -> Self {
        Self {
            spec: &GTA_SA_10_US_SPEC,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gta_sa_10_us_owns_the_verified_game_process_target() {
        let profile = GtaProfile::gta_sa_10_us();
        assert_eq!(profile.spec.identity.name, "GTA SA 1.0 US");
        assert_eq!(profile.spec.game.process.get(), 0x53BEE0);
    }
}
