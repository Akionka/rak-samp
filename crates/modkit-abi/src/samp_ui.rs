//! Exact-version SA-MP local UI service ABI.

use crate::{CommandReceiptId, ModResult, ServiceHeader};

pub const SAMP_UI_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_MAX_CHAT_ENTRIES: u16 = 100;
pub const SAMP_MAX_CHAT_ENTRY_TEXT_BYTES: usize = 143;
pub const SAMP_MAX_CHAT_ENTRY_PREFIX_BYTES: usize = 27;
pub const SAMP_MAX_CHAT_INPUT_TEXT_BYTES: usize = 128;
pub const SAMP_UI_MAX_CHAT_COMMAND_NAME_BYTES: usize = 32;
pub const SAMP_MAX_DIALOG_TITLE_BYTES: usize = 65;
pub const SAMP_MAX_DIALOG_TEXT_BYTES: usize = 4_096;
pub const SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES: usize = 128;
pub const SAMP_MAX_DIALOG_LIST_ITEMS: usize = 100;
pub const SAMP_MAX_DIALOG_LIST_ITEM_BYTES: usize = 255;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampChatInputTextV1 {
    pub len: u8,
    pub bytes: [u8; SAMP_MAX_CHAT_INPUT_TEXT_BYTES],
}

impl Default for SampChatInputTextV1 {
    fn default() -> Self {
        Self {
            len: 0,
            bytes: [0; SAMP_MAX_CHAT_INPUT_TEXT_BYTES],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampDialogListItemV1 {
    pub len: u8,
    pub bytes: [u8; SAMP_MAX_DIALOG_LIST_ITEM_BYTES],
}

impl Default for SampDialogListItemV1 {
    fn default() -> Self {
        Self {
            len: 0,
            bytes: [0; SAMP_MAX_DIALOG_LIST_ITEM_BYTES],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampDialogSnapshotV1 {
    pub active: u8,
    pub style: u8,
    pub server_side: u8,
    pub has_editbox: u8,
    pub id: i32,
    pub title_len: u8,
    pub editbox_text_len: u8,
    pub listbox_item_count: u8,
    pub reserved: u8,
    pub text_len: u16,
    pub reserved2: [u8; 2],
    pub title: [u8; SAMP_MAX_DIALOG_TITLE_BYTES],
    pub editbox_text: [u8; SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES],
    pub text: [u8; SAMP_MAX_DIALOG_TEXT_BYTES],
    pub listbox_items: [SampDialogListItemV1; SAMP_MAX_DIALOG_LIST_ITEMS],
}

impl Default for SampDialogSnapshotV1 {
    fn default() -> Self {
        Self {
            active: 0,
            style: 0,
            server_side: 0,
            has_editbox: 0,
            id: 0,
            title_len: 0,
            editbox_text_len: 0,
            listbox_item_count: 0,
            reserved: 0,
            text_len: 0,
            reserved2: [0; 2],
            title: [0; SAMP_MAX_DIALOG_TITLE_BYTES],
            editbox_text: [0; SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES],
            text: [0; SAMP_MAX_DIALOG_TEXT_BYTES],
            listbox_items: [SampDialogListItemV1::default(); SAMP_MAX_DIALOG_LIST_ITEMS],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampDialogResponseV1 {
    pub available: u8,
    pub button: u8,
    pub input_len: u8,
    pub reserved: u8,
    pub dialog_id: u16,
    pub reserved2: u16,
    pub list_item: i32,
    pub input: [u8; SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES],
}

impl Default for SampDialogResponseV1 {
    fn default() -> Self {
        Self {
            available: 0,
            button: 0,
            input_len: 0,
            reserved: 0,
            dialog_id: 0,
            reserved2: 0,
            list_item: 0,
            input: [0; SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampChatEntryV1 {
    pub id: u16,
    pub text_len: u8,
    pub prefix_len: u8,
    pub text_colour: u32,
    pub prefix_colour: u32,
    pub text: [u8; SAMP_MAX_CHAT_ENTRY_TEXT_BYTES],
    pub prefix: [u8; SAMP_MAX_CHAT_ENTRY_PREFIX_BYTES],
}

impl Default for SampChatEntryV1 {
    fn default() -> Self {
        Self {
            id: 0,
            text_len: 0,
            prefix_len: 0,
            text_colour: 0,
            prefix_colour: 0,
            text: [0; SAMP_MAX_CHAT_ENTRY_TEXT_BYTES],
            prefix: [0; SAMP_MAX_CHAT_ENTRY_PREFIX_BYTES],
        }
    }
}

/// Borrowed dialog request. The Host copies every byte slice before returning.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SampDialogRequestV1 {
    pub id: u16,
    pub reserved: u16,
    pub style: u32,
    pub title: *const u8,
    pub title_len: u32,
    pub text: *const u8,
    pub text_len: u32,
    pub button1: *const u8,
    pub button1_len: u32,
    pub button2: *const u8,
    pub button2_len: u32,
}

/// `ANY_THREAD + CALLBACK_SAFE`; submissions are non-blocking and return Core receipts.
#[repr(C)]
pub struct SampUiServiceV1 {
    pub header: ServiceHeader,
    pub chat_display_mode: unsafe extern "system" fn(out: *mut i32) -> ModResult,
    pub chat_entry: unsafe extern "system" fn(id: u16, out: *mut SampChatEntryV1) -> ModResult,
    pub chat_input_active: unsafe extern "system" fn(out: *mut u8) -> ModResult,
    pub chat_input_text: unsafe extern "system" fn(out: *mut SampChatInputTextV1) -> ModResult,
    pub chat_command_defined:
        unsafe extern "system" fn(name: *const u8, name_len: u32, out: *mut u8) -> ModResult,
    pub cursor_mode: unsafe extern "system" fn(out: *mut i32) -> ModResult,
    pub scoreboard_open: unsafe extern "system" fn(out: *mut u8) -> ModResult,
    pub dialog_active: unsafe extern "system" fn(out: *mut u8) -> ModResult,
    pub dialog_snapshot: unsafe extern "system" fn(out: *mut SampDialogSnapshotV1) -> ModResult,
    pub take_dialog_response:
        unsafe extern "system" fn(out: *mut SampDialogResponseV1) -> ModResult,
    pub dialog_selected_item: unsafe extern "system" fn(out: *mut i32) -> ModResult,
    pub dialog_list_item_count: unsafe extern "system" fn(out: *mut i32) -> ModResult,
    pub submit_chat_message: unsafe extern "system" fn(
        style: u32,
        text: *const u8,
        text_len: u32,
        prefix: *const u8,
        prefix_len: u32,
        text_colour: u32,
        prefix_colour: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_death_message: unsafe extern "system" fn(
        killer: *const u8,
        killer_len: u32,
        victim: *const u8,
        victim_len: u32,
        killer_colour: u32,
        victim_colour: u32,
        weapon: u8,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_chat_display_mode:
        unsafe extern "system" fn(mode: i32, out: *mut CommandReceiptId) -> ModResult,
    pub submit_chat_entry: unsafe extern "system" fn(
        id: u16,
        text: *const u8,
        text_len: u32,
        prefix: *const u8,
        prefix_len: u32,
        text_colour: u32,
        prefix_colour: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_chat_input_text: unsafe extern "system" fn(
        text: *const u8,
        text_len: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_chat_input_enabled:
        unsafe extern "system" fn(enabled: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_chat_input_process: unsafe extern "system" fn(
        text: *const u8,
        text_len: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_cursor_mode:
        unsafe extern "system" fn(mode: i32, out: *mut CommandReceiptId) -> ModResult,
    pub submit_cursor_toggle:
        unsafe extern "system" fn(show: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_scoreboard_open:
        unsafe extern "system" fn(open: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_dialog: unsafe extern "system" fn(
        request: *const SampDialogRequestV1,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_dialog_client_side:
        unsafe extern "system" fn(client_side: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_dialog_selected_item:
        unsafe extern "system" fn(selected: i32, out: *mut CommandReceiptId) -> ModResult,
    pub submit_dialog_editbox_text: unsafe extern "system" fn(
        text: *const u8,
        text_len: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_dialog_close:
        unsafe extern "system" fn(button: u8, out: *mut CommandReceiptId) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_use_fixed_owned_storage() {
        assert_eq!(SampChatInputTextV1::default().bytes.len(), 128);
        assert_eq!(SampDialogSnapshotV1::default().listbox_items.len(), 100);
        assert_eq!(SampChatEntryV1::default().text.len(), 143);
    }

    #[test]
    fn service_layout_is_header_plus_twenty_seven_functions() {
        let pointer = core::mem::size_of::<usize>();
        assert_eq!(
            core::mem::offset_of!(SampUiServiceV1, chat_display_mode),
            16
        );
        assert_eq!(core::mem::size_of::<SampUiServiceV1>(), 16 + 27 * pointer);
    }
}
