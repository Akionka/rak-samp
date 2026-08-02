//! In-game validation plugin for the rak-rs native packet and RPC paths.

use rak_rs_plugin_api::{
    HostApi, RakRsApiV1, RakRsDirection, RakRsEventCallbackV1, RakRsEventV1, RakRsHookAction,
    RakRsResult, RakRsSendOptions, RakRsSubscription, ResolveError, wait_for_default_host,
};
use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    ptr,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicPtr, AtomicU8, AtomicU32, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};
use windows_sys::core::BOOL;

const HOST_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const HOST_POLL_INTERVAL: Duration = Duration::from_millis(100);
const REPORT_INTERVAL: Duration = Duration::from_secs(5);
const SEND_TEST_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const ID_COUNT: usize = 256;
const ID_TIMESTAMP: u8 = 40;
const ID_STATS_UPDATE: u8 = 205;
const RPC_UPDATE_SCORES_AND_PINGS: u8 = 155;
const STATS_PAYLOAD_LEN: usize = 8;
const SEND_TEST_MARKER: &str = "rak-rs-validation-send.enabled";
const SHUTDOWN_TEST_MARKER: &str = "rak-rs-validation-shutdown.enabled";
const TEST_PACKET_ID: u8 = 254;
const TEST_RPC_ID: u8 = 255;
const TEST_PACKET_INPUT: [u8; 16] = *b"rak-rs-packet-in";
const TEST_PACKET_REPLACEMENT: [u8; 16] = *b"rak-rs-packet-ok";
const TEST_RPC_INPUT: [u8; 16] = *b"rak-rs-rpc-input";
const TEST_RPC_REPLACEMENT: [u8; 16] = *b"rak-rs-rpc-pass!";
const SELF_TEST_PENDING: u32 = 0;
const SELF_TEST_REWRITTEN: u32 = 1;
const SELF_TEST_PASSED: u32 = 2;
const SELF_TEST_FAILED: u32 = 3;
const SELF_TEST_TIMED_OUT: u32 = 4;
const SELF_TEST_CALL_FAILED: u32 = 5;
const SELF_TEST_DISABLED: u32 = 6;

type IdHistogram = [AtomicU32; ID_COUNT];
type RegisterFn = unsafe extern "system" fn(
    RakRsDirection,
    Option<RakRsEventCallbackV1>,
    *mut c_void,
    *mut RakRsSubscription,
) -> RakRsResult;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static API: AtomicPtr<RakRsApiV1> = AtomicPtr::new(ptr::null_mut());
static STOP: AtomicBool = AtomicBool::new(false);
static INCOMING_PACKETS: AtomicU32 = AtomicU32::new(0);
static OUTGOING_PACKETS: AtomicU32 = AtomicU32::new(0);
static INCOMING_RPCS: AtomicU32 = AtomicU32::new(0);
static OUTGOING_RPCS: AtomicU32 = AtomicU32::new(0);
static NULL_EVENTS: AtomicU32 = AtomicU32::new(0);
static TIMESTAMP_DECODE_ERRORS: AtomicU32 = AtomicU32::new(0);
static PACKET_SELF_TEST: AtomicU32 = AtomicU32::new(SELF_TEST_PENDING);
static RPC_SELF_TEST: AtomicU32 = AtomicU32::new(SELF_TEST_PENDING);
static SEND_PACKET_SELF_TEST: AtomicU32 = AtomicU32::new(SELF_TEST_PENDING);
static SEND_RPC_SELF_TEST: AtomicU32 = AtomicU32::new(SELF_TEST_PENDING);
static STATS_PAYLOAD_READY: AtomicBool = AtomicBool::new(false);
static STATS_PAYLOAD: [AtomicU8; STATS_PAYLOAD_LEN] =
    [const { AtomicU8::new(0) }; STATS_PAYLOAD_LEN];
static INCOMING_PACKET_IDS: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];
static OUTGOING_PACKET_IDS: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];
static INCOMING_TIMESTAMP_INNER_IDS: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];
static OUTGOING_TIMESTAMP_INNER_IDS: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];
static INCOMING_RPC_IDS: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];
static OUTGOING_RPC_IDS: IdHistogram = [const { AtomicU32::new(0) }; ID_COUNT];

