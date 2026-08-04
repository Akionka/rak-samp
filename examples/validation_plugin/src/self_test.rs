use crate::{
    logging,
    state::{
        HOST_WAIT_TIMEOUT, ID_STATS_UPDATE, RPC_UPDATE_SCORES_AND_PINGS, SELF_TESTS, STATS_PAYLOAD,
        STATS_PAYLOAD_LEN, STATS_PAYLOAD_READY, STOP, SelfTestStatus, self_test_finished,
        self_test_label,
    },
};
use rak_samp_plugin_api::{
    HostApi, LocalChatDisplayMode, LocalChatMessage, LocalChatMessageStyle, LocalCursorMode,
    LocalDeathMessage, LocalDialog, LocalDialogState, LocalDialogStyle, LocalPlayer,
    MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABELS,
    MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES, RakSampHookAction, RakSampResult, RakSampSendOptions,
    RemotePlayerState,
    events::{EncodedPayload, Event, EventError, rpc::incoming},
};
use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{Duration, Instant},
};

const SEND_TEST_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const DIRECT_SNAPSHOT_STATE_TIMEOUT: Duration = Duration::from_secs(120);
const DIRECT_DIALOG_ID: u16 = 0x7FFC;
const DIRECT_DIALOG_TITLE: &[u8] = b"rak-samp validation";
pub(crate) const SEND_TEST_MARKER: &str = "rak-samp-validation-send.enabled";
pub(crate) const DIRECT_CLIENT_TEST_MARKER: &str = "rak-samp-validation-direct-client.enabled";
pub(crate) const PLAYER_DIRECTORY_TEST_MARKER: &str =
    "rak-samp-validation-player-directory.enabled";
pub(crate) const REMOTE_PLAYER_STATE_TEST_MARKER: &str =
    "rak-samp-validation-remote-player-state.enabled";
pub(crate) const VEHICLE_EXISTS_TEST_MARKER: &str = "rak-samp-validation-vehicle-exists.enabled";
pub(crate) const TEXT_LABEL_EXISTS_TEST_MARKER: &str =
    "rak-samp-validation-text-label-exists.enabled";
pub(crate) const TEXT_LABEL_TEST_MARKER: &str = "rak-samp-validation-text-label.enabled";
pub(crate) const TEXTDRAW_EXISTS_TEST_MARKER: &str = "rak-samp-validation-textdraw-exists.enabled";
pub(crate) const TEXTDRAW_TEST_MARKER: &str = "rak-samp-validation-textdraw.enabled";
pub(crate) const OBJECT_EXISTS_TEST_MARKER: &str = "rak-samp-validation-object-exists.enabled";
pub(crate) const GANGZONE_TEST_MARKER: &str = "rak-samp-validation-gangzone.enabled";
pub(crate) const SHUTDOWN_TEST_MARKER: &str = "rak-samp-validation-shutdown.enabled";
pub(crate) const TEST_PACKET_ID: u8 = 254;
pub(crate) const TEST_RPC_ID: u8 = 255;
pub(crate) const TEST_PACKET_INPUT: [u8; 18] = *b"rak-samp-packet-in";
pub(crate) const TEST_PACKET_REPLACEMENT: [u8; 18] = *b"rak-samp-packet-ok";
pub(crate) const TEST_RPC_INPUT: [u8; 18] = *b"rak-samp-rpc-input";
pub(crate) const TEST_RPC_REPLACEMENT: [u8; 18] = *b"rak-samp-rpc-pass!";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DirectSnapshotChanges {
    position: bool,
    health: bool,
    armour: bool,
    vehicle: bool,
}

impl DirectSnapshotChanges {
    fn observe(&mut self, baseline: &LocalPlayer, snapshot: &LocalPlayer) {
        self.position |= snapshot.position != baseline.position;
        self.health |= snapshot.health != baseline.health;
        self.armour |= snapshot.armour != baseline.armour;
        self.vehicle |= snapshot.vehicle_id != baseline.vehicle_id;
    }

