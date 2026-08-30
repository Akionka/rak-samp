pub mod dl;
pub mod r1;
pub mod r3;
pub mod r5;

use super::profile::ProfileSpec;
use crate::SampVersion;

/// Returns the immutable profile specification for an exact executable identity.
pub fn for_identity(version: SampVersion, entry_point: u32) -> Option<&'static ProfileSpec> {
    let spec = match version {
        SampVersion::R1 => &r1::R1_SPEC,
        SampVersion::R3_1 => &r3::R3_SPEC,
        SampVersion::R5_1 => &r5::R5_SPEC,
        SampVersion::Dl => &dl::DL_SPEC,
        SampVersion::R2 | SampVersion::R4_2 => return None,
    };

    (spec.identity.entry_point == entry_point).then_some(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_only_exact_supported_identities() {
        for (version, entry_point, name) in [
            (SampVersion::R1, 0x31DF13, "SA-MP 0.3.7 R1"),
            (SampVersion::R3_1, 0x0CC4D0, "SA-MP 0.3.7 R3-1"),
            (SampVersion::R5_1, 0x0CBC90, "SA-MP 0.3.7 R5-1"),
            (SampVersion::Dl, 0x0FDB60, "SA-MP 0.3.DL-R1"),
        ] {
            let spec = for_identity(version, entry_point).expect("supported identity");
            assert_eq!(spec.identity.name, name);
        }

        for (version, entry_point) in [
            (SampVersion::R1, 0x0CC4D0),
            (SampVersion::R3_1, 0x31DF13),
            (SampVersion::R5_1, 0x0FDB60),
            (SampVersion::Dl, 0x0CBC90),
            (SampVersion::R2, SampVersion::R2.entry_point()),
            (SampVersion::R4_2, SampVersion::R4_2.entry_point()),
        ] {
            assert!(for_identity(version, entry_point).is_none());
        }
    }

    #[test]
    fn pool_function_rvas_remain_profile_specific() {
        for (spec, player, vehicle, vehicle_exists) in [
            (&r1::R1_SPEC, 0x1160, 0x1170, 0x1140),
            (&r3::R3_SPEC, 0x1160, 0x1170, 0x1140),
            (&r5::R5_SPEC, 0x1170, 0x1180, 0x1150),
            (&dl::DL_SPEC, 0x1170, 0x1180, 0x1150),
        ] {
            assert_eq!(spec.net_game.get_player_pool_rva.get(), player);
            assert_eq!(spec.net_game.get_vehicle_pool_rva.get(), vehicle);
            assert_eq!(spec.pools.vehicle.does_exist_rva.get(), vehicle_exists);
        }
    }
}
