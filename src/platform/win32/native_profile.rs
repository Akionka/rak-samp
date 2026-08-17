//! Version-selected direct client profiles.
//!
//! Network hooks use [`crate::AddressSet`] and are independently supported for
//! several SA-MP builds. Direct object layouts and native calls require a
//! separately verified profile, so this boundary deliberately selects no
//! profile for a recognized build until its layout gates are complete.

use super::{
    native_client::profile::NativeClientProfile, r1_client::R1ClientProfile,
    r3_client::ClassicClientProfile,
};
use crate::{SampVersion, runtime::DirectClientError};

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

impl NativeProfile {
    /// Selects a direct-native profile independently of the network
    /// [`crate::AddressSet`].
    #[cfg(test)]
    pub(super) fn select(
        module_base: usize,
        version: SampVersion,
        entry_point: u32,
    ) -> Option<Self> {
        NativeClientProfile::select(module_base, version, entry_point)
            .map(Self::from_native_client_profile)
    }

    /// Converts the selected immutable specification into the temporary
    /// legacy operation dispatch until each operation reads the specification.
    pub(super) fn from_native_client_profile(profile: NativeClientProfile) -> Self {
        match profile.spec.identity.version {
            SampVersion::R1 => Self::R1(R1ClientProfile::from_selected(profile.module_base)),
            SampVersion::R3_1 => {
                Self::R3(ClassicClientProfile::from_selected_r3(profile.module_base))
            }
            SampVersion::R5_1 => {
                Self::R5(ClassicClientProfile::from_selected_r5(profile.module_base))
            }
            SampVersion::Dl => {
                Self::Dl(ClassicClientProfile::from_selected_dl(profile.module_base))
            }
            SampVersion::R2 | SampVersion::R4_2 => unreachable!("unsupported profile identity"),
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
        let native_client = NativeClientProfile::select(0x7000_0000, SampVersion::R1, 0x31DF13)
            .expect("the exact R1 entry point must select an immutable profile");

        assert!(matches!(
            native_client.player_info(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.remote_player_state(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.remote_player_is_streamed_out(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.onfoot_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.incar_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.passenger_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.trailer_sync(0),
            Err(DirectClientError::NotReady)
        ));
        assert!(matches!(
            native_client.aim_sync(0),
            Err(DirectClientError::NotReady)
        ));
    }
}
