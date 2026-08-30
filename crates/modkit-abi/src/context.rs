//! Stable scalar types for runtime-validated native execution contexts.

/// Opaque token proving that a native execution scope is currently active.
///
/// The host owns token issuance and validation. Plugins must not synthesize
/// token values or retain them after the callback that supplied the context.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GameContextTokenV1(u64);

impl GameContextTokenV1 {
    pub const INVALID: Self = Self(0);

    /// Reconstructs a token received through the raw ABI.
    ///
    /// # Safety
    ///
    /// `raw` must have been issued by the active host for the current process.
    /// This does not extend the token's callback scope.
    #[must_use]
    pub const unsafe fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

/// Native-operation execution constraint declared by a service method.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct NativeExecutionConstraintV1(u32);

impl NativeExecutionConstraintV1 {
    pub const GAME_THREAD_ANY_PHASE: Self = Self(0);
    pub const POST_GAME_PROCESS_ONLY: Self = Self(1);
    pub const RENDER_PHASE_ONLY: Self = Self(2);
    pub const QUEUED_ONLY: Self = Self(3);

    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{GameContextTokenV1, NativeExecutionConstraintV1};
    use std::mem::{align_of, size_of};

    #[test]
    fn context_token_has_stable_scalar_layout() {
        assert_eq!(size_of::<GameContextTokenV1>(), size_of::<u64>());
        assert_eq!(align_of::<GameContextTokenV1>(), align_of::<u64>());
        assert!(!GameContextTokenV1::INVALID.is_valid());
        assert!(unsafe { GameContextTokenV1::from_raw(1) }.is_valid());
    }

    #[test]
    fn execution_constraint_has_stable_scalar_layout() {
        assert_eq!(size_of::<NativeExecutionConstraintV1>(), size_of::<u32>());
        assert_eq!(align_of::<NativeExecutionConstraintV1>(), align_of::<u32>());
        assert_eq!(NativeExecutionConstraintV1::GAME_THREAD_ANY_PHASE.raw(), 0);
        assert_eq!(NativeExecutionConstraintV1::POST_GAME_PROCESS_ONLY.raw(), 1);
        assert_eq!(NativeExecutionConstraintV1::RENDER_PHASE_ONLY.raw(), 2);
        assert_eq!(NativeExecutionConstraintV1::QUEUED_ONLY.raw(), 3);
    }
}