    fn complete(self) -> bool {
        self.position && self.health && self.armour && self.vehicle
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RemotePlayerStateChanges {
    health_or_armour: bool,
    special_action: bool,
    animation: bool,
}

impl RemotePlayerStateChanges {
    fn observe(&mut self, baseline: RemotePlayerState, snapshot: RemotePlayerState) {
        self.health_or_armour |=
            snapshot.health != baseline.health || snapshot.armour != baseline.armour;
        self.special_action |= snapshot.special_action != baseline.special_action;
        self.animation |= snapshot.animation_id != baseline.animation_id;
    }

    fn complete(self) -> bool {
        self.health_or_armour && self.special_action && self.animation
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DirectChatDisplayModes {
    off: bool,
    no_shadow: bool,
    normal: bool,
}

impl DirectChatDisplayModes {
    fn observe(&mut self, mode: LocalChatDisplayMode) {
        match mode {
            LocalChatDisplayMode::Off => self.off = true,
            LocalChatDisplayMode::NoShadow => self.no_shadow = true,
            LocalChatDisplayMode::Normal => self.normal = true,
        }
    }

    fn complete(self) -> bool {
        self.off && self.no_shadow && self.normal
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DirectCursorStates {
    none: bool,
    active: bool,
}

impl DirectCursorStates {
    fn observe(&mut self, mode: LocalCursorMode) {
        if mode == LocalCursorMode::None {
            self.none = true;
        } else {
            self.active = true;
        }
    }

    fn complete(self) -> bool {
        self.none && self.active
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DirectScoreboardStates {
    closed: bool,
    open: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DirectVisibilityStates {
    inactive: bool,
    active: bool,
}

impl DirectVisibilityStates {
    fn observe(&mut self, active: bool) {
        if active {
            self.active = true;
        } else {
            self.inactive = true;
        }
    }

    fn complete(self) -> bool {
        self.inactive && self.active
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DirectUiStates {
    chat_modes: DirectChatDisplayModes,
    cursor: DirectCursorStates,
    scoreboard: DirectScoreboardStates,
    dialog: DirectVisibilityStates,
    active_dialog_core: DirectVisibilityStates,
    chat_input: DirectVisibilityStates,
}

impl DirectUiStates {
    fn complete(self) -> bool {
        self.chat_modes.complete()
            && self.cursor.complete()
            && self.scoreboard.complete()
            && self.dialog.complete()
            && self.active_dialog_core.complete()
            && self.chat_input.complete()
    }
}

impl DirectScoreboardStates {
    fn observe(&mut self, open: bool) {
        if open {
            self.open = true;
        } else {
            self.closed = true;
        }
    }

    fn complete(self) -> bool {
        self.closed && self.open
    }
}

fn direct_validation_dialog() -> LocalDialog<'static> {
    LocalDialog {
        id: DIRECT_DIALOG_ID,
        style: LocalDialogStyle::MessageBox,
        title: DIRECT_DIALOG_TITLE,
        text: b"This is a direct local dialog validation request.",
        button1: b"Close",
        button2: b"",
    }
}

fn is_direct_validation_dialog(dialog: &LocalDialogState) -> bool {
    dialog.id == i32::from(DIRECT_DIALOG_ID)
        && dialog.style == LocalDialogStyle::MessageBox
        && dialog.title == DIRECT_DIALOG_TITLE
        && !dialog.server_side
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectDialogAction {
    Matched,
    Wait,
    Interference,
    Requeue,
}

#[derive(Default)]
struct DirectDialogWaitState {
    retry_after_interference: bool,
}

impl DirectDialogWaitState {
    fn observe(&mut self, dialog: Option<&LocalDialogState>) -> DirectDialogAction {
        match dialog {
            Some(dialog) if is_direct_validation_dialog(dialog) => DirectDialogAction::Matched,
            Some(_) => {
                self.retry_after_interference = true;
                DirectDialogAction::Interference
            }
            None if std::mem::take(&mut self.retry_after_interference) => {
                DirectDialogAction::Requeue
            }
            None => DirectDialogAction::Wait,
        }
    }
}

pub(crate) fn rewrite_test_packet(event: &mut Event<'_>) -> RakSampHookAction {
    rewrite_test_event(
        event,
        TEST_PACKET_ID,
        &TEST_PACKET_INPUT,
        &TEST_PACKET_REPLACEMENT,
        &SELF_TESTS.packet,
    );
    RakSampHookAction::Continue
}

pub(crate) fn rewrite_test_rpc(event: &mut Event<'_>) -> RakSampHookAction {
    rewrite_test_event(
        event,
        TEST_RPC_ID,
        &TEST_RPC_INPUT,
        &TEST_RPC_REPLACEMENT,
        &SELF_TESTS.rpc,
    );
    RakSampHookAction::Continue
}

fn rewrite_test_event<const N: usize>(
    event: &mut Event<'_>,
    expected_id: u8,
    input: &[u8; N],
    replacement: &[u8; N],
    status: &AtomicU8,
) {
    if event.id() != expected_id {
        return;
    }
    if !event_matches(event, input) {
        return;
    }
    status.store(
        if event.replace_bytes(replacement).is_ok() {
            SelfTestStatus::Rewritten.as_raw()
        } else {
            SelfTestStatus::Failed.as_raw()
        },
        Ordering::Release,
    );
}

pub(crate) fn test_verdict<const N: usize>(
    event: &mut Event<'_>,
    id: u8,
    expected_id: u8,
    input: &[u8; N],
    replacement: &[u8; N],
    status: &AtomicU8,
) -> RakSampHookAction {
    if id != expected_id {
        return RakSampHookAction::Continue;
    }
    if event_matches(event, replacement) {
        status.store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
        return RakSampHookAction::Block;
    }
    if event_matches(event, input) {
        status.store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
        return RakSampHookAction::Block;
    }
    RakSampHookAction::Continue
}

fn event_matches<const N: usize>(event: &mut Event<'_>, expected: &[u8; N]) -> bool {
    read_exact_event(event).as_ref() == Some(expected)
}

fn read_exact_event<const N: usize>(event: &mut Event<'_>) -> Option<[u8; N]> {
    let mut actual = [0; N];
    let read = event.reset_read().is_ok()
        && event
            .read_bytes(N)
            .map(|bytes| {
                actual.copy_from_slice(&bytes);
            })
            .is_ok()
        && matches!(
            event.read_u8(),
            Err(EventError::Host(RakSampResult::ReadOutOfBounds))
        );
    let restored = event.reset_read().is_ok();
    (read && restored).then_some(actual)
}

pub(crate) fn capture_stats_payload(event: &mut Event<'_>) {
    if STATS_PAYLOAD_READY.load(Ordering::Acquire) {
        return;
    }
    let Some(payload) = read_exact_event::<STATS_PAYLOAD_LEN>(event) else {
        return;
    };
    for (destination, source) in STATS_PAYLOAD.iter().zip(payload) {
        destination.store(source, Ordering::Relaxed);
    }
    STATS_PAYLOAD_READY.store(true, Ordering::Release);
}

pub(crate) fn test_dialog_input() -> incoming::ShowDialog {
    incoming::ShowDialog {
        dialog_id: 0x7FFE,
        style: 2,
        title: b"rak-samp input".to_vec(),
        button1: b"accept".to_vec(),
        button2: b"cancel".to_vec(),
        text: b"native encoded dialog input".to_vec(),
    }
}

pub(crate) fn test_dialog_replacement() -> incoming::ShowDialog {
    incoming::ShowDialog {
        dialog_id: 0x7FFD,
        style: 5,
        title: b"rak-samp replacement".to_vec(),
        button1: b"yes".to_vec(),
        button2: b"no".to_vec(),
        text: b"native encoded dialog replacement".to_vec(),
    }
}

pub(crate) fn run(api: HostApi) {
    let rpc_result = emulate_when_ready(|| {
        api.emulate_incoming_rpc(TEST_RPC_ID, &TEST_RPC_INPUT, TEST_RPC_INPUT.len() * 8)
    });
    record_emulation_result("RPC", rpc_result, &SELF_TESTS.rpc);

    match encode_dialog_when_ready(api) {
        Ok(payload) => {
            let dialog_result = emulate_when_ready(|| {
                api.emulate_incoming_rpc(
                    incoming::SHOW_DIALOG.id(),
                    payload.as_bytes(),
                    payload.len_bits(),
                )
            });
            record_emulation_result("dialog RPC", dialog_result, &SELF_TESTS.dialog);
        }
        Err(error) => {
            SELF_TESTS
                .dialog
                .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
            logging::write(&format!("dialog self-test encode failed: {error}"));
        }
    }

    let packet_result = emulate_when_ready(|| {
        api.emulate_incoming_packet(
            TEST_PACKET_ID,
            &TEST_PACKET_INPUT,
            TEST_PACKET_INPUT.len() * 8,
        )
    });
    record_emulation_result("packet", packet_result, &SELF_TESTS.packet);

    let deadline = Instant::now() + Duration::from_secs(10);
    while !STOP.load(Ordering::Acquire)
        && Instant::now() < deadline
        && (!self_test_finished(SELF_TESTS.packet.load(Ordering::Acquire))
            || !self_test_finished(SELF_TESTS.rpc.load(Ordering::Acquire))
            || !self_test_finished(SELF_TESTS.dialog.load(Ordering::Acquire)))
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    mark_timeout(&SELF_TESTS.packet);
    mark_timeout(&SELF_TESTS.rpc);
    mark_timeout(&SELF_TESTS.dialog);
    logging::write(&format!(
        "self-test completed: packet={} RPC={} dialog={}",
        self_test_label(SELF_TESTS.packet.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.rpc.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.dialog.load(Ordering::Acquire)),
    ));
    run_direct_client(api);
    run_player_directory(api);
    run_remote_player_state(api);
    run_vehicle_exists(api);
    run_text_label_exists(api);
    run_text_label(api);
    run_textdraw_exists(api);
    run_textdraw(api);
    run_object_exists(api);
    run_gangzone(api);
    run_send(api);
    schedule_shutdown();
}

fn run_player_directory(api: HostApi) {
    if !logging::plugin_path(PLAYER_DIRECTORY_TEST_MARKER).is_file() {
        SELF_TESTS
            .player_directory
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "player-directory self-test disabled; opt in with rak-samp-validation-player-directory.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.player_info(id) {
            Ok(Some(player)) if !player.is_local && !player.nickname.is_empty() => {
                let state = match api.remote_player_state(id) {
                    Ok(Some(state)) => state,
                    Ok(None) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {
                        id = id.wrapping_add(1) % MAX_SAMP_PLAYERS;
                        std::thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .player_directory
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "player-directory remote-state returned {error:?}: player_id={id}"
                        ));
                        return;
                    }
                };
                if api.is_player_connected(id) == Ok(true)
                    && api.is_player_defined(id) == Ok(true)
                    && api.is_player_paused(id).is_ok()
                    && api.player_nickname(id) == Ok(Some(player.nickname))
                    && api.is_player_npc(id) == Ok(Some(player.is_npc))
                    && api.player_colour(id) == Ok(Some(player.colour))
                    && api.player_score(id) == Ok(Some(player.score))
                    && api.player_ping(id) == Ok(Some(player.ping))
                    && api.player_health(id) == Ok(Some(state.health))
                    && api.player_armour(id) == Ok(Some(state.armour))
                    && api.player_special_action(id) == Ok(Some(state.special_action))
                    && api.player_animation_id(id) == Ok(Some(state.animation_id))
                {
                    SELF_TESTS
                        .player_directory
                        .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                    logging::write(&format!(
                        "player-directory self-test passed: player_id={id}"
                    ));
                    return;
                }
                SELF_TESTS
                    .player_directory
                    .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "player-directory self-test failed: player_id={id}"
                ));
                return;
            }
            Ok(Some(_)) | Ok(None) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .player_directory
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "player-directory self-test returned {error:?}: player_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_PLAYERS;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .player_directory
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("player-directory self-test timed out without a remote player");
}

fn run_remote_player_state(api: HostApi) {
    if !logging::plugin_path(REMOTE_PLAYER_STATE_TEST_MARKER).is_file() {
        SELF_TESTS
            .remote_player_state
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "remote-player-state self-test disabled; opt in with rak-samp-validation-remote-player-state.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    let baseline = loop {
        if STOP.load(Ordering::Acquire) || Instant::now() >= deadline {
            SELF_TESTS
                .remote_player_state
                .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
            logging::write("remote-player-state self-test timed out without a remote player");
            return;
        }
        match api.remote_player_state(id) {
            Ok(Some(state)) => break (id, state),
            Ok(None) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .remote_player_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "remote-player-state self-test returned {error:?}: player_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_PLAYERS;
        std::thread::sleep(Duration::from_millis(20));
    };

    let (id, baseline) = baseline;
    let mut changes = RemotePlayerStateChanges::default();
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.remote_player_state(id) {
            Ok(Some(state)) => {
                changes.observe(baseline, state);
                if changes.complete() {
                    SELF_TESTS
                        .remote_player_state
                        .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                    logging::write(&format!(
                        "remote-player-state self-test passed: player_id={id}"
                    ));
                    return;
                }
            }
            Ok(None) => {
                SELF_TESTS
                    .remote_player_state
                    .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "remote-player-state self-test disconnected before transitions: player_id={id}"
                ));
                return;
            }
            Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .remote_player_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "remote-player-state self-test returned {error:?}: player_id={id}"
                ));
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .remote_player_state
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write(&format!(
        "remote-player-state self-test timed out before all transitions: player_id={id}"
    ));
}

fn run_vehicle_exists(api: HostApi) {
    if !logging::plugin_path(VEHICLE_EXISTS_TEST_MARKER).is_file() {
        SELF_TESTS
            .vehicle_exists
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "vehicle-exists self-test disabled; opt in with rak-samp-validation-vehicle-exists.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.is_vehicle_defined(id) {
            Ok(true) => {
                SELF_TESTS
                    .vehicle_exists
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!("vehicle-exists self-test passed: vehicle_id={id}"));
                return;
            }
            Ok(false) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .vehicle_exists
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "vehicle-exists self-test returned {error:?}: vehicle_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_VEHICLES;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .vehicle_exists
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("vehicle-exists self-test timed out without a defined vehicle");
}

fn run_text_label_exists(api: HostApi) {
    if !logging::plugin_path(TEXT_LABEL_EXISTS_TEST_MARKER).is_file() {
        SELF_TESTS
            .text_label_exists
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "text-label-exists self-test disabled; opt in with rak-samp-validation-text-label-exists.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.is_text_label_defined(id) {
            Ok(true) => {
                SELF_TESTS
                    .text_label_exists
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "text-label-exists self-test passed: label_id={id}"
                ));
                return;
            }
            Ok(false) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .text_label_exists
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "text-label-exists self-test returned {error:?}: label_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_TEXT_LABELS;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .text_label_exists
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("text-label-exists self-test timed out without a defined label");
}

fn run_text_label(api: HostApi) {
    if !logging::plugin_path(TEXT_LABEL_TEST_MARKER).is_file() {
        SELF_TESTS
            .text_label
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "text-label self-test disabled; opt in with rak-samp-validation-text-label.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.text_label(id) {
            Ok(Some(label)) if label.id == id && api.is_text_label_defined(id) == Ok(true) => {
                SELF_TESTS
                    .text_label
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!("text-label self-test passed: label_id={id}"));
                return;
            }
            Ok(Some(_)) => {
                SELF_TESTS
                    .text_label
                    .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
                logging::write(&format!("text-label self-test failed: label_id={id}"));
                return;
            }
            Ok(None) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .text_label
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "text-label self-test returned {error:?}: label_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_TEXT_LABELS;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .text_label
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("text-label self-test timed out without a copied label");
}

fn run_textdraw_exists(api: HostApi) {
    if !logging::plugin_path(TEXTDRAW_EXISTS_TEST_MARKER).is_file() {
        SELF_TESTS
            .textdraw_exists
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "textdraw-exists self-test disabled; opt in with rak-samp-validation-textdraw-exists.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut pool_index = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.is_textdraw_defined(pool_index) {
            Ok(true) => {
                SELF_TESTS
                    .textdraw_exists
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "textdraw-exists self-test passed: pool_index={pool_index}"
                ));
                return;
            }
            Ok(false) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .textdraw_exists
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "textdraw-exists self-test returned {error:?}: pool_index={pool_index}"
                ));
                return;
            }
        }
        pool_index = (pool_index + 1) % MAX_SAMP_TEXTDRAWS;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .textdraw_exists
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("textdraw-exists self-test timed out without a defined textdraw");
}

