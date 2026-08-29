//! Platform-neutral host runtime primitives.
//!
//! This crate owns the concurrency and lifecycle primitives that the host
//! runtime reuses across GTA and SA-MP services: the bounded command queue and
//! its receipts, callback activity tracking, non-blocking reclamation of
//! dropped callback state, and active-scope state for runtime-validated
//! game-thread contexts. It deliberately contains no native addresses and no
//! dependency on Windows, MinHook, or any game backend.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod callback;
pub mod command;
pub mod reclaim;
pub mod scope;

pub use callback::{
    CallbackContext, CallbackContextGuard, CallbackGate, CallbackGateGuard, DispatchGate,
    DispatchGuard,
};
pub use command::{
    CommandError, CommandId, CommandQueue, GAME_COMMAND_QUEUE_CAPACITY, QueuedCommand,
};
pub use reclaim::{DeferredReclamation, Reclaimable};
pub use scope::{GameThreadScope, GameThreadToken};