struct PluginState {
    api: Option<HostApi>,
    subscriptions: Vec<RakRsSubscription>,
    initialization_worker: Option<JoinHandle<()>>,
    reporter_worker: Option<JoinHandle<()>>,
    self_test_worker: Option<JoinHandle<()>>,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            api: None,
            subscriptions: Vec::new(),
            initialization_worker: None,
            reporter_worker: None,
            self_test_worker: None,
            shutting_down: false,
        }
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
            let worker = std::thread::Builder::new()
                .name("rak-rs-validation-init".into())
                .spawn(initialize);
            if let Ok(worker) = worker {
                STATE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .initialization_worker = Some(worker);
            }
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

fn initialize() {
    initialize_log();
    write_log(&format!(
        "session started: process_id={}",
        std::process::id()
    ));

    let Some(api) = wait_for_host() else {
        return;
    };
    if is_shutting_down() {
        write_log("shutdown requested before callback registration");
        return;
    }

    let registrations: [(&str, RegisterFn, RakRsDirection, RakRsEventCallbackV1); 6] = [
        (
            "incoming packet self-test rewriter",
            api.raw().register_packet,
            RakRsDirection::Incoming,
            rewrite_test_packet,
        ),
        (
            "incoming RPC self-test rewriter",
            api.raw().register_rpc,
            RakRsDirection::Incoming,
            rewrite_test_rpc,
        ),
        (
            "incoming packet",
            api.raw().register_packet,
            RakRsDirection::Incoming,
            on_incoming_packet,
        ),
        (
            "outgoing packet",
            api.raw().register_packet,
            RakRsDirection::Outgoing,
            on_outgoing_packet,
        ),
        (
            "incoming RPC",
            api.raw().register_rpc,
            RakRsDirection::Incoming,
            on_incoming_rpc,
        ),
        (
            "outgoing RPC",
            api.raw().register_rpc,
            RakRsDirection::Outgoing,
            on_outgoing_rpc,
        ),
    ];
    let mut subscriptions = Vec::with_capacity(registrations.len());
    for (label, register_fn, direction, callback) in registrations {
        match register(register_fn, direction, Some(callback)) {
            Ok(subscription) => subscriptions.push(subscription),
            Err(error) => {
                write_log(&format!("{label} registration failed: {error:?}"));
                unregister_all(api, subscriptions);
                return;
            }
        }
    }

    {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutting_down {
            drop(state);
            unregister_all(api, subscriptions);
            return;
        }
        state.api = Some(api);
        state.subscriptions = subscriptions;
        API.store(
            api.raw() as *const RakRsApiV1 as *mut RakRsApiV1,
            Ordering::Release,
        );
    }
    write_log("ready: six packet/RPC validation callbacks registered");
    report_counts(0);

    match std::thread::Builder::new()
        .name("rak-rs-validation-report".into())
        .spawn(report_loop)
    {
        Ok(worker) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .reporter_worker = Some(worker);
        }
        Err(error) => {
            write_log(&format!("reporter thread failed to start: {error}"));
            API.store(ptr::null_mut(), Ordering::Release);
            let subscriptions = {
                let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
                state.api = None;
                std::mem::take(&mut state.subscriptions)
            };
            unregister_all(api, subscriptions);
            return;
        }
    }

    match std::thread::Builder::new()
        .name("rak-rs-validation-self-test".into())
        .spawn(move || run_self_test(api))
    {
        Ok(worker) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .self_test_worker = Some(worker);
        }
        Err(error) => write_log(&format!("self-test thread failed to start: {error}")),
    }
}

fn wait_for_host() -> Option<HostApi> {
    let deadline = Instant::now() + HOST_WAIT_TIMEOUT;
    loop {
        if STOP.load(Ordering::Acquire) {
            return None;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            write_log("host discovery timed out after 30 seconds");
            return None;
        }
        match wait_for_default_host(remaining.min(HOST_POLL_INTERVAL)) {
            Ok(api) => return Some(api),
            Err(ResolveError::TimedOut) => {}
            Err(error) => {
                write_log(&format!("host discovery failed: {error}"));
                return None;
            }
        }
    }
}

