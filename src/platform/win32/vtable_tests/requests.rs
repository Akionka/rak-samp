//! Native refresh request queue tests.

use super::*;

#[test]
fn handle_reads_are_deduplicated_queued_and_published_per_pump() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(state.object_handle(7), Err(DirectClientError::NotReady));
    state
        .queue_handle_request(&state.object_handle_requests, 32, 7)
        .unwrap();
    state
        .queue_handle_request(&state.object_handle_requests, 32, 7)
        .unwrap();
    assert_eq!(state.object_handle_requests.lock().unwrap().len(), 1);

    state.object_handle_cache.lock().unwrap()[7] = HandleCacheEntry::Known(None);
    assert_eq!(state.object_handle(7), Ok(None));

    assert_eq!(
        state.object_id_by_handle(42),
        Err(DirectClientError::NotReady)
    );
    state
        .object_handle_reverse_cache
        .lock()
        .unwrap()
        .insert(42, Some(7));
    assert_eq!(state.object_id_by_handle(42), Ok(Some(7)));
}

#[test]
fn active_profile_limit_rejects_an_id_before_queueing() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();

    assert_eq!(
        state.object_handle(1000),
        Err(DirectClientError::InvalidArgument)
    );
    assert!(state.object_handle_requests.lock().unwrap().is_empty());
}

#[test]
fn handle_reverse_requests_are_deduplicated() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    state
        .queue_handle_id_request(&state.object_handle_reverse_requests, 16, 42)
        .unwrap();
    state
        .queue_handle_id_request(&state.object_handle_reverse_requests, 16, 42)
        .unwrap();
    assert_eq!(
        state.object_handle_reverse_requests.lock().unwrap().len(),
        1
    );

    state
        .object_handle_reverse_cache
        .lock()
        .unwrap()
        .insert(42, None);
    assert_eq!(state.object_id_by_handle(42), Ok(None));
}

#[test]
fn player_directory_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_player_info_request(7).unwrap();
    state.queue_player_info_request(7).unwrap();
    assert_eq!(state.player_info_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + PLAYER_INFO_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_player_info_request(id).unwrap();
    }
    assert_eq!(
        state.queue_player_info_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_player_info_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.player_info_requests.lock().unwrap().len(),
        PLAYER_INFO_REQUEST_QUEUE_CAPACITY - PLAYER_INFO_REQUESTS_PER_PUMP
    );
}

#[test]
fn remote_player_state_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_remote_player_state_request(7).unwrap();
    state.queue_remote_player_state_request(7).unwrap();
    for id in 8..(7 + REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_remote_player_state_request(id).unwrap();
    }
    assert_eq!(
        state.queue_remote_player_state_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_remote_player_state_requests();
    assert_eq!(drained.len(), REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.remote_player_state_requests.lock().unwrap().len(),
        REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY - REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP
    );
}

#[test]
fn streamed_out_player_position_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_streamed_out_player_position_request(7).unwrap();
    state.queue_streamed_out_player_position_request(7).unwrap();
    for id in 8..(7 + STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY as u16) {
        state
            .queue_streamed_out_player_position_request(id)
            .unwrap();
    }
    assert_eq!(
        state.queue_streamed_out_player_position_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_streamed_out_player_position_requests();
    assert_eq!(
        drained.len(),
        STREAMED_OUT_PLAYER_POSITION_REQUESTS_PER_PUMP
    );
    assert_eq!(drained[0], 7);
    assert_eq!(
        state
            .streamed_out_player_position_requests
            .lock()
            .unwrap()
            .len(),
        STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY
            - STREAMED_OUT_PLAYER_POSITION_REQUESTS_PER_PUMP
    );
}

#[test]
fn vehicle_exists_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_vehicle_exists_request(7).unwrap();
    state.queue_vehicle_exists_request(7).unwrap();
    assert_eq!(state.vehicle_exists_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_vehicle_exists_request(id).unwrap();
    }
    assert_eq!(
        state.queue_vehicle_exists_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_vehicle_exists_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.vehicle_exists_requests.lock().unwrap().len(),
        VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY - VEHICLE_EXISTS_REQUESTS_PER_PUMP
    );
}

#[test]
fn onfoot_sync_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_onfoot_sync_request(7).unwrap();
    state.queue_onfoot_sync_request(7).unwrap();
    for id in 8..(7 + ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_onfoot_sync_request(id).unwrap();
    }
    assert_eq!(
        state.queue_onfoot_sync_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_onfoot_sync_requests();
    assert_eq!(drained.len(), ONFOOT_SYNC_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.onfoot_sync_requests.lock().unwrap().len(),
        ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY - ONFOOT_SYNC_REQUESTS_PER_PUMP
    );
}

#[test]
fn incar_sync_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_incar_sync_request(7).unwrap();
    state.queue_incar_sync_request(7).unwrap();
    for id in 8..(7 + INCAR_SYNC_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_incar_sync_request(id).unwrap();
    }
    assert_eq!(
        state.queue_incar_sync_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_incar_sync_requests();
    assert_eq!(drained.len(), INCAR_SYNC_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.incar_sync_requests.lock().unwrap().len(),
        INCAR_SYNC_REQUEST_QUEUE_CAPACITY - INCAR_SYNC_REQUESTS_PER_PUMP
    );
}

