//! Host-visible game-thread command queue.
//!
//! The queue and its receipt/error types now live in `modkit-runtime`; this
//! module re-exports them so existing host call sites keep their paths.

pub(crate) use modkit_runtime::{CommandError, CommandId, CommandQueue, QueuedCommand};

#[cfg(test)]
pub(crate) use modkit_runtime::GAME_COMMAND_QUEUE_CAPACITY;
