//! GTA-owned `CGame::Process` hook and tick orchestration.

use crate::GtaProfile;
use modkit_win32::InlineHook;
use std::{
    mem,
    sync::{
        Arc, Condvar, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    },
};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

/// Verified zero-argument ABI of `CGame::Process` for the supported target.
pub type GameProcessFn = unsafe extern "C" fn();

/// Host-internal work that brackets the original `CGame::Process` call.
pub trait GameTickParticipant: Send + Sync {
    /// Captures work accepted before this tick's original native call.
    fn before_game_process(&self) {}

    /// Runs captured work after the original native call returns.
    fn after_game_process(&self) {}
}

/// Failure to create or enable the GTA game-process hook.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameTickInstallError {
    AlreadyInstalled,
    CreateHook,
    EnableHook,
}

/// Failure to stop and remove the GTA game-process hook safely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameTickShutdownError {
    CalledFromGameThread,
    DisableHook,
    RemoveHook,
}

/// Owns the GTA game-process hook, trampoline, thread identity, and phases.
#[derive(Clone)]
pub struct GameTickRuntime {
    inner: Arc<GameTickRuntimeInner>,
}

struct GameTickRuntimeInner {
    profile: GtaProfile,
    participant: Mutex<Option<Weak<dyn GameTickParticipant>>>,
    hook: Mutex<Option<InlineHook>>,
    trampoline: AtomicUsize,
    game_thread_id: AtomicU32,
    detour_diagnostic_logged: AtomicBool,
    stopping: AtomicBool,
    in_flight: Mutex<usize>,
    idle: Condvar,
}

static ACTIVE_RUNTIME: OnceLock<Mutex<Option<Arc<GameTickRuntimeInner>>>> = OnceLock::new();

impl GameTickRuntime {
    #[must_use]
    pub fn new(profile: GtaProfile) -> Self {
        Self {
            inner: Arc::new(GameTickRuntimeInner {
                profile,
                participant: Mutex::new(None),
                hook: Mutex::new(None),
                trampoline: AtomicUsize::new(0),
                game_thread_id: AtomicU32::new(0),
                detour_diagnostic_logged: AtomicBool::new(false),
                stopping: AtomicBool::new(false),
                in_flight: Mutex::new(0),
                idle: Condvar::new(),
            }),
        }
    }

    /// Returns the verified GTA profile selected for this runtime.
    #[must_use]
    pub fn profile(&self) -> GtaProfile {
        self.inner.profile
    }

    /// Registers the host-owned participant driven by the GTA tick.
    pub fn register_participant(&self, participant: Weak<dyn GameTickParticipant>) {
        *self
            .inner
            .participant
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(participant);
    }

    /// Installs and enables the GTA `CGame::Process` detour.
    pub fn install(&self) -> Result<(), GameTickInstallError> {
        let active_slot = ACTIVE_RUNTIME.get_or_init(|| Mutex::new(None));
        if active_slot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_some()
        {
            return Err(GameTickInstallError::AlreadyInstalled);
        }
        let (mut hook, trampoline) = unsafe {
            InlineHook::create(
                "CGame::Process",
                self.inner.profile.spec.game.process.get(),
                game_process_detour as *const () as usize,
            )
        }
        .map_err(|_| GameTickInstallError::CreateHook)?;

        self.inner.stopping.store(false, Ordering::Release);
        self.inner.trampoline.store(trampoline, Ordering::Release);
        let mut active = active_slot
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if active.is_some() {
            self.inner.trampoline.store(0, Ordering::Release);
            return Err(GameTickInstallError::AlreadyInstalled);
        }
        *active = Some(Arc::clone(&self.inner));
        drop(active);

        if hook.enable().is_err() {
            self.inner.trampoline.store(0, Ordering::Release);
            clear_active_runtime(&self.inner);
            return Err(GameTickInstallError::EnableHook);
        }

        *self
            .inner
            .hook
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(hook);
        Ok(())
    }

    /// Disables and removes the hook after all active detours have returned.
    pub fn shutdown(&self) -> Result<(), GameTickShutdownError> {
        self.inner.stopping.store(true, Ordering::Release);
        if self.is_game_thread() {
            return Err(GameTickShutdownError::CalledFromGameThread);
        }

        let mut hook_slot = self
            .inner
            .hook
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(hook) = hook_slot.as_mut() {
            hook.try_disable()
                .map_err(|_| GameTickShutdownError::DisableHook)?;

            let mut in_flight = self
                .inner
                .in_flight
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            while *in_flight != 0 {
                in_flight = self
                    .inner
                    .idle
                    .wait(in_flight)
                    .unwrap_or_else(|error| error.into_inner());
            }
            drop(in_flight);

            hook.try_remove()
                .map_err(|_| GameTickShutdownError::RemoveHook)?;
            hook_slot.take();
        }
        self.inner.trampoline.store(0, Ordering::Release);
        self.inner.game_thread_id.store(0, Ordering::Release);
        clear_active_runtime(&self.inner);
        Ok(())
    }

