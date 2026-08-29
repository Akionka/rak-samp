//! Dialog, chat, input, display, text-label, and textdraw validation.

use super::*;

pub(super) fn verify_cached_chat_display_mode(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.chat().display_mode() {
            Ok(_) => return Ok(()),
            Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_cached_cursor_mode(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.cursor().mode() {
            Ok(_) => return Ok(()),
            Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_cached_chat_input(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + CHAT_INPUT_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let input = samp.chat_input();
        match (
            input.is_active(),
            input.is_command_defined(b"quit"),
            input.is_command_defined(MISSING_COMMAND_NAME),
        ) {
            (Ok(true), Ok(true), Ok(false)) => return Ok(()),
            (Ok(_), Ok(_), Ok(false)) => {}
            (Err(SampClientSdkResult::NotReady), _, _)
            | (_, Err(SampClientSdkResult::NotReady), _)
            | (_, _, Err(SampClientSdkResult::NotReady)) => {}
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => return Err(error),
            _ => return Err(SampClientSdkResult::NativeCallFailed),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_cached_chat_input_text(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + CHAT_INPUT_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.chat_input().text() {
            Ok(text) if text == CHAT_INPUT_TEXT_MARKER => return Ok(()),
            Ok(_) | Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_cached_scoreboard_transition(samp: Samp) -> Result<(), SampClientSdkResult> {
    let scoreboard = samp.scoreboard();
    wait_for_receipt(scoreboard.toggle(true)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || scoreboard.is_open())?;
    wait_for_receipt(scoreboard.toggle(false)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || {
        scoreboard.is_open().map(|open| !open)
    })
}

pub(super) fn verify_cached_dialog_active(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + DIALOG_ACTIVE_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.dialogs().is_active() {
            Ok(true) => return Ok(()),
            Ok(false) | Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_ui_mutations(samp: Samp) -> Result<(), SampClientSdkResult> {
    let chat = samp.chat();
    let original_chat_mode = wait_for_value(SCALAR_CACHE_TIMEOUT, || chat.display_mode())?;
    for mode in [
        LocalChatDisplayMode::Off,
        LocalChatDisplayMode::NoShadow,
        LocalChatDisplayMode::Normal,
    ] {
        wait_for_receipt(chat.set_display_mode(mode)?)?;
        wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
            chat.display_mode().map(|current| current == mode)
        })?;
    }
    wait_for_receipt(chat.set_display_mode(original_chat_mode)?)?;

    wait_for_receipt(chat.set_entry(
        99,
        LOCAL_CHAT_MARKER,
        LOCAL_CHAT_PREFIX,
        0xFF6FCF97,
        0xFFFFFFFF,
    )?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        chat.entry(99).map(|entry| {
            entry.text == LOCAL_CHAT_MARKER
                && entry.prefix == LOCAL_CHAT_PREFIX
                && entry.text_colour == 0xFF6FCF97
                && entry.prefix_colour == 0xFFFFFFFF
        })
    })?;
    wait_for_receipt(chat.add(LocalChatMessage {
        style: LocalChatMessageStyle::Info,
        text: LOCAL_CHAT_MARKER,
        prefix: b"",
        text_colour: 0xFF6FCF97,
        prefix_colour: 0,
    })?)?;
    wait_for_receipt(chat.death_window().add(LocalDeathMessage {
        killer: LOCAL_CHAT_PREFIX,
        victim: b"validation",
        killer_colour: 0xFF6FCF97,
        victim_colour: 0xFFFFFFFF,
        weapon: 24,
    })?)?;

    let input = samp.chat_input();
    wait_for_receipt(input.set_text(LOCAL_CHAT_INPUT_MUTATION)?)?;
    wait_for_condition(CHAT_INPUT_CACHE_TIMEOUT, || {
        input.text().map(|text| text == LOCAL_CHAT_INPUT_MUTATION)
    })?;
    wait_for_receipt(input.set_enabled(false)?)?;
    wait_for_condition(CHAT_INPUT_CACHE_TIMEOUT, || {
        input.is_active().map(|active| !active)
    })?;

    let cursor = samp.cursor();
    let original_cursor_mode = wait_for_value(SCALAR_CACHE_TIMEOUT, || cursor.mode())?;
    wait_for_receipt(cursor.set_mode(LocalCursorMode::LockCamera)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        cursor
            .mode()
            .map(|mode| mode == LocalCursorMode::LockCamera)
    })?;
    wait_for_receipt(cursor.set_mode(original_cursor_mode)?)?;
    wait_for_receipt(cursor.toggle(true)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || cursor.is_active())?;
    wait_for_receipt(cursor.toggle(false)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        cursor.is_active().map(|active| !active)
    })?;

    let scoreboard = samp.scoreboard();
    wait_for_receipt(scoreboard.toggle(true)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || scoreboard.is_open())?;
    wait_for_receipt(scoreboard.toggle(false)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || {
        scoreboard.is_open().map(|open| !open)
    })
}

pub(super) fn verify_dialog_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    let dialogs = samp.dialogs();
    wait_for_receipt(dialogs.close_with_button(0)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.is_active().map(|active| !active)
    })?;

    wait_for_receipt(dialogs.show(LocalDialog {
        id: 26_000,
        style: LocalDialogStyle::Input,
        title: LOCAL_INPUT_DIALOG_TITLE,
        text: LOCAL_INPUT_DIALOG_BODY,
        button1: b"Accept",
        button2: b"Cancel",
    })?)?;
    wait_for_receipt(dialogs.set_client_side(true)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.active().map(|dialog| {
            dialog.is_some_and(|dialog| {
                dialog.id == 26_000
                    && dialog.style == LocalDialogStyle::Input
                    && dialog.title == LOCAL_INPUT_DIALOG_TITLE
                    && dialog.text == LOCAL_INPUT_DIALOG_BODY
                    && !dialog.server_side
            })
        })
    })?;
    wait_for_receipt(dialogs.set_editbox_text(LOCAL_DIALOG_INPUT_TEXT)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.active().map(|dialog| {
            dialog.is_some_and(|dialog| {
                dialog.editbox_text.as_deref() == Some(LOCAL_DIALOG_INPUT_TEXT)
            })
        })
    })?;
    wait_for_receipt(dialogs.close_with_button(1)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.last_response().map(|response| {
            response.is_some_and(|response| {
                response.dialog_id == 26_000
                    && response.button == 1
                    && response.input == LOCAL_DIALOG_INPUT_TEXT
            })
        })
    })?;

    wait_for_receipt(dialogs.show(LocalDialog {
        id: 26_001,
        style: LocalDialogStyle::List,
        title: LOCAL_LIST_DIALOG_TITLE,
        text: b"first\nsecond",
        button1: b"Select",
        button2: b"Cancel",
    })?)?;
    wait_for_receipt(dialogs.set_selected_item(1)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        let state = dialogs.active()?;
        Ok(state.is_some_and(|dialog| {
            dialog.id == 26_001
                && dialog.style == LocalDialogStyle::List
                && dialog.items == [b"first".to_vec(), b"second".to_vec()]
        }) && dialogs.selected_item()? == 1
            && dialogs.list_item_count()? == 2)
    })?;
    wait_for_receipt(dialogs.close_with_button(0)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.last_response().map(|response| {
            response.is_some_and(|response| {
                response.dialog_id == 26_001 && response.button == 0 && response.list_item == 1
            })
        })
    })
}

