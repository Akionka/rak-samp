//! Game command queue and tick orchestration tests.

use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

static GAME_PROCESS_CALLS: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn fake_game_process() {
    GAME_PROCESS_CALLS.fetch_add(1, Ordering::AcqRel);
}

#[test]
fn game_tick_uses_one_generation_bracket_for_every_native_profile() {
    let profiles = [
        r1_native_profile().expect("R1 must select its verified native profile"),
        r3_native_profile().expect("R3 must select its verified native profile"),
        NativeClientProfile::select(0x10000, SampVersion::R5_1, SampVersion::R5_1.entry_point())
            .expect("R5 must select its verified native profile"),
        NativeClientProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
            .expect("DL must select its verified native profile"),
    ];

    for profile in profiles {
        let mut state = test_backend_state();
        state.context.native_client_profile = Some(profile);
        state.cache_generation.store(2, Ordering::Release);

        state.pump_game_tick(Vec::new());

        assert_eq!(state.cache_generation.load(Ordering::Acquire), 4);
    }
}

#[test]
fn dialog_editbox_text_command_is_bounded_and_queued() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    let mut oversized = vec![b'x'; 129];
    oversized.push(0);
    assert_eq!(
        state.submit_local_dialog_editbox_text(oversized),
        Err(DirectClientError::NotReady)
    );
    let id = state
        .submit_local_dialog_editbox_text(b"fixture".to_vec())
        .unwrap();
    let snapshot = state.game_commands.take_tick_snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].id, id);
    assert!(matches!(
        &snapshot[0].command,
        GameCommand::Ui(UiCommand::SetDialogEditboxText(text)) if text == b"fixture"
    ));
}

#[test]
fn game_command_queue_is_shared_fifo_and_bounded() {
    let state = test_backend_state();
    state.queue_local_dialog(test_dialog(7)).unwrap();
    state.queue_local_chat_message(test_chat_message()).unwrap();
    state
        .queue_local_death_message(test_death_message())
        .unwrap();
    for id in 3..GAME_COMMAND_QUEUE_CAPACITY as u16 {
        state.queue_local_dialog(test_dialog(id)).unwrap();
    }
    assert_eq!(
        state.queue_local_chat_message(test_chat_message()),
        Err(DirectClientError::QueueFull)
    );

    let snapshot = state.game_commands.take_tick_snapshot();
    assert_eq!(snapshot.len(), GAME_COMMAND_QUEUE_CAPACITY);
    assert!(matches!(
        &snapshot[0].command,
        GameCommand::Ui(UiCommand::ShowDialog(request)) if request.id == 7
    ));
    assert!(matches!(
        &snapshot[1].command,
        GameCommand::Ui(UiCommand::AddChatMessage(_))
    ));
    assert!(matches!(
        &snapshot[2].command,
        GameCommand::Ui(UiCommand::AddDeathMessage(_))
    ));
    assert!(matches!(
        &snapshot[3].command,
        GameCommand::Ui(UiCommand::ShowDialog(request)) if request.id == 3
    ));
}

#[test]
fn typed_text_label_receipt_returns_the_game_thread_selected_id() {
    let state = test_backend_state();
    let command = state
        .game_commands
        .submit(GameCommand::TextLabel(TextLabelCommand::DeleteTextLabel(0)))
        .unwrap();
    state
        .auto_text_label_creates
        .lock()
        .unwrap()
        .insert(command, Some(7));
    state.game_commands.complete(command, Ok(()));

    assert_eq!(state.try_take_created_text_label(command), Ok(Some(Ok(7))));
    assert!(state.auto_text_label_creates.lock().unwrap().is_empty());
}

