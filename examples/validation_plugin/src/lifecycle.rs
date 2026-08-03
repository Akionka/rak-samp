use crate::{
    callbacks, logging, reporting, self_test,
    state::{HOST_WAIT_TIMEOUT, STOP},
};
use rak_samp_plugin_api::{
    HostApi, RakSampDirection, ResolveError, SubscriptionSet, register_handlers,
    wait_for_default_host,
};
use std::{
    sync::{Mutex, atomic::Ordering},
    thread::JoinHandle,
    time::Duration,
};
use windows_sys::{
    Win32::Foundation::{HINSTANCE, TRUE},
    core::BOOL,
};

const HOST_POLL_INTERVAL: Duration = Duration::from_millis(100);

struct PluginState {
    subscriptions: SubscriptionSet,
    initialization_worker: Option<JoinHandle<()>>,
    reporter_worker: Option<JoinHandle<()>>,
    self_test_worker: Option<JoinHandle<()>>,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            subscriptions: SubscriptionSet::new(),
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

    let subscriptions = match register_handlers!(api;
        packet(RakSampDirection::Incoming, self_test::rewrite_test_packet),
        rpc(RakSampDirection::Incoming, self_test::rewrite_test_rpc),
        packet(RakSampDirection::Incoming, callbacks::on_incoming_packet),
        packet(RakSampDirection::Outgoing, callbacks::on_outgoing_packet),
        rpc(RakSampDirection::Incoming, callbacks::on_incoming_rpc),
        rpc(RakSampDirection::Outgoing, callbacks::on_outgoing_rpc),
    ) {
        Ok(subscriptions) => subscriptions,
        Err(error) => {
            logging::write(&format!(
                "callback registration failed: {:?}",
                error.result()
            ));
            retain_failed_subscriptions(error.into_subscriptions());
            return;
        }
    };

    {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutting_down {
            drop(state);
            retain_failed_subscriptions(subscriptions);
            return;
        }
        state.subscriptions = subscriptions;
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
            let subscriptions = {
                let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
                std::mem::take(&mut state.subscriptions)
            };
            retain_failed_subscriptions(subscriptions);
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

fn is_shutting_down() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .shutting_down
}

fn retain_failed_subscriptions(subscriptions: SubscriptionSet) {
    let subscriptions = match subscriptions.unregister_and_wait() {
        Ok(()) => return,
        Err(error) => {
            for failure in error.failures() {
                logging::write(&format!(
                    "subscription {} failed to stop: {:?}",
                    failure.id(),
                    failure.result()
                ));
            }
            error.into_subscriptions()
        }
    };
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .subscriptions = subscriptions;
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

    let subscriptions = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        std::mem::take(&mut state.subscriptions)
    };
    if subscriptions.is_empty() {
        logging::write("shutdown completed before host registration");
        return TRUE;
    };

    match subscriptions.unregister_and_wait() {
        Ok(()) => {
            logging::write("shutdown completed; all callbacks quiesced");
            TRUE
        }
        Err(error) => {
            for failure in error.failures() {
                logging::write(&format!(
                    "subscription {} failed to stop: {:?}",
                    failure.id(),
                    failure.result()
                ));
            }
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .subscriptions = error.into_subscriptions();
            0
        }
    }
}