pub(super) fn verify_chat_command_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    CHAT_COMMAND_INVOKED.store(false, Ordering::Release);
    let command = samp
        .chat_input()
        .register_command(LOCAL_COMMAND_NAME, |arguments| {
            if arguments == b"consolidated" {
                CHAT_COMMAND_INVOKED.store(true, Ordering::Release);
            }
        })?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.chat_input().is_command_defined(LOCAL_COMMAND_NAME)
    })?;
    wait_for_receipt(samp.chat_input().process(LOCAL_COMMAND_TEXT)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(CHAT_COMMAND_INVOKED.load(Ordering::Acquire))
    })?;
    command
        .unregister_and_wait()
        .map_err(|error| error.result())?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.chat_input()
            .is_command_defined(LOCAL_COMMAND_NAME)
            .map(|defined| !defined)
    })
}

pub(super) fn verify_animation_table(samp: Samp) -> Result<(), SampClientSdkResult> {
    let animation = wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.anim().get(0))?;
    if animation.name.is_empty() || animation.file.is_empty() {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    match wait_for_value(SCALAR_CACHE_TIMEOUT, || {
        samp.anim().find(&animation.name, &animation.file)
    })? {
        Some(0) => Ok(()),
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(super) fn verify_text_label_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    set_text_label_phase("local_player_wait");
    let local = wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?;
    set_text_label_phase("create_wait");
    let mut create = samp.labels().create(
        LOCAL_LABEL_TEXT,
        0xFF6FCF97,
        Vector3 {
            x: local.position.x,
            y: local.position.y,
            z: local.position.z + 1.0,
        },
        40.0,
        false,
        None,
        None,
    )?;
    let id = loop {
        match create.wait(INITIALIZATION_TIMEOUT) {
            Ok(id) => break id,
            Err(SampClientSdkResult::TimedOut) => continue,
            Err(error) => return Err(error),
        }
    };
    set_text_label_phase("initial_cache_wait");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        let label = match samp.labels().get(id) {
            Ok(label) => {
                let result = if label.is_some() { 2 } else { 1 };
                if TEXT_LABEL_INITIAL_RESULT.swap(result, Ordering::AcqRel) != result {
                    publish_status();
                }
                label
            }
            Err(error) => {
                let result = 0x100 | error as u32;
                if TEXT_LABEL_INITIAL_RESULT.swap(result, Ordering::AcqRel) != result {
                    publish_status();
                }
                return Err(error);
            }
        };
        let fields = label.map_or(0, |label| {
            (1 << 0)
                | (u32::from(label.id == id.get()) << 1)
                | (u32::from(label.text == LOCAL_LABEL_TEXT) << 2)
                | (u32::from(label.colour == 0xFF6FCF97) << 3)
                | (u32::from(label.draw_distance == 40.0) << 4)
                | (u32::from(!label.behind_walls) << 5)
        });
        if TEXT_LABEL_INITIAL_FIELDS.swap(fields, Ordering::AcqRel) != fields {
            publish_status();
        }
        Ok(fields == 0b11_1111)
    })?;
    set_text_label_phase("set_wait");
    wait_for_receipt(samp.labels().set_text(id, LOCAL_LABEL_UPDATED_TEXT)?)?;
    set_text_label_phase("updated_cache_wait");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.labels()
            .get(id)
            .map(|label| label.is_some_and(|label| label.text == LOCAL_LABEL_UPDATED_TEXT))
    })?;
    set_text_label_phase("delete_wait");
    wait_for_receipt(samp.labels().delete(id)?)?;
    set_text_label_phase("deleted_cache_wait");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.labels().exists(id).map(|exists| !exists)
    })?;
    set_text_label_phase("complete");
    Ok(())
}

