//! Active-scope state for runtime-validated game-thread contexts.
//!
//! A host issues a [`GameThreadToken`] only on its recorded game thread. Each
//! token also records the engine phase in which it was issued. Native services
//! can therefore reject stale, cross-thread, or wrong-phase use without
//! containing GTA or SA-MP-specific code in this crate.

use std::{
    collections::HashMap,
    sync::Mutex,
    thread::{self, ThreadId},
};

/// Engine phase in which a native execution scope was opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GamePhase {
    BeforeGameProcess,
    DuringGameProcess,
    PostGameProcess,
    Render,
}

/// Constraint that a native operation places on the supplied context token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeExecutionConstraint {
    GameThreadAnyPhase,
    PostGameProcessOnly,
    RenderPhaseOnly,
    QueuedOnly,
}

/// Reason why a native execution scope or token was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeError {
    Exhausted,
    Invalid,
    Stale,
    WrongThread,
    WrongPhase,
    ShuttingDown,
}

/// Opaque runtime token value. Zero is always invalid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScopeToken(u64);

impl ScopeToken {
    pub const INVALID: Self = Self(0);

    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Tracks the game thread and all currently active native execution scopes.
pub struct GameThreadScope {
    state: Mutex<GameThreadScopeState>,
}

struct GameThreadScopeState {
    game_thread: Option<ThreadId>,
    next_token: u64,
    active: HashMap<ScopeToken, ActiveScope>,
    shutting_down: bool,
}

#[derive(Clone, Copy)]
struct ActiveScope {
    owner_thread: ThreadId,
    phase: GamePhase,
}

impl GameThreadScope {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(GameThreadScopeState {
                game_thread: None,
                next_token: 1,
                active: HashMap::new(),
                shutting_down: false,
            }),
        }
    }

    /// Records the calling thread as the host game thread.
    pub fn set_current_as_game_thread(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let current_thread = thread::current().id();
        if state.game_thread != Some(current_thread) {
            state.active.clear();
        }
        state.game_thread = Some(current_thread);
        state.shutting_down = false;
    }

    /// Invalidates all active tokens and forgets the game thread.
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.game_thread = None;
        state.active.clear();
    }

    /// Rejects new scopes and invalidates every active token.
    pub fn shutdown(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        state.game_thread = None;
        state.active.clear();
    }

    /// Opens a runtime-validated scope for the current engine phase.
    pub fn enter(&self, phase: GamePhase) -> Result<GameThreadToken<'_>, ScopeError> {
        let owner_thread = thread::current().id();
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutting_down {
            return Err(ScopeError::ShuttingDown);
        }
        if state.game_thread != Some(owner_thread) {
            return Err(ScopeError::WrongThread);
        }

        let raw = state.next_token;
        if raw == 0 {
            return Err(ScopeError::Exhausted);
        }
        state.next_token = raw.checked_add(1).unwrap_or(0);
        let token = ScopeToken(raw);
        state.active.insert(
            token,
            ActiveScope {
                owner_thread,
                phase,
            },
        );
        Ok(GameThreadToken { scope: self, token })
    }

    /// Validates token lifetime, thread ownership, and operation phase.
    pub fn validate(
        &self,
        token: ScopeToken,
        constraint: NativeExecutionConstraint,
    ) -> Result<(), ScopeError> {
        if token == ScopeToken::INVALID {
            return Err(ScopeError::Invalid);
        }

        let current_thread = thread::current().id();
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutting_down {
            return Err(ScopeError::ShuttingDown);
        }
        let active = state.active.get(&token).ok_or(ScopeError::Stale)?;
        if active.owner_thread != current_thread {
            return Err(ScopeError::WrongThread);
        }

        match constraint {
            NativeExecutionConstraint::GameThreadAnyPhase => Ok(()),
            NativeExecutionConstraint::PostGameProcessOnly
                if active.phase == GamePhase::PostGameProcess =>
            {
                Ok(())
            }
            NativeExecutionConstraint::RenderPhaseOnly if active.phase == GamePhase::Render => {
                Ok(())
            }
            NativeExecutionConstraint::QueuedOnly => Err(ScopeError::WrongPhase),
            NativeExecutionConstraint::PostGameProcessOnly
            | NativeExecutionConstraint::RenderPhaseOnly => Err(ScopeError::WrongPhase),
        }
    }

    #[must_use]
    pub fn is_active_on_current_thread(&self) -> bool {
        let current_thread = thread::current().id();
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state
            .active
            .values()
            .any(|active| active.owner_thread == current_thread)
    }

    #[cfg(test)]
    fn set_next_token(&self, next_token: u64) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .next_token = next_token;
    }
}