fn run_textdraw(api: HostApi) {
    if !logging::plugin_path(TEXTDRAW_TEST_MARKER).is_file() {
        SELF_TESTS
            .textdraw
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "textdraw self-test disabled; opt in with rak-samp-validation-textdraw.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut pool_index = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.textdraw(pool_index) {
            Ok(Some(draw))
                if draw.pool_index == pool_index
                    && api.is_textdraw_defined(pool_index) == Ok(true) =>
            {
                SELF_TESTS
                    .textdraw
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "textdraw self-test passed: pool_index={pool_index}"
                ));
                return;
            }
            Ok(Some(_)) => {
                SELF_TESTS
                    .textdraw
                    .store(SelfTestStatus::Failed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "textdraw self-test failed: pool_index={pool_index}"
                ));
                return;
            }
            Ok(None) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .textdraw
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "textdraw self-test returned {error:?}: pool_index={pool_index}"
                ));
                return;
            }
        }
        pool_index = (pool_index + 1) % MAX_SAMP_TEXTDRAWS;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .textdraw
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("textdraw self-test timed out without a copied numeric snapshot");
}

fn run_object_exists(api: HostApi) {
    if !logging::plugin_path(OBJECT_EXISTS_TEST_MARKER).is_file() {
        SELF_TESTS
            .object_exists
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "object-exists self-test disabled; opt in with rak-samp-validation-object-exists.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.is_object_defined(id) {
            Ok(true) => {
                SELF_TESTS
                    .object_exists
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!("object-exists self-test passed: object_id={id}"));
                return;
            }
            Ok(false) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .object_exists
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "object-exists self-test returned {error:?}: object_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_OBJECTS;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .object_exists
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("object-exists self-test timed out without a defined object");
}

