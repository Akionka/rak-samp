//! Probe status bits, failure state, and status-file reporting.

use super::*;

/// The SDK host API resolved for this probe.
pub const STATUS_HOST_CONNECTED: u32 = 1 << 0;
/// Public host status, version, and the opaque module base identify the pinned profile image.
pub const STATUS_RUNTIME_IDENTITY: u32 = 1 << 1;
/// The selected profile's CNetGame scalar cache reported the expected connected server values.
pub const STATUS_CNETGAME_SCALARS: u32 = 1 << 6;
/// The selected profile's cached local-player snapshot passed bounded sanity checks.
pub const STATUS_LOCAL_PLAYER_SNAPSHOT: u32 = 1 << 7;
/// The selected profile's cached player-pool count pair and largest ID matched the loopback session.
pub const STATUS_PLAYER_POOL_SCALARS: u32 = 1 << 12;
/// The selected profile's cached scoreboard flag observed both the open and closed states.
pub const STATUS_SCOREBOARD_CACHE: u32 = 1 << 13;
/// The selected profile's cached chat display mode returned a documented native value.
pub const STATUS_CHAT_DISPLAY_MODE: u32 = 1 << 14;
/// The selected profile's cached cursor mode returned a documented native value.
pub const STATUS_CURSOR_MODE: u32 = 1 << 15;
/// The selected profile's cached remote-player directory observed another loopback client.
pub const STATUS_REMOTE_PLAYER_DIRECTORY: u32 = 1 << 16;
/// Object, pickup, vehicle, gangzone, and player-ped handle paths round-tripped.
pub const STATUS_ENTITY_HANDLES: u32 = 1 << 17;
/// Native local-player force-sync methods completed on the game thread.
pub const STATUS_FORCE_SYNC_RECEIPTS: u32 = 1 << 18;
/// Chat, input, cursor, scoreboard, and death-window mutations completed.
pub const STATUS_UI_MUTATIONS: u32 = 1 << 19;
/// Client-side input/list dialogs and the close-response hook passed.
pub const STATUS_DIALOG_LIFECYCLE: u32 = 1 << 20;
/// A native chat command was installed, invoked, and removed synchronously.
pub const STATUS_CHAT_COMMAND_LIFECYCLE: u32 = 1 << 21;
/// The animation table round-tripped one name/file pair.
pub const STATUS_ANIMATION_TABLE: u32 = 1 << 22;
/// Local-player colour/action and all send-rate writes completed and restored.
pub const STATUS_LOCAL_MUTATIONS: u32 = 1 << 23;
/// An automatically allocated text label completed create/read/write/delete.
pub const STATUS_TEXT_LABEL_LIFECYCLE: u32 = 1 << 24;
/// A free R5 textdraw slot completed create/read/write/delete.
pub const STATUS_TEXTDRAW_LIFECYCLE: u32 = 1 << 25;
/// Local on-foot/aim and remote NPC on-foot snapshots passed bounded checks.
pub const STATUS_SYNC_SNAPSHOTS: u32 = 1 << 26;
/// Controlled local vehicle states and all vehicle force-sync packets passed.
pub const STATUS_VEHICLE_SYNC: u32 = 1 << 27;
/// Disconnect invalidated the R5 connection-bound caches.
pub const STATUS_DISCONNECT_INVALIDATION: u32 = 1 << 28;
/// Reconnect restored R5 caches and post-reconnect packet/RPC delivery.
pub const STATUS_RECONNECT_RESTORED: u32 = 1 << 29;
/// Complete connected-session validation before the opt-in reconnect phase.
pub const MAIN_SUCCESS_STATUS: u32 = 0x0FFF_FFFF;
/// Complete validation including disconnect invalidation and reconnect restoration.
pub const FULL_SUCCESS_STATUS: u32 = 0x3FFF_FFFF;
/// The R5 cached chat-input active flag and command names passed the live check.
pub const STATUS_CHAT_INPUT_CACHE: u32 = 1 << 8;
/// The R5 cached dialog active flag observed the disposable server dialog.
pub const STATUS_DIALOG_ACTIVE_CACHE: u32 = 1 << 9;
/// The loopback dialog-request command completed on the game thread.
pub const STATUS_DIALOG_REQUEST_RECEIPT: u32 = 1 << 10;
/// The cached chat-input text matched the operator-entered marker.
pub const STATUS_CHAT_INPUT_TEXT_CACHE: u32 = 1 << 11;
/// The non-blocking RPC 93 listener was registered.
pub const STATUS_REPLY_LISTENER_REGISTERED: u32 = 1 << 2;
/// A real inbound RPC supplied the native receiver required for a connected client.
pub const STATUS_INCOMING_READY: u32 = 1 << 3;
/// The single outgoing chat command completed successfully on the game thread.
pub const STATUS_OUTBOUND_RECEIPT: u32 = 1 << 4;
/// The matching server reply was observed before continuing to the original handler.
pub const STATUS_REPLY_OBSERVED: u32 = 1 << 5;
/// An initialization, command, or reply stage failed.
pub const STATUS_FAILED: u32 = 1 << 31;