fn register(
    register: unsafe extern "system" fn(
        RakRsDirection,
        Option<RakRsEventCallbackV1>,
        *mut c_void,
        *mut RakRsSubscription,
    ) -> RakRsResult,
    direction: RakRsDirection,
    callback: Option<RakRsEventCallbackV1>,
) -> Result<RakRsSubscription, RakRsResult> {
    let mut subscription = RakRsSubscription::default();
    let result = unsafe { register(direction, callback, ptr::null_mut(), &raw mut subscription) };
    if result == RakRsResult::Ok {
        Ok(subscription)
    } else {
        Err(result)
    }
}

unsafe extern "system" fn on_incoming_packet(
    _user_data: *mut c_void,
    event: *mut RakRsEventV1,
) -> RakRsHookAction {
    let observed = observe_packet(
        event,
        &INCOMING_PACKETS,
        &INCOMING_PACKET_IDS,
        &INCOMING_TIMESTAMP_INNER_IDS,
    );
    test_verdict(
        observed,
        event,
        TEST_PACKET_ID,
        &TEST_PACKET_INPUT,
        &TEST_PACKET_REPLACEMENT,
        &PACKET_SELF_TEST,
    )
}

unsafe extern "system" fn on_outgoing_packet(
    _user_data: *mut c_void,
    event: *mut RakRsEventV1,
) -> RakRsHookAction {
    let observed = observe_packet(
        event,
        &OUTGOING_PACKETS,
        &OUTGOING_PACKET_IDS,
        &OUTGOING_TIMESTAMP_INNER_IDS,
    );
    if let Some((api, ID_STATS_UPDATE)) = observed {
        capture_stats_payload(api, event);
    }
    RakRsHookAction::Continue
}

unsafe extern "system" fn on_incoming_rpc(
    _user_data: *mut c_void,
    event: *mut RakRsEventV1,
) -> RakRsHookAction {
    let observed = observe(event, &INCOMING_RPCS, &INCOMING_RPC_IDS);
    test_verdict(
        observed,
        event,
        TEST_RPC_ID,
        &TEST_RPC_INPUT,
        &TEST_RPC_REPLACEMENT,
        &RPC_SELF_TEST,
    )
}

unsafe extern "system" fn on_outgoing_rpc(
    _user_data: *mut c_void,
    event: *mut RakRsEventV1,
) -> RakRsHookAction {
    observe(event, &OUTGOING_RPCS, &OUTGOING_RPC_IDS);
    RakRsHookAction::Continue
}

fn observe_packet(
    event: *mut RakRsEventV1,
    count: &AtomicU32,
    ids: &IdHistogram,
    timestamp_inner_ids: &IdHistogram,
) -> Option<(*mut RakRsApiV1, u8)> {
    let (api, id) = observe(event, count, ids)?;
    if id != ID_TIMESTAMP {
        return Some((api, id));
    }

    let mut timestamp = 0;
    let mut inner_id = 0;
    let decoded = unsafe { ((*api).event_reset_read)(event) } == RakRsResult::Ok
        && unsafe { ((*api).event_read_u32)(event, &raw mut timestamp) } == RakRsResult::Ok
        && unsafe { ((*api).event_read_u8)(event, &raw mut inner_id) } == RakRsResult::Ok;
    let restored = unsafe { ((*api).event_reset_read)(event) } == RakRsResult::Ok;
    if decoded && restored {
        timestamp_inner_ids[usize::from(inner_id)].fetch_add(1, Ordering::Relaxed);
    } else {
        TIMESTAMP_DECODE_ERRORS.fetch_add(1, Ordering::Relaxed);
    }
    Some((api, id))
}

unsafe extern "system" fn rewrite_test_packet(
    _user_data: *mut c_void,
    event: *mut RakRsEventV1,
) -> RakRsHookAction {
    rewrite_test_event(
        event,
        TEST_PACKET_ID,
        &TEST_PACKET_INPUT,
        &TEST_PACKET_REPLACEMENT,
        &PACKET_SELF_TEST,
    );
    RakRsHookAction::Continue
}

unsafe extern "system" fn rewrite_test_rpc(
    _user_data: *mut c_void,
    event: *mut RakRsEventV1,
) -> RakRsHookAction {
    rewrite_test_event(
        event,
        TEST_RPC_ID,
        &TEST_RPC_INPUT,
        &TEST_RPC_REPLACEMENT,
        &RPC_SELF_TEST,
    );
    RakRsHookAction::Continue
}