fn run_gangzone(api: HostApi) {
    if !logging::plugin_path(GANGZONE_TEST_MARKER).is_file() {
        SELF_TESTS
            .gangzone
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "gangzone self-test disabled; opt in with rak-samp-validation-gangzone.enabled",
        );
        return;
    }

    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut id = 0_u16;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.gangzone(id) {
            Ok(Some(_)) => {
                SELF_TESTS
                    .gangzone
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!("gangzone self-test passed: gangzone_id={id}"));
                return;
            }
            Ok(None) | Err(RakSampResult::NotReady | RakSampResult::QueueFull) => {}
            Err(error) => {
                SELF_TESTS
                    .gangzone
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "gangzone self-test returned {error:?}: gangzone_id={id}"
                ));
                return;
            }
        }
        id = (id + 1) % MAX_SAMP_GANGZONES;
        std::thread::sleep(Duration::from_millis(20));
    }
    SELF_TESTS
        .gangzone
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write("gangzone self-test timed out without a defined gangzone");
}

fn run_direct_client(api: HostApi) {
    if !logging::plugin_path(DIRECT_CLIENT_TEST_MARKER).is_file() {
        SELF_TESTS
            .direct_client
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        SELF_TESTS
            .direct_snapshot_state
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write(
            "direct-client self-test disabled; opt in with rak-samp-validation-direct-client.enabled",
        );
        return;
    }

    logging::write(
        "direct-client self-test waiting for a spawned local player and idle dialog state",
    );
    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.local_player() {
            Ok(snapshot) if snapshot.spawned && !snapshot.nickname.is_empty() => {
                match api.samp_game_state() {
                    Ok(_) => {}
                    Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test game-state returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.server_info() {
                    Ok(info) if !info.address.is_empty() && info.port != 0 => {}
                    Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Ok(_) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write("direct-client self-test server-info snapshot was empty");
                        return;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test server-info returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.local_chat_display_mode() {
                    Ok(_) => {}
                    Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test chat-display-mode returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.local_cursor_mode() {
                    Ok(_) => {}
                    Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test cursor-mode returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.is_local_scoreboard_open() {
                    Ok(_) => {}
                    Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test scoreboard-state returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.is_local_dialog_active() {
                    Ok(false) => {}
                    Ok(true) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test dialog-state returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.active_local_dialog() {
                    Ok(None) => {}
                    Ok(Some(_)) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test active-dialog snapshot returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.is_local_chat_input_active() {
                    Ok(false) => {}
                    Ok(true) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test chat-input-state returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.local_animation(0) {
                    Ok(animation)
                        if animation.name == b"AIRPORT"
                            && animation.file == b"THRW_BARL_THRW"
                            && api.local_animation_id(&animation.name, &animation.file)
                                == Ok(Some(0)) => {}
                    Ok(_) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test animation-table returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.player_info(snapshot.id) {
                    Ok(Some(player))
                        if player.is_local
                            && !player.is_npc
                            && player.nickname == snapshot.nickname
                            && player.colour == snapshot.colour
                            && player.score == snapshot.score
                            && player.ping == snapshot.ping => {}
                    Ok(_) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test player-directory returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.player_count(true) {
                    Ok(count) if count > 0 => {}
                    Ok(_) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test player-count returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.player_max_id() {
                    Ok(max_id) if max_id >= snapshot.id => {}
                    Ok(_) | Err(RakSampResult::NotReady) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => {
                        SELF_TESTS
                            .direct_client
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        SELF_TESTS
                            .direct_snapshot_state
                            .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                        logging::write(&format!(
                            "direct-client self-test player-max-id returned {error:?}"
                        ));
                        return;
                    }
                }

                let dialog_result = api.show_local_dialog(direct_validation_dialog());
                if dialog_result != RakSampResult::Ok {
                    SELF_TESTS
                        .direct_client
                        .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                    SELF_TESTS
                        .direct_snapshot_state
                        .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                    logging::write(&format!(
                        "direct-client self-test dialog request returned {dialog_result:?}"
                    ));
                    return;
                }

                let chat_result = api.show_local_chat_message(LocalChatMessage {
                    style: LocalChatMessageStyle::Debug,
                    text: b"Direct local chat validation request.",
                    prefix: b"[rak-samp]",
                    text_colour: 0xFF_A9_C4_E4,
                    prefix_colour: u32::MAX,
                });
                if chat_result != RakSampResult::Ok {
                    SELF_TESTS
                        .direct_client
                        .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                    SELF_TESTS
                        .direct_snapshot_state
                        .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                    logging::write(&format!(
                        "direct-client self-test chat request returned {chat_result:?}"
                    ));
                    return;
                }

                let death_result = api.show_local_death_message(LocalDeathMessage {
                    killer: b"killer",
                    victim: b"victim",
                    killer_colour: 0xFFFF_0000,
                    victim_colour: 0xFF00_FF00,
                    weapon: 24,
                });
                if death_result != RakSampResult::Ok {
                    SELF_TESTS
                        .direct_client
                        .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                    SELF_TESTS
                        .direct_snapshot_state
                        .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                    logging::write(&format!(
                        "direct-client self-test death-window request returned {death_result:?}"
                    ));
                    return;
                }

                logging::write(&format!(
                    "direct-client self-test queued after spawned preflight: local_player_id={}",
                    snapshot.id
                ));
                let dialog_deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
                let mut dialog_wait = DirectDialogWaitState::default();
                let mut interference_logged = false;
                let dialog_ready = loop {
                    if STOP.load(Ordering::Acquire) || Instant::now() >= dialog_deadline {
                        break false;
                    }
                    match api.active_local_dialog() {
                        Ok(dialog) => match dialog_wait.observe(dialog.as_ref()) {
                            DirectDialogAction::Matched => break true,
                            DirectDialogAction::Wait => {}
                            DirectDialogAction::Interference => {
                                if !interference_logged {
                                    interference_logged = true;
                                    logging::write(
                                        "direct-client self-test observed non-validation dialog interference; waiting to retry the local dialog",
                                    );
                                }
                            }
                            DirectDialogAction::Requeue => {
                                let retry_result =
                                    api.show_local_dialog(direct_validation_dialog());
                                if retry_result != RakSampResult::Ok {
                                    SELF_TESTS.direct_client.store(
                                        SelfTestStatus::CallFailed.as_raw(),
                                        Ordering::Release,
                                    );
                                    SELF_TESTS.direct_snapshot_state.store(
                                        SelfTestStatus::CallFailed.as_raw(),
                                        Ordering::Release,
                                    );
                                    logging::write(&format!(
                                        "direct-client self-test dialog retry returned {retry_result:?}"
                                    ));
                                    return;
                                }
                                logging::write(
                                    "direct-client self-test requeued the local dialog after interference",
                                );
                            }
                        },
                        Err(RakSampResult::NotReady) => {}
                        Err(error) => {
                            SELF_TESTS
                                .direct_client
                                .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                            SELF_TESTS
                                .direct_snapshot_state
                                .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                            logging::write(&format!(
                                "direct-client self-test active-dialog snapshot returned {error:?}"
                            ));
                            return;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(10));
                };
                if !dialog_ready {
                    SELF_TESTS
                        .direct_client
                        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
                    SELF_TESTS
                        .direct_snapshot_state
                        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
                    logging::write(
                        "direct-client self-test timed out before its local dialog became active",
                    );
                    return;
                }

                SELF_TESTS
                    .direct_client
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client self-test passed: dialog=Ok active_dialog=Ok chat=Ok death_window=Ok game_state=Ok server_info=Ok chat_display_mode=Ok cursor_mode=Ok scoreboard_state=Ok dialog_state=Ok chat_input_state=Ok animation_table=Ok player_directory=Ok player_count=Ok player_max_id=Ok local_player_id={}",
                    snapshot.id
                ));
                run_direct_snapshot_state(api, snapshot.id);
                return;
            }
            Ok(_) | Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_client
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client self-test snapshot returned {error:?}"
                ));
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    SELF_TESTS
        .direct_client
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    SELF_TESTS
        .direct_snapshot_state
        .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
    logging::write(
        "direct-client self-test timed out before a spawned snapshot and idle dialog state",
    );
}

fn run_direct_snapshot_state(api: HostApi, expected_id: u16) {
    let spawn_deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let baseline = loop {
        if STOP.load(Ordering::Acquire) || Instant::now() >= spawn_deadline {
            SELF_TESTS
                .direct_snapshot_state
                .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
            logging::write(&format!(
                "direct-client state validation timed out before a baseline snapshot: local_player_id={expected_id}"
            ));
            return;
        }
        match api.local_player() {
            Ok(snapshot) if snapshot.id == expected_id => break snapshot,
            Ok(_) | Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation snapshot returned {error:?}"
                ));
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    };

    logging::write(&format!(
        "direct-client state validation observing local_player_id={expected_id}"
    ));
    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    let mut changes = DirectSnapshotChanges::default();
    let mut ui_states = DirectUiStates::default();
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.local_player() {
            Ok(snapshot) if snapshot.id == expected_id => {
                changes.observe(&baseline, &snapshot);
            }
            Ok(_) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation changed local-player identity: local_player_id={expected_id}"
                ));
                return;
            }
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation snapshot returned {error:?}"
                ));
                return;
            }
        }
        match api.local_chat_display_mode() {
            Ok(mode) => ui_states.chat_modes.observe(mode),
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation chat-display-mode returned {error:?}"
                ));
                return;
            }
        }
        match api.local_cursor_mode() {
            Ok(mode) => ui_states.cursor.observe(mode),
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation cursor-mode returned {error:?}"
                ));
                return;
            }
        }
        match api.is_local_scoreboard_open() {
            Ok(open) => ui_states.scoreboard.observe(open),
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation scoreboard-state returned {error:?}"
                ));
                return;
            }
        }
        match api.is_local_dialog_active() {
            Ok(active) => ui_states.dialog.observe(active),
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation dialog-state returned {error:?}"
                ));
                return;
            }
        }
        match api.active_local_dialog() {
            Ok(dialog) => ui_states.active_dialog_core.observe(dialog.is_some()),
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation active-dialog snapshot returned {error:?}"
                ));
                return;
            }
        }
        match api.is_local_chat_input_active() {
            Ok(active) => ui_states.chat_input.observe(active),
            Err(RakSampResult::NotReady) => {}
            Err(error) => {
                SELF_TESTS
                    .direct_snapshot_state
                    .store(SelfTestStatus::CallFailed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client state validation chat-input-state returned {error:?}"
                ));
                return;
            }
        }
        if changes.complete() && ui_states.complete() {
            SELF_TESTS
                .direct_snapshot_state
                .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
            log_direct_snapshot_state("passed", expected_id, changes, ui_states);
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    if !STOP.load(Ordering::Acquire) {
        SELF_TESTS
            .direct_snapshot_state
            .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
        log_direct_snapshot_state("timed-out", expected_id, changes, ui_states);
    }
}