pub(super) fn record_failure(error: SampClientSdkResult) {
    FAILURE
        .compare_exchange(
            SampClientSdkResult::Ok as u32,
            error as u32,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .ok();
    STATUS.fetch_or(STATUS_FAILED, Ordering::AcqRel);
}

pub(super) fn stored_failure() -> SampClientSdkResult {
    match FAILURE.load(Ordering::Acquire) {
        value if value == SampClientSdkResult::NotReady as u32 => SampClientSdkResult::NotReady,
        value if value == SampClientSdkResult::TimedOut as u32 => SampClientSdkResult::TimedOut,
        value if value == SampClientSdkResult::ShuttingDown as u32 => {
            SampClientSdkResult::ShuttingDown
        }
        _ => SampClientSdkResult::NativeCallFailed,
    }
}

pub(super) fn publish_status() {
    let _ = fs::write(
        STATUS_FILE,
        status_record(
            STATUS.load(Ordering::Acquire),
            FAILURE.load(Ordering::Acquire),
            SCALAR_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref(),
            PLAYER_POOL_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref(),
            RECONNECT_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref(),
            ProbePhaseStatus {
                text_label_phase: *TEXT_LABEL_PHASE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()),
                text_label_initial_fields: TEXT_LABEL_INITIAL_FIELDS.load(Ordering::Acquire),
                text_label_initial_result: TEXT_LABEL_INITIAL_RESULT.load(Ordering::Acquire),
                textdraw_phase: *TEXTDRAW_PHASE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()),
                textdraw_snapshot_fields: TEXTDRAW_SNAPSHOT_FIELDS.load(Ordering::Acquire),
                textdraw_snapshot_result: TEXTDRAW_SNAPSHOT_RESULT.load(Ordering::Acquire),
                vehicle_phase: *VEHICLE_PHASE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()),
            },
        ),
    );
}

pub(super) fn status_record(
    status: u32,
    failure: u32,
    scalar: Option<&ScalarObservation>,
    player_pool: Option<&PlayerPoolObservation>,
    reconnect: Option<&ReconnectObservation>,
    phases: ProbePhaseStatus,
) -> String {
    use std::fmt::Write;

    let mut record = format!("status=0x{status:08X}\nfailure={failure}\n");
    if let Some(scalar) = scalar {
        let _ = writeln!(record, "game_state={}", scalar.game_state);
        let _ = writeln!(record, "address_hex={}", hex(&scalar.address));
        let _ = writeln!(record, "hostname_hex={}", hex(&scalar.hostname));
        let _ = writeln!(record, "port={}", scalar.port);
    }
    if let Some(player_pool) = player_pool {
        let _ = writeln!(
            record,
            "player_count_including_npcs={}",
            player_pool.including_npcs
        );
        let _ = writeln!(
            record,
            "player_count_excluding_npcs={}",
            player_pool.excluding_npcs
        );
        let _ = writeln!(record, "player_max_id={:?}", player_pool.max_id);
    }
    if let Some(reconnect) = reconnect {
        let _ = writeln!(record, "reconnect_server_ready={}", reconnect.server_ready);
        let _ = writeln!(record, "reconnect_local_ready={}", reconnect.local_ready);
        let _ = writeln!(record, "reconnect_game_state={:?}", reconnect.game_state);
        let _ = writeln!(record, "reconnect_spawned={:?}", reconnect.spawned);
        let _ = writeln!(
            record,
            "reconnect_incoming_ready={}",
            reconnect.incoming_ready
        );
    }
    let _ = writeln!(record, "text_label_phase={}", phases.text_label_phase);
    let _ = writeln!(
        record,
        "text_label_initial_fields=0x{:02X}",
        phases.text_label_initial_fields
    );
    let _ = writeln!(
        record,
        "text_label_initial_result=0x{:03X}",
        phases.text_label_initial_result
    );
    let _ = writeln!(record, "textdraw_phase={}", phases.textdraw_phase);
    let _ = writeln!(
        record,
        "textdraw_snapshot_fields=0x{:03X}",
        phases.textdraw_snapshot_fields
    );
    let _ = writeln!(
        record,
        "textdraw_snapshot_result=0x{:03X}",
        phases.textdraw_snapshot_result
    );
    let _ = writeln!(record, "vehicle_phase={}", phases.vehicle_phase);
    #[cfg(feature = "r1-probe")]
    {
        let _ = writeln!(
            record,
            "codec_round_trip={}",
            R1_CODEC_ROUND_TRIP.load(Ordering::Acquire)
        );
        let _ = writeln!(
            record,
            "incoming_packet_bits={}",
            usize::from(R1_PACKET_EXACT_BITS.load(Ordering::Acquire)) * R1_EXACT_BIT_COUNT
        );
        let _ = writeln!(
            record,
            "incoming_rpc_bits={}",
            usize::from(R1_RPC_EXACT_BITS.load(Ordering::Acquire)) * R1_EXACT_BIT_COUNT
        );
    }
    record
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}
