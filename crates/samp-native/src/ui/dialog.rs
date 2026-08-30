//! Dialog operations and reads.

use super::*;

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

impl NativeProfile {
    pub fn dialog_response_on_close(
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

    pub fn set_dialog_editbox_text(self, text: &[u8]) -> Result<(), DirectClientError> {
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

    pub fn show_dialog(self, request: LocalDialogRequest) -> Result<(), DirectClientError> {
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

    pub fn close_dialog(self, button: u8) -> Result<(), DirectClientError> {
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

    pub fn set_dialog_client_side(self, client_side: bool) -> Result<(), DirectClientError> {
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

    pub fn set_dialog_selected_item(self, selected: i32) -> Result<(), DirectClientError> {
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

    pub fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        read_i32_bool(
            (dialog as usize)
                .checked_add(self.spec.ui.dialog.active_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )
    }

    /// Copies bounded metadata and dynamic text from the active dialog.
    pub fn dialog_state(self) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
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
