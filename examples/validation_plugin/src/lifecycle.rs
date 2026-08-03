use crate::{
    callbacks, logging, reporting, self_test,
    state::{HOST_WAIT_TIMEOUT, STOP},
};
use rak_samp_plugin_api::{
    HostApi, RakSampDirection, RakSampHookAction, ResolveError, Subscription, events::Event,
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

#[derive(Clone, Copy)]
enum ListenerKind {
    Packet,
    Rpc,
}

type EventHandler = for<'event> fn(&mut Event<'event>) -> RakSampHookAction;

struct PluginState {
    subscriptions: Vec<Subscription>,
    initialization_worker: Option<JoinHandle<()>>,
    reporter_worker: Option<JoinHandle<()>>,
    self_test_worker: Option<JoinHandle<()>>,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
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

    let registrations: [(&str, ListenerKind, RakSampDirection, EventHandler); 6] = [
        (
            "incoming packet self-test rewriter",
            ListenerKind::Packet,
            RakSampDirection::Incoming,
            self_test::rewrite_test_packet,
        ),
        (
            "incoming RPC self-test rewriter",
            ListenerKind::Rpc,
            RakSampDirection::Incoming,
            self_test::rewrite_test_rpc,
        ),
        (
            "incoming packet",
            ListenerKind::Packet,
            RakSampDirection::Incoming,
            callbacks::on_incoming_packet,
        ),
        (
            "outgoing packet",
            ListenerKind::Packet,
            RakSampDirection::Outgoing,
            callbacks::on_outgoing_packet,
        ),
        (
            "incoming RPC",
            ListenerKind::Rpc,
            RakSampDirection::Incoming,
            callbacks::on_incoming_rpc,
        ),
        (
            "outgoing RPC",
            ListenerKind::Rpc,
            RakSampDirection::Outgoing,
            callbacks::on_outgoing_rpc,
        ),
    ];
    let mut subscriptions = Vec::with_capacity(registrations.len());
    for (label, kind, direction, callback) in registrations {
        let registration = match kind {
            ListenerKind::Packet => api.on_packet(direction, callback),
            ListenerKind::Rpc => api.on_rpc(direction, callback),
        };
        match registration {
            Ok(subscription) => subscriptions.push(subscription),
            Err(error) => {
                logging::write(&format!("{label} registration failed: {error:?}"));
                unregister_all(subscriptions);
                return;
            }
        }
    }

    {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutting_down {
            drop(state);
            unregister_all(subscriptions);
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
            unregister_all(subscriptions);
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

fn unregister_all(subscriptions: Vec<Subscription>) {
    for subscription in subscriptions {
        let _ = subscription.unregister_and_wait();
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

    let subscriptions = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        std::mem::take(&mut state.subscriptions)
    };
    if subscriptions.is_empty() {
        logging::write("shutdown completed before host registration");
        return TRUE;
    };

    let mut failed = Vec::new();
    for subscription in subscriptions {
        if let Err(error) = subscription.unregister_and_wait() {
            let result = error.result();
            let subscription = error.into_subscription();
            logging::write(&format!(
                "subscription {} failed to stop: {result:?}",
                subscription.id()
            ));
            failed.push(subscription);
        }
    }
    if failed.is_empty() {
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
