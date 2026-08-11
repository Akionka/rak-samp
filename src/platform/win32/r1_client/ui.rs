use super::*;

impl R1ClientProfile {
    pub(in super::super) fn show_dialog(
        self,
        request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;

        let title = nul_terminated(request.title);
        let text = nul_terminated(request.text);
        let button1 = nul_terminated(request.button1);
        let button2 = nul_terminated(request.button2);
        let show: DialogShowFn = unsafe { mem::transmute(self.module_base + DIALOG_SHOW_RVA) };
        unsafe {
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
        Ok(())
    }

    /// Invokes R1 `CDialog::Close` with one response-button selection.
    pub(in super::super) fn close_dialog(self, button: u8) -> Result<(), DirectClientError> {
        if button > 1 {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let close: DialogCloseFn = unsafe { mem::transmute(self.module_base + DIALOG_CLOSE_RVA) };
        unsafe { close(dialog, button) };
        Ok(())
    }

    pub(in super::super) fn show_chat_message(
        self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let text = nul_terminated(request.text);
        let prefix = nul_terminated(request.prefix);
        let add_entry: ChatAddEntryFn =
            unsafe { mem::transmute(self.module_base + CHAT_ADD_ENTRY_RVA) };
        unsafe {
            add_entry(
                chat,
                request.style.as_raw(),
                text.as_ptr().cast(),
                prefix.as_ptr().cast(),
                request.text_colour,
                request.prefix_colour,
            );
        }
        Ok(())
    }

    pub(in super::super) fn show_death_message(
        self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let death_window = self.death_window().ok_or(DirectClientError::NotReady)?;
        let killer = nul_terminated(request.killer);
        let victim = nul_terminated(request.victim);
        let add_message: DeathWindowAddMessageFn =
            unsafe { mem::transmute(self.module_base + DEATH_WINDOW_ADD_MESSAGE_RVA) };
        unsafe {
            add_message(
                death_window,
                killer.as_ptr().cast(),
                victim.as_ptr().cast(),
                request.killer_colour,
                request.victim_colour,
                request.weapon,
            );
        }
        Ok(())
    }

    pub(in super::super) fn dialog_is_ready(self) -> bool {
        self.dialog().is_some()
    }

    pub(in super::super) fn chat_is_ready(self) -> bool {
        self.chat().is_some()
    }

    pub(in super::super) fn chat_display_mode(self) -> Result<i32, DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let get_mode: ChatGetModeFn =
            unsafe { mem::transmute(self.module_base + CHAT_GET_MODE_RVA) };
        let mode = unsafe { get_mode(chat) };
        matches!(mode, 0..=2)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    /// Replaces one fixed R1 chat-history entry on the game thread.
    pub(in super::super) fn set_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<(), DirectClientError> {
        if id >= MAX_CHAT_ENTRIES
            || text.len() >= CHAT_ENTRY_TEXT_CAPACITY
            || prefix.len() >= CHAT_ENTRY_PREFIX_CAPACITY
            || text.contains(&0)
            || prefix.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)? as *mut u8;
        let entry = unsafe { chat.add(CHAT_ENTRIES_OFFSET + usize::from(id) * CHAT_ENTRY_SIZE) };
        if !writable_range(entry, CHAT_ENTRY_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_bytes(
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                0,
                CHAT_ENTRY_PREFIX_CAPACITY,
            );
            ptr::write_bytes(
                entry.add(CHAT_ENTRY_TEXT_OFFSET),
                0,
                CHAT_ENTRY_TEXT_CAPACITY,
            );
            ptr::copy_nonoverlapping(
                prefix.as_ptr(),
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                prefix.len(),
            );
            ptr::copy_nonoverlapping(text.as_ptr(), entry.add(CHAT_ENTRY_TEXT_OFFSET), text.len());
            ptr::write_unaligned(
                entry.add(CHAT_ENTRY_TEXT_COLOUR_OFFSET).cast::<u32>(),
                text_colour,
            );
            ptr::write_unaligned(
                entry.add(CHAT_ENTRY_PREFIX_COLOUR_OFFSET).cast::<u32>(),
                prefix_colour,
            );
        }
        Ok(())
    }

    /// Copies one fixed R1 chat-history entry on the game thread.
    pub(in super::super) fn chat_entry(
        self,
        id: u16,
    ) -> Result<ChatEntrySnapshot, DirectClientError> {
        if id >= MAX_CHAT_ENTRIES {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)? as *const u8;
        let entry = unsafe { chat.add(CHAT_ENTRIES_OFFSET + usize::from(id) * CHAT_ENTRY_SIZE) };
        if !readable_range(entry, CHAT_ENTRY_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let prefix = unsafe {
            bounded_c_string(
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                CHAT_ENTRY_PREFIX_CAPACITY,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            bounded_c_string(entry.add(CHAT_ENTRY_TEXT_OFFSET), CHAT_ENTRY_TEXT_CAPACITY)
        }
        .ok_or(DirectClientError::NotReady)?;
        let text_colour =
            unsafe { read_unaligned::<u32>(entry.add(CHAT_ENTRY_TEXT_COLOUR_OFFSET) as usize) }
                .ok_or(DirectClientError::NotReady)?;
        let prefix_colour =
            unsafe { read_unaligned::<u32>(entry.add(CHAT_ENTRY_PREFIX_COLOUR_OFFSET) as usize) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(ChatEntrySnapshot {
            id,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
    }

    /// Writes one established R1 `CChat::m_nMode` value from the game-thread
    /// command pump.
    pub(in super::super) fn set_chat_display_mode(
        self,
        mode: i32,
    ) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=2) {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (chat as *mut u8)
                .add(CHAT_DISPLAY_MODE_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, mode) };
        Ok(())
    }

    pub(in super::super) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe { read_unaligned::<i32>(game as usize + GAME_CURSOR_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    /// Invokes the validated R1 cursor-mode transition from the game-thread
    /// command pump.
    pub(in super::super) fn set_cursor_mode(self, mode: i32) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let set_cursor_mode: GameSetCursorModeFn =
            unsafe { mem::transmute(self.module_base + GAME_SET_CURSOR_MODE_RVA) };
        unsafe { set_cursor_mode(game, mode, i32::from(mode != 0)) };
        Ok(())
    }

    /// Implements SF.lua's R1 cursor toggle, including input re-enabling when
    /// the cursor is hidden.
    pub(in super::super) fn toggle_cursor(self, show: bool) -> Result<(), DirectClientError> {
        self.set_cursor_mode(if show { 3 } else { 0 })?;
        if !show {
            let game = self.game().ok_or(DirectClientError::NotReady)?;
            let process_input_enabling: GameProcessInputEnablingFn =
                unsafe { mem::transmute(self.module_base + GAME_PROCESS_INPUT_ENABLING_RVA) };
            unsafe { process_input_enabling(game) };
        }
        Ok(())
    }

    pub(in super::super) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(scoreboard as usize + SCOREBOARD_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Writes the R1 scoreboard-enabled field from the game-thread command
    /// pump after proving that the complete native field remains writable.
    pub(in super::super) fn set_scoreboard_open(self, open: bool) -> Result<(), DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (scoreboard as *mut u8)
                .add(SCOREBOARD_ENABLED_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, i32::from(open)) };
        Ok(())
    }

    pub(in super::super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)
    }

    /// Sets whether the active R1 dialog is client-side. The native field has
    /// inverse semantics: it stores whether the dialog is server-side.
    pub(in super::super) fn set_dialog_client_side(
        self,
        client_side: bool,
    ) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (dialog as *mut u8)
                .add(DIALOG_SERVER_SIDE_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, i32::from(!client_side)) };
        Ok(())
    }

    /// Writes the selected index of an active R1 list dialog on the game thread.
    pub(in super::super) fn set_dialog_selected_item(
        self,
        selected: i32,
    ) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            dialog.cast(),
            DIALOG_LISTBOX_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (listbox + DXUT_LISTBOX_SELECTED_OFFSET) as *mut i32;
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, selected) };
        Ok(())
    }

    /// Copies bounded metadata and dynamic text from an active R1 dialog on
    /// the game thread. All text and item strings are bounded copies; no
    /// native or DXUT pointer crosses this boundary.
    pub(in super::super) fn dialog_state(
        self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            dialog.cast(),
            DIALOG_SERVER_SIDE_OFFSET + mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)? {
            return Ok(None);
        }
        let style = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_TYPE_OFFSET) }
            .and_then(|style| u32::try_from(style).ok())
            .and_then(LocalDialogStyle::from_raw)
            .ok_or(DirectClientError::NotReady)?;
        let id = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let title = unsafe {
            bounded_c_string(
                (dialog as usize + DIALOG_CAPTION_OFFSET) as *const u8,
                DIALOG_CAPTION_CAPACITY,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let server_side = read_r1_bool(dialog as usize + DIALOG_SERVER_SIDE_OFFSET)?;
        let text = self.dialog_text()?;
        let editbox_text = self.dialog_editbox_text()?;
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let (selected_item, list_item_count, listbox_items) = if listbox == 0 {
            (None, None, Vec::new())
        } else {
            let selected = (listbox + DXUT_LISTBOX_SELECTED_OFFSET) as *const i32;
            let item_count = (listbox + DXUT_LISTBOX_ITEM_COUNT_OFFSET) as *const i32;
            if !readable_range(selected.cast(), mem::size_of::<i32>())
                || !readable_range(item_count.cast(), mem::size_of::<i32>())
            {
                return Err(DirectClientError::NotReady);
            }
            let selected_item = unsafe { read_unaligned::<i32>(selected as usize) };
            let list_item_count = unsafe { read_unaligned::<i32>(item_count as usize) }
                .filter(|count| *count >= 0)
                .ok_or(DirectClientError::NotReady)?;
            let mut items = Vec::new();
            for index in 0..usize::try_from(list_item_count)
                .map_err(|_| DirectClientError::NotReady)?
                .min(MAX_DIALOG_LISTBOX_ITEMS)
            {
                items.push(self.dialog_listbox_item_text(index)?);
            }
            (selected_item, Some(list_item_count), items)
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

    /// Copies the bounded R1 dialog body text on the game thread. The native
    /// `m_szText` pointer is validated and read through a bounded copy.
    pub(in super::super) fn dialog_text(self) -> Result<Vec<u8>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let text = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_TEXT_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if text == 0 {
            return Ok(Vec::new());
        }
        unsafe { bounded_c_string(text as *const u8, MAX_DIALOG_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)
    }

    /// Copies the bounded R1 dialog editbox text on the game thread. Dialogs
    /// without an editbox report `None` rather than failing the snapshot.
    pub(in super::super) fn dialog_editbox_text(
        self,
    ) -> Result<Option<Vec<u8>>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_EDITBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if editbox == 0 {
            return Ok(None);
        }
        if !readable_range(editbox as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_text: DxutEditBoxGetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_GET_TEXT_RVA) };
        unsafe {
            bounded_c_string(
                get_text(editbox as *mut c_void).cast(),
                MAX_DIALOG_EDITBOX_TEXT_BYTES + 1,
            )
        }
        .map(Some)
        .ok_or(DirectClientError::NotReady)
    }

    /// Replaces the R1 dialog editbox text through its native DXUT method.
    pub(in super::super) fn set_dialog_editbox_text(
        self,
        text: &[u8],
    ) -> Result<(), DirectClientError> {
        if text.len() > MAX_DIALOG_EDITBOX_TEXT_BYTES || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_EDITBOX_OFFSET) }
            .filter(|editbox| *editbox != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(editbox as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let text = nul_terminated(text.to_vec());
        let set_text: DxutEditBoxSetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_SET_TEXT_RVA) };
        unsafe { set_text(editbox as *mut c_void, text.as_ptr().cast(), false) };
        Ok(())
    }

    /// Copies one bounded R1 dialog listbox item string on the game thread.
    pub(in super::super) fn dialog_listbox_item_text(
        self,
        index: usize,
    ) -> Result<Vec<u8>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .filter(|listbox| *listbox != 0)
            .ok_or(DirectClientError::NotReady)?;
        let items = unsafe { read_unaligned::<usize>(listbox + DXUT_LISTBOX_ITEMS_OFFSET) }
            .filter(|items| *items != 0)
            .ok_or(DirectClientError::NotReady)?;
        let item = unsafe { read_unaligned::<usize>(items + index * mem::size_of::<usize>()) }
            .filter(|item| *item != 0)
            .ok_or(DirectClientError::NotReady)?;
        unsafe {
            bounded_dxut_listbox_item_text((item + DXUT_LISTBOX_ITEM_TEXT_OFFSET) as *const u8)
        }
        .ok_or(DirectClientError::NotReady)
    }

    pub(in super::super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(input as usize + INPUT_ENABLED_OFFSET)
    }

    /// Updates the R1 chat edit box through its native DXUT method.
    pub(in super::super) fn set_chat_input_text(
        self,
        text: &[u8],
    ) -> Result<(), DirectClientError> {
        if text.len() > MAX_CHAT_INPUT_TEXT_BYTES || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let edit_box: *mut c_void = unsafe { read_pointer(input as usize + INPUT_EDIT_BOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if edit_box.is_null() || !readable_range(edit_box.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let text = nul_terminated(text.to_vec());
        let set_text: DxutEditBoxSetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_SET_TEXT_RVA) };
        unsafe { set_text(edit_box, text.as_ptr().cast(), false) };
        Ok(())
    }

    /// Copies the current R1 chat edit-box text while running on the game
    /// thread; callers publish the owned bytes through the cache.
    pub(in super::super) fn chat_input_text(self) -> Result<Vec<u8>, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let edit_box: *mut c_void = unsafe { read_pointer(input as usize + INPUT_EDIT_BOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if edit_box.is_null() || !readable_range(edit_box.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_text: DxutEditBoxGetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_GET_TEXT_RVA) };
        unsafe { bounded_c_string(get_text(edit_box).cast(), MAX_CHAT_INPUT_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)
    }

    /// Opens or closes R1's chat input through its native transition methods.
    pub(in super::super) fn set_chat_input_enabled(
        self,
        enabled: bool,
    ) -> Result<(), DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let operation: InputNoArgFn = unsafe {
            mem::transmute(
                self.module_base
                    + if enabled {
                        INPUT_OPEN_RVA
                    } else {
                        INPUT_CLOSE_RVA
                    },
            )
        };
        unsafe { operation(input) };
        Ok(())
    }

    /// Replaces the R1 chat-input text and dispatches its native command path.
    pub(in super::super) fn process_chat_input(self, text: &[u8]) -> Result<(), DirectClientError> {
        self.set_chat_input_text(text)?;
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let process: InputNoArgFn = unsafe { mem::transmute(self.module_base + INPUT_PROCESS_RVA) };
        unsafe { process(input) };
        Ok(())
    }

    /// Registers one bounded native chat-command callback on the game thread.
    pub(in super::super) fn register_chat_command(
        self,
        name: &[u8],
        callback: unsafe extern "cdecl" fn(*const i8),
    ) -> Result<(), DirectClientError> {
        if name.is_empty() || name.len() > MAX_CHAT_COMMAND_NAME_BYTES || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let command_count =
            unsafe { read_unaligned::<i32>(input as usize + INPUT_COMMAND_COUNT_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        if !(0..MAX_CHAT_COMMANDS as i32).contains(&command_count) {
            return Err(DirectClientError::NotReady);
        }
        let name = nul_terminated(name.to_vec());
        let get_handler: InputGetCommandHandlerFn =
            unsafe { mem::transmute(self.module_base + INPUT_GET_COMMAND_HANDLER_RVA) };
        if !unsafe { get_handler(input, name.as_ptr().cast()) }.is_null() {
            return Err(DirectClientError::NotReady);
        }
        let add_command: InputAddCommandFn =
            unsafe { mem::transmute(self.module_base + INPUT_ADD_COMMAND_RVA) };
        unsafe { add_command(input, name.as_ptr().cast(), callback) };
        Ok(())
    }

    /// Removes one R1 chat-command entry using the bounded shifting sequence
    /// used by the pinned SF.lua reference. R1 exposes no native remove call.
    pub(in super::super) fn unregister_chat_command(
        self,
        name: &[u8],
    ) -> Result<(), DirectClientError> {
        if name.is_empty() || name.len() > MAX_CHAT_COMMAND_NAME_BYTES || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)? as *mut u8;
        let command_count =
            unsafe { read_unaligned::<i32>(input as usize + INPUT_COMMAND_COUNT_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        if !(1..=MAX_CHAT_COMMANDS as i32).contains(&command_count) {
            return Err(DirectClientError::NotReady);
        }
        let command_count = command_count as usize;
        let proc_base = unsafe { input.add(INPUT_COMMAND_PROC_OFFSET) };
        let name_base = unsafe { input.add(INPUT_COMMAND_NAME_OFFSET) };
        let count = unsafe { input.add(INPUT_COMMAND_COUNT_OFFSET) };
        if !writable_range(proc_base, command_count * mem::size_of::<usize>())
            || !writable_range(name_base, command_count * INPUT_COMMAND_NAME_CAPACITY)
            || !writable_range(count, mem::size_of::<i32>())
        {
            return Err(DirectClientError::NotReady);
        }
        let Some(index) = (0..command_count).find(|index| {
            let candidate = unsafe {
                bounded_c_string(
                    name_base.wrapping_add(index * INPUT_COMMAND_NAME_CAPACITY),
                    INPUT_COMMAND_NAME_CAPACITY,
                )
            };
            candidate.as_deref() == Some(name)
        }) else {
            return Err(DirectClientError::NotReady);
        };
        let remaining = command_count - index - 1;
        if remaining != 0 {
            unsafe {
                ptr::copy(
                    name_base.add((index + 1) * INPUT_COMMAND_NAME_CAPACITY),
                    name_base.add(index * INPUT_COMMAND_NAME_CAPACITY),
                    remaining * INPUT_COMMAND_NAME_CAPACITY,
                );
                ptr::copy(
                    proc_base.add((index + 1) * mem::size_of::<usize>()),
                    proc_base.add(index * mem::size_of::<usize>()),
                    remaining * mem::size_of::<usize>(),
                );
            }
        }
        let last = command_count - 1;
        unsafe {
            ptr::write_bytes(
                name_base.add(last * INPUT_COMMAND_NAME_CAPACITY),
                0,
                INPUT_COMMAND_NAME_CAPACITY,
            );
            ptr::write_bytes(
                proc_base.add(last * mem::size_of::<usize>()),
                0,
                mem::size_of::<usize>(),
            );
            ptr::write_unaligned(count.cast::<i32>(), (command_count - 1) as i32);
        }
        Ok(())
    }
}
