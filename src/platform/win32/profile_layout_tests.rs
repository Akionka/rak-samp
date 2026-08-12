//! Independent minimal layout gates for unenabled native client profiles.
//!
//! The runtime keeps direct helpers R1-only apart from the narrow, read-only
//! R3 CNetGame scalar cache. These tests record the first three structures any
//! future R3-1, R5-1, or DL profile must prove before a broader gate is relaxed.

use crate::client::SampVersion;

type FixtureFn = unsafe extern "C" fn() -> usize;

unsafe extern "C" {
    fn samp_client_sdk_fixture_r3_1_netgame_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_rak_client_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_host_address_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_hostname_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_port_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_pools_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_command_count_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_caption_offset() -> usize;

    fn samp_client_sdk_fixture_r5_1_netgame_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_netgame_rak_client_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_netgame_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_netgame_pools_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_input_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_input_command_count_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_dialog_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_dialog_caption_offset() -> usize;

    fn samp_client_sdk_fixture_dl_netgame_size() -> usize;
    fn samp_client_sdk_fixture_dl_netgame_rak_client_offset() -> usize;
    fn samp_client_sdk_fixture_dl_netgame_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_dl_netgame_pools_offset() -> usize;
    fn samp_client_sdk_fixture_dl_input_size() -> usize;
    fn samp_client_sdk_fixture_dl_input_command_count_offset() -> usize;
    fn samp_client_sdk_fixture_dl_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_dl_dialog_size() -> usize;
    fn samp_client_sdk_fixture_dl_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_dl_dialog_caption_offset() -> usize;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProfileLayout {
    netgame_size: usize,
    netgame_rak_client_offset: usize,
    netgame_game_state_offset: usize,
    netgame_pools_offset: usize,
    input_size: usize,
    input_command_count_offset: usize,
    input_enabled_offset: usize,
    dialog_size: usize,
    dialog_active_offset: usize,
    dialog_caption_offset: usize,
}

#[derive(Clone, Copy)]
struct ProfileLayoutFixture {
    version: SampVersion,
    netgame_size: FixtureFn,
    netgame_rak_client_offset: FixtureFn,
    netgame_game_state_offset: FixtureFn,
    netgame_pools_offset: FixtureFn,
    input_size: FixtureFn,
    input_command_count_offset: FixtureFn,
    input_enabled_offset: FixtureFn,
    dialog_size: FixtureFn,
    dialog_active_offset: FixtureFn,
    dialog_caption_offset: FixtureFn,
}

impl ProfileLayoutFixture {
    unsafe fn observed(self) -> ProfileLayout {
        ProfileLayout {
            netgame_size: unsafe { (self.netgame_size)() },
            netgame_rak_client_offset: unsafe { (self.netgame_rak_client_offset)() },
            netgame_game_state_offset: unsafe { (self.netgame_game_state_offset)() },
            netgame_pools_offset: unsafe { (self.netgame_pools_offset)() },
            input_size: unsafe { (self.input_size)() },
            input_command_count_offset: unsafe { (self.input_command_count_offset)() },
            input_enabled_offset: unsafe { (self.input_enabled_offset)() },
            dialog_size: unsafe { (self.dialog_size)() },
            dialog_active_offset: unsafe { (self.dialog_active_offset)() },
            dialog_caption_offset: unsafe { (self.dialog_caption_offset)() },
        }
    }
}

const R3_1_LAYOUT: ProfileLayout = ProfileLayout {
    netgame_size: 0x3E2,
    netgame_rak_client_offset: 0x2C,
    netgame_game_state_offset: 0x3CD,
    netgame_pools_offset: 0x3DE,
    input_size: 0x1AFC,
    input_command_count_offset: 0x14DC,
    input_enabled_offset: 0x14E0,
    dialog_size: 0x29D,
    dialog_active_offset: 0x28,
    dialog_caption_offset: 0x40,
};

const R5_1_LAYOUT: ProfileLayout = ProfileLayout {
    netgame_rak_client_offset: 0x00,
    ..R3_1_LAYOUT
};

const DL_LAYOUT: ProfileLayout = R3_1_LAYOUT;

#[test]
fn r3_scalar_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_netgame_host_address_offset(),
            samp_client_sdk_fixture_r3_1_netgame_hostname_offset(),
            samp_client_sdk_fixture_r3_1_netgame_port_offset(),
        )
    };
    assert_eq!(observed, (0x30, 0x131, 0x235));
}

