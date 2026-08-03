use crate::{
    callbacks, logging, reporting, self_test,
    state::{API, HOST_WAIT_TIMEOUT, STOP},
};
use rak_samp_plugin_api::{
    HostApi, RakSampApiV1, RakSampDirection, RakSampEventCallbackV1, RakSampResult,
    RakSampSubscription, ResolveError, wait_for_default_host,
};
use std::{
    ffi::c_void,
    ptr,
    sync::{Mutex, atomic::Ordering},
    thread::JoinHandle,
    time::Duration,
};
use windows_sys::{
    Win32::Foundation::{HINSTANCE, TRUE},
    core::BOOL,
};

const HOST_POLL_INTERVAL: Duration = Duration::from_millis(100);

type RegisterFn = unsafe extern "system" fn(
    RakSampDirection,
    Option<RakSampEventCallbackV1>,
    *mut c_void,
    *mut RakSampSubscription,
) -> RakSampResult;

struct PluginState {
    api: Option<HostApi>,
    subscriptions: Vec<RakSampSubscription>,
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

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());

pub(crate) fn start(instance: HINSTANCE) {
    let instance = instance as usize;
    let worker = std::thread::Builder::new()
        .name("rak-samp-validation-init".into())
        .spawn(move || initialize(instance));
    if let Ok(worker) = worker {
        STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .initialization_worker = Some(worker);
    }
}

fn initialize(instance: usize) {
    logging::initialize(instance as HINSTANCE);
    logging::write(&format!(
        "session started: process_id={}",
        std::process::id()
    ));

    let Some(api) = wait_for_host() else {
        return;
    };
    if is_shutting_down() {
        logging::write("shutdown requested before callback registration");
        return;
    }

    let registrations: [(&str, RegisterFn, RakSampDirection, RakSampEventCallbackV1); 6] = [
        (
            "incoming packet self-test rewriter",
            api.raw().register_packet,
            RakSampDirection::Incoming,
            self_test::rewrite_test_packet,
        ),
        (
            "incoming RPC self-test rewriter",
            api.raw().register_rpc,
            RakSampDirection::Incoming,
            self_test::rewrite_test_rpc,
        ),
        (
            "incoming packet",
            api.raw().register_packet,
            RakSampDirection::Incoming,
            callbacks::on_incoming_packet,
        ),
        (
            "outgoing packet",
            api.raw().register_packet,
            RakSampDirection::Outgoing,
            callbacks::on_outgoing_packet,
        ),
        (
            "incoming RPC",
            api.raw().register_rpc,
            RakSampDirection::Incoming,
            callbacks::on_incoming_rpc,
        ),
        (
            "outgoing RPC",
            api.raw().register_rpc,
            RakSampDirection::Outgoing,
            callbacks::on_outgoing_rpc,
        ),
    ];
    let mut subscriptions = Vec::with_capacity(registrations.len());
    for (label, register_fn, direction, callback) in registrations {
        match register(register_fn, direction, Some(callback)) {
            Ok(subscription) => subscriptions.push(subscription),
            Err(error) => {
                logging::write(&format!("{label} registration failed: {error:?}"));
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
            api.raw() as *const RakSampApiV1 as *mut RakSampApiV1,
            Ordering::Release,
        );
    }
    logging::write("ready: six packet/RPC validation callbacks registered");
    reporting::report_counts(0);

    match std::thread::Builder::new()
        .name("rak-samp-validation-report".into())
        .spawn(reporting::report_loop)
    {
        Ok(worker) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .reporter_worker = Some(worker);
        }
        Err(error) => {
            logging::write(&format!("reporter thread failed to start: {error}"));
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
        .name("rak-samp-validation-self-test".into())
        .spawn(move || self_test::run(api))
    {
        Ok(worker) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .self_test_worker = Some(worker);
        }
        Err(error) => logging::write(&format!("self-test thread failed to start: {error}")),
    }
}

fn wait_for_host() -> Option<HostApi> {
    let deadline = std::time::Instant::now() + HOST_WAIT_TIMEOUT;
    loop {
        if STOP.load(Ordering::Acquire) {
            return None;
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            logging::write("host discovery timed out after 30 seconds");
            return None;
        }
        match wait_for_default_host(remaining.min(HOST_POLL_INTERVAL)) {
            Ok(api) => return Some(api),
            Err(ResolveError::TimedOut) => {}
            Err(error) => {
                logging::write(&format!("host discovery failed: {error}"));
                return None;
            }
        }
    }
}

fn register(
    register: RegisterFn,
    direction: RakSampDirection,
    callback: Option<RakSampEventCallbackV1>,
) -> Result<RakSampSubscription, RakSampResult> {
    let mut subscription = RakSampSubscription::default();
    let result = unsafe { register(direction, callback, ptr::null_mut(), &raw mut subscription) };
    if result == RakSampResult::Ok {
        Ok(subscription)
    } else {
        Err(result)
    }
}

fn is_shutting_down() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .shutting_down
}

fn unregister_all(api: HostApi, subscriptions: Vec<RakSampSubscription>) {
    for subscription in subscriptions {
        let _ = api.unregister_and_wait(subscription);
    }
}

pub(crate) fn shutdown() -> BOOL {
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
        logging::write("shutdown completed before host registration");
        return TRUE;
    };

    let mut failed = Vec::new();
    for subscription in subscriptions {
        let result = api.unregister_and_wait(subscription);
        if !matches!(
            result,
            RakSampResult::Ok | RakSampResult::SubscriptionNotFound
        ) {
            logging::write(&format!(
                "subscription {} failed to stop: {result:?}",
                subscription.id
            ));
            failed.push(subscription);
        }
    }
    if failed.is_empty() {
        API.store(ptr::null_mut(), Ordering::Release);
        logging::write("shutdown completed; all callbacks quiesced");
        TRUE
    } else {
        STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .subscriptions = failed;
        0
    }
}