fn rewrite_test_event<const N: usize>(
    event: *mut RakRsEventV1,
    expected_id: u8,
    input: &[u8; N],
    replacement: &[u8; N],
    status: &AtomicU32,
) {
    if event.is_null() {
        return;
    }
    let api = API.load(Ordering::Acquire);
    if api.is_null() || unsafe { ((*api).event_id)(event) } != expected_id {
        return;
    }
    if !event_matches(api, event, input) {
        return;
    }
    let result = unsafe { ((*api).event_replace_bytes)(event, replacement.as_ptr(), N) };
    status.store(
        if result == RakRsResult::Ok {
            SELF_TEST_REWRITTEN
        } else {
            SELF_TEST_FAILED
        },
        Ordering::Release,
    );
}

fn test_verdict<const N: usize>(
    observed: Option<(*mut RakRsApiV1, u8)>,
    event: *mut RakRsEventV1,
    expected_id: u8,
    input: &[u8; N],
    replacement: &[u8; N],
    status: &AtomicU32,
) -> RakRsHookAction {
    let Some((api, id)) = observed else {
        return RakRsHookAction::Continue;
    };
    if id != expected_id {
        return RakRsHookAction::Continue;
    }
    if event_matches(api, event, replacement) {
        status.store(SELF_TEST_PASSED, Ordering::Release);
        return RakRsHookAction::Block;
    }
    if event_matches(api, event, input) {
        status.store(SELF_TEST_FAILED, Ordering::Release);
        return RakRsHookAction::Block;
    }
    RakRsHookAction::Continue
}

fn event_matches<const N: usize>(
    api: *mut RakRsApiV1,
    event: *mut RakRsEventV1,
    expected: &[u8; N],
) -> bool {
    read_exact_event(api, event).as_ref() == Some(expected)
}

fn read_exact_event<const N: usize>(
    api: *mut RakRsApiV1,
    event: *mut RakRsEventV1,
) -> Option<[u8; N]> {
    let mut actual = [0; N];
    let mut trailing = 0;
    let read = unsafe { ((*api).event_reset_read)(event) } == RakRsResult::Ok
        && unsafe { ((*api).event_read_bytes)(event, actual.as_mut_ptr(), N) } == RakRsResult::Ok
        && unsafe { ((*api).event_read_u8)(event, &raw mut trailing) }
            == RakRsResult::ReadOutOfBounds;
    let restored = unsafe { ((*api).event_reset_read)(event) } == RakRsResult::Ok;
    (read && restored).then_some(actual)
}

fn capture_stats_payload(api: *mut RakRsApiV1, event: *mut RakRsEventV1) {
    if STATS_PAYLOAD_READY.load(Ordering::Acquire) {
        return;
    }
    let Some(payload) = read_exact_event::<STATS_PAYLOAD_LEN>(api, event) else {
        return;
    };
    for (destination, source) in STATS_PAYLOAD.iter().zip(payload) {
        destination.store(source, Ordering::Relaxed);
    }
    STATS_PAYLOAD_READY.store(true, Ordering::Release);
}

