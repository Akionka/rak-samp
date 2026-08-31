//! Adapters from the exact-version local UI service to the frozen legacy host implementation.

use modkit_abi::{
    CommandReceiptId, MOD_INVALID_ARGUMENT, MOD_OK, ModResult, SampChatEntryV1,
    SampChatInputTextV1, SampDialogRequestV1, SampDialogResponseV1, SampDialogSnapshotV1,
};
use sdk_abi::{
    SampClientSdkChatEntryV1, SampClientSdkChatInputTextV1, SampClientSdkCommandReceipt,
    SampClientSdkDialogResponseV1, SampClientSdkDialogSnapshotV1, SampClientSdkResult,
};

use super::modkit::subscription_result;

fn submit_with_receipt(
    out: *mut CommandReceiptId,
    submit: impl FnOnce(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = submit(&mut legacy);
    if result == SampClientSdkResult::Ok {
        *out = CommandReceiptId(legacy.id);
    }
    subscription_result(result)
}

pub(super) unsafe extern "system" fn chat_display_mode(out: *mut i32) -> ModResult {
    subscription_result(unsafe { super::local_state::local_chat_display_mode(out) })
}

pub(super) unsafe extern "system" fn chat_entry(id: u16, out: *mut SampChatEntryV1) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkChatEntryV1::default();
    let result = unsafe { super::snapshots::chat_entry_info(id, &mut legacy) };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    *out = SampChatEntryV1 {
        id: legacy.id,
        text_len: legacy.text_len,
        prefix_len: legacy.prefix_len,
        text_colour: legacy.text_colour,
        prefix_colour: legacy.prefix_colour,
        text: legacy.text,
        prefix: legacy.prefix,
    };
    MOD_OK
}

pub(super) unsafe extern "system" fn chat_input_active(out: *mut u8) -> ModResult {
    subscription_result(unsafe { super::local_state::local_chat_input_active(out) })
}

pub(super) unsafe extern "system" fn chat_input_text(out: *mut SampChatInputTextV1) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkChatInputTextV1 {
        len: 0,
        bytes: [0; 128],
    };
    let result = unsafe { super::chat_input::local_chat_input_text(&mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = SampChatInputTextV1 {
            len: legacy.len,
            bytes: legacy.bytes,
        };
    }
    subscription_result(result)
}

pub(super) unsafe extern "system" fn chat_command_defined(
    name: *const u8,
    name_len: u32,
    out: *mut u8,
) -> ModResult {
    subscription_result(unsafe {
        super::chat_input::local_chat_command_defined(name, name_len as usize, out)
    })
}

pub(super) unsafe extern "system" fn cursor_mode(out: *mut i32) -> ModResult {
    subscription_result(unsafe { super::local_state::local_cursor_mode(out) })
}

pub(super) unsafe extern "system" fn scoreboard_open(out: *mut u8) -> ModResult {
    subscription_result(unsafe { super::local_state::local_scoreboard_open(out) })
}

pub(super) unsafe extern "system" fn dialog_active(out: *mut u8) -> ModResult {
    subscription_result(unsafe { super::local_state::local_dialog_active(out) })
}

pub(super) unsafe extern "system" fn dialog_snapshot(out: *mut SampDialogSnapshotV1) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = unsafe { core::mem::zeroed::<SampClientSdkDialogSnapshotV1>() };
    let result = unsafe { super::dialog::local_dialog_snapshot(&mut legacy) };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    let mut snapshot = SampDialogSnapshotV1 {
        active: legacy.active,
        style: legacy.style,
        server_side: legacy.server_side,
        has_editbox: legacy.has_editbox,
        id: legacy.id,
        title_len: legacy.title_len,
        editbox_text_len: legacy.editbox_text_len,
        listbox_item_count: legacy.listbox_item_count,
        reserved: 0,
        text_len: legacy.text_len,
        reserved2: [0; 2],
        title: legacy.title,
        editbox_text: legacy.editbox_text,
        text: legacy.text,
        ..SampDialogSnapshotV1::default()
    };
    for (target, source) in snapshot
        .listbox_items
        .iter_mut()
        .zip(legacy.listbox_items.iter())
    {
        target.len = source.len;
        target.bytes = source.bytes;
    }
    *out = snapshot;
    MOD_OK
}