pub(super) fn set_text_label_phase(phase: &'static str) {
    *TEXT_LABEL_PHASE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = phase;
    publish_status();
}

pub(super) fn verify_textdraw_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    let textdraws = samp.textdraws();
    let mut free = None;
    for raw in (0..2_304).rev() {
        let id = TextdrawId::new(raw).ok_or(SampClientSdkResult::NativeCallFailed)?;
        if !wait_for_value(SCALAR_CACHE_TIMEOUT, || textdraws.exists(id))? {
            free = Some(id);
            break;
        }
    }
    let id = free.ok_or(SampClientSdkResult::NativeCallFailed)?;
    verify_textdraw_mutation("create_before", "create_after", || {
        textdraws.create(id, LOCAL_TEXTDRAW_TEXT, 320.0, 180.0)
    })?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        textdraws
            .get(id)
            .map(|textdraw| textdraw.is_some_and(|textdraw| textdraw.text == LOCAL_TEXTDRAW_TEXT))
    })?;
    verify_textdraw_mutation("position_before", "position_after", || {
        textdraws.set_position(id, 300.0, 170.0)
    })?;
    verify_textdraw_mutation("style_before", "style_after", || textdraws.set_style(id, 1))?;
    verify_textdraw_mutation("letter_before", "letter_after", || {
        textdraws.set_letter_style(id, 0.3, 1.2, 0xFFFFFFFF)
    })?;
    verify_textdraw_mutation("proportional_before", "proportional_after", || {
        textdraws.set_proportional(id, false)
    })?;
    verify_textdraw_mutation("shadow_before", "shadow_after", || {
        textdraws.set_shadow(id, 2, 0xFF101010)
    })?;
    verify_textdraw_mutation("outline_before", "outline_after", || {
        textdraws.set_outline(id, 1, 0xFF202020)
    })?;
    verify_textdraw_mutation("box_before", "box_after", || {
        textdraws.set_box(id, true, 0x80202020, 180.0, 30.0)
    })?;
    verify_textdraw_mutation("alignment_before", "alignment_after", || {
        textdraws.set_alignment(id, 2)
    })?;
    verify_textdraw_mutation("string_before", "string_after", || {
        textdraws.set_text(id, LOCAL_TEXTDRAW_UPDATED_TEXT)
    })?;
    verify_textdraw_mutation("model_before", "model_after", || {
        textdraws.set_model_style(
            id,
            Vector3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            1.25,
            1,
            2,
        )
    })?;
    set_textdraw_phase("snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || match textdraws.get(id) {
        Ok(textdraw) => {
            let fields = textdraw.as_ref().map_or(0, textdraw_snapshot_fields);
            publish_textdraw_snapshot_observation(if textdraw.is_some() { 2 } else { 1 }, fields);
            Ok(fields == 0x7FF)
        }
        Err(error) => {
            publish_textdraw_snapshot_observation(0x100 | error as u32, 0);
            Err(error)
        }
    })?;
    set_textdraw_phase("snapshot_after");
    verify_textdraw_mutation("delete_before", "delete_after", || textdraws.delete(id))?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        textdraws.exists(id).map(|exists| !exists)
    })?;
    set_textdraw_phase("complete");
    Ok(())
}