#[test]
fn text_label_text_update_copies_nonempty_text_into_the_game_command() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);

    let mut text = b"updated".to_vec();
    state.submit_set_text_label_text(7, text.clone()).unwrap();
    text[0] = b'X';

    let snapshot = state.game_commands.take_tick_snapshot();
    assert!(matches!(
        &snapshot[0].command,
        GameCommand::TextLabel(TextLabelCommand::SetTextLabelText { id: 7, text })
            if text.as_slice() == b"updated"
    ));
    assert_eq!(
        state.submit_set_text_label_text(7, Vec::new()),
        Err(DirectClientError::NotReady)
    );
}

#[test]
fn network_commands_copy_payloads_and_detach_the_legacy_waiter() {
    let state = test_backend_state();
    let mut payload = BitStream::new();
    payload.write_u8(0xAB).unwrap();

    assert_eq!(
        state.send_packet(99, &payload, SendOptions::default()),
        Ok(true)
    );
    payload.write_u8(0xCD).unwrap();

    let snapshot = state.game_commands.take_tick_snapshot();
    assert_eq!(snapshot.len(), 1);
    assert!(matches!(
        &snapshot[0].command,
        GameCommand::Network(NetworkCommand::SendPacket {
            id: 99,
            payload: queued,
            options: SendOptions { .. },
        }) if queued.as_bytes() == [0xAB]
    ));
}

#[test]
fn game_tick_calls_original_once_and_marks_the_game_thread() {
    let state = test_backend_state();
    GAME_PROCESS_CALLS.store(0, Ordering::Release);

    unsafe { state.game_tick.run_tick(&state, fake_game_process) };

    assert_eq!(GAME_PROCESS_CALLS.load(Ordering::Acquire), 1);
    assert!(state.is_game_thread());
}

#[test]
fn game_tick_leaves_commands_pending_until_the_rak_client_is_ready() {
    let state = test_backend_state();
    let id = state
        .submit_game_command(GameCommand::Ui(UiCommand::ShowDialog(test_dialog(1))))
        .unwrap();

    unsafe { state.game_tick.run_tick(&state, fake_game_process) };

    assert_eq!(state.game_commands.try_take(id), Ok(None));
}

#[test]
fn game_tick_completes_commands_after_the_rak_client_is_ready() {
    let state = test_backend_state();
    state.rak_client.store(1, Ordering::Release);
    let id = state
        .submit_game_command(GameCommand::Ui(UiCommand::ShowDialog(test_dialog(1))))
        .unwrap();

    unsafe { state.game_tick.run_tick(&state, fake_game_process) };

    assert_eq!(
        state.game_commands.try_take(id),
        Ok(Some(Err(CommandError::NativeFailure)))
    );
}

#[test]
fn disconnect_invalidation_preserves_the_captured_rak_client_for_reconnect() {
    let mut state = test_backend_state();
    state.context.version = SampVersion::R3_1;
    state.context.native_client_profile = r3_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.rpc_receiver.store(0x2000, Ordering::Release);
    state.player_address.store(0x0100007F, Ordering::Release);
    state.player_port.store(7777, Ordering::Release);

    state.invalidate_after_disconnect();

    assert_eq!(state.rak_client.load(Ordering::Acquire), 0x1000);
    assert_eq!(state.rpc_receiver.load(Ordering::Acquire), 0);
    assert_eq!(state.player_address.load(Ordering::Acquire), 0);
    assert_eq!(state.player_port.load(Ordering::Acquire), 0);
    assert!(
        state
            .submit_connect_to_server(b"127.0.0.1".to_vec(), 7777)
            .is_ok()
    );
}

#[test]
fn command_wait_is_rejected_on_the_published_game_thread() {
    let state = Arc::new(test_backend_state());
    state.game_tick.mark_current_game_thread();
    let id = state
        .game_commands
        .submit(GameCommand::Ui(UiCommand::ShowDialog(test_dialog(1))))
        .unwrap();
    let backend = Backend {
        state: Arc::clone(&state),
    };

    assert_eq!(
        backend.wait_for_command(id, Duration::ZERO),
        Err(CommandError::WaitRejected)
    );
}
