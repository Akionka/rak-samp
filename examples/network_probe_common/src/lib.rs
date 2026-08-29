//! Loopback-only version-selected network delivery validation.
//!
//! This probe sends one bounded chat marker only to the disposable local server
//! filter. Its matching server-message listener always continues, allowing a
//! human to verify that SA-MP's original incoming-RPC handler displayed reply.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp-client-sdk network probes support only 32-bit Windows x86 targets");

mod config;
mod entities;
mod network;
mod reconnect;
mod state;
mod status;
mod sync;
mod ui;

use config::*;
use entities::*;
use network::*;
use reconnect::*;
#[cfg(test)]
use samp_client_sdk::SampClientSdkClientVersion;
use samp_client_sdk::{
    CommandReceipt, GangzoneId, LocalChatDisplayMode, LocalChatMessage, LocalChatMessageStyle,
    LocalCursorMode, LocalDeathMessage, LocalDialog, LocalDialogStyle, ObjectId, PlayerId,
    ProtocolSendError, Samp, SampClientSdkDirection, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkResult, SendRateKind, SpecialAction, SubscriptionSet,
    TextdrawId, Vector3, VehicleId, events::ProtocolAction, raw,
};
#[cfg(feature = "r1-probe")]
use samp_protocol::BitStream;
use samp_protocol::{
    WireDescriptor,
    packet::common::{
        SendAimSync, SendPassengerSync, SendPlayerSync, SendStatsUpdate, SendTrailerSync,
        SendUnoccupiedSync, SendVehicleSync, SendWeaponsUpdate,
    },
    rpc::incoming::common::SERVER_MESSAGE,
};
use state::*;
use status::*;
pub use status::{FULL_SUCCESS_STATUS, MAIN_SUCCESS_STATUS};
use std::{
    ffi::c_void,
    fs, ptr,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use sync::*;
use ui::*;
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};
use windows_sys::core::BOOL;

struct InitializationGuard;