impl Default for GameThreadScope {
    fn default() -> Self {
        Self::new()
    }
}

/// An active native execution scope. Dropping it invalidates its token.
#[must_use]
pub struct GameThreadToken<'scope> {
    scope: &'scope GameThreadScope,
    token: ScopeToken,
}

impl GameThreadToken<'_> {
    #[must_use]
    pub const fn token(&self) -> ScopeToken {
        self.token
    }
}

impl Drop for GameThreadToken<'_> {
    fn drop(&mut self) {
        self.scope
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .active
            .remove(&self.token);
    }
}

#[cfg(test)]
mod tests {
    use super::{GamePhase, GameThreadScope, NativeExecutionConstraint, ScopeError, ScopeToken};
    use std::{sync::Arc, thread};

    #[test]
    fn validates_active_token_and_phase() {
        let scope = GameThreadScope::new();
        scope.set_current_as_game_thread();
        let token = scope.enter(GamePhase::PostGameProcess).unwrap();

        assert!(scope.is_active_on_current_thread());
        assert_eq!(
            scope.validate(token.token(), NativeExecutionConstraint::GameThreadAnyPhase),
            Ok(())
        );
        assert_eq!(
            scope.validate(
                token.token(),
                NativeExecutionConstraint::PostGameProcessOnly
            ),
            Ok(())
        );
        assert_eq!(
            scope.validate(token.token(), NativeExecutionConstraint::RenderPhaseOnly),
            Err(ScopeError::WrongPhase)
        );
        assert_eq!(
            scope.validate(token.token(), NativeExecutionConstraint::QueuedOnly),
            Err(ScopeError::WrongPhase)
        );
    }

    #[test]
    fn rejects_invalid_stale_and_cross_thread_tokens() {
        let scope = Arc::new(GameThreadScope::new());
        scope.set_current_as_game_thread();
        let token = scope.enter(GamePhase::Render).unwrap();
        let raw = token.token();

        assert_eq!(
            scope.validate(
                ScopeToken::INVALID,
                NativeExecutionConstraint::GameThreadAnyPhase
            ),
            Err(ScopeError::Invalid)
        );

        let other = Arc::clone(&scope);
        assert_eq!(
            thread::spawn(move || {
                other.validate(raw, NativeExecutionConstraint::GameThreadAnyPhase)
            })
            .join()
            .unwrap(),
            Err(ScopeError::WrongThread)
        );

        drop(token);
        assert_eq!(
            scope.validate(raw, NativeExecutionConstraint::GameThreadAnyPhase),
            Err(ScopeError::Stale)
        );
    }

    #[test]
    fn nested_tokens_are_independent() {
        let scope = GameThreadScope::new();
        scope.set_current_as_game_thread();
        let outer = scope.enter(GamePhase::DuringGameProcess).unwrap();
        let inner = scope.enter(GamePhase::Render).unwrap();

        drop(inner);
        assert_eq!(
            scope.validate(outer.token(), NativeExecutionConstraint::GameThreadAnyPhase),
            Ok(())
        );
        drop(outer);
        assert!(!scope.is_active_on_current_thread());
    }

    #[test]
    fn clear_and_shutdown_make_late_drop_harmless() {
        let scope = GameThreadScope::new();
        scope.set_current_as_game_thread();
        let cleared = scope.enter(GamePhase::BeforeGameProcess).unwrap();
        let cleared_raw = cleared.token();
        scope.clear();
        assert_eq!(
            scope.validate(cleared_raw, NativeExecutionConstraint::GameThreadAnyPhase),
            Err(ScopeError::Stale)
        );
        drop(cleared);

        scope.set_current_as_game_thread();
        let shutdown = scope.enter(GamePhase::BeforeGameProcess).unwrap();
        scope.shutdown();
        assert_eq!(
            scope.validate(
                shutdown.token(),
                NativeExecutionConstraint::GameThreadAnyPhase
            ),
            Err(ScopeError::ShuttingDown)
        );
        drop(shutdown);
        assert!(matches!(
            scope.enter(GamePhase::BeforeGameProcess),
            Err(ScopeError::ShuttingDown)
        ));
    }

    #[test]
    fn token_ids_never_wrap_or_reuse_zero() {
        let scope = GameThreadScope::new();
        scope.set_current_as_game_thread();
        scope.set_next_token(u64::MAX);
        let last = scope.enter(GamePhase::Render).unwrap();
        assert_eq!(last.token().raw(), u64::MAX);
        assert!(matches!(
            scope.enter(GamePhase::Render),
            Err(ScopeError::Exhausted)
        ));
    }
}