pub(super) unsafe extern "system" fn take_dialog_response(
    out: *mut SampDialogResponseV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkDialogResponseV1::default();
    let result = unsafe { super::dialog::take_local_dialog_response(&mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = SampDialogResponseV1 {
            available: legacy.available,
            button: legacy.button,
            input_len: legacy.input_len,
            reserved: 0,
            dialog_id: legacy.dialog_id,
            reserved2: 0,
            list_item: legacy.list_item,
            input: legacy.input,
        };
    }
    subscription_result(result)
}

pub(super) unsafe extern "system" fn dialog_selected_item(out: *mut i32) -> ModResult {
    subscription_result(unsafe { super::dialog::local_dialog_selected_item(out) })
}

pub(super) unsafe extern "system" fn dialog_list_item_count(out: *mut i32) -> ModResult {
    subscription_result(unsafe { super::dialog::local_dialog_list_item_count(out) })
}

pub(super) unsafe extern "system" fn submit_chat_message(
    style: u32,
    text: *const u8,
    text_len: u32,
    prefix: *const u8,
    prefix_len: u32,
    text_colour: u32,
    prefix_colour: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::messages::submit_local_chat_message(
            style,
            text,
            text_len as usize,
            prefix,
            prefix_len as usize,
            text_colour,
            prefix_colour,
            receipt,
        )
    })
}

pub(super) unsafe extern "system" fn submit_death_message(
    killer: *const u8,
    killer_len: u32,
    victim: *const u8,
    victim_len: u32,
    killer_colour: u32,
    victim_colour: u32,
    weapon: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::messages::submit_local_death_message(
            killer,
            killer_len as usize,
            victim,
            victim_len as usize,
            killer_colour,
            victim_colour,
            weapon,
            receipt,
        )
    })
}

pub(super) unsafe extern "system" fn submit_chat_display_mode(
    mode: i32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::local_commands::submit_local_chat_display_mode(mode, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_chat_entry(
    id: u16,
    text: *const u8,
    text_len: u32,
    prefix: *const u8,
    prefix_len: u32,
    text_colour: u32,
    prefix_colour: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::submit_local_chat_entry(
            id,
            text,
            text_len as usize,
            prefix,
            prefix_len as usize,
            text_colour,
            prefix_colour,
            receipt,
        )
    })
}

pub(super) unsafe extern "system" fn submit_chat_input_text(
    text: *const u8,
    text_len: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::chat_input::submit_local_chat_input_text(text, text_len as usize, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_chat_input_enabled(
    enabled: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::chat_input::submit_local_chat_input_enabled(enabled, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_chat_input_process(
    text: *const u8,
    text_len: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::chat_input::submit_local_chat_input_process(text, text_len as usize, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_cursor_mode(
    mode: i32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::local_commands::submit_local_cursor_mode(mode, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_cursor_toggle(
    show: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::local_commands::submit_local_cursor_toggle(show, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_scoreboard_open(
    open: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::local_commands::submit_local_scoreboard_open(open, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_dialog(
    request: *const SampDialogRequestV1,
    out: *mut CommandReceiptId,
) -> ModResult {
    let Some(request) = (unsafe { request.as_ref() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    submit_with_receipt(out, |receipt| unsafe {
        super::dialog::submit_local_dialog(
            request.id,
            request.style,
            request.title,
            request.title_len as usize,
            request.text,
            request.text_len as usize,
            request.button1,
            request.button1_len as usize,
            request.button2,
            request.button2_len as usize,
            receipt,
        )
    })
}

pub(super) unsafe extern "system" fn submit_dialog_client_side(
    client_side: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::dialog::submit_local_dialog_client_side(client_side, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_dialog_selected_item(
    selected: i32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::dialog::submit_local_dialog_selected_item(selected, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_dialog_editbox_text(
    text: *const u8,
    text_len: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::dialog::submit_local_dialog_editbox_text(text, text_len as usize, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_dialog_close(
    button: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::dialog::submit_local_dialog_close(button, receipt)
    })
}
