//! Guarded UI cache reads shared by immutable client profiles.

mod chat;
mod dialog;
mod display;
mod input;

use super::{
    memory::{
        bounded_c_string, read_i32_bool, read_pointer, read_unaligned, readable_range,
        writable_range, write_unaligned,
    },
    profile::{ListItemTextLayout, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{
    ChatEntrySnapshot, DirectClientError, LocalChatMessageRequest, LocalDeathMessageRequest,
    LocalDialogRequest, LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalDialogStyle,
};
use std::{ffi::c_void, mem};

type R1DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type ClassicDxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type R1DialogShowFn = unsafe extern "thiscall" fn(
    *mut c_void,
    i32,
    i32,
    *const i8,
    *const i8,
    *const i8,
    *const i8,
    i32,
);
type ClassicDialogShowFn = unsafe extern "thiscall" fn(
    *mut c_void,
    i32,
    i32,
    *const i8,
    *const i8,
    *const i8,
    *const i8,
    i32,
);
type R1DialogCloseFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type ClassicDialogCloseFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type R1ChatAddEntryFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type ClassicChatAddEntryFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type R1DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type ClassicDeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type R1GameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type ClassicGameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type R1GameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);
type ClassicGameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);
type R1InputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type ClassicInputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type R1DxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);
type ClassicDxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);
type R1InputGetCommandHandlerFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8) -> *mut c_void;
type ClassicInputGetCommandHandlerFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8) -> *mut c_void;
type R1InputAddCommandFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, unsafe extern "cdecl" fn(*const i8));
type ClassicInputAddCommandFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, unsafe extern "cdecl" fn(*const i8));

