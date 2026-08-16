//! Version-selected direct client profiles.
//!
//! Network hooks use [`crate::AddressSet`] and are independently supported for
//! several SA-MP builds. Direct object layouts and native calls require a
//! separately verified profile, so this boundary deliberately selects no
//! profile for a recognized build until its layout gates are complete.

use super::{r1_client::R1ClientProfile, r3_client::ClassicClientProfile};
use crate::{
    SampVersion,
    runtime::{DirectClientError, LocalPlayerSnapshot, ServerInfoSnapshot},
};

/// A verified direct-native profile selected for the loaded SA-MP build.
///
/// More variants are added only with their own fixture-backed layout and live
/// validation.
#[derive(Clone, Copy, Debug)]
pub(super) enum NativeProfile {
    R1(R1ClientProfile),
    R3(ClassicClientProfile),
    R5(ClassicClientProfile),
    Dl(ClassicClientProfile),
}

/// Owned local-player data prepared for publication by the cache refresher.
///
/// `raw_r1_address` stays inside the host and is non-zero only for the
/// connected R1 profile.
pub(super) struct LocalPlayerCacheSnapshot {
    pub(super) snapshot: Option<LocalPlayerSnapshot>,
    pub(super) raw_r1_address: usize,
}

impl NativeProfile {
    /// Selects a direct-native profile independently of the network
    /// [`crate::AddressSet`].
    pub(super) fn select(
        module_base: usize,
        version: SampVersion,
        entry_point: u32,
    ) -> Option<Self> {
        match version {
            SampVersion::R1 => R1ClientProfile::verify(module_base, entry_point).map(Self::R1),
            SampVersion::R3_1 => {
                ClassicClientProfile::verify(module_base, entry_point).map(Self::R3)
            }
            SampVersion::R5_1 => {
                ClassicClientProfile::verify_r5(module_base, entry_point).map(Self::R5)
            }
            SampVersion::Dl => {
                ClassicClientProfile::verify_dl(module_base, entry_point).map(Self::Dl)
            }
            SampVersion::R2 | SampVersion::R4_2 => None,
        }
    }

    pub(super) const fn dialog_close_target(self) -> usize {
        match self {
            Self::R1(profile) => profile.dialog_close_target(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.dialog_close_target()
            }
        }
    }