#[test]
fn passenger_sync_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_passenger_sync_request(7).unwrap();
    state.queue_passenger_sync_request(7).unwrap();
    for id in 8..(7 + PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_passenger_sync_request(id).unwrap();
    }
    assert_eq!(
        state.queue_passenger_sync_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_passenger_sync_requests();
    assert_eq!(drained.len(), PASSENGER_SYNC_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.passenger_sync_requests.lock().unwrap().len(),
        PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY - PASSENGER_SYNC_REQUESTS_PER_PUMP
    );
}

#[test]
fn text_label_exists_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_text_label_exists_request(7).unwrap();
    state.queue_text_label_exists_request(7).unwrap();
    assert_eq!(state.text_label_exists_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_text_label_exists_request(id).unwrap();
    }
    assert_eq!(
        state.queue_text_label_exists_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_text_label_exists_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.text_label_exists_requests.lock().unwrap().len(),
        TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY - TEXT_LABEL_EXISTS_REQUESTS_PER_PUMP
    );
}

#[test]
fn text_label_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_text_label_request(7).unwrap();
    state.queue_text_label_request(7).unwrap();
    assert_eq!(state.text_label_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + TEXT_LABEL_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_text_label_request(id).unwrap();
    }
    assert_eq!(
        state.queue_text_label_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_text_label_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.text_label_requests.lock().unwrap().len(),
        TEXT_LABEL_REQUEST_QUEUE_CAPACITY - TEXT_LABEL_REQUESTS_PER_PUMP
    );
}

#[test]
fn textdraw_exists_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_textdraw_exists_request(7).unwrap();
    state.queue_textdraw_exists_request(7).unwrap();
    assert_eq!(state.textdraw_exists_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_textdraw_exists_request(id).unwrap();
    }
    assert_eq!(
        state.queue_textdraw_exists_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_textdraw_exists_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.textdraw_exists_requests.lock().unwrap().len(),
        TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY - TEXTDRAW_EXISTS_REQUESTS_PER_PUMP
    );
}

#[test]
fn textdraw_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_textdraw_request(7).unwrap();
    state.queue_textdraw_request(7).unwrap();
    assert_eq!(state.textdraw_requests.lock().unwrap().len(), 1);
    for pool_index in 8..(7 + TEXTDRAW_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_textdraw_request(pool_index).unwrap();
    }
    assert_eq!(
        state.queue_textdraw_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_textdraw_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.textdraw_requests.lock().unwrap().len(),
        TEXTDRAW_REQUEST_QUEUE_CAPACITY - TEXTDRAW_REQUESTS_PER_PUMP
    );
}

#[test]
fn chat_entry_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_chat_entry_request(7).unwrap();
    state.queue_chat_entry_request(7).unwrap();
    assert_eq!(state.chat_entry_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + CHAT_ENTRY_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_chat_entry_request(id).unwrap();
    }
    assert_eq!(
        state.queue_chat_entry_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_chat_entry_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.chat_entry_requests.lock().unwrap().len(),
        CHAT_ENTRY_REQUEST_QUEUE_CAPACITY - CHAT_ENTRY_REQUESTS_PER_PUMP
    );
}

#[test]
fn object_exists_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_object_exists_request(7).unwrap();
    state.queue_object_exists_request(7).unwrap();
    assert_eq!(state.object_exists_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_object_exists_request(id).unwrap();
    }
    assert_eq!(
        state.queue_object_exists_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_object_exists_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.object_exists_requests.lock().unwrap().len(),
        OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY - OBJECT_EXISTS_REQUESTS_PER_PUMP
    );
}

#[test]
fn gangzone_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_gangzone_request(7).unwrap();
    state.queue_gangzone_request(7).unwrap();
    assert_eq!(state.gangzone_requests.lock().unwrap().len(), 1);
    for id in 8..(7 + GANGZONE_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_gangzone_request(id).unwrap();
    }
    assert_eq!(
        state.queue_gangzone_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_gangzone_requests();
    assert_eq!(drained, vec![7, 8, 9, 10]);
    assert_eq!(
        state.gangzone_requests.lock().unwrap().len(),
        GANGZONE_REQUEST_QUEUE_CAPACITY - GANGZONE_REQUESTS_PER_PUMP
    );
}

#[test]
fn contended_request_enqueue_returns_busy_without_losing_work() {
    let state = test_backend_state();
    let _guard = state.player_info_requests.lock().unwrap();

    assert_eq!(
        state.queue_player_info_request(7),
        Err(DirectClientError::Busy)
    );
}

#[test]
fn contended_request_drain_preserves_the_queue() {
    let state = test_backend_state();
    state.queue_player_info_request(7).unwrap();
    let guard = state.player_info_requests.lock().unwrap();

    assert!(state.take_player_info_requests().is_empty());
    drop(guard);
    assert_eq!(state.take_player_info_requests(), vec![7]);
}
