//! Scoped proof that direct native execution is currently permitted.

use modkit_abi::GameContextTokenV1;
use std::marker::PhantomData;

/// Callback-scoped proof for direct native service calls.
///
/// The host will construct this value when Phase 9 adds typed native callback
/// delivery. It cannot be created safely by plugins, sent to another thread,
/// or retained beyond the callback lifetime.
///
/// ```compile_fail
/// fn require_send<T: Send>() {}
/// require_send::<modkit_sdk::GameContext<'static>>();
/// ```
///
/// ```compile_fail
/// fn require_sync<T: Sync>() {}
/// require_sync::<modkit_sdk::GameContext<'static>>();
/// ```
pub struct GameContext<'scope> {
    #[allow(dead_code)] // Consumed by the Phase 9 callback and service adapters.
    token: GameContextTokenV1,
    _scope: PhantomData<&'scope mut ()>,
    _not_send_or_sync: PhantomData<*mut ()>,
}

impl<'scope> GameContext<'scope> {
    /// Wraps a host-issued callback token without extending its lifetime.
    ///
    /// # Safety
    ///
    /// `token` must be supplied by the host for the active callback. The
    /// returned value must not outlive that callback.
    #[must_use]
    pub unsafe fn from_raw(token: GameContextTokenV1) -> Self {
        Self {
            token,
            _scope: PhantomData,
            _not_send_or_sync: PhantomData,
        }
    }

    #[must_use]
    pub(crate) const fn token(&self) -> GameContextTokenV1 {
        self.token
    }
}

#[cfg(test)]
mod tests {
    use super::GameContext;
    use modkit_abi::GameContextTokenV1;
    use std::{marker::PhantomData, mem::size_of};

    #[test]
    fn context_is_a_thin_scoped_token() {
        assert_eq!(
            size_of::<GameContext<'_>>(),
            size_of::<GameContextTokenV1>()
        );

        let context = GameContext {
            token: unsafe { GameContextTokenV1::from_raw(7) },
            _scope: PhantomData,
            _not_send_or_sync: PhantomData,
        };
        assert_eq!(context.token.raw(), 7);
    }
}
