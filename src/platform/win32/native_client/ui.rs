//! Guarded UI cache reads shared by immutable client profiles.

use super::{
    memory::{bounded_c_string, read_i32_bool, read_pointer, read_unaligned, readable_range},
    profile::{ListItemTextLayout, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{ChatEntrySnapshot, DirectClientError, LocalDialogSnapshot, LocalDialogStyle};
use std::{ffi::c_void, mem};

type R1DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type ClassicDxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;

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

    /// Copies the bounded command names stored by the guarded chat input.
    pub(crate) fn chat_input_commands(self) -> Result<Vec<Vec<u8>>, DirectClientError> {
        let layout = self.spec.ui.input;
        let required = layout
            .command_count_offset
            .get()
            .checked_add(mem::size_of::<i32>())
            .ok_or(DirectClientError::NotReady)?;
        let input = self
            .singleton(layout.singleton_rva, required)
            .ok_or(DirectClientError::NotReady)?;
        let count = unsafe {
            read_unaligned::<i32>(
                (input as usize)
                    .checked_add(layout.command_count_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|count| (0..=layout.max_commands.get() as i32).contains(count))
        .ok_or(DirectClientError::NotReady)? as usize;
        let names = (input as usize)
            .checked_add(layout.command_name_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let names_length = count
            .checked_mul(layout.command_name_capacity.get())
            .ok_or(DirectClientError::NotReady)?;
        if names_length != 0 && !super::memory::readable_range(names as *const u8, names_length) {
            return Err(DirectClientError::NotReady);
        }
        (0..count)
            .map(|index| {
                let address = names
                    .checked_add(
                        index
                            .checked_mul(layout.command_name_capacity.get())
                            .ok_or(DirectClientError::NotReady)?,
                    )
                    .ok_or(DirectClientError::NotReady)?;
                unsafe {
                    bounded_c_string(address as *const u8, layout.command_name_capacity.get())
                }
                .ok_or(DirectClientError::NotReady)
            })
            .collect()
    }

    /// Copies the bounded text from the chat-input DXUT edit box.
    pub(crate) fn chat_input_text(self) -> Result<Vec<u8>, DirectClientError> {
        let layout = self.spec.ui.input;
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe {
            read_pointer(
                (input as usize)
                    .checked_add(layout.edit_box_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, 1))
        .ok_or(DirectClientError::NotReady)?;
        let rva = layout
            .edit_box_get_text_rva
            .ok_or(DirectClientError::NotReady)?;
        let target = self
            .module_base
            .checked_add(rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let get_text: R1DxutEditBoxGetTextFn = mem::transmute(target);
                    get_text(editbox.cast())
                }
                PoolGetterAbi::Classic => {
                    let get_text: ClassicDxutEditBoxGetTextFn = mem::transmute(target);
                    get_text(editbox.cast())
                }
            }
        };
        unsafe { bounded_c_string(text, layout.max_text_bytes.get().saturating_add(1)) }
            .ok_or(DirectClientError::NotReady)
    }

    /// Copies bounded metadata and dynamic text from the active dialog.
    pub(crate) fn dialog_state(self) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        let layout = self.spec.ui.dialog;
        let required = layout
            .server_side_offset
            .get()
            .checked_add(mem::size_of::<i32>())
            .ok_or(DirectClientError::NotReady)?;
        let dialog = self
            .singleton(layout.singleton_rva, required)
            .ok_or(DirectClientError::NotReady)?;
        if !self.dialog_is_active()? {
            return Ok(None);
        }
        let style = unsafe {
            read_unaligned::<i32>(
                (dialog as usize)
                    .checked_add(layout.dialog_type_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .and_then(|value| u32::try_from(value).ok())
        .and_then(LocalDialogStyle::from_raw)
        .ok_or(DirectClientError::NotReady)?;
        let id = unsafe {
            read_unaligned::<i32>(
                (dialog as usize)
                    .checked_add(layout.id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let title = unsafe {
            bounded_c_string(
                (dialog as usize)
                    .checked_add(layout.caption_offset.get())
                    .ok_or(DirectClientError::NotReady)? as *const u8,
                layout.caption_capacity.get(),
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let server_side = read_i32_bool(
            (dialog as usize)
                .checked_add(layout.server_side_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )?;
        let text = self.dialog_text()?;
        let editbox_text = self.dialog_editbox_text()?;
        let listbox = unsafe {
            read_pointer(
                (dialog as usize)
                    .checked_add(layout.listbox_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let (selected_item, list_item_count, listbox_items) = if listbox.is_null() {
            (None, None, Vec::new())
        } else {
            let selected = unsafe {
                read_unaligned::<i32>(
                    (listbox as usize)
                        .checked_add(layout.listbox.selected_offset.get())
                        .ok_or(DirectClientError::NotReady)?,
                )
            };
            let count = unsafe {
                read_unaligned::<i32>(
                    (listbox as usize)
                        .checked_add(layout.listbox.item_count_offset.get())
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
            .filter(|value| *value >= 0)
            .ok_or(DirectClientError::NotReady)?;
            let count = usize::try_from(count).map_err(|_| DirectClientError::NotReady)?;
            let items = (0..count.min(layout.max_listbox_items.get()))
                .map(|index| self.dialog_listbox_item_text(index))
                .collect::<Result<Vec<_>, _>>()?;
            (selected, Some(count as i32), items)
        };
        Ok(Some(LocalDialogSnapshot {
            id,
            style,
            title,
            server_side,
            selected_item,
            list_item_count,
            text,
            editbox_text,
            listbox_items,
        }))
    }

    fn dialog_text(self) -> Result<Vec<u8>, DirectClientError> {
        let layout = self.spec.ui.dialog;
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            read_pointer(
                (dialog as usize)
                    .checked_add(layout.text_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        if text.is_null() {
            return Ok(Vec::new());
        }
        unsafe { bounded_c_string(text, layout.max_text_bytes.get().saturating_add(1)) }
            .ok_or(DirectClientError::NotReady)
    }

    fn dialog_editbox_text(self) -> Result<Option<Vec<u8>>, DirectClientError> {
        let layout = self.spec.ui.dialog;
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe {
            read_pointer(
                (dialog as usize)
                    .checked_add(layout.editbox_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        if editbox.is_null() {
            return Ok(None);
        }
        if !readable_range(editbox, 1) {
            return Err(DirectClientError::NotReady);
        }
        let rva = self
            .spec
            .ui
            .input
            .edit_box_get_text_rva
            .ok_or(DirectClientError::NotReady)?;
        let target = self
            .module_base
            .checked_add(rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let get_text: R1DxutEditBoxGetTextFn = mem::transmute(target);
                    get_text(editbox.cast())
                }
                PoolGetterAbi::Classic => {
                    let get_text: ClassicDxutEditBoxGetTextFn = mem::transmute(target);
                    get_text(editbox.cast())
                }
            }
        };
        unsafe { bounded_c_string(text, layout.max_editbox_text_bytes.get().saturating_add(1)) }
            .map(Some)
            .ok_or(DirectClientError::NotReady)
    }

    fn dialog_listbox_item_text(self, index: usize) -> Result<Vec<u8>, DirectClientError> {
        let layout = self.spec.ui.dialog;
        if index >= layout.max_listbox_items.get() {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let listbox = unsafe {
            read_pointer(
                (dialog as usize)
                    .checked_add(layout.listbox_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let items = unsafe {
            read_pointer(
                (listbox as usize)
                    .checked_add(layout.listbox.items_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let item = unsafe {
            read_pointer(
                (items as usize)
                    .checked_add(
                        index
                            .checked_mul(mem::size_of::<usize>())
                            .ok_or(DirectClientError::NotReady)?,
                    )
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let text = match self.spec.strategies.list_item_text_layout {
            ListItemTextLayout::DxutComboBoxItem => (item as usize)
                .checked_add(layout.listbox.item_text_offset.get())
                .ok_or(DirectClientError::NotReady)?
                as *const u8,
            ListItemTextLayout::DirectPointer => item.cast_const(),
        };
        unsafe { bounded_c_string(text, layout.listbox.item_text_capacity.get()) }
            .ok_or(DirectClientError::NotReady)
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
