use crate::{
    logging,
    self_test::{TEST_PACKET_ID, TEST_RPC_ID},
    state::{IdHistogram, METRICS, SELF_TESTS, self_test_label},
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

pub(crate) const REPORT_INTERVAL: Duration = Duration::from_secs(5);

pub(crate) fn report_loop() {
    let started = Instant::now();
    let mut next_report = REPORT_INTERVAL;
    while !crate::state::STOP.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = started.elapsed();
        if elapsed >= next_report {
            report_counts(elapsed.as_secs());
            next_report = elapsed + REPORT_INTERVAL;
        }
    }
    report_counts(started.elapsed().as_secs());
    logging::write("reporter stopped");
}

pub(crate) fn report_counts(elapsed_seconds: u64) {
    logging::write(&format!(
        "t={elapsed_seconds}s incoming_packets={} outgoing_packets={} incoming_rpcs={} outgoing_rpcs={} player_skin_decodes={} typed_r1_rpcs={} typed_r1_packets={} typed_r1_rpc_replaced={} typed_r1_packet_replaced={} null_events={} timestamp_decode_errors={}",
        METRICS.incoming_packets.load(Ordering::Relaxed),
        METRICS.outgoing_packets.load(Ordering::Relaxed),
        METRICS.incoming_rpcs.load(Ordering::Relaxed),
        METRICS.outgoing_rpcs.load(Ordering::Relaxed),
        METRICS.player_skin_decoded.load(Ordering::Relaxed),
        METRICS.typed_r1_rpc_callbacks.load(Ordering::Relaxed),
        METRICS.typed_r1_packet_callbacks.load(Ordering::Relaxed),
        METRICS.typed_r1_rpc_replaced.load(Ordering::Relaxed),
        METRICS.typed_r1_packet_replaced.load(Ordering::Relaxed),
        METRICS.null_events.load(Ordering::Relaxed),
        METRICS.timestamp_decode_errors.load(Ordering::Relaxed),
    ));
    report_histogram(
        "incoming_packet_ids",
        &METRICS.incoming_packet_ids,
        packet_name,
    );
    report_histogram(
        "outgoing_packet_ids",
        &METRICS.outgoing_packet_ids,
        packet_name,
    );
    report_histogram(
        "incoming_timestamp_inner_ids",
        &METRICS.incoming_timestamp_inner_ids,
        packet_name,
    );
    report_histogram(
        "outgoing_timestamp_inner_ids",
        &METRICS.outgoing_timestamp_inner_ids,
        packet_name,
    );
    report_histogram(
        "incoming_rpc_ids",
        &METRICS.incoming_rpc_ids,
        incoming_rpc_name,
    );
    report_histogram(
        "outgoing_rpc_ids",
        &METRICS.outgoing_rpc_ids,
        outgoing_rpc_name,
    );
    logging::write(&format!(
        "self_test: packet={} RPC={} dialog={} direct_client={} direct_snapshot_state={} player_directory={} remote_player_state={} send_packet={} send_RPC={}",
        self_test_label(SELF_TESTS.packet.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.rpc.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.dialog.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.direct_client.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.direct_snapshot_state.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.player_directory.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.remote_player_state.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.send_packet.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.send_rpc.load(Ordering::Acquire)),
    ));
}

fn report_histogram(label: &str, histogram: &IdHistogram, name: fn(u8) -> Option<&'static str>) {
    logging::write(&format!("{label}: {}", format_histogram(histogram, name)));
}

pub(crate) fn format_histogram(
    histogram: &IdHistogram,
    name: fn(u8) -> Option<&'static str>,
) -> String {
    let entries: Vec<_> = histogram
        .iter()
        .enumerate()
        .filter_map(|(id, count)| {
            let count = count.load(Ordering::Relaxed);
            if count == 0 {
                return None;
            }
            let id = id as u8;
            Some(match name(id) {
                Some(name) => format!("{id}({name})={count}"),
                None => format!("{id}={count}"),
            })
        })
        .collect();
    if entries.is_empty() {
        "none".to_owned()
    } else {
        entries.join(", ")
    }
}

