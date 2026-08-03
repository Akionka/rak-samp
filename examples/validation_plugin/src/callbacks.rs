use crate::{
    logging, self_test,
    state::{API, ID_STATS_UPDATE, ID_TIMESTAMP, METRICS, SELF_TESTS, SelfTestStatus},
};
use rak_samp_plugin_api::{
    HostApi, RakSampApiV1, RakSampEventV1, RakSampHookAction, RakSampResult,
    events::{RpcAction, packet, rpc::incoming},
};
use std::{ffi::c_void, sync::atomic::Ordering};

pub(crate) unsafe extern "system" fn on_incoming_packet(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    let observed = observe_packet(
        event,
        &METRICS.incoming_packets,
        &METRICS.incoming_packet_ids,
        &METRICS.incoming_timestamp_inner_ids,
    );
    let action = self_test::test_verdict(
        observed,
        event,
        self_test::TEST_PACKET_ID,
        &self_test::TEST_PACKET_INPUT,
        &self_test::TEST_PACKET_REPLACEMENT,
        &SELF_TESTS.packet,
    );
    if action == RakSampHookAction::Block {
        return action;
    }
    let Some((raw_api, packet_id @ (200 | 207 | 208))) = observed else {
        return action;
    };
    let Ok(api) = (unsafe { HostApi::from_raw(raw_api) }) else {
        return action;
    };
    validate_typed_r1_packet(api, event, packet_id)
}

pub(crate) unsafe extern "system" fn on_outgoing_packet(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    let observed = observe_packet(
        event,
        &METRICS.outgoing_packets,
        &METRICS.outgoing_packet_ids,
        &METRICS.outgoing_timestamp_inner_ids,
    );
    if let Some((api, ID_STATS_UPDATE)) = observed {
        self_test::capture_stats_payload(api, event);
    }
    RakSampHookAction::Continue
}

pub(crate) unsafe extern "system" fn on_incoming_rpc(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    let observed = observe(event, &METRICS.incoming_rpcs, &METRICS.incoming_rpc_ids);
    let action = self_test::test_verdict(
        observed,
        event,
        self_test::TEST_RPC_ID,
        &self_test::TEST_RPC_INPUT,
        &self_test::TEST_RPC_REPLACEMENT,
        &SELF_TESTS.rpc,
    );
    if action == RakSampHookAction::Block {
        return action;
    }
    let Some((raw_api, rpc_id)) = observed else {
        return action;
    };
    let Ok(api) = (unsafe { HostApi::from_raw(raw_api) }) else {
        match rpc_id {
            61 => SELF_TESTS
                .dialog
                .store(SelfTestStatus::Failed.as_raw(), Ordering::Release),
            153 => logging::write("SetPlayerSkin callback could not read the host API"),
            _ => logging::write("R1 typed RPC callback could not read the host API"),
        }
        return action;
    };
    match rpc_id {
        61 => validate_dialog_rewrite(api, event),
        153 => validate_player_skin(api, event),
        32 | 36 | 44 | 68 | 76 | 82 | 83 | 84 | 86 | 104 | 112 | 113 | 117 | 124 | 128 | 134
        | 135 | 139 | 155 | 164 | 167 | 170 | 173 => validate_typed_r1_rpc(api, event, rpc_id),
        _ => action,
    }
}

pub(crate) unsafe extern "system" fn on_outgoing_rpc(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    observe(event, &METRICS.outgoing_rpcs, &METRICS.outgoing_rpc_ids);
    RakSampHookAction::Continue
}

fn validate_player_skin(api: HostApi, event: *mut RakSampEventV1) -> RakSampHookAction {
    match unsafe {
        incoming::on_set_player_skin(api, event, |_skin| {
            METRICS.player_skin_decoded.fetch_add(1, Ordering::Relaxed);
            RpcAction::Continue
        })
    } {
        Ok(action) => action,
        Err(error) => {
            logging::write(&format!("SetPlayerSkin decode failed: {error}"));
            RakSampHookAction::Continue
        }
    }
}

fn typed_rpc_action<T>(value: T) -> RpcAction<T> {
    METRICS
        .typed_r1_rpc_callbacks
        .fetch_add(1, Ordering::Relaxed);
    if METRICS
        .typed_r1_rpc_replaced
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        RpcAction::Replace(value)
    } else {
        RpcAction::Continue
    }
}

fn typed_packet_action<T>(value: T) -> RpcAction<T> {
    METRICS
        .typed_r1_packet_callbacks
        .fetch_add(1, Ordering::Relaxed);
    if METRICS
        .typed_r1_packet_replaced
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        RpcAction::Replace(value)
    } else {
        RpcAction::Continue
    }
}

fn validate_typed_r1_packet(
    api: HostApi,
    event: *mut RakSampEventV1,
    packet_id: u8,
) -> RakSampHookAction {
    macro_rules! handle_packet {
        ($helper:path) => {
            match unsafe { $helper(api, event, typed_packet_action) } {
                Ok(action) => action,
                Err(error) => {
                    logging::write(&format!("R1 typed packet {packet_id} failed: {error}"));
                    RakSampHookAction::Continue
                }
            }
        };
    }
    match packet_id {
        200 => handle_packet!(packet::incoming::on_vehicle_sync),
        207 => handle_packet!(packet::incoming::on_player_sync),
        208 => handle_packet!(packet::incoming::on_markers_sync),
        _ => RakSampHookAction::Continue,
    }
}