impl Drop for InitializationGuard {
    fn drop(&mut self) {
        STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .initializing = false;
        INITIALIZATION_FINISHED.notify_all();
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(instance) };
            STATUS.store(0, Ordering::Release);
            FAILURE.store(SampClientSdkResult::Ok as u32, Ordering::Release);
            *SCALAR_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            *PLAYER_POOL_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            *RECONNECT_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            CHAT_COMMAND_INVOKED.store(false, Ordering::Release);
            SYNC_PACKETS_OBSERVED.store(0, Ordering::Release);
            for count in &SYNC_PACKET_COUNTS {
                count.store(0, Ordering::Release);
            }
            *TEXT_LABEL_PHASE
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = "none";
            TEXT_LABEL_INITIAL_FIELDS.store(0, Ordering::Release);
            TEXT_LABEL_INITIAL_RESULT.store(0, Ordering::Release);
            *TEXTDRAW_PHASE
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = "none";
            TEXTDRAW_SNAPSHOT_FIELDS.store(0, Ordering::Release);
            TEXTDRAW_SNAPSHOT_RESULT.store(0, Ordering::Release);
            *VEHICLE_PHASE
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = "none";
            RECONNECT_REQUESTED.store(false, Ordering::Release);
            INCOMING_REPLY_COUNT.store(0, Ordering::Release);
            #[cfg(feature = "r1-probe")]
            {
                R1_CODEC_ROUND_TRIP.store(false, Ordering::Release);
                R1_PACKET_EXACT_BITS.store(false, Ordering::Release);
                R1_RPC_EXACT_BITS.store(false, Ordering::Release);
            }
            *VEHICLE_PHASES
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = VehiclePhases::new();
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .initializing = true;
            if thread::Builder::new()
                .name("samp-client-sdk-r5-network-probe-init".into())
                .spawn(initialize)
                .is_err()
            {
                record_failure(SampClientSdkResult::NativeCallFailed);
                STATE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .initializing = false;
                INITIALIZATION_FINISHED.notify_all();
            }
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

fn initialize() {
    let _initialization = InitializationGuard;
    publish_status();
    let Some(samp) = connect_host() else {
        publish_status();
        return;
    };
    STATUS.fetch_or(STATUS_HOST_CONNECTED, Ordering::AcqRel);
    publish_status();
    if let Err(error) = verify_runtime_identity(samp) {
        record_failure(error);
        publish_status();
        return;
    }
    STATUS.fetch_or(STATUS_RUNTIME_IDENTITY, Ordering::AcqRel);
    publish_status();

    let subscriptions = match register_listeners(samp) {
        Ok(subscriptions) => subscriptions,
        Err(error) => {
            record_failure(error);
            publish_status();
            return;
        }
    };

    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    if state.shutting_down {
        return;
    }
    state.subscriptions = Some(subscriptions);
    drop(state);
    STATUS.fetch_or(STATUS_REPLY_LISTENER_REGISTERED, Ordering::AcqRel);
    publish_status();

    let result = run_probe(samp);
    if let Err(error) = result {
        record_failure(error);
    }
    publish_status();
}

fn run_probe(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_incoming_ready(samp)?;
    STATUS.fetch_or(STATUS_INCOMING_READY, Ordering::AcqRel);
    publish_status();
    #[cfg(feature = "r1-probe")]
    verify_r1_codec_and_exact_bits(samp)?;
    verify_cached_cnetgame_scalars(samp)?;
    STATUS.fetch_or(STATUS_CNETGAME_SCALARS, Ordering::AcqRel);
    publish_status();
    verify_cached_local_player(samp)?;
    STATUS.fetch_or(STATUS_LOCAL_PLAYER_SNAPSHOT, Ordering::AcqRel);
    publish_status();
    verify_entity_handles(samp)?;
    STATUS.fetch_or(STATUS_ENTITY_HANDLES, Ordering::AcqRel);
    publish_status();
    verify_force_sync_receipts(samp)?;
    STATUS.fetch_or(STATUS_FORCE_SYNC_RECEIPTS, Ordering::AcqRel);
    publish_status();
    verify_cached_player_pool_scalars(samp)?;
    STATUS.fetch_or(STATUS_PLAYER_POOL_SCALARS, Ordering::AcqRel);
    publish_status();
    #[cfg(feature = "r1-probe")]
    verify_r1_raw_addresses(samp)?;
    verify_cached_remote_player_directory(samp)?;
    STATUS.fetch_or(STATUS_REMOTE_PLAYER_DIRECTORY, Ordering::AcqRel);
    publish_status();
    verify_cached_chat_display_mode(samp)?;
    STATUS.fetch_or(STATUS_CHAT_DISPLAY_MODE, Ordering::AcqRel);
    publish_status();
    verify_cached_cursor_mode(samp)?;
    STATUS.fetch_or(STATUS_CURSOR_MODE, Ordering::AcqRel);
    publish_status();

    let receipt = probe_protocol_send(samp.net().send_chat(OUTBOUND_MARKER))?;
    wait_for_receipt(receipt)?;
    STATUS.fetch_or(STATUS_OUTBOUND_RECEIPT, Ordering::AcqRel);
    publish_status();

    wait_for_status(STATUS_REPLY_OBSERVED, CALLBACK_TIMEOUT)
        .and_then(|()| verify_cached_scoreboard_transition(samp))?;
    STATUS.fetch_or(STATUS_SCOREBOARD_CACHE, Ordering::AcqRel);
    publish_status();
    wait_for_receipt(samp.chat_input().set_enabled(true)?)?;
    wait_for_receipt(samp.chat_input().set_text(CHAT_INPUT_TEXT_MARKER)?)?;
    verify_cached_chat_input(samp)?;
    STATUS.fetch_or(STATUS_CHAT_INPUT_CACHE, Ordering::AcqRel);
    publish_status();
    verify_cached_chat_input_text(samp)?;
    STATUS.fetch_or(STATUS_CHAT_INPUT_TEXT_CACHE, Ordering::AcqRel);
    publish_status();
    wait_for_receipt(samp.chat_input().set_enabled(false)?)?;
    let dialog_receipt = probe_protocol_send(samp.net().send_chat(DIALOG_REQUEST_MARKER))?;
    wait_for_receipt(dialog_receipt)?;
    STATUS.fetch_or(STATUS_DIALOG_REQUEST_RECEIPT, Ordering::AcqRel);
    publish_status();
    verify_cached_dialog_active(samp)?;
    STATUS.fetch_or(STATUS_DIALOG_ACTIVE_CACHE, Ordering::AcqRel);
    publish_status();
    verify_dialog_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_DIALOG_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_ui_mutations(samp)?;
    STATUS.fetch_or(STATUS_UI_MUTATIONS, Ordering::AcqRel);
    publish_status();
    verify_chat_command_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_CHAT_COMMAND_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_animation_table(samp)?;
    STATUS.fetch_or(STATUS_ANIMATION_TABLE, Ordering::AcqRel);
    publish_status();
    verify_local_mutations(samp)?;
    STATUS.fetch_or(STATUS_LOCAL_MUTATIONS, Ordering::AcqRel);
    publish_status();
    verify_text_label_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_TEXT_LABEL_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_textdraw_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_TEXTDRAW_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_sync_snapshots(samp)?;
    STATUS.fetch_or(STATUS_SYNC_SNAPSHOTS, Ordering::AcqRel);
    publish_status();
    verify_vehicle_sync(samp)?;
    STATUS.fetch_or(STATUS_VEHICLE_SYNC, Ordering::AcqRel);
    publish_status();
    verify_reconnect_on_request(samp)?;
    Ok(())
}

fn wait_for_condition(
    timeout: Duration,
    mut condition: impl FnMut() -> Result<bool, SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match condition() {
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

fn wait_for_value<T>(
    timeout: Duration,
    mut read: impl FnMut() -> Result<T, SampClientSdkResult>,
) -> Result<T, SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match read() {
            Ok(value) => return Ok(value),
            Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn is_shutting_down() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .shutting_down
}

/// Stops the callbacks before an unload manager calls `FreeLibrary`.
///
/// This must run on a worker thread, not from `DllMain` or a callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Shutdown() -> BOOL {
    let subscriptions = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        state.subscriptions.take()
    };

    let Some(subscriptions) = subscriptions else {
        return TRUE;
    };
    match subscriptions.unregister_and_wait() {
        Ok(()) => TRUE,
        Err(error) => {
            STATE
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .subscriptions = Some(error.into_subscriptions());
            0
        }
    }
}

/// Returns the probe stage bitset.
///
/// The connected pass is complete at `0x0FFFFFFF`; the opt-in reconnect pass
/// completes at `0x3FFFFFFF`.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Status() -> u32 {
    STATUS.load(Ordering::Acquire)
}