pub(crate) fn packet_name(id: u8) -> Option<&'static str> {
    Some(match id {
        6 => "ID_INTERNAL_PING",
        7 => "ID_PING",
        8 => "ID_PING_OPEN_CONNECTIONS",
        9 => "ID_CONNECTED_PONG",
        10 => "ID_REQUEST_STATIC_DATA",
        11 => "ID_CONNECTION_REQUEST",
        12 => "ID_AUTH_KEY",
        14 => "ID_BROADCAST_PINGS",
        15 => "ID_SECURED_CONNECTION_RESPONSE",
        16 => "ID_SECURED_CONNECTION_CONFIRMATION",
        17 => "ID_RPC_MAPPING",
        19 => "ID_SET_RANDOM_NUMBER_SEED",
        20 => "ID_RPC",
        21 => "ID_RPC_REPLY",
        23 => "ID_DETECT_LOST_CONNECTIONS",
        24 => "ID_OPEN_CONNECTION_REQUEST",
        25 => "ID_OPEN_CONNECTION_REPLY",
        26 => "ID_OPEN_CONNECTION_COOKIE",
        28 => "ID_RSA_PUBLIC_KEY_MISMATCH",
        29 => "ID_CONNECTION_ATTEMPT_FAILED",
        30 => "ID_NEW_INCOMING_CONNECTION",
        31 => "ID_NO_FREE_INCOMING_CONNECTIONS",
        32 => "ID_DISCONNECTION_NOTIFICATION",
        33 => "ID_CONNECTION_LOST",
        34 => "ID_CONNECTION_REQUEST_ACCEPTED",
        36 => "ID_CONNECTION_BANNED",
        37 => "ID_INVALID_PASSWORD",
        38 => "ID_MODIFIED_PACKET",
        39 => "ID_PONG",
        40 => "ID_TIMESTAMP",
        41 => "ID_RECEIVED_STATIC_DATA",
        42 => "ID_REMOTE_DISCONNECTION_NOTIFICATION",
        43 => "ID_REMOTE_CONNECTION_LOST",
        44 => "ID_REMOTE_NEW_INCOMING_CONNECTION",
        45 => "ID_REMOTE_EXISTING_CONNECTION",
        46 => "ID_REMOTE_STATIC_DATA",
        55 => "ID_ADVERTISE_SYSTEM",
        200 => "ID_VEHICLE_SYNC",
        201 => "ID_RCON_COMMAND",
        202 => "ID_RCON_RESPONSE",
        203 => "ID_AIM_SYNC",
        204 => "ID_WEAPONS_UPDATE",
        205 => "ID_STATS_UPDATE",
        206 => "ID_BULLET_SYNC",
        207 => "ID_PLAYER_SYNC",
        208 => "ID_MARKERS_SYNC",
        209 => "ID_UNOCCUPIED_SYNC",
        210 => "ID_TRAILER_SYNC",
        211 => "ID_PASSENGER_SYNC",
        212 => "ID_SPECTATOR_SYNC",
        TEST_PACKET_ID => "rak_samp_SELF_TEST",
        _ => return None,
    })
}

