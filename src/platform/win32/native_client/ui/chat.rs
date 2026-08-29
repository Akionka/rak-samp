//! Chat and death-window operations.

use super::*;

type R1ChatAddEntryFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type ClassicChatAddEntryFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type R1DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type ClassicDeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);

impl NativeClientProfile {
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
}
