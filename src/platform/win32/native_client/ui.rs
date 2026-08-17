//! Guarded UI cache reads shared by immutable client profiles.

use super::{
    memory::{bounded_c_string, read_i32_bool, read_unaligned},
    profile::NativeClientProfile,
};
use crate::runtime::{ChatEntrySnapshot, DirectClientError};

impl NativeClientProfile {
    /// Copies one bounded chat-history entry from the guarded chat singleton.
    pub(crate) fn chat_entry(self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        let layout = self.spec.ui.chat;
        if usize::from(id) >= layout.max_entries.get() {
            return Err(DirectClientError::NotReady);
        }
        let required = layout
            .entries_offset
            .get()
            .checked_add(
                (usize::from(id) + 1)
                    .checked_mul(layout.entry_size.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)?;
        let chat = self
            .singleton(self.spec.ui.chat.singleton_rva, required)
            .ok_or(DirectClientError::NotReady)?;
        let entry = (chat as usize)
            .checked_add(layout.entries_offset.get())
            .and_then(|address| {
                address.checked_add(usize::from(id).checked_mul(layout.entry_size.get())?)
            })
            .ok_or(DirectClientError::NotReady)?;
        let prefix = unsafe {
            bounded_c_string(
                entry
                    .checked_add(layout.prefix_offset.get())
                    .ok_or(DirectClientError::NotReady)? as *const u8,
                layout.prefix_capacity.get(),
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            bounded_c_string(
                entry
                    .checked_add(layout.text_offset.get())
                    .ok_or(DirectClientError::NotReady)? as *const u8,
                layout.text_capacity.get(),
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let text_colour = unsafe {
            read_unaligned::<u32>(
                entry
                    .checked_add(layout.text_colour_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let prefix_colour = unsafe {
            read_unaligned::<u32>(
                entry
                    .checked_add(layout.prefix_colour_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(ChatEntrySnapshot {
            id,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
    }

    pub(crate) fn chat_display_mode(self) -> Result<i32, DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe {
            read_unaligned::<i32>(
                (chat as usize)
                    .checked_add(self.spec.ui.chat.display_mode_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=2)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe {
            read_unaligned::<i32>(
                (game as usize)
                    .checked_add(self.spec.ui.game.cursor_mode_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        read_i32_bool(
            (scoreboard as usize)
                .checked_add(self.spec.ui.scoreboard.enabled_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )
    }

    pub(crate) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        read_i32_bool(
            (dialog as usize)
                .checked_add(self.spec.ui.dialog.active_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )
    }

    pub(crate) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        read_i32_bool(
            (input as usize)
                .checked_add(self.spec.ui.input.enabled_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    #[test]
    fn ui_scalars_reject_invalid_values_for_every_profile() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.chat_entry(u16::MAX),
                Err(DirectClientError::NotReady)
            );
        }
    }
}