impl NativeClientProfile {
    pub(crate) fn dialog_response_on_close(
        self,
        dialog: *mut c_void,
        button: u8,
    ) -> Result<Option<LocalDialogResponseSnapshot>, DirectClientError> {
        if button > 1 || dialog.is_null() || self.dialog() != Some(dialog) {
            return Ok(None);
        }
        let layout = self.spec.ui.dialog;
        let required = layout
            .server_side_offset
            .get()
            .checked_add(mem::size_of::<i32>())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(dialog.cast(), required) || !self.dialog_is_active()? {
            return Ok(None);
        }
        if read_i32_bool(
            (dialog as usize)
                .checked_add(layout.server_side_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )? {
            return Ok(None);
        }
        let dialog_id = unsafe {
            read_unaligned::<i32>(
                (dialog as usize)
                    .checked_add(layout.id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .and_then(|id| u16::try_from(id).ok())
        .filter(|id| *id != 1);
        let Some(dialog_id) = dialog_id else {
            return Ok(None);
        };
        let listbox = unsafe {
            read_pointer(
                (dialog as usize)
                    .checked_add(layout.listbox_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let list_item = if listbox.is_null() {
            0
        } else {
            unsafe {
                read_unaligned::<i32>(
                    (listbox as usize)
                        .checked_add(layout.listbox.selected_offset.get())
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
            .ok_or(DirectClientError::NotReady)?
        };
        let input = self.dialog_editbox_text()?.unwrap_or_default();
        Ok(Some(LocalDialogResponseSnapshot {
            dialog_id,
            button,
            list_item,
            input,
        }))
    }

    pub(crate) fn register_chat_command(
        self,
        name: &[u8],
        callback: unsafe extern "cdecl" fn(*const i8),
    ) -> Result<(), DirectClientError> {
        let layout = self.spec.ui.input;
        if name.is_empty() || name.len() > layout.max_command_name_bytes.get() || name.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let count = unsafe {
            read_unaligned::<i32>(
                (input as usize)
                    .checked_add(layout.command_count_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|count| (0..layout.max_commands.get() as i32).contains(count))
        .ok_or(DirectClientError::NotReady)?;
        let _ = count;
        let mut name = name.to_vec();
        name.push(0);
        let handler_target = self.ui_target(layout.get_command_handler_rva)?;
        let occupied = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let get_handler: R1InputGetCommandHandlerFn = mem::transmute(handler_target);
                    get_handler(input, name.as_ptr().cast())
                }
                PoolGetterAbi::Classic => {
                    let get_handler: ClassicInputGetCommandHandlerFn =
                        mem::transmute(handler_target);
                    get_handler(input, name.as_ptr().cast())
                }
            }
        };
        if !occupied.is_null() {
            return Err(DirectClientError::NotReady);
        }
        let add_target = self.ui_target(layout.add_command_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let add: R1InputAddCommandFn = mem::transmute(add_target);
                    add(input, name.as_ptr().cast(), callback);
                }
                PoolGetterAbi::Classic => {
                    let add: ClassicInputAddCommandFn = mem::transmute(add_target);
                    add(input, name.as_ptr().cast(), callback);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn unregister_chat_command(self, name: &[u8]) -> Result<(), DirectClientError> {
        let layout = self.spec.ui.input;
        if name.is_empty() || name.len() > layout.max_command_name_bytes.get() || name.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)? as *mut u8;
        let count = unsafe {
            read_unaligned::<i32>(
                (input as usize)
                    .checked_add(layout.command_count_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|count| (1..=layout.max_commands.get() as i32).contains(count))
        .ok_or(DirectClientError::NotReady)? as usize;
        let proc_base = (input as usize)
            .checked_add(layout.command_proc_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let name_base = (input as usize)
            .checked_add(layout.command_name_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let count_field = (input as usize)
            .checked_add(layout.command_count_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let proc_bytes = count
            .checked_mul(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        let name_bytes = count
            .checked_mul(layout.command_name_capacity.get())
            .ok_or(DirectClientError::NotReady)?;
        if !writable_range(proc_base as *const u8, proc_bytes)
            || !writable_range(name_base as *const u8, name_bytes)
            || !writable_range(count_field as *const u8, mem::size_of::<i32>())
        {
            return Err(DirectClientError::NotReady);
        }
        let index = (0..count)
            .find(|index| {
                let Some(address) = index
                    .checked_mul(layout.command_name_capacity.get())
                    .and_then(|offset| name_base.checked_add(offset))
                else {
                    return false;
                };
                unsafe {
                    bounded_c_string(address as *const u8, layout.command_name_capacity.get())
                }
                .as_deref()
                    == Some(name)
            })
            .ok_or(DirectClientError::NotReady)?;
        let remaining = count - index - 1;
        let name_size = layout.command_name_capacity.get();
        if remaining != 0 {
            unsafe {
                std::ptr::copy(
                    (name_base + (index + 1) * name_size) as *const u8,
                    (name_base + index * name_size) as *mut u8,
                    remaining * name_size,
                );
                std::ptr::copy(
                    (proc_base + (index + 1) * mem::size_of::<usize>()) as *const u8,
                    (proc_base + index * mem::size_of::<usize>()) as *mut u8,
                    remaining * mem::size_of::<usize>(),
                );
            }
        }
        let last = count - 1;
        unsafe {
            std::ptr::write_bytes((name_base + last * name_size) as *mut u8, 0, name_size);
            std::ptr::write_bytes(
                (proc_base + last * mem::size_of::<usize>()) as *mut u8,
                0,
                mem::size_of::<usize>(),
            );
        }
        if !unsafe { write_unaligned(count_field, (count - 1) as i32) } {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    fn set_editbox_text(
        self,
        editbox: *mut u8,
        rva: Option<super::profile::NativeRva>,
        text: &[u8],
        maximum: usize,
    ) -> Result<(), DirectClientError> {
        if text.len() > maximum
            || text.contains(&0)
            || editbox.is_null()
            || !readable_range(editbox, 1)
        {
            return Err(DirectClientError::NotReady);
        }
        let target = self.ui_target(rva.ok_or(DirectClientError::NotReady)?)?;
        let mut text = text.to_vec();
        text.push(0);
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set_text: R1DxutEditBoxSetTextFn = mem::transmute(target);
                    set_text(editbox.cast(), text.as_ptr().cast(), false);
                }
                PoolGetterAbi::Classic => {
                    let set_text: ClassicDxutEditBoxSetTextFn = mem::transmute(target);
                    set_text(editbox.cast(), text.as_ptr().cast(), false);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn set_dialog_editbox_text(self, text: &[u8]) -> Result<(), DirectClientError> {
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
        self.set_editbox_text(
            editbox,
            self.spec.ui.input.edit_box_set_text_rva,
            text,
            layout.max_editbox_text_bytes.get(),
        )
    }

    pub(crate) fn set_chat_input_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        let layout = self.spec.ui.input;
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe {
            read_pointer(
                (input as usize)
                    .checked_add(layout.edit_box_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        self.set_editbox_text(
            editbox,
            layout.edit_box_set_text_rva,
            text,
            layout.max_text_bytes.get(),
        )
    }

    pub(crate) fn set_chat_input_enabled(self, enabled: bool) -> Result<(), DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let rva = if enabled {
            self.spec.ui.input.open_rva
        } else {
            self.spec.ui.input.close_rva
        };
        let target = self.ui_target(rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let operation: R1InputNoArgFn = mem::transmute(target);
                    operation(input);
                }
                PoolGetterAbi::Classic => {
                    let operation: ClassicInputNoArgFn = mem::transmute(target);
                    operation(input);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn process_chat_input(self, text: &[u8]) -> Result<(), DirectClientError> {
        self.set_chat_input_text(text)?;
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let target = self.ui_target(self.spec.ui.input.process_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let process: R1InputNoArgFn = mem::transmute(target);
                    process(input);
                }
                PoolGetterAbi::Classic => {
                    let process: ClassicInputNoArgFn = mem::transmute(target);
                    process(input);
                }
            }
        }
        Ok(())
    }

    fn ui_target(self, rva: super::profile::NativeRva) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn show_dialog(self, request: LocalDialogRequest) -> Result<(), DirectClientError> {
        if request.title.contains(&0)
            || request.text.contains(&0)
            || request.button1.contains(&0)
            || request.button2.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let mut title = request.title;
        let mut text = request.text;
        let mut button1 = request.button1;
        let mut button2 = request.button2;
        title.push(0);
        text.push(0);
        button1.push(0);
        button2.push(0);
        let target = self.ui_target(self.spec.ui.dialog.show_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let show: R1DialogShowFn = mem::transmute(target);
                    show(
                        dialog,
                        i32::from(request.id),
                        request.style.as_raw() as i32,
                        title.as_ptr().cast(),
                        text.as_ptr().cast(),
                        button1.as_ptr().cast(),
                        button2.as_ptr().cast(),
                        0,
                    );
                }
                PoolGetterAbi::Classic => {
                    let show: ClassicDialogShowFn = mem::transmute(target);
                    show(
                        dialog,
                        i32::from(request.id),
                        request.style.as_raw() as i32,
                        title.as_ptr().cast(),
                        text.as_ptr().cast(),
                        button1.as_ptr().cast(),
                        button2.as_ptr().cast(),
                        0,
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn close_dialog(self, button: u8) -> Result<(), DirectClientError> {
        if button > 1 {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let target = self.ui_target(self.spec.ui.dialog.close_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let close: R1DialogCloseFn = mem::transmute(target);
                    close(dialog, button);
                }
                PoolGetterAbi::Classic => {
                    let close: ClassicDialogCloseFn = mem::transmute(target);
                    close(dialog, button);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn show_chat_message(
        self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        if request.text.contains(&0) || request.prefix.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let target = self.ui_target(self.spec.ui.chat.add_entry_rva)?;
        let mut text = request.text;
        let mut prefix = request.prefix;
        text.push(0);
        prefix.push(0);
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let add: R1ChatAddEntryFn = mem::transmute(target);
                    add(
                        chat,
                        request.style.as_raw(),
                        text.as_ptr().cast(),
                        prefix.as_ptr().cast(),
                        request.text_colour,
                        request.prefix_colour,
                    );
                }
                PoolGetterAbi::Classic => {
                    let add: ClassicChatAddEntryFn = mem::transmute(target);
                    add(
                        chat,
                        request.style.as_raw(),
                        text.as_ptr().cast(),
                        prefix.as_ptr().cast(),
                        request.text_colour,
                        request.prefix_colour,
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn show_death_message(
        self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        if request.killer.contains(&0) || request.victim.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let death_window = self.death_window().ok_or(DirectClientError::NotReady)?;
        let target = self.ui_target(
            self.spec
                .ui
                .death_window
                .add_message_rva
                .ok_or(DirectClientError::NotReady)?,
        )?;
        let mut killer = request.killer;
        let mut victim = request.victim;
        killer.push(0);
        victim.push(0);
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let add: R1DeathWindowAddMessageFn = mem::transmute(target);
                    add(
                        death_window,
                        killer.as_ptr().cast(),
                        victim.as_ptr().cast(),
                        request.killer_colour,
                        request.victim_colour,
                        request.weapon,
                    );
                }
                PoolGetterAbi::Classic => {
                    let add: ClassicDeathWindowAddMessageFn = mem::transmute(target);
                    add(
                        death_window,
                        killer.as_ptr().cast(),
                        victim.as_ptr().cast(),
                        request.killer_colour,
                        request.victim_colour,
                        request.weapon,
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn set_cursor_mode(self, mode: i32) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let target = self.ui_target(self.spec.ui.game.set_cursor_mode_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set_mode: R1GameSetCursorModeFn = mem::transmute(target);
                    set_mode(game, mode, i32::from(mode != 0));
                }
                PoolGetterAbi::Classic => {
                    let set_mode: ClassicGameSetCursorModeFn = mem::transmute(target);
                    set_mode(game, mode, i32::from(mode != 0));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn toggle_cursor(self, show: bool) -> Result<(), DirectClientError> {
        self.set_cursor_mode(if show { 3 } else { 0 })?;
        if !show {
            let game = self.game().ok_or(DirectClientError::NotReady)?;
            let target = self.ui_target(self.spec.ui.game.process_input_enabling_rva)?;
            unsafe {
                match self.spec.strategies.pool_getter_abi {
                    PoolGetterAbi::R1 => {
                        let process: R1GameProcessInputEnablingFn = mem::transmute(target);
                        process(game);
                    }
                    PoolGetterAbi::Classic => {
                        let process: ClassicGameProcessInputEnablingFn = mem::transmute(target);
                        process(game);
                    }
                }
            }
        }
        Ok(())
    }

    /// Replaces one bounded chat-history entry on the game thread.
    pub(crate) fn set_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<(), DirectClientError> {
        let layout = self.spec.ui.chat;
        if usize::from(id) >= layout.max_entries.get()
            || text.len() >= layout.text_capacity.get()
            || prefix.len() >= layout.prefix_capacity.get()
            || text.contains(&0)
            || prefix.contains(&0)
        {
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
            .singleton(layout.singleton_rva, required)
            .ok_or(DirectClientError::NotReady)?;
        let entry = (chat as usize)
            .checked_add(layout.entries_offset.get())
            .and_then(|address| {
                address.checked_add(usize::from(id).checked_mul(layout.entry_size.get())?)
            })
            .ok_or(DirectClientError::NotReady)?;
        if !writable_range(entry as *const u8, layout.entry_size.get()) {
            return Err(DirectClientError::NotReady);
        }
        let prefix_address = entry
            .checked_add(layout.prefix_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let text_address = entry
            .checked_add(layout.text_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        unsafe {
            std::ptr::write_bytes(prefix_address as *mut u8, 0, layout.prefix_capacity.get());
            std::ptr::write_bytes(text_address as *mut u8, 0, layout.text_capacity.get());
            std::ptr::copy_nonoverlapping(prefix.as_ptr(), prefix_address as *mut u8, prefix.len());
            std::ptr::copy_nonoverlapping(text.as_ptr(), text_address as *mut u8, text.len());
        }
        if !unsafe {
            write_unaligned(
                entry
                    .checked_add(layout.text_colour_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                text_colour,
            ) && write_unaligned(
                entry
                    .checked_add(layout.prefix_colour_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                prefix_colour,
            )
        } {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    pub(crate) fn set_chat_display_mode(self, mode: i32) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=2) {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        unsafe {
            write_unaligned(
                (chat as usize)
                    .checked_add(self.spec.ui.chat.display_mode_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                mode,
            )
            .then_some(())
            .ok_or(DirectClientError::NotReady)
        }
    }

    pub(crate) fn set_scoreboard_open(self, open: bool) -> Result<(), DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        unsafe {
            write_unaligned(
                (scoreboard as usize)
                    .checked_add(self.spec.ui.scoreboard.enabled_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                i32::from(open),
            )
            .then_some(())
            .ok_or(DirectClientError::NotReady)
        }
    }

    pub(crate) fn set_dialog_client_side(self, client_side: bool) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        unsafe {
            write_unaligned(
                (dialog as usize)
                    .checked_add(self.spec.ui.dialog.server_side_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                i32::from(!client_side),
            )
            .then_some(())
            .ok_or(DirectClientError::NotReady)
        }
    }

    pub(crate) fn set_dialog_selected_item(self, selected: i32) -> Result<(), DirectClientError> {
        let layout = self.spec.ui.dialog;
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
        unsafe {
            write_unaligned(
                (listbox as usize)
                    .checked_add(layout.listbox.selected_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                selected,
            )
            .then_some(())
            .ok_or(DirectClientError::NotReady)
        }
    }

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
            assert_eq!(
                profile.set_chat_entry(0, b"text\0", b"prefix", 0, 0),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(
                profile.set_chat_display_mode(3),
                Err(DirectClientError::NotReady)
            );
        }
    }

    #[test]
    fn ui_specs_keep_verified_profile_behavior_boundaries() {
        let r1 =
            NativeClientProfile::select(0x10000, SampVersion::R1, SampVersion::R1.entry_point())
                .expect("the R1 identity must select");
        let r3 = NativeClientProfile::select(
            0x10000,
            SampVersion::R3_1,
            SampVersion::R3_1.entry_point(),
        )
        .expect("the R3 identity must select");
        let r5 = NativeClientProfile::select(
            0x10000,
            SampVersion::R5_1,
            SampVersion::R5_1.entry_point(),
        )
        .expect("the R5 identity must select");
        let dl =
            NativeClientProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
                .expect("the DL identity must select");

        assert_eq!(
            r1.spec.strategies.list_item_text_layout,
            ListItemTextLayout::DxutComboBoxItem
        );
        for profile in [r3, r5, dl] {
            assert_eq!(
                profile.spec.strategies.list_item_text_layout,
                ListItemTextLayout::DirectPointer
            );
        }
        assert_eq!(
            r3.spec
                .ui
                .input
                .edit_box_set_text_rva
                .map(|value| value.get()),
            Some(0x84E70)
        );
        assert_eq!(
            r3.spec
                .ui
                .input
                .edit_box_get_text_rva
                .map(|value| value.get()),
            Some(0x84F40)
        );
        assert_eq!(dl.spec.ui.dialog.show_rva.get(), 0x6FA50);
    }
}
