//! Active-scope state for runtime-validated game-thread contexts.
//!
//! A host issues a [`GameThreadToken`] only after validating that the current
//! thread is the game thread. The token proves thread confinement, not a
//! universal engine phase; each native operation records its own execution
//! constraint. This state is generic and contains no GTA or SA-MP addresses.

use std::{
    sync::Mutex,
    thread::{self, ThreadId},
};

/// Tracks the validated game thread and the currently open scopes on it.
pub struct GameThreadScope {
    state: Mutex<GameThreadScopeState>,
}

#[derive(Default)]
struct GameThreadScopeState {
    game_thread: Option<ThreadId>,
    open_scopes: usize,
}

impl GameThreadScope {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(GameThreadScopeState::default()),
        }
    }

    /// Records the game thread once the host has identified it.
    pub fn set_game_thread(&self, id: ThreadId) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .game_thread = Some(id);
    }

    /// Clears the recorded game thread, e.g. during shutdown.
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.game_thread = None;
        state.open_scopes = 0;
    }

    /// Reports whether the current thread is inside an open game-thread scope.
    #[must_use]
    pub fn is_active_on_current_thread(&self) -> bool {
        let current = thread::current().id();
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.game_thread == Some(current) && state.open_scopes != 0
    }

    /// Validates the current thread and opens a scope, returning a token.
    ///
    /// Returns `None` when the current thread is not the recorded game thread.
    pub fn enter(&self) -> Option<GameThreadToken<'_>> {
        let current = thread::current().id();
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.game_thread != Some(current) {
            return None;
        }
        state.open_scopes += 1;
        Some(GameThreadToken { scope: self })
    }
}

impl Default for GameThreadScope {
    fn default() -> Self {
        Self::new()
    }
}

/// A validated game-thread scope. Closes the scope when dropped.
#[must_use]
pub struct GameThreadToken<'scope> {
    scope: &'scope GameThreadScope,
}

impl Drop for GameThreadToken<'_> {
    fn drop(&mut self) {
        let mut state = self
            .scope
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        debug_assert!(state.open_scopes != 0);
        state.open_scopes -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::GameThreadScope;
    use std::thread;

    #[test]
    fn token_is_issued_only_on_the_validated_game_thread() {
        let scope = GameThreadScope::new();
        let game_thread = thread::current().id();
        scope.set_game_thread(game_thread);

        let _token = scope.enter().unwrap();
        assert!(scope.is_active_on_current_thread());
    }

    #[test]
    fn token_is_rejected_on_an_unvalidated_thread() {
        let scope = GameThreadScope::new();
        let game_thread = thread::current().id();
        scope.set_game_thread(game_thread);

        let handle = thread::spawn(move || scope.enter().is_none());
        assert!(handle.join().unwrap());
    }

    #[test]
    fn scope_is_inactive_before_validation_and_after_clear() {
        let scope = GameThreadScope::new();
        assert!(!scope.is_active_on_current_thread());
        assert!(scope.enter().is_none());

        scope.set_game_thread(thread::current().id());
        let token = scope.enter().unwrap();
        assert!(scope.is_active_on_current_thread());
        drop(token);
        assert!(!scope.is_active_on_current_thread());

        scope.set_game_thread(thread::current().id());
        let token = scope.enter().unwrap();
        assert!(scope.is_active_on_current_thread());
        drop(token);
        scope.clear();
        assert!(!scope.is_active_on_current_thread());
    }
}
