//! GTA-owned `CGame::Process` hook and tick orchestration.

use crate::GtaProfile;
use modkit_win32::InlineHook;
use std::{
    mem,
    sync::{
        Arc, Mutex, OnceLock, Weak,
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
    CreateHook,
    EnableHook,
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
}

static ACTIVE_RUNTIME: OnceLock<Mutex<Option<Weak<GameTickRuntimeInner>>>> = OnceLock::new();

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
            }),
        }
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
        let (mut hook, trampoline) = InlineHook::create(
            "CGame::Process",
            self.inner.profile.spec.game.process.get(),
            game_process_detour as *const () as usize,
        )
        .map_err(|_| GameTickInstallError::CreateHook)?;

        self.inner.trampoline.store(trampoline, Ordering::Release);
        *ACTIVE_RUNTIME
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(Arc::downgrade(&self.inner));

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

    /// Disables the hook and clears the published game-thread identity.
    pub fn shutdown(&self) {
        if let Some(hook) = self
            .inner
            .hook
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
        {
            hook.disable();
        }
        self.inner.trampoline.store(0, Ordering::Release);
        self.inner.game_thread_id.store(0, Ordering::Release);
        clear_active_runtime(&self.inner);
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
    let Some(runtime) = active_runtime() else {
        return;
    };
    let trampoline = runtime.inner.trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
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
    if let Some(participant) = runtime.registered_participant() {
        unsafe { runtime.run_tick(participant.as_ref(), original) };
    } else {
        unsafe { original() };
    }
}

fn active_runtime() -> Option<GameTickRuntime> {
    ACTIVE_RUNTIME.get().and_then(|slot| {
        slot.lock()
            .ok()
            .and_then(|runtime| runtime.as_ref().and_then(Weak::upgrade))
            .map(|inner| GameTickRuntime { inner })
    })
}

fn clear_active_runtime(target: &Arc<GameTickRuntimeInner>) {
    let Some(slot) = ACTIVE_RUNTIME.get() else {
        return;
    };
    let mut active = slot.lock().unwrap_or_else(|error| error.into_inner());
    if active
        .as_ref()
        .and_then(Weak::upgrade)
        .is_some_and(|runtime| Arc::ptr_eq(&runtime, target))
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
        let runtime = GameTickRuntime::new(GtaProfile::gta_sa_10_us());
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
