use crate::{
    logging,
    state::{
        HOST_WAIT_TIMEOUT, ID_STATS_UPDATE, RPC_UPDATE_SCORES_AND_PINGS, SELF_TESTS, STATS_PAYLOAD,
        STATS_PAYLOAD_LEN, STATS_PAYLOAD_READY, STOP, SelfTestStatus, self_test_finished,
        self_test_label,
    },
};
use rak_samp_plugin_api::{
    HostApi, RakSampHookAction, RakSampResult, RakSampSendOptions,
    events::{EncodedPayload, Event, EventError, rpc::incoming},
};
use std::{
    sync::atomic::{AtomicU8, Ordering},
    time::{Duration, Instant},
};

const SEND_TEST_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
pub(crate) const SEND_TEST_MARKER: &str = "rak-samp-validation-send.enabled";
pub(crate) const SHUTDOWN_TEST_MARKER: &str = "rak-samp-validation-shutdown.enabled";
pub(crate) const TEST_PACKET_ID: u8 = 254;
pub(crate) const TEST_RPC_ID: u8 = 255;
pub(crate) const TEST_PACKET_INPUT: [u8; 18] = *b"rak-samp-packet-in";
pub(crate) const TEST_PACKET_REPLACEMENT: [u8; 18] = *b"rak-samp-packet-ok";
pub(crate) const TEST_RPC_INPUT: [u8; 18] = *b"rak-samp-rpc-input";
pub(crate) const TEST_RPC_REPLACEMENT: [u8; 18] = *b"rak-samp-rpc-pass!";

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
    run_send(api);
    schedule_shutdown();
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