    /// Publishes the current thread as the observed GTA game thread.
    pub fn mark_current_game_thread(&self) {
        self.inner
            .game_thread_id
            .store(unsafe { GetCurrentThreadId() }, Ordering::Release);
    }

    /// Reports whether the caller is the observed GTA game thread.
    #[must_use]
    pub fn is_game_thread(&self) -> bool {
        let game_thread = self.inner.game_thread_id.load(Ordering::Acquire);
        game_thread != 0 && game_thread == unsafe { GetCurrentThreadId() }
    }

    /// Runs one ordered game-process tick with an explicit participant.
    ///
    /// # Safety
    ///
    /// `original` must be the captured `CGame::Process` trampoline with the
    /// declared ABI and must remain callable for the duration of this call.
    pub unsafe fn run_tick(&self, participant: &dyn GameTickParticipant, original: GameProcessFn) {
        self.mark_current_game_thread();
        participant.before_game_process();
        unsafe { original() };
        participant.after_game_process();
    }

    fn registered_participant(&self) -> Option<Arc<dyn GameTickParticipant>> {
        self.inner
            .participant
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .and_then(Weak::upgrade)
    }
}

unsafe extern "C" fn game_process_detour() {
    let Some(inner) = active_runtime() else {
        return;
    };
    let activity = DetourActivity::enter(Arc::clone(&inner));
    let runtime = GameTickRuntime { inner };
    let trampoline = runtime.inner.trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        log::error!("CGame::Process detour has no published trampoline");
        return;
    }
    if !runtime
        .inner
        .detour_diagnostic_logged
        .swap(true, Ordering::AcqRel)
    {
        log::debug!("entered CGame::Process detour for the first time");
    }

    let original: GameProcessFn = unsafe { mem::transmute(trampoline) };
    if runtime.inner.stopping.load(Ordering::Acquire) {
        unsafe { original() };
    } else if let Some(participant) = runtime.registered_participant() {
        unsafe { runtime.run_tick(participant.as_ref(), original) };
    } else {
        unsafe { original() };
    }
    drop(activity);
}

struct DetourActivity {
    inner: Arc<GameTickRuntimeInner>,
}

impl DetourActivity {
    fn enter(inner: Arc<GameTickRuntimeInner>) -> Self {
        let mut in_flight = inner
            .in_flight
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *in_flight += 1;
        drop(in_flight);
        Self { inner }
    }
}

impl Drop for DetourActivity {
    fn drop(&mut self) {
        let mut in_flight = self
            .inner
            .in_flight
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *in_flight -= 1;
        if *in_flight == 0 {
            self.inner.idle.notify_all();
        }
    }
}

fn active_runtime() -> Option<Arc<GameTickRuntimeInner>> {
    ACTIVE_RUNTIME.get().and_then(|slot| {
        slot.lock()
            .ok()
            .and_then(|runtime| runtime.as_ref().map(Arc::clone))
    })
}

fn clear_active_runtime(target: &Arc<GameTickRuntimeInner>) {
    let Some(slot) = ACTIVE_RUNTIME.get() else {
        return;
    };
    let mut active = slot.lock().unwrap_or_else(|error| error.into_inner());
    if active
        .as_ref()
        .is_some_and(|runtime| Arc::ptr_eq(runtime, target))
    {
        *active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static ORIGINAL_CALLS: AtomicU32 = AtomicU32::new(0);

    unsafe extern "C" fn fake_original() {
        ORIGINAL_CALLS.fetch_add(1, Ordering::AcqRel);
    }

    struct CountingParticipant {
        before: AtomicU32,
        after: AtomicU32,
    }

    impl GameTickParticipant for CountingParticipant {
        fn before_game_process(&self) {
            self.before.fetch_add(1, Ordering::AcqRel);
        }

        fn after_game_process(&self) {
            self.after.fetch_add(1, Ordering::AcqRel);
        }
    }

    #[test]
    fn tick_marks_thread_calls_original_once_and_runs_participant() {
        let runtime = GameTickRuntime::new(
            GtaProfile::select(0x0040_0000, crate::GTA_SA_10_US_SHA256).unwrap(),
        );
        let participant = CountingParticipant {
            before: AtomicU32::new(0),
            after: AtomicU32::new(0),
        };
        ORIGINAL_CALLS.store(0, Ordering::Release);

        unsafe { runtime.run_tick(&participant, fake_original) };

        assert!(runtime.is_game_thread());
        assert_eq!(ORIGINAL_CALLS.load(Ordering::Acquire), 1);
        assert_eq!(participant.before.load(Ordering::Acquire), 1);
        assert_eq!(participant.after.load(Ordering::Acquire), 1);
    }
}
