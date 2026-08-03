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
    LocalDeathMessage, LocalDialog, LocalDialogStyle, LocalPlayer, MAX_SAMP_PLAYERS,
    RakSampHookAction, RakSampResult, RakSampSendOptions,
    events::{EncodedPayload, Event, EventError, rpc::incoming},
};
use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{Duration, Instant},
};

const SEND_TEST_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const DIRECT_SNAPSHOT_STATE_TIMEOUT: Duration = Duration::from_secs(120);
pub(crate) const SEND_TEST_MARKER: &str = "rak-samp-validation-send.enabled";
pub(crate) const DIRECT_CLIENT_TEST_MARKER: &str = "rak-samp-validation-direct-client.enabled";
pub(crate) const PLAYER_DIRECTORY_TEST_MARKER: &str =
    "rak-samp-validation-player-directory.enabled";
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
    chat_input: DirectVisibilityStates,
}

impl DirectUiStates {
    fn complete(self) -> bool {
        self.chat_modes.complete()
            && self.cursor.complete()
            && self.scoreboard.complete()
            && self.dialog.complete()
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
                if api.is_player_connected(id) == Ok(true)
                    && api.player_nickname(id) == Ok(Some(player.nickname))
                    && api.is_player_npc(id) == Ok(Some(player.is_npc))
                    && api.player_colour(id) == Ok(Some(player.colour))
                    && api.player_score(id) == Ok(Some(player.score))
                    && api.player_ping(id) == Ok(Some(player.ping))
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

    let dialog_result = api.show_local_dialog(LocalDialog {
        id: 0x7FFC,
        style: LocalDialogStyle::MessageBox,
        title: b"rak-samp validation",
        text: b"This is a direct local dialog validation request.",
        button1: b"Close",
        button2: b"",
    });
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

    let deadline = Instant::now() + HOST_WAIT_TIMEOUT;
    while !STOP.load(Ordering::Acquire) && Instant::now() < deadline {
        match api.local_player() {
            Ok(snapshot) if !snapshot.nickname.is_empty() => {
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
                        logging::write(
                            "direct-client self-test animation-table entry did not match the R1 fingerprint",
                        );
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
                            "direct-client self-test dialog-state returned {error:?}"
                        ));
                        return;
                    }
                }
                match api.is_local_chat_input_active() {
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
                SELF_TESTS
                    .direct_client
                    .store(SelfTestStatus::Passed.as_raw(), Ordering::Release);
                logging::write(&format!(
                    "direct-client self-test passed: dialog=Ok chat=Ok death_window=Ok game_state=Ok server_info=Ok chat_display_mode=Ok cursor_mode=Ok scoreboard_state=Ok dialog_state=Ok chat_input_state=Ok animation_table=Ok player_directory=Ok player_count=Ok local_player_id={}",
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
    logging::write("direct-client self-test timed out before a populated snapshot");
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
        "direct-client state validation {outcome}: local_player_id={id} position_changed={} health_changed={} armour_changed={} vehicle_changed={} chat_mode_off={} chat_mode_no_shadow={} chat_mode_normal={} cursor_none={} cursor_active={} scoreboard_closed={} scoreboard_open={} dialog_inactive={} dialog_active={} chat_input_inactive={} chat_input_active={}",
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
    let deadline = Instant::now() + HOST_WAIT_TIMEOUT;
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
        DirectChatDisplayModes, DirectCursorStates, DirectScoreboardStates, DirectSnapshotChanges,
        DirectVisibilityStates,
    };
    use rak_samp_plugin_api::{LocalChatDisplayMode, LocalCursorMode, LocalPlayer, Vector3};

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
}