fn validate_typed_r1_rpc(
    api: HostApi,
    event: *mut RakSampEventV1,
    rpc_id: u8,
) -> RakSampHookAction {
    macro_rules! handle_rpc {
        ($helper:path) => {
            match unsafe { $helper(api, event, typed_rpc_action) } {
                Ok(action) => action,
                Err(error) => {
                    logging::write(&format!("R1 typed RPC {rpc_id} failed: {error}"));
                    RakSampHookAction::Continue
                }
            }
        };
    }
    match rpc_id {
        32 => handle_rpc!(incoming::on_player_stream_in),
        36 => handle_rpc!(incoming::on_create_3d_text),
        44 => handle_rpc!(incoming::on_create_object),
        68 => handle_rpc!(incoming::on_set_spawn_info),
        76 => handle_rpc!(incoming::on_init_menu),
        82 => handle_rpc!(incoming::on_interpolate_camera),
        83 => handle_rpc!(incoming::on_toggle_select_text_draw),
        84 => handle_rpc!(incoming::on_set_object_material),
        86 => handle_rpc!(incoming::on_apply_player_animation),
        104 => handle_rpc!(incoming::on_enable_stunt_bonus),
        112 => handle_rpc!(incoming::on_play_crime_report),
        113 => handle_rpc!(incoming::on_set_player_attached_object),
        117 => handle_rpc!(incoming::on_enter_edit_object),
        124 => handle_rpc!(incoming::on_toggle_player_spectating),
        128 => handle_rpc!(incoming::on_request_class_response),
        134 => handle_rpc!(incoming::on_show_text_draw),
        135 => handle_rpc!(incoming::on_text_draw_hide),
        139 => handle_rpc!(incoming::on_init_game),
        155 => handle_rpc!(incoming::on_update_scores_and_pings),
        164 => handle_rpc!(incoming::on_vehicle_stream_in),
        167 => handle_rpc!(incoming::on_disable_vehicle_collisions),
        170 => handle_rpc!(incoming::on_toggle_camera_target_notifying),
        173 => handle_rpc!(incoming::on_apply_actor_animation),
        _ => RakSampHookAction::Continue,
    }
}

fn validate_dialog_rewrite(api: HostApi, event: *mut RakSampEventV1) -> RakSampHookAction {
    let first = unsafe {
        incoming::on_show_dialog(api, event, |dialog| {
            if dialog == self_test::test_dialog_input() {
                SELF_TESTS
                    .dialog
                    .store(SelfTestStatus::Rewritten.as_raw(), Ordering::Release);
                RpcAction::Replace(self_test::test_dialog_replacement())
            } else {
                RpcAction::Continue
            }
        })
    };
    if let Err(error) = first {
        SELF_TESTS
            .dialog
            .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
        logging::write(&format!("dialog decode/rewrite failed: {error}"));
        return RakSampHookAction::Continue;
    }
    if SELF_TESTS.dialog.load(Ordering::Acquire) != SelfTestStatus::Rewritten.as_raw() {
        return first.unwrap_or(RakSampHookAction::Continue);
    }
    match unsafe {
        incoming::on_show_dialog(api, event, |dialog| {
            if dialog == self_test::test_dialog_replacement() {
                SELF_TESTS
                    .dialog
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                RpcAction::Block
            } else {
                SELF_TESTS
                    .dialog
                    .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
                RpcAction::Continue
            }
        })
    } {
        Ok(action) => action,
        Err(error) => {
            SELF_TESTS
                .dialog
                .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
            logging::write(&format!("dialog replacement verification failed: {error}"));
            RakSampHookAction::Continue
        }
    }
}

fn observe_packet(
    event: *mut RakSampEventV1,
    count: &std::sync::atomic::AtomicU32,
    ids: &crate::state::IdHistogram,
    timestamp_inner_ids: &crate::state::IdHistogram,
) -> Option<(*mut RakSampApiV1, u8)> {
    let (api, id) = observe(event, count, ids)?;
    if id != ID_TIMESTAMP {
        return Some((api, id));
    }

    let mut timestamp = 0;
    let mut inner_id = 0;
    let decoded = unsafe { ((*api).event_reset_read)(event) } == RakSampResult::Ok
        && unsafe { ((*api).event_read_u32)(event, &raw mut timestamp) } == RakSampResult::Ok
        && unsafe { ((*api).event_read_u8)(event, &raw mut inner_id) } == RakSampResult::Ok;
    let restored = unsafe { ((*api).event_reset_read)(event) } == RakSampResult::Ok;
    if decoded && restored {
        timestamp_inner_ids[usize::from(inner_id)].fetch_add(1, Ordering::Relaxed);
    } else {
        METRICS
            .timestamp_decode_errors
            .fetch_add(1, Ordering::Relaxed);
    }
    Some((api, id))
}

fn observe(
    event: *mut RakSampEventV1,
    count: &std::sync::atomic::AtomicU32,
    ids: &crate::state::IdHistogram,
) -> Option<(*mut RakSampApiV1, u8)> {
    if event.is_null() {
        METRICS.null_events.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    let api = API.load(Ordering::Acquire);
    if api.is_null() {
        return None;
    }
    let id = unsafe { ((*api).event_id)(event) };
    ids[usize::from(id)].fetch_add(1, Ordering::Relaxed);
    count.fetch_add(1, Ordering::Relaxed);
    Some((api, id))
}