/// Returns the first failing `SampClientSdkResult` value, if any.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Failure() -> u32 {
    FAILURE.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_probe_configuration_matches_its_client_version() {
        #[cfg(feature = "r1-probe")]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::R1,
                0x31DF13,
                14,
                14,
                b"SDK R1 loopback probe".as_slice()
            )
        );
        #[cfg(feature = "r3-probe")]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::R3_1,
                0x0CC4D0,
                15,
                14,
                b"SA-MP".as_slice()
            )
        );
        #[cfg(feature = "dl-probe")]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::Dl,
                0x0FDB60,
                5,
                5,
                b"SDK DL loopback probe".as_slice()
            )
        );
        #[cfg(not(any(feature = "r1-probe", feature = "r3-probe", feature = "dl-probe")))]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::R5_1,
                0x0CBC90,
                5,
                5,
                b"SDK R5 loopback probe".as_slice()
            )
        );
    }

    #[test]
    fn selected_probe_ui_labels_match_its_client_version() {
        let labels = (
            LOCAL_CHAT_PREFIX,
            LOCAL_CHAT_INPUT_MUTATION,
            LOCAL_INPUT_DIALOG_TITLE,
            LOCAL_INPUT_DIALOG_BODY,
            LOCAL_LIST_DIALOG_TITLE,
            MAIN_PASS_MESSAGE,
        );
        #[cfg(feature = "r1-probe")]
        assert_eq!(
            labels,
            (
                b"R1 SDK".as_slice(),
                b"R1_SDK_MUTATION".as_slice(),
                b"R1 input dialog".as_slice(),
                b"R1 input body".as_slice(),
                b"R1 list dialog".as_slice(),
                b"R1 main pass complete. Type /r1sdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
        #[cfg(feature = "r3-probe")]
        assert_eq!(
            labels,
            (
                b"R3 SDK".as_slice(),
                b"R3_SDK_MUTATION".as_slice(),
                b"R3 input dialog".as_slice(),
                b"R3 input body".as_slice(),
                b"R3 list dialog".as_slice(),
                b"R3 main pass complete. Type /r3sdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
        #[cfg(feature = "dl-probe")]
        assert_eq!(
            labels,
            (
                b"DL SDK".as_slice(),
                b"DL_SDK_MUTATION".as_slice(),
                b"DL input dialog".as_slice(),
                b"DL input body".as_slice(),
                b"DL list dialog".as_slice(),
                b"DL main pass complete. Type /dlsdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
        #[cfg(not(any(feature = "r1-probe", feature = "r3-probe", feature = "dl-probe")))]
        assert_eq!(
            labels,
            (
                b"R5 SDK".as_slice(),
                b"R5_SDK_MUTATION".as_slice(),
                b"R5 input dialog".as_slice(),
                b"R5 input body".as_slice(),
                b"R5 list dialog".as_slice(),
                b"R5 main pass complete. Type /r5sdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
    }

    #[test]
    fn status_record_is_bounded_and_machine_readable() {
        let record = status_record(
            STATUS_HOST_CONNECTED | STATUS_OUTBOUND_RECEIPT,
            0,
            None,
            None,
            None,
            ProbePhaseStatus {
                text_label_phase: "none",
                text_label_initial_fields: 0,
                text_label_initial_result: 0,
                textdraw_phase: "none",
                textdraw_snapshot_fields: 0,
                textdraw_snapshot_result: 0,
                vehicle_phase: "none",
            },
        );
        #[cfg(not(feature = "r1-probe"))]
        assert_eq!(
            record,
            "status=0x00000011\nfailure=0\ntext_label_phase=none\ntext_label_initial_fields=0x00\ntext_label_initial_result=0x000\ntextdraw_phase=none\ntextdraw_snapshot_fields=0x000\ntextdraw_snapshot_result=0x000\nvehicle_phase=none\n"
        );
        #[cfg(feature = "r1-probe")]
        assert_eq!(
            record,
            concat!(
                "status=0x00000011\nfailure=0\ntext_label_phase=none\ntext_label_initial_fields=0x00\ntext_label_initial_result=0x000\ntextdraw_phase=none\ntextdraw_snapshot_fields=0x000\ntextdraw_snapshot_result=0x000\nvehicle_phase=none\n",
                "codec_round_trip=false\nincoming_packet_bits=0\nincoming_rpc_bits=0\n"
            )
        );
    }

    #[test]
    fn parses_the_r5_entry_point_from_a_bounded_pe_header() {
        let mut image = vec![0_u8; 0x200];
        image[..2].copy_from_slice(b"MZ");
        image[0x3C..0x40].copy_from_slice(&(0x80_u32).to_le_bytes());
        image[0x80..0x84].copy_from_slice(b"PE\0\0");
        image[0x98..0x9A].copy_from_slice(&0x10B_u16.to_le_bytes());
        image[0xA8..0xAC].copy_from_slice(&PROFILE_ENTRY_POINT_RVA.to_le_bytes());

        assert_eq!(
            parse_pe_entry_point_rva(|offset, destination| {
                let Some(source) = image.get(offset..offset.saturating_add(destination.len()))
                else {
                    return false;
                };
                destination.copy_from_slice(source);
                true
            }),
            Some(PROFILE_ENTRY_POINT_RVA)
        );
    }

    #[test]
    fn valid_unspawned_local_player_remains_retryable() {
        let player = samp_client_sdk::LocalPlayer {
            id: 0,
            nickname: b"R5 probe".to_vec(),
            colour: 0,
            spawned: false,
            health: 100.0,
            armour: 0.0,
            position: samp_client_sdk::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            velocity: samp_client_sdk::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            special_action: 0,
            animation_id: 0,
            vehicle_id: None,
            score: 0,
            ping: 0,
        };

        assert!(local_player_snapshot_is_valid(&player));
        assert!(!player.spawned);
    }

    #[test]
    fn parses_the_server_owned_entity_id_marker() {
        let marker = [ENTITY_IDS_PREFIX, b"17,18,19,20"].concat();
        let ids = parse_entity_ids(&marker).unwrap();
        assert_eq!(ids.object, 17);
        assert_eq!(ids.vehicle, 18);
        assert_eq!(ids.pickup, 19);
        assert_eq!(ids.gangzone, 20);
        assert!(parse_entity_ids(&[ENTITY_IDS_PREFIX, b"17,18,19"].concat()).is_none());
        assert!(parse_entity_ids(&[ENTITY_IDS_PREFIX, b"17,18,19,20,21"].concat()).is_none());
    }

    #[test]
    fn parses_bounded_vehicle_phase_fields() {
        assert_eq!(
            parse_u16_fields(
                b"R5_SDK_REMOTE_TRAILER_READY_7,18,19",
                b"R5_SDK_REMOTE_TRAILER_READY_"
            ),
            Some(vec![7, 18, 19])
        );
        assert!(
            parse_u16_fields(
                b"R5_SDK_REMOTE_TRAILER_READY_7,nope,19",
                b"R5_SDK_REMOTE_TRAILER_READY_"
            )
            .is_none()
        );
    }

    #[test]
    fn retries_transient_not_ready_values() {
        let mut attempts = 0;
        let value = wait_for_value(Duration::from_millis(500), || {
            attempts += 1;
            if attempts < 3 {
                Err(SampClientSdkResult::NotReady)
            } else {
                Ok(37)
            }
        })
        .expect("a published cache value should be returned");

        assert_eq!(value, 37);
        assert_eq!(attempts, 3);
    }

    #[test]
    fn success_masks_cover_contiguous_status_bits() {
        assert_eq!(MAIN_SUCCESS_STATUS, (1_u32 << 28) - 1);
        assert_eq!(FULL_SUCCESS_STATUS, (1_u32 << 30) - 1);
    }
}