fn log_direct_snapshot_state(
    outcome: &str,
    id: u16,
    changes: DirectSnapshotChanges,
    ui_states: DirectUiStates,
) {
    logging::write(&format!(
        "direct-client state validation {outcome}: local_player_id={id} position_changed={} health_changed={} armour_changed={} vehicle_changed={} chat_mode_off={} chat_mode_no_shadow={} chat_mode_normal={} cursor_none={} cursor_active={} scoreboard_closed={} scoreboard_open={} dialog_inactive={} dialog_active={} active_dialog_snapshot_inactive={} active_dialog_snapshot_active={} chat_input_inactive={} chat_input_active={}",
        changes.position,
        changes.health,
        changes.armour,
        changes.vehicle,
        ui_states.chat_modes.off,
        ui_states.chat_modes.no_shadow,
        ui_states.chat_modes.normal,
        ui_states.cursor.none,
        ui_states.cursor.active,
        ui_states.scoreboard.closed,
        ui_states.scoreboard.open,
        ui_states.dialog.inactive,
        ui_states.dialog.active,
        ui_states.active_dialog_core.inactive,
        ui_states.active_dialog_core.active,
        ui_states.chat_input.inactive,
        ui_states.chat_input.active,
    ));
}

fn run_send(api: HostApi) {
    if !logging::plugin_path(SEND_TEST_MARKER).is_file() {
        SELF_TESTS
            .send_packet
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        SELF_TESTS
            .send_rpc
            .store(SelfTestStatus::Disabled.as_raw(), Ordering::Release);
        logging::write("send self-test disabled; opt in with rak-samp-validation-send.enabled");
        return;
    }
    logging::write("send self-test enabled; waiting for an outgoing ID_STATS_UPDATE payload");
    let deadline = Instant::now() + SEND_TEST_WAIT_TIMEOUT;
    while !STOP.load(Ordering::Acquire)
        && !STATS_PAYLOAD_READY.load(Ordering::Acquire)
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    if !STATS_PAYLOAD_READY.load(Ordering::Acquire) {
        SELF_TESTS
            .send_packet
            .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
        SELF_TESTS
            .send_rpc
            .store(SelfTestStatus::TimedOut.as_raw(), Ordering::Release);
        logging::write("send self-test timed out before an ID_STATS_UPDATE payload was captured");
        return;
    }

    let mut payload = [0; STATS_PAYLOAD_LEN];
    for (destination, source) in payload.iter_mut().zip(&STATS_PAYLOAD) {
        *destination = source.load(Ordering::Relaxed);
    }
    let packet_options = RakSampSendOptions {
        reliability: 6,
        ..RakSampSendOptions::default()
    };
    let packet_result = api.send_packet(
        ID_STATS_UPDATE,
        &payload,
        payload.len() * u8::BITS as usize,
        packet_options,
    );
    record_send_result("packet", packet_result, &SELF_TESTS.send_packet);

    let rpc_result = api.send_rpc(
        RPC_UPDATE_SCORES_AND_PINGS,
        &[],
        0,
        RakSampSendOptions::default(),
    );
    record_send_result("RPC", rpc_result, &SELF_TESTS.send_rpc);
    logging::write(&format!(
        "send self-test completed: packet={} RPC={}",
        self_test_label(SELF_TESTS.send_packet.load(Ordering::Acquire)),
        self_test_label(SELF_TESTS.send_rpc.load(Ordering::Acquire)),
    ));
}

