use crate::{
    logging, self_test,
    state::{ID_STATS_UPDATE, ID_TIMESTAMP, METRICS, SELF_TESTS, SelfTestStatus},
};
use rak_samp_plugin_api::{
    RakSampHookAction,
    events::{Event, RpcAction, packet, rpc::incoming},
};
use std::sync::atomic::Ordering;

pub(crate) fn on_incoming_packet(event: &mut Event<'_>) -> RakSampHookAction {
    let packet_id = observe_packet(
        event,
        &METRICS.incoming_packets,
        &METRICS.incoming_packet_ids,
        &METRICS.incoming_timestamp_inner_ids,
    );
    let action = self_test::test_verdict(
        event,
        packet_id,
        self_test::TEST_PACKET_ID,
        &self_test::TEST_PACKET_INPUT,
        &self_test::TEST_PACKET_REPLACEMENT,
        &SELF_TESTS.packet,
    );
    if action == RakSampHookAction::Block {
        return action;
    }
    match packet_id {
        200 | 207 | 208 => validate_typed_r1_packet(event, packet_id),
        _ => action,
    }
}

pub(crate) fn on_outgoing_packet(event: &mut Event<'_>) -> RakSampHookAction {
    let packet_id = observe_packet(
        event,
        &METRICS.outgoing_packets,
        &METRICS.outgoing_packet_ids,
        &METRICS.outgoing_timestamp_inner_ids,
    );
    if packet_id == ID_STATS_UPDATE {
        self_test::capture_stats_payload(event);
    }
    RakSampHookAction::Continue
}

pub(crate) fn on_incoming_rpc(event: &mut Event<'_>) -> RakSampHookAction {
    let rpc_id = observe(event, &METRICS.incoming_rpcs, &METRICS.incoming_rpc_ids);
    let action = self_test::test_verdict(
        event,
        rpc_id,
        self_test::TEST_RPC_ID,
        &self_test::TEST_RPC_INPUT,
        &self_test::TEST_RPC_REPLACEMENT,
        &SELF_TESTS.rpc,
    );
    if action == RakSampHookAction::Block {
        return action;
    }
    match rpc_id {
        61 => validate_dialog_rewrite(event),
        153 => validate_player_skin(event),
        32 | 36 | 44 | 68 | 76 | 82 | 83 | 84 | 86 | 104 | 112 | 113 | 117 | 124 | 128 | 134
        | 135 | 139 | 155 | 164 | 167 | 170 | 173 => validate_typed_r1_rpc(event, rpc_id),
        _ => action,
    }
}

pub(crate) fn on_outgoing_rpc(event: &mut Event<'_>) -> RakSampHookAction {
    observe(event, &METRICS.outgoing_rpcs, &METRICS.outgoing_rpc_ids);
    RakSampHookAction::Continue
}