fn observe(
    event: *mut RakRsEventV1,
    count: &AtomicU32,
    ids: &IdHistogram,
) -> Option<(*mut RakRsApiV1, u8)> {
    if event.is_null() {
        NULL_EVENTS.fetch_add(1, Ordering::Relaxed);
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

fn run_self_test(api: HostApi) {
    let rpc_result = emulate_when_ready(|| {
        api.emulate_incoming_rpc(TEST_RPC_ID, &TEST_RPC_INPUT, TEST_RPC_INPUT.len() * 8)
    });
    record_emulation_result("RPC", rpc_result, &RPC_SELF_TEST);

    let packet_result = emulate_when_ready(|| {
        api.emulate_incoming_packet(
            TEST_PACKET_ID,
            &TEST_PACKET_INPUT,
            TEST_PACKET_INPUT.len() * 8,
        )
    });
    record_emulation_result("packet", packet_result, &PACKET_SELF_TEST);

    let deadline = Instant::now() + Duration::from_secs(10);
    while !STOP.load(Ordering::Acquire)
        && Instant::now() < deadline
        && (!self_test_finished(PACKET_SELF_TEST.load(Ordering::Acquire))
            || !self_test_finished(RPC_SELF_TEST.load(Ordering::Acquire)))
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    mark_self_test_timeout(&PACKET_SELF_TEST);
    mark_self_test_timeout(&RPC_SELF_TEST);
    write_log(&format!(
        "self-test completed: packet={} RPC={}",
        self_test_label(PACKET_SELF_TEST.load(Ordering::Acquire)),
        self_test_label(RPC_SELF_TEST.load(Ordering::Acquire)),
    ));
    run_send_self_test(api);
    schedule_shutdown_self_test();
}

fn run_send_self_test(api: HostApi) {
    if !Path::new(SEND_TEST_MARKER).is_file() {
        SEND_PACKET_SELF_TEST.store(SELF_TEST_DISABLED, Ordering::Release);
        SEND_RPC_SELF_TEST.store(SELF_TEST_DISABLED, Ordering::Release);
        write_log("send self-test disabled; opt in with rak-rs-validation-send.enabled");
        return;
    }
    write_log("send self-test enabled; waiting for an outgoing ID_STATS_UPDATE payload");
    let deadline = Instant::now() + SEND_TEST_WAIT_TIMEOUT;
    while !STOP.load(Ordering::Acquire)
        && !STATS_PAYLOAD_READY.load(Ordering::Acquire)
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    if !STATS_PAYLOAD_READY.load(Ordering::Acquire) {
        SEND_PACKET_SELF_TEST.store(SELF_TEST_TIMED_OUT, Ordering::Release);
        SEND_RPC_SELF_TEST.store(SELF_TEST_TIMED_OUT, Ordering::Release);
        write_log("send self-test timed out before an ID_STATS_UPDATE payload was captured");
        return;
    }

    let mut payload = [0; STATS_PAYLOAD_LEN];
    for (destination, source) in payload.iter_mut().zip(&STATS_PAYLOAD) {
        *destination = source.load(Ordering::Relaxed);
    }
    let packet_options = RakRsSendOptions {
        reliability: 6,
        ..RakRsSendOptions::default()
    };
    let packet_result = api.send_packet(
        ID_STATS_UPDATE,
        &payload,
        payload.len() * u8::BITS as usize,
        packet_options,
    );
    record_send_result("packet", packet_result, &SEND_PACKET_SELF_TEST);

    let rpc_result = api.send_rpc(
        RPC_UPDATE_SCORES_AND_PINGS,
        &[],
        0,
        RakRsSendOptions::default(),
    );
    record_send_result("RPC", rpc_result, &SEND_RPC_SELF_TEST);
    write_log(&format!(
        "send self-test completed: packet={} RPC={}",
        self_test_label(SEND_PACKET_SELF_TEST.load(Ordering::Acquire)),
        self_test_label(SEND_RPC_SELF_TEST.load(Ordering::Acquire)),
    ));
}

fn record_send_result(label: &str, result: RakRsResult, status: &AtomicU32) {
    write_log(&format!("send self-test {label} returned {result:?}"));
    status.store(
        if result == RakRsResult::Ok {
            SELF_TEST_PASSED
        } else {
            SELF_TEST_CALL_FAILED
        },
        Ordering::Release,
    );
}

fn schedule_shutdown_self_test() {
    if !Path::new(SHUTDOWN_TEST_MARKER).is_file() {
        return;
    }
    write_log("shutdown self-test enabled; scheduling coordinated callback shutdown");
    if let Err(error) = std::thread::Builder::new()
        .name("rak-rs-validation-shutdown".into())
        .spawn(|| {
            std::thread::sleep(Duration::from_millis(250));
            let result = RakRsPlugin_Shutdown();
            write_log(&format!("shutdown self-test returned {result}"));
        })
    {
        write_log(&format!(
            "shutdown self-test thread failed to start: {error}"
        ));
    }
}

fn emulate_when_ready(mut emulate: impl FnMut() -> RakRsResult) -> RakRsResult {
    let deadline = Instant::now() + HOST_WAIT_TIMEOUT;
    loop {
        if STOP.load(Ordering::Acquire) {
            return RakRsResult::NotReady;
        }
        let result = emulate();
        if result != RakRsResult::NotReady || Instant::now() >= deadline {
            return result;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn record_emulation_result(label: &str, result: RakRsResult, status: &AtomicU32) {
    write_log(&format!("self-test {label} emulation returned {result:?}"));
    if result != RakRsResult::Ok {
        let _ = status.compare_exchange(
            SELF_TEST_PENDING,
            SELF_TEST_CALL_FAILED,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

fn self_test_finished(status: u32) -> bool {
    matches!(
        status,
        SELF_TEST_PASSED
            | SELF_TEST_FAILED
            | SELF_TEST_TIMED_OUT
            | SELF_TEST_CALL_FAILED
            | SELF_TEST_DISABLED
    )
}

fn mark_self_test_timeout(status: &AtomicU32) {
    let _ = status.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        (!self_test_finished(current)).then_some(SELF_TEST_TIMED_OUT)
    });
}

fn self_test_label(status: u32) -> &'static str {
    match status {
        SELF_TEST_PENDING => "pending",
        SELF_TEST_REWRITTEN => "rewritten",
        SELF_TEST_PASSED => "passed",
        SELF_TEST_FAILED => "failed",
        SELF_TEST_TIMED_OUT => "timed-out",
        SELF_TEST_CALL_FAILED => "call-failed",
        SELF_TEST_DISABLED => "disabled",
        _ => "invalid",
    }
}

fn report_loop() {
    let started = Instant::now();
    let mut next_report = REPORT_INTERVAL;
    while !STOP.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = started.elapsed();
        if elapsed >= next_report {
            report_counts(elapsed.as_secs());
            next_report = elapsed + REPORT_INTERVAL;
        }
    }
    report_counts(started.elapsed().as_secs());
    write_log("reporter stopped");
}

fn report_counts(elapsed_seconds: u64) {
    write_log(&format!(
        "t={elapsed_seconds}s incoming_packets={} outgoing_packets={} incoming_rpcs={} outgoing_rpcs={} null_events={} timestamp_decode_errors={}",
        INCOMING_PACKETS.load(Ordering::Relaxed),
        OUTGOING_PACKETS.load(Ordering::Relaxed),
        INCOMING_RPCS.load(Ordering::Relaxed),
        OUTGOING_RPCS.load(Ordering::Relaxed),
        NULL_EVENTS.load(Ordering::Relaxed),
        TIMESTAMP_DECODE_ERRORS.load(Ordering::Relaxed),
    ));
    report_histogram("incoming_packet_ids", &INCOMING_PACKET_IDS, packet_name);
    report_histogram("outgoing_packet_ids", &OUTGOING_PACKET_IDS, packet_name);
    report_histogram(
        "incoming_timestamp_inner_ids",
        &INCOMING_TIMESTAMP_INNER_IDS,
        packet_name,
    );
    report_histogram(
        "outgoing_timestamp_inner_ids",
        &OUTGOING_TIMESTAMP_INNER_IDS,
        packet_name,
    );
    report_histogram("incoming_rpc_ids", &INCOMING_RPC_IDS, incoming_rpc_name);
    report_histogram("outgoing_rpc_ids", &OUTGOING_RPC_IDS, outgoing_rpc_name);
    write_log(&format!(
        "self_test: packet={} RPC={} send_packet={} send_RPC={}",
        self_test_label(PACKET_SELF_TEST.load(Ordering::Acquire)),
        self_test_label(RPC_SELF_TEST.load(Ordering::Acquire)),
        self_test_label(SEND_PACKET_SELF_TEST.load(Ordering::Acquire)),
        self_test_label(SEND_RPC_SELF_TEST.load(Ordering::Acquire)),
    ));
}

fn report_histogram(label: &str, histogram: &IdHistogram, name: fn(u8) -> Option<&'static str>) {
    write_log(&format!("{label}: {}", format_histogram(histogram, name)));
}

fn format_histogram(histogram: &IdHistogram, name: fn(u8) -> Option<&'static str>) -> String {
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

fn packet_name(id: u8) -> Option<&'static str> {
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
        TEST_PACKET_ID => "RAK_RS_SELF_TEST",
        _ => return None,
    })
}

fn incoming_rpc_name(id: u8) -> Option<&'static str> {
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
        156 => "SET_INTERIOR",
        159 => "SET_VEHICLE_POSITION",
        160 => "SET_VEHICLE_ANGLE",
        162 => "SET_CAMERA_BEHIND",
        163 => "PLAYER_STREAM_OUT",
        165 => "VEHICLE_STREAM_OUT",
        TEST_RPC_ID => "RAK_RS_SELF_TEST",
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

fn initialize_log() {
    let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rak-rs-validation.log")
    else {
        return;
    };
    let _ = LOG_FILE.set(Mutex::new(file));
}

fn write_log(message: &str) {
    let Some(file) = LOG_FILE.get() else {
        return;
    };
    let mut file = file.lock().unwrap_or_else(|error| error.into_inner());
    let _ = writeln!(file, "{message}");
    let _ = file.flush();
}

fn is_shutting_down() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .shutting_down
}

fn unregister_all(api: HostApi, subscriptions: Vec<RakRsSubscription>) {
    for subscription in subscriptions {
        let _ = api.unregister_and_wait(subscription);
    }
}

/// Stops workers and callbacks before an unload manager calls `FreeLibrary`.
#[unsafe(no_mangle)]
pub extern "system" fn RakRsPlugin_Shutdown() -> BOOL {
    STOP.store(true, Ordering::Release);
    let initialization = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        state.initialization_worker.take()
    };
    if let Some(worker) = initialization {
        let _ = worker.join();
    }

    let self_test = STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .self_test_worker
        .take();
    if let Some(worker) = self_test {
        let _ = worker.join();
    }

    let reporter = STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .reporter_worker
        .take();
    if let Some(worker) = reporter {
        let _ = worker.join();
    }

    let (api, subscriptions) = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        (state.api, std::mem::take(&mut state.subscriptions))
    };
    let Some(api) = api else {
        write_log("shutdown completed before host registration");
        return TRUE;
    };

    let mut failed = Vec::new();
    for subscription in subscriptions {
        let result = api.unregister_and_wait(subscription);
        if !matches!(result, RakRsResult::Ok | RakRsResult::SubscriptionNotFound) {
            write_log(&format!(
                "subscription {} failed to stop: {result:?}",
                subscription.id
            ));
            failed.push(subscription);
        }
    }
    if failed.is_empty() {
        API.store(ptr::null_mut(), Ordering::Release);
        write_log("shutdown completed; all callbacks quiesced");
        TRUE
    } else {
        STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .subscriptions = failed;
        0
    }
}