fn record_send_result(label: &str, result: RakSampResult, status: &AtomicU8) {
    logging::write(&format!("send self-test {label} returned {result:?}"));
    status.store(
        if result == RakSampResult::Ok {
            SelfTestStatus::Passed.as_raw()
        } else {
            SelfTestStatus::CallFailed.as_raw()
        },
        Ordering::Release,
    );
}

fn schedule_shutdown() {
    if !logging::plugin_path(SHUTDOWN_TEST_MARKER).is_file() {
        return;
    }
    logging::write("shutdown self-test enabled; scheduling coordinated callback shutdown");
    if let Err(error) = std::thread::Builder::new()
        .name("rak-samp-validation-shutdown".into())
        .spawn(|| {
            std::thread::sleep(Duration::from_millis(250));
            let result = crate::RakSampPlugin_Shutdown();
            logging::write(&format!("shutdown self-test returned {result}"));
        })
    {
        logging::write(&format!(
            "shutdown self-test thread failed to start: {error}"
        ));
    }
}

fn emulate_when_ready(mut emulate: impl FnMut() -> RakSampResult) -> RakSampResult {
    let deadline = Instant::now() + HOST_WAIT_TIMEOUT;
    loop {
        if STOP.load(Ordering::Acquire) {
            return RakSampResult::NotReady;
        }
        let result = emulate();
        if result != RakSampResult::NotReady || Instant::now() >= deadline {
            return result;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn encode_dialog_when_ready(api: HostApi) -> Result<EncodedPayload, EventError> {
    // Encoding uses the host's captured R1 native string codec, which may not
    // exist until the user has finished connecting. Keep the general host
    // discovery timeout short, but give this opt-in live step the same
    // two-minute window as the other connection-dependent checks.
    let deadline = Instant::now() + DIRECT_SNAPSHOT_STATE_TIMEOUT;
    loop {
        if STOP.load(Ordering::Acquire) {
            return Err(EventError::Host(RakSampResult::NotReady));
        }
        let result = incoming::SHOW_DIALOG.encode(api, test_dialog_input());
        if !matches!(result, Err(EventError::Host(RakSampResult::NotReady)))
            || Instant::now() >= deadline
        {
            return result;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn record_emulation_result(label: &str, result: RakSampResult, status: &AtomicU8) {
    logging::write(&format!("self-test {label} emulation returned {result:?}"));
    if result != RakSampResult::Ok {
        let _ = status.compare_exchange(
            SelfTestStatus::Pending.as_raw(),
            SelfTestStatus::CallFailed.as_raw(),
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

fn mark_timeout(status: &AtomicU8) {
    let _ = status.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        (!self_test_finished(current)).then_some(SelfTestStatus::TimedOut.as_raw())
    });
}

#[cfg(test)]
mod tests {
    use super::{
        DirectChatDisplayModes, DirectCursorStates, DirectDialogAction, DirectDialogWaitState,
        DirectScoreboardStates, DirectSnapshotChanges, DirectUiStates, DirectVisibilityStates,
        RemotePlayerStateChanges, direct_validation_dialog, is_direct_validation_dialog,
    };
    use rak_samp_plugin_api::{
        LocalChatDisplayMode, LocalCursorMode, LocalDialogState, LocalDialogStyle, LocalPlayer,
        RemotePlayerState, Vector3,
    };

    fn snapshot() -> LocalPlayer {
        LocalPlayer {
            id: 7,
            nickname: b"fixture".to_vec(),
            colour: 0,
            spawned: true,
            health: 100.0,
            armour: 50.0,
            position: Vector3::default(),
            velocity: Vector3::default(),
            special_action: 0,
            animation_id: 0,
            vehicle_id: None,
            score: 0,
            ping: 0,
        }
    }

    #[test]
    fn direct_snapshot_monitor_requires_each_requested_state_change() {
        let baseline = snapshot();
        let mut changed = baseline.clone();
        changed.position.x = 1.0;
        changed.health = 90.0;
        changed.armour = 40.0;
        changed.vehicle_id = Some(12);

        let mut changes = DirectSnapshotChanges::default();
        changes.observe(&baseline, &changed);
        assert!(changes.complete());
    }

    #[test]
    fn direct_snapshot_monitor_does_not_pass_when_a_state_is_unchanged() {
        let baseline = snapshot();
        let mut changed = baseline.clone();
        changed.position.x = 1.0;
        changed.health = 90.0;
        changed.vehicle_id = Some(12);

        let mut changes = DirectSnapshotChanges::default();
        changes.observe(&baseline, &changed);
        assert!(!changes.complete());
    }

    #[test]
    fn remote_player_state_monitor_requires_all_transition_categories() {
        let baseline = RemotePlayerState {
            id: 7,
            health: 100.0,
            armour: 50.0,
            special_action: 0,
            animation_id: 0,
        };
        let mut changes = RemotePlayerStateChanges::default();
        changes.observe(
            baseline,
            RemotePlayerState {
                armour: 40.0,
                ..baseline
            },
        );
        changes.observe(
            baseline,
            RemotePlayerState {
                special_action: 1,
                ..baseline
            },
        );
        assert!(!changes.complete());
        changes.observe(
            baseline,
            RemotePlayerState {
                animation_id: 2,
                ..baseline
            },
        );
        assert!(changes.complete());
    }

    #[test]
    fn direct_chat_mode_monitor_requires_all_three_r1_modes() {
        let mut modes = DirectChatDisplayModes::default();
        modes.observe(LocalChatDisplayMode::Normal);
        modes.observe(LocalChatDisplayMode::Off);
        assert!(!modes.complete());
        modes.observe(LocalChatDisplayMode::NoShadow);
        assert!(modes.complete());
    }

    #[test]
    fn direct_cursor_and_scoreboard_monitors_require_both_visible_states() {
        let mut cursor = DirectCursorStates::default();
        cursor.observe(LocalCursorMode::None);
        assert!(!cursor.complete());
        cursor.observe(LocalCursorMode::LockCamera);
        assert!(cursor.complete());

        let mut scoreboard = DirectScoreboardStates::default();
        scoreboard.observe(false);
        assert!(!scoreboard.complete());
        scoreboard.observe(true);
        assert!(scoreboard.complete());

        let mut visibility = DirectVisibilityStates::default();
        visibility.observe(false);
        assert!(!visibility.complete());
        visibility.observe(true);
        assert!(visibility.complete());
    }

    #[test]
    fn direct_ui_monitor_requires_active_dialog_core_transitions() {
        let mut states = DirectUiStates::default();
        states.chat_modes.observe(LocalChatDisplayMode::Off);
        states.chat_modes.observe(LocalChatDisplayMode::NoShadow);
        states.chat_modes.observe(LocalChatDisplayMode::Normal);
        states.cursor.observe(LocalCursorMode::None);
        states.cursor.observe(LocalCursorMode::LockCamera);
        states.scoreboard.observe(false);
        states.scoreboard.observe(true);
        states.dialog.observe(false);
        states.dialog.observe(true);
        states.chat_input.observe(false);
        states.chat_input.observe(true);
        assert!(!states.complete());

        states.active_dialog_core.observe(true);
        assert!(!states.complete());
        states.active_dialog_core.observe(false);
        assert!(states.complete());
    }

    #[test]
    fn direct_dialog_match_requires_the_owned_local_validation_core() {
        let request = direct_validation_dialog();
        let expected = LocalDialogState {
            id: i32::from(request.id),
            style: request.style,
            title: request.title.to_vec(),
            server_side: false,
        };
        assert!(is_direct_validation_dialog(&expected));

        assert!(!is_direct_validation_dialog(&LocalDialogState {
            server_side: true,
            ..expected.clone()
        }));
        assert!(!is_direct_validation_dialog(&LocalDialogState {
            style: LocalDialogStyle::Input,
            ..expected
        }));
    }

    #[test]
    fn direct_dialog_wait_requeues_only_after_interference_clears() {
        let mut wait = DirectDialogWaitState::default();
        assert_eq!(wait.observe(None), DirectDialogAction::Wait);

        let server_dialog = LocalDialogState {
            id: 7,
            style: LocalDialogStyle::MessageBox,
            title: b"server".to_vec(),
            server_side: true,
        };
        assert_eq!(
            wait.observe(Some(&server_dialog)),
            DirectDialogAction::Interference
        );
        assert_eq!(wait.observe(None), DirectDialogAction::Requeue);
        assert_eq!(wait.observe(None), DirectDialogAction::Wait);

        let request = direct_validation_dialog();
        let direct_dialog = LocalDialogState {
            id: i32::from(request.id),
            style: request.style,
            title: request.title.to_vec(),
            server_side: false,
        };
        assert_eq!(
            wait.observe(Some(&direct_dialog)),
            DirectDialogAction::Matched
        );
    }
}