    pub(super) fn rakpeer_address(
        self,
        rakclient: *mut std::ffi::c_void,
    ) -> Result<*mut std::ffi::c_void, DirectClientError> {
        match self {
            Self::R1(profile) => profile.rakpeer_address(rakclient),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.rakpeer_address(rakclient)
            }
        }
    }

    pub(super) fn disconnect_with_reason(
        self,
        rak_client: *mut std::ffi::c_void,
        block_duration: u32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.disconnect_with_reason(rak_client, block_duration),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.disconnect_with_reason(rak_client, block_duration)
            }
        }
    }

    pub(super) fn player_pool(self) -> Result<*mut std::ffi::c_void, DirectClientError> {
        match self {
            Self::R1(profile) => profile.player_pool(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.player_pool(),
        }
    }

    pub(super) fn vehicle_pool(self) -> Result<*mut std::ffi::c_void, DirectClientError> {
        match self {
            Self::R1(profile) => profile.vehicle_pool(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.vehicle_pool(),
        }
    }

    /// Reads the narrow scalar cache surface available on the selected build.
    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        match self {
            Self::R1(profile) => profile.game_state(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.game_state(),
        }
    }

    pub(super) fn set_game_state(self, state: i32) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_game_state(state),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_game_state(state)
            }
        }
    }

    pub(super) fn connect_to_server(
        self,
        address: &[u8],
        port: u16,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.connect_to_server(address, port),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.connect_to_server(address, port)
            }
        }
    }

    pub(super) fn animation_catalog(
        self,
    ) -> Result<Vec<crate::runtime::AnimationSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.animation_catalog(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.animation_catalog()
            }
        }
    }

    /// Reads copied server metadata from the selected scalar profile.
    pub(super) fn server_info(self) -> Result<ServerInfoSnapshot, DirectClientError> {
        match self {
            Self::R1(profile) => profile.server_info(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.server_info(),
        }
    }

    /// Prepares the local-player cache data while keeping R1's raw-address
    /// contract private to the selected native profile.
    pub(super) fn local_player_cache_snapshot(
        self,
        r1_connected: bool,
    ) -> LocalPlayerCacheSnapshot {
        match self {
            Self::R1(profile) if r1_connected => LocalPlayerCacheSnapshot {
                snapshot: profile.local_player().ok(),
                raw_r1_address: profile
                    .local_player_address()
                    .map_or(0, |player| player as usize),
            },
            Self::R1(_) => LocalPlayerCacheSnapshot {
                snapshot: None,
                raw_r1_address: 0,
            },
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => LocalPlayerCacheSnapshot {
                snapshot: profile.local_player().ok(),
                raw_r1_address: 0,
            },
        }
    }

    /// Reads the copied player-pool count pair available on this profile.
    pub(super) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        match self {
            Self::R1(profile) => profile.player_counts(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.player_counts(),
        }
    }

    /// Reads the copied player-pool largest ID available on this profile.
    pub(super) fn player_max_id(self) -> Result<u16, DirectClientError> {
        match self {
            Self::R1(profile) => profile.player_max_id(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.player_max_id(),
        }
    }

    /// Copies one remote-player record on profiles that independently verified
    /// the required native calls and layouts.
    pub(super) fn player_info(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::PlayerInfoSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.player_info(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.player_info(id),
        }
    }

    /// Copies one remote-player state snapshot on the profiles with a verified
    /// remote-player layout prefix.
    pub(super) fn remote_player_state(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::RemotePlayerStateSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.remote_player_state(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.remote_player_state(id)
            }
        }
    }

    /// Determines whether a remote player is currently streamed out on a
    /// profile with a verified `CRemotePlayer` and `CPed` pointer chain.
    pub(super) fn remote_player_is_streamed_out(
        self,
        id: u16,
    ) -> Result<Option<bool>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.remote_player_is_streamed_out(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.remote_player_is_streamed_out(id)
            }
        }
    }

    pub(super) fn object_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.object_handle(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.object_handle(id),
        }
    }

    pub(super) fn object_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.object_id_by_handle(handle),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.object_id_by_handle(handle)
            }
        }
    }

    pub(super) fn pickup_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.pickup_handle(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.pickup_handle(id),
        }
    }

    pub(super) fn pickup_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.pickup_id_by_handle(handle),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.pickup_id_by_handle(handle)
            }
        }
    }

    pub(super) fn vehicle_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.vehicle_handle(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.vehicle_handle(id),
        }
    }

    pub(super) fn vehicle_id_by_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.vehicle_id_by_handle(handle),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.vehicle_id_by_handle(handle)
            }
        }
    }

    pub(super) fn player_ped_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.player_ped_handle(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.player_ped_handle(id)
            }
        }
    }

    pub(super) fn player_id_by_ped_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.player_id_by_ped_handle(handle),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.player_id_by_ped_handle(handle)
            }
        }
    }

    /// Copies an owned on-foot synchronization snapshot on the profiles with a
    /// verified local and remote sync layout.
    pub(super) fn onfoot_sync(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::OnFootSyncSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.onfoot_sync(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.onfoot_sync(id),
        }
    }

    /// Copies an owned in-car synchronization snapshot on profiles with a
    /// verified local and remote sync layout.
    pub(super) fn incar_sync(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::InCarSyncSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.incar_sync(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.incar_sync(id),
        }
    }

    pub(super) fn passenger_sync(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::PassengerSyncSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.passenger_sync(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.passenger_sync(id),
        }
    }

    pub(super) fn trailer_sync(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::TrailerSyncSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.trailer_sync(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.trailer_sync(id),
        }
    }

    pub(super) fn aim_sync(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::AimSyncSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.aim_sync(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.aim_sync(id),
        }
    }

    pub(super) fn force_aim_sync(self) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_aim_sync(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.force_aim_sync(),
        }
    }
    pub(super) fn force_onfoot_sync(self) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_onfoot_sync(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.force_onfoot_sync()
            }
        }
    }
    pub(super) fn force_stats_sync(self) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_stats_sync(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.force_stats_sync(),
        }
    }
    pub(super) fn force_trailer_sync(self, trailer: u16) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_trailer_sync(trailer),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.force_trailer_sync(trailer)
            }
        }
    }
    pub(super) fn force_vehicle_sync(self, vehicle: u16) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_vehicle_sync(vehicle),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.force_vehicle_sync(vehicle)
            }
        }
    }
    pub(super) fn force_weapons_sync(self) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_weapons_sync(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.force_weapons_sync()
            }
        }
    }
    pub(super) fn force_passenger_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_passenger_sync(vehicle, seat),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.force_passenger_sync(vehicle, seat)
            }
        }
    }
    pub(super) fn force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: i32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.force_unoccupied_sync(vehicle, seat),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.force_unoccupied_sync(vehicle, seat)
            }
        }
    }
    pub(super) fn set_send_rate(
        self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_send_rate(kind, milliseconds),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_send_rate(kind, milliseconds)
            }
        }
    }
    pub(super) fn spawn_local_player(self) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.spawn_local_player(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.spawn_local_player()
            }
        }
    }
    pub(super) fn set_local_player_name(self, name: &[u8]) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_local_player_name(name),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_local_player_name(name)
            }
        }
    }
    pub(super) fn set_player_colour(self, id: u16, colour: u32) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_player_colour(id, colour),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_player_colour(id, colour)
            }
        }
    }
    pub(super) fn set_local_player_special_action(
        self,
        action: u8,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_local_player_special_action(action),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_local_player_special_action(action)
            }
        }
    }
    pub(super) fn show_chat_message(
        self,
        request: crate::runtime::LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.show_chat_message(request),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.show_chat_message(request)
            }
        }
    }
    pub(super) fn chat_is_ready(self) -> bool {
        match self {
            Self::R1(profile) => profile.chat_is_ready(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.chat_is_ready(),
        }
    }
    pub(super) fn show_death_message(
        self,
        request: crate::runtime::LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.show_death_message(request),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.show_death_message(request)
            }
        }
    }
    pub(super) fn death_window_is_ready(self) -> bool {
        match self {
            Self::R1(profile) => profile.death_window_is_ready(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.death_window_is_ready()
            }
        }
    }

    pub(super) fn show_dialog(
        self,
        request: crate::runtime::LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.show_dialog(request),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.show_dialog(request)
            }
        }
    }

    pub(super) fn close_dialog(self, button: u8) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.close_dialog(button),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.close_dialog(button)
            }
        }
    }

    pub(super) fn dialog_response_on_close(
        self,
        dialog: *mut std::ffi::c_void,
        button: u8,
    ) -> Result<Option<crate::runtime::LocalDialogResponseSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.dialog_response_on_close(dialog, button),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.dialog_response_on_close(dialog, button)
            }
        }
    }

    pub(super) fn dialog_is_ready(self) -> bool {
        match self {
            Self::R1(profile) => profile.dialog_is_ready(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.dialog_is_ready(),
        }
    }

    pub(super) fn dialog_state(
        self,
    ) -> Result<Option<crate::runtime::LocalDialogSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.dialog_state(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.dialog_state(),
        }
    }

    pub(super) fn set_dialog_selected_item(self, selected: i32) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_dialog_selected_item(selected),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_dialog_selected_item(selected)
            }
        }
    }

    pub(super) fn set_dialog_client_side(self, client_side: bool) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_dialog_client_side(client_side),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_dialog_client_side(client_side)
            }
        }
    }

    pub(super) fn set_dialog_editbox_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_dialog_editbox_text(text),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_dialog_editbox_text(text)
            }
        }
    }

    pub(super) fn set_cursor_mode(self, mode: i32) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_cursor_mode(mode),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_cursor_mode(mode)
            }
        }
    }

    pub(super) fn toggle_cursor(self, show: bool) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.toggle_cursor(show),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.toggle_cursor(show)
            }
        }
    }

    pub(super) fn set_scoreboard_open(self, open: bool) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_scoreboard_open(open),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_scoreboard_open(open)
            }
        }
    }

    pub(super) fn set_chat_input_enabled(self, enabled: bool) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_chat_input_enabled(enabled),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_chat_input_enabled(enabled)
            }
        }
    }

    pub(super) fn set_chat_input_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_chat_input_text(text),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_chat_input_text(text)
            }
        }
    }

    pub(super) fn process_chat_input(self, text: &[u8]) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.process_chat_input(text),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.process_chat_input(text)
            }
        }
    }

    pub(super) fn set_chat_display_mode(self, mode: i32) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.set_chat_display_mode(mode),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_chat_display_mode(mode)
            }
        }
    }

    pub(super) fn set_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => {
                profile.set_chat_entry(id, text, prefix, text_colour, prefix_colour)
            }
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.set_chat_entry(id, text, prefix, text_colour, prefix_colour)
            }
        }
    }

    pub(super) fn chat_entry(
        self,
        id: u16,
    ) -> Result<crate::runtime::ChatEntrySnapshot, DirectClientError> {
        match self {
            Self::R1(profile) => profile.chat_entry(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.chat_entry(id),
        }
    }

    pub(super) fn register_chat_command(
        self,
        name: &[u8],
        callback: unsafe extern "cdecl" fn(*const i8),
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.register_chat_command(name, callback),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.register_chat_command(name, callback)
            }
        }
    }

    pub(super) fn unregister_chat_command(self, name: &[u8]) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.unregister_chat_command(name),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.unregister_chat_command(name)
            }
        }
    }

    pub(super) fn text_label_exists(self, id: u16) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(profile) => profile.text_label_exists(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.text_label_exists(id)
            }
        }
    }
    pub(super) fn first_free_text_label_id(self) -> Result<u16, DirectClientError> {
        match self {
            Self::R1(profile) => profile.first_free_text_label_id(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.first_free_text_label_id()
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn create_text_label(
        self,
        id: u16,
        text: &[u8],
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.create_text_label(
                id,
                text,
                colour,
                position,
                draw_distance,
                behind_walls,
                attached_player_id,
                attached_vehicle_id,
            ),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.create_text_label(
                id,
                text,
                colour,
                position,
                draw_distance,
                behind_walls,
                attached_player_id,
                attached_vehicle_id,
            ),
        }
    }
    pub(super) fn delete_text_label(self, id: u16) -> Result<(), DirectClientError> {
        match self {
            Self::R1(profile) => profile.delete_text_label(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.delete_text_label(id)
            }
        }
    }
    pub(super) fn text_label(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::TextLabelSnapshot>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.text_label(id),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.text_label(id),
        }
    }

    pub(super) fn textdraw_exists(self, id: u16) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(p) => p.textdraw_exists(id),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.textdraw_exists(id),
        }
    }
    pub(super) fn vehicle_exists(self, id: u16) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(p) => p.vehicle_exists(id),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.vehicle_exists(id),
        }
    }
    pub(super) fn object_exists(self, id: u16) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(p) => p.object_exists(id),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.object_exists(id),
        }
    }
    pub(super) fn gangzone(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::GangzoneSnapshot>, DirectClientError> {
        match self {
            Self::R1(p) => p.gangzone(id),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.gangzone(id),
        }
    }
    pub(super) fn textdraw(
        self,
        id: u16,
    ) -> Result<Option<crate::runtime::TextdrawSnapshot>, DirectClientError> {
        match self {
            Self::R1(p) => p.textdraw(id),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.textdraw(id),
        }
    }
    pub(super) fn delete_textdraw(self, id: u16) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.delete_textdraw(id),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.delete_textdraw(id),
        }
    }
    pub(super) fn create_textdraw(
        self,
        id: u16,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.create_textdraw(id, text, x, y),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.create_textdraw(id, text, x, y),
        }
    }
    pub(super) fn set_textdraw_position(
        self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_position(id, x, y),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_position(id, x, y),
        }
    }
    pub(super) fn set_textdraw_style(self, id: u16, style: i32) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_style(id, style),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_style(id, style),
        }
    }
    pub(super) fn set_textdraw_letter_style(
        self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_letter_style(id, width, height, colour),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => {
                p.set_textdraw_letter_style(id, width, height, colour)
            }
        }
    }
    pub(super) fn set_textdraw_proportional(
        self,
        id: u16,
        value: bool,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_proportional(id, value),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_proportional(id, value),
        }
    }
    pub(super) fn set_textdraw_shadow(
        self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_shadow(id, shadow, colour),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_shadow(id, shadow, colour),
        }
    }
    pub(super) fn set_textdraw_outline(
        self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_outline(id, outline, colour),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_outline(id, outline, colour),
        }
    }
    pub(super) fn set_textdraw_string(self, id: u16, text: &[u8]) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_string(id, text),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_string(id, text),
        }
    }
    pub(super) fn set_textdraw_box(
        self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_box(id, enabled, colour, width, height),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => {
                p.set_textdraw_box(id, enabled, colour, width, height)
            }
        }
    }
    pub(super) fn set_textdraw_alignment(
        self,
        id: u16,
        alignment: u8,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_alignment(id, alignment),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => p.set_textdraw_alignment(id, alignment),
        }
    }
    pub(super) fn set_textdraw_model_style(
        self,
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<(), DirectClientError> {
        match self {
            Self::R1(p) => p.set_textdraw_model_style(id, rotation, zoom, colour1, colour2),
            Self::R3(p) | Self::R5(p) | Self::Dl(p) => {
                p.set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
            }
        }
    }

    /// Reads the copied chat-input enabled flag available on this profile.
    pub(super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(profile) => profile.chat_input_is_active(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.chat_input_is_active()
            }
        }
    }

    /// Reads copied native chat-command names available on this profile.
    pub(super) fn chat_input_commands(self) -> Result<Vec<Vec<u8>>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.chat_input_commands(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.chat_input_commands()
            }
        }
    }

    /// Reads copied chat-input text available on this profile.
    pub(super) fn chat_input_text(self) -> Result<Vec<u8>, DirectClientError> {
        match self {
            Self::R1(profile) => profile.chat_input_text(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.chat_input_text(),
        }
    }

    /// Reads the copied chat display mode available on this profile.
    pub(super) fn chat_display_mode(self) -> Result<i32, DirectClientError> {
        match self {
            Self::R1(profile) => profile.chat_display_mode(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.chat_display_mode()
            }
        }
    }

    /// Reads the copied dialog active flag available on this profile.
    pub(super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(profile) => profile.dialog_is_active(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.dialog_is_active(),
        }
    }

    /// Reads the copied scoreboard enabled flag available on this profile.
    pub(super) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        match self {
            Self::R1(profile) => profile.scoreboard_is_open(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => {
                profile.scoreboard_is_open()
            }
        }
    }

    /// Reads the copied cursor mode available on this profile.
    pub(super) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        match self {
            Self::R1(profile) => profile.cursor_mode(),
            Self::R3(profile) | Self::R5(profile) | Self::Dl(profile) => profile.cursor_mode(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_enables_only_verified_direct_profiles() {
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R1, 0x31DF13),
            Some(NativeProfile::R1(_))
        ));
        assert!(NativeProfile::select(0x10000, SampVersion::R1, 0x31DF14).is_none());
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R3_1, SampVersion::R3_1.entry_point()),
            Some(NativeProfile::R3(_))
        ));
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::R5_1, SampVersion::R5_1.entry_point()),
            Some(NativeProfile::R5(_))
        ));
        assert!(matches!(
            NativeProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point()),
            Some(NativeProfile::Dl(_))
        ));
        for version in [SampVersion::R2, SampVersion::R4_2] {
            assert!(NativeProfile::select(0x10000, version, version.entry_point()).is_none());
        }
    }

    #[test]
    fn r1_player_and_sync_reads_reach_the_verified_profile() {
        let profile = NativeProfile::select(0x10000, SampVersion::R1, 0x31DF13)
            .expect("the exact R1 entry point must select a native profile");

        assert!(matches!(
            profile.player_info(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.remote_player_state(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.remote_player_is_streamed_out(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.onfoot_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.incar_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.passenger_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.trailer_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            profile.aim_sync(0),
            Err(DirectClientError::NotReady)
        ));
    }

    #[test]
    fn local_player_cache_keeps_the_raw_address_r1_only() {
        let r1 = NativeProfile::select(0x10000, SampVersion::R1, 0x31DF13)
            .expect("the exact R1 entry point must select a native profile");
        let r1_disconnected = r1.local_player_cache_snapshot(false);
        assert!(r1_disconnected.snapshot.is_none());
        assert_eq!(r1_disconnected.raw_r1_address, 0);

        for (version, entry_point) in [
            (SampVersion::R3_1, SampVersion::R3_1.entry_point()),
            (SampVersion::R5_1, SampVersion::R5_1.entry_point()),
            (SampVersion::Dl, SampVersion::Dl.entry_point()),
        ] {
            let profile = NativeProfile::select(0x10000, version, entry_point)
                .expect("every verified direct profile must select");
            assert_eq!(profile.local_player_cache_snapshot(false).raw_r1_address, 0);
        }
    }
}