pub(crate) fn incoming_rpc_name(id: u8) -> Option<&'static str> {
    Some(match id {
        11 => "SET_PLAYER_NAME",
        12 => "SET_PLAYER_POS",
        13 => "SET_PLAYER_POS_FIND_Z",
        14 => "SET_PLAYER_HEALTH",
        15 => "TOGGLE_PLAYER_CONTROLLABLE",
        16 => "PLAY_SOUND",
        17 => "SET_WORLD_BOUNDS",
        18 => "GIVE_PLAYER_MONEY",
        19 => "SET_PLAYER_FACING_ANGLE",
        20 => "RESET_PLAYER_MONEY",
        21 => "RESET_PLAYER_WEAPONS",
        22 => "GIVE_PLAYER_WEAPON",
        29 => "SET_PLAYER_TIME",
        37 => "DISABLE_CHECKPOINT",
        39 => "DISABLE_RACE_CHECKPOINT",
        40 => "GAMEMODE_RESTART",
        42 => "STOP_AUDIO_STREAM",
        59 => "CHAT_BUBBLE",
        61 => "SHOW_DIALOG",
        66 => "SET_PLAYER_ARMOUR",
        67 => "SET_PLAYER_ARMED_WEAPON",
        69 => "SET_PLAYER_TEAM",
        70 => "PUT_PLAYER_IN_VEHICLE",
        71 => "REMOVE_PLAYER_FROM_VEHICLE",
        73 => "DISPLAY_GAME_TEXT",
        74 => "FORCE_CLASS_SELECTION",
        93 => "SERVER_MESSAGE",
        94 => "SET_WORLD_TIME",
        101 => "CHAT_MESSAGE",
        107 => "SET_CHECKPOINT",
        133 => "SET_PLAYER_WANTED_LEVEL",
        137 => "PLAYER_JOIN",
        138 => "PLAYER_QUIT",
        147 => "SET_VEHICLE_HEALTH",
        152 => "SET_WEATHER",
        153 => "SET_PLAYER_SKIN",
        156 => "SET_INTERIOR",
        159 => "SET_VEHICLE_POSITION",
        160 => "SET_VEHICLE_ANGLE",
        162 => "SET_CAMERA_BEHIND",
        163 => "PLAYER_STREAM_OUT",
        165 => "VEHICLE_STREAM_OUT",
        TEST_RPC_ID => "rak_samp_SELF_TEST",
        _ => return None,
    })
}

fn outgoing_rpc_name(id: u8) -> Option<&'static str> {
    Some(match id {
        23 => "SEND_CLICK_PLAYER",
        26 => "SEND_ENTER_VEHICLE",
        50 => "SEND_COMMAND",
        52 => "SEND_SPAWN",
        53 => "SEND_DEATH_NOTIFICATION",
        62 => "SEND_DIALOG_RESPONSE",
        83 => "SEND_CLICK_TEXT_DRAW",
        101 => "SEND_CHAT",
        118 => "SEND_INTERIOR_CHANGE",
        119 => "SEND_MAP_MARKER",
        128 => "SEND_REQUEST_CLASS",
        129 => "SEND_REQUEST_SPAWN",
        132 => "SEND_MENU_SELECT",
        136 => "SEND_VEHICLE_DESTROYED",
        154 => "SEND_EXIT_VEHICLE",
        155 => "SEND_UPDATE_SCORES_AND_PINGS",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{format_histogram, incoming_rpc_name, packet_name};
    use crate::{
        self_test::{TEST_PACKET_ID, TEST_RPC_ID},
        state::{ID_COUNT, IdHistogram},
    };
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn names_known_packet_and_rpc_ids() {
        assert_eq!(packet_name(41), Some("ID_RECEIVED_STATIC_DATA"));
        assert_eq!(packet_name(207), Some("ID_PLAYER_SYNC"));
        assert_eq!(incoming_rpc_name(93), Some("SERVER_MESSAGE"));
        assert_eq!(incoming_rpc_name(153), Some("SET_PLAYER_SKIN"));
        assert_eq!(packet_name(TEST_PACKET_ID), Some("rak_samp_SELF_TEST"));
        assert_eq!(incoming_rpc_name(TEST_RPC_ID), Some("rak_samp_SELF_TEST"));
    }

    #[test]
    fn formats_only_nonzero_ids_in_numeric_order() {
        let histogram: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];
        histogram[207].store(3, Ordering::Relaxed);
        histogram[41].store(2, Ordering::Relaxed);

        assert_eq!(
            format_histogram(&histogram, packet_name),
            "41(ID_RECEIVED_STATIC_DATA)=2, 207(ID_PLAYER_SYNC)=3"
        );
    }
}