pub(super) fn verify_textdraw_mutation(
    before: &'static str,
    after: &'static str,
    submit: impl FnOnce() -> Result<CommandReceipt<()>, SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    set_textdraw_phase(before);
    wait_for_receipt(submit()?)?;
    set_textdraw_phase(after);
    Ok(())
}

pub(super) fn set_textdraw_phase(phase: &'static str) {
    *TEXTDRAW_PHASE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = phase;
    publish_status();
}

pub(super) fn textdraw_snapshot_fields(textdraw: &samp_client_sdk::TextDraw) -> u32 {
    (1 << 0)
        | (u32::from(textdraw.text == LOCAL_TEXTDRAW_UPDATED_TEXT) << 1)
        | (u32::from(textdraw.position() == (300.0, 170.0)) << 2)
        | (u32::from(textdraw.style() == 1) << 3)
        | (u32::from(textdraw.letter_style() == (0.3, 1.2, 0xFFFFFFFF)) << 4)
        | (u32::from(!textdraw.is_proportional()) << 5)
        | (u32::from(textdraw.shadow() == 2) << 6)
        | (u32::from(textdraw.outline() == 1) << 7)
        | (u32::from(textdraw.alignment() == (false, true, false)) << 8)
        | (u32::from(textdraw.box_style() == (true, 180.0, 30.0, 0x80202020)) << 9)
        | (u32::from(
            textdraw.model_style()
                == (
                    0,
                    Vector3 {
                        x: 10.0,
                        y: 20.0,
                        z: 30.0,
                    },
                    1.25,
                    1,
                    2,
                ),
        ) << 10)
}

pub(super) fn publish_textdraw_snapshot_observation(result: u32, fields: u32) {
    let result_changed = TEXTDRAW_SNAPSHOT_RESULT.swap(result, Ordering::AcqRel) != result;
    let fields_changed = TEXTDRAW_SNAPSHOT_FIELDS.swap(fields, Ordering::AcqRel) != fields;
    if result_changed || fields_changed {
        publish_status();
    }
}