fn validate_player_skin(event: &mut Event<'_>) -> RakSampHookAction {
    match incoming::SET_PLAYER_SKIN.handle(event, |_skin| {
        METRICS.player_skin_decoded.fetch_add(1, Ordering::Relaxed);
        RpcAction::Continue
    }) {
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

fn validate_typed_r1_packet(event: &mut Event<'_>, packet_id: u8) -> RakSampHookAction {
    macro_rules! handle_packet {
        ($descriptor:path) => {
            match $descriptor.handle(event, typed_packet_action) {
                Ok(action) => action,
                Err(error) => {
                    logging::write(&format!("R1 typed packet {packet_id} failed: {error}"));
                    RakSampHookAction::Continue
                }
            }
        };
    }
    match packet_id {
        200 => handle_packet!(packet::incoming::VEHICLE_SYNC),
        207 => handle_packet!(packet::incoming::PLAYER_SYNC),
        208 => handle_packet!(packet::incoming::MARKERS_SYNC),
        _ => RakSampHookAction::Continue,
    }
}

fn validate_typed_r1_rpc(event: &mut Event<'_>, rpc_id: u8) -> RakSampHookAction {
    macro_rules! handle_rpc {
        ($descriptor:path) => {
            match $descriptor.handle(event, typed_rpc_action) {
                Ok(action) => action,
                Err(error) => {
                    logging::write(&format!("R1 typed RPC {rpc_id} failed: {error}"));
                    RakSampHookAction::Continue
                }
            }
        };
    }
    match rpc_id {
        32 => handle_rpc!(incoming::PLAYER_STREAM_IN),
        36 => handle_rpc!(incoming::CREATE_3D_TEXT),
        44 => handle_rpc!(incoming::CREATE_OBJECT),
        68 => handle_rpc!(incoming::SET_SPAWN_INFO),
        76 => handle_rpc!(incoming::INIT_MENU),
        82 => handle_rpc!(incoming::INTERPOLATE_CAMERA),
        83 => handle_rpc!(incoming::TOGGLE_SELECT_TEXT_DRAW),
        84 => handle_rpc!(incoming::SET_OBJECT_MATERIAL),
        86 => handle_rpc!(incoming::APPLY_PLAYER_ANIMATION),
        104 => handle_rpc!(incoming::ENABLE_STUNT_BONUS),
        112 => handle_rpc!(incoming::PLAY_CRIME_REPORT),
        113 => handle_rpc!(incoming::SET_PLAYER_ATTACHED_OBJECT),
        117 => handle_rpc!(incoming::ENTER_EDIT_OBJECT),
        124 => handle_rpc!(incoming::TOGGLE_PLAYER_SPECTATING),
        128 => handle_rpc!(incoming::REQUEST_CLASS_RESPONSE),
        134 => handle_rpc!(incoming::SHOW_TEXT_DRAW),
        135 => handle_rpc!(incoming::TEXT_DRAW_HIDE),
        139 => handle_rpc!(incoming::INIT_GAME),
        155 => handle_rpc!(incoming::UPDATE_SCORES_AND_PINGS),
        164 => handle_rpc!(incoming::VEHICLE_STREAM_IN),
        167 => handle_rpc!(incoming::DISABLE_VEHICLE_COLLISIONS),
        170 => handle_rpc!(incoming::TOGGLE_CAMERA_TARGET_NOTIFYING),
        173 => handle_rpc!(incoming::APPLY_ACTOR_ANIMATION),
        _ => RakSampHookAction::Continue,
    }
}

fn validate_dialog_rewrite(event: &mut Event<'_>) -> RakSampHookAction {
    let first = incoming::SHOW_DIALOG.handle(event, |dialog| {
        if dialog == self_test::test_dialog_input() {
            SELF_TESTS
                .dialog
                .store(SelfTestStatus::Rewritten.as_raw(), Ordering::Release);
            RpcAction::Replace(self_test::test_dialog_replacement())
        } else {
            RpcAction::Continue
        }
    });
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
    match incoming::SHOW_DIALOG.handle(event, |dialog| {
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
    }) {
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
    event: &mut Event<'_>,
    count: &std::sync::atomic::AtomicU32,
    ids: &crate::state::IdHistogram,
    timestamp_inner_ids: &crate::state::IdHistogram,
) -> u8 {
    let id = observe(event, count, ids);
    if id != ID_TIMESTAMP {
        return id;
    }

    let decoded = event.reset_read().and_then(|_| {
        let _timestamp = event.read_u32()?;
        event.read_u8()
    });
    let restored = event.reset_read().is_ok();
    if let (Ok(inner_id), true) = (decoded, restored) {
        timestamp_inner_ids[usize::from(inner_id)].fetch_add(1, Ordering::Relaxed);
    } else {
        METRICS
            .timestamp_decode_errors
            .fetch_add(1, Ordering::Relaxed);
    }
    id
}

fn observe(
    event: &Event<'_>,
    count: &std::sync::atomic::AtomicU32,
    ids: &crate::state::IdHistogram,
) -> u8 {
    let id = event.id();
    ids[usize::from(id)].fetch_add(1, Ordering::Relaxed);
    count.fetch_add(1, Ordering::Relaxed);
    id
}
