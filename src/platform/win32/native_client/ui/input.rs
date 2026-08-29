//! Chat-input and chat-command operations.

use super::super::memory;
use super::*;

type R1InputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type ClassicInputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type R1InputGetCommandHandlerFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8) -> *mut c_void;
type ClassicInputGetCommandHandlerFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8) -> *mut c_void;
type R1InputAddCommandFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, unsafe extern "cdecl" fn(*const i8));
type ClassicInputAddCommandFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, unsafe extern "cdecl" fn(*const i8));

impl NativeClientProfile {
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
        if names_length != 0 && !memory::readable_range(names as *const u8, names_length) {
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
}