#[test]
fn non_r1_profile_layout_gates_match_the_independent_cpp_fixture() {
    let fixtures = [
        (
            ProfileLayoutFixture {
                version: SampVersion::R3_1,
                netgame_size: samp_client_sdk_fixture_r3_1_netgame_size,
                netgame_rak_client_offset: samp_client_sdk_fixture_r3_1_netgame_rak_client_offset,
                netgame_game_state_offset: samp_client_sdk_fixture_r3_1_netgame_game_state_offset,
                netgame_pools_offset: samp_client_sdk_fixture_r3_1_netgame_pools_offset,
                input_size: samp_client_sdk_fixture_r3_1_input_size,
                input_command_count_offset: samp_client_sdk_fixture_r3_1_input_command_count_offset,
                input_enabled_offset: samp_client_sdk_fixture_r3_1_input_enabled_offset,
                dialog_size: samp_client_sdk_fixture_r3_1_dialog_size,
                dialog_active_offset: samp_client_sdk_fixture_r3_1_dialog_active_offset,
                dialog_caption_offset: samp_client_sdk_fixture_r3_1_dialog_caption_offset,
            },
            R3_1_LAYOUT,
        ),
        (
            ProfileLayoutFixture {
                version: SampVersion::R5_1,
                netgame_size: samp_client_sdk_fixture_r5_1_netgame_size,
                netgame_rak_client_offset: samp_client_sdk_fixture_r5_1_netgame_rak_client_offset,
                netgame_game_state_offset: samp_client_sdk_fixture_r5_1_netgame_game_state_offset,
                netgame_pools_offset: samp_client_sdk_fixture_r5_1_netgame_pools_offset,
                input_size: samp_client_sdk_fixture_r5_1_input_size,
                input_command_count_offset: samp_client_sdk_fixture_r5_1_input_command_count_offset,
                input_enabled_offset: samp_client_sdk_fixture_r5_1_input_enabled_offset,
                dialog_size: samp_client_sdk_fixture_r5_1_dialog_size,
                dialog_active_offset: samp_client_sdk_fixture_r5_1_dialog_active_offset,
                dialog_caption_offset: samp_client_sdk_fixture_r5_1_dialog_caption_offset,
            },
            R5_1_LAYOUT,
        ),
        (
            ProfileLayoutFixture {
                version: SampVersion::Dl,
                netgame_size: samp_client_sdk_fixture_dl_netgame_size,
                netgame_rak_client_offset: samp_client_sdk_fixture_dl_netgame_rak_client_offset,
                netgame_game_state_offset: samp_client_sdk_fixture_dl_netgame_game_state_offset,
                netgame_pools_offset: samp_client_sdk_fixture_dl_netgame_pools_offset,
                input_size: samp_client_sdk_fixture_dl_input_size,
                input_command_count_offset: samp_client_sdk_fixture_dl_input_command_count_offset,
                input_enabled_offset: samp_client_sdk_fixture_dl_input_enabled_offset,
                dialog_size: samp_client_sdk_fixture_dl_dialog_size,
                dialog_active_offset: samp_client_sdk_fixture_dl_dialog_active_offset,
                dialog_caption_offset: samp_client_sdk_fixture_dl_dialog_caption_offset,
            },
            DL_LAYOUT,
        ),
    ];

    for (fixture, expected) in fixtures {
        let actual = unsafe { fixture.observed() };
        assert_eq!(actual, expected, "{:#?} layout fixture", fixture.version);
    }
}