/// Returns the number of incoming packet callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakRsValidation_IncomingPacketCount() -> u32 {
    INCOMING_PACKETS.load(Ordering::Relaxed)
}

/// Returns the number of outgoing packet callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakRsValidation_OutgoingPacketCount() -> u32 {
    OUTGOING_PACKETS.load(Ordering::Relaxed)
}

/// Returns the number of incoming RPC callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakRsValidation_IncomingRpcCount() -> u32 {
    INCOMING_RPCS.load(Ordering::Relaxed)
}

/// Returns the number of outgoing RPC callbacks observed in this session.
#[unsafe(no_mangle)]
pub extern "system" fn RakRsValidation_OutgoingRpcCount() -> u32 {
    OUTGOING_RPCS.load(Ordering::Relaxed)
}

/// Reports whether all enabled local, send, and emulation self-tests finished.
#[unsafe(no_mangle)]
pub extern "system" fn RakRsValidation_SelfTestsComplete() -> BOOL {
    let statuses = [
        PACKET_SELF_TEST.load(Ordering::Acquire),
        RPC_SELF_TEST.load(Ordering::Acquire),
        SEND_PACKET_SELF_TEST.load(Ordering::Acquire),
        SEND_RPC_SELF_TEST.load(Ordering::Acquire),
    ];
    BOOL::from(statuses.into_iter().all(self_test_finished))
}

#[cfg(test)]
mod tests {
    use super::{
        ID_COUNT, IdHistogram, TEST_PACKET_ID, TEST_RPC_ID, format_histogram, incoming_rpc_name,
        packet_name,
    };
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn names_known_packet_and_rpc_ids() {
        assert_eq!(packet_name(41), Some("ID_RECEIVED_STATIC_DATA"));
        assert_eq!(packet_name(207), Some("ID_PLAYER_SYNC"));
        assert_eq!(incoming_rpc_name(93), Some("SERVER_MESSAGE"));
        assert_eq!(packet_name(TEST_PACKET_ID), Some("RAK_RS_SELF_TEST"));
        assert_eq!(incoming_rpc_name(TEST_RPC_ID), Some("RAK_RS_SELF_TEST"));
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
