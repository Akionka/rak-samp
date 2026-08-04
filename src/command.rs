//! Bounded, host-owned game-thread command queue.
//!
//! This module owns the concurrency semantics that the Windows game-process
//! hook will use. It deliberately contains no native calls: producers submit
//! fully owned commands, a tick takes one FIFO snapshot, and the game thread
//! publishes a completion for the corresponding receipt.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Condvar, Mutex},
    time::Duration,
};

pub(crate) const GAME_COMMAND_QUEUE_CAPACITY: usize = 256;

pub(crate) type CommandId = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommandError {
    QueueFull,
    ShuttingDown,
    NativeFailure,
    UnknownReceipt,
    #[allow(dead_code)] // Consumed by the pending C ABI receipt poll/wait slots.
    TimedOut,
    #[allow(dead_code)] // Produced by the pending game-thread/callback wait guards.
    WaitRejected,
}

#[derive(Debug)]
pub(crate) struct QueuedCommand<C> {
    pub(crate) id: CommandId,
    pub(crate) command: C,
}

pub(crate) struct CommandQueue<C, R> {
    state: Mutex<CommandQueueState<C, R>>,
    completion: Condvar,
}

struct CommandQueueState<C, R> {
    next_id: CommandId,
    commands: VecDeque<QueuedCommand<C>>,
    receipts: HashSet<CommandId>,
    completions: HashMap<CommandId, Result<R, CommandError>>,
    shutting_down: bool,
}

impl<C, R> Default for CommandQueue<C, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, R> CommandQueue<C, R> {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(CommandQueueState {
                next_id: 1,
                commands: VecDeque::with_capacity(GAME_COMMAND_QUEUE_CAPACITY),
                receipts: HashSet::new(),
                completions: HashMap::new(),
                shutting_down: false,
            }),
            completion: Condvar::new(),
        }
    }

    pub(crate) fn submit(&self, command: C) -> Result<CommandId, CommandError> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.shutting_down {
            return Err(CommandError::ShuttingDown);
        }
        if state.commands.len() == GAME_COMMAND_QUEUE_CAPACITY {
            return Err(CommandError::QueueFull);
        }

        let id = next_command_id(&mut state);
        state.receipts.insert(id);
        state.commands.push_back(QueuedCommand { id, command });
        Ok(id)
    }

    /// Takes every command accepted before this lock acquisition. Producers
    /// that submit after the snapshot is taken remain queued for the next tick.
    pub(crate) fn take_tick_snapshot(&self) -> Vec<QueuedCommand<C>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.commands.drain(..).collect()
    }

    /// Completes an accepted command. A detached or shutdown-completed receipt
    /// intentionally discards this late result without cancelling execution.
    pub(crate) fn complete(&self, id: CommandId, result: Result<R, CommandError>) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.receipts.contains(&id) && !state.completions.contains_key(&id) {
            state.completions.insert(id, result);
            self.completion.notify_all();
        }
    }

    #[allow(dead_code)] // Called by the pending C ABI receipt-poll slot.
    pub(crate) fn try_take(
        &self,
        id: CommandId,
    ) -> Result<Option<Result<R, CommandError>>, CommandError> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if !state.receipts.contains(&id) {
            return Err(CommandError::UnknownReceipt);
        }
        if let Some(result) = state.completions.remove(&id) {
            state.receipts.remove(&id);
            return Ok(Some(result));
        }
        Ok(None)
    }

    /// Waits for one completion. A timeout deliberately leaves the receipt
    /// intact so callers can poll or wait again.
    #[allow(dead_code)] // Called by the pending C ABI timed-wait slot.
    pub(crate) fn wait(
        &self,
        id: CommandId,
        timeout: Duration,
        wait_allowed: bool,
    ) -> Result<Result<R, CommandError>, CommandError> {
        if !wait_allowed {
            return Err(CommandError::WaitRejected);
        }

        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if !state.receipts.contains(&id) {
            return Err(CommandError::UnknownReceipt);
        }
        let (mut state, timeout_result) = self
            .completion
            .wait_timeout_while(state, timeout, |state| {
                state.receipts.contains(&id) && !state.completions.contains_key(&id)
            })
            .unwrap_or_else(|error| error.into_inner());
        if let Some(result) = state.completions.remove(&id) {
            state.receipts.remove(&id);
            return Ok(result);
        }
        debug_assert!(timeout_result.timed_out());
        Err(CommandError::TimedOut)
    }

    /// Drops the caller's receipt without cancelling queued or in-flight work.
    pub(crate) fn detach(&self, id: CommandId) -> Result<(), CommandError> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if !state.receipts.remove(&id) {
            return Err(CommandError::UnknownReceipt);
        }
        state.completions.remove(&id);
        Ok(())
    }

    /// Rejects further submissions and completes every attached receipt, even
    /// if a command has already left the tick snapshot for native execution.
    pub(crate) fn shutdown(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        state.commands.clear();
        let outstanding = state.receipts.iter().copied().collect::<Vec<_>>();
        for id in outstanding {
            state
                .completions
                .entry(id)
                .or_insert(Err(CommandError::ShuttingDown));
        }
        self.completion.notify_all();
    }

    #[cfg(test)]
    fn queued_len(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .commands
            .len()
    }
}

fn next_command_id<C, R>(state: &mut CommandQueueState<C, R>) -> CommandId {
    loop {
        let id = state.next_id;
        state.next_id = state.next_id.wrapping_add(1).max(1);
        if !state.receipts.contains(&id) {
            return id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandError, CommandQueue, GAME_COMMAND_QUEUE_CAPACITY};
    use std::time::Duration;

    #[test]
    fn tick_snapshot_is_fifo_and_defers_new_commands() {
        let queue = CommandQueue::<u8, u8>::new();
        let first = queue.submit(1).unwrap();
        let second = queue.submit(2).unwrap();

        let first_tick = queue.take_tick_snapshot();
        let third = queue.submit(3).unwrap();
        let second_tick = queue.take_tick_snapshot();

        assert_eq!(
            first_tick
                .into_iter()
                .map(|command| (command.id, command.command))
                .collect::<Vec<_>>(),
            vec![(first, 1), (second, 2)]
        );
        assert_eq!(
            second_tick
                .into_iter()
                .map(|command| (command.id, command.command))
                .collect::<Vec<_>>(),
            vec![(third, 3)]
        );
    }

    #[test]
    fn queue_enforces_the_shared_capacity() {
        let queue = CommandQueue::<u8, ()>::new();
        for value in 0..GAME_COMMAND_QUEUE_CAPACITY {
            queue.submit(value as u8).unwrap();
        }

        assert_eq!(queue.queued_len(), GAME_COMMAND_QUEUE_CAPACITY);
        assert_eq!(queue.submit(0), Err(CommandError::QueueFull));
    }

    #[test]
    fn timeout_keeps_the_receipt_for_retry() {
        let queue = CommandQueue::<(), u8>::new();
        let id = queue.submit(()).unwrap();

        assert_eq!(
            queue.wait(id, Duration::ZERO, true),
            Err(CommandError::TimedOut)
        );
        queue.complete(id, Ok(42));
        assert_eq!(queue.wait(id, Duration::ZERO, true), Ok(Ok(42)));
    }

    #[test]
    fn detached_receipt_does_not_cancel_execution() {
        let queue = CommandQueue::<u8, ()>::new();
        let id = queue.submit(9).unwrap();
        assert_eq!(queue.detach(id), Ok(()));

        let snapshot = queue.take_tick_snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].command, 9);
        queue.complete(id, Ok(()));
        assert_eq!(queue.try_take(id), Err(CommandError::UnknownReceipt));
    }

    #[test]
    fn waits_can_be_rejected_without_touching_the_receipt() {
        let queue = CommandQueue::<(), ()>::new();
        let id = queue.submit(()).unwrap();

        assert_eq!(
            queue.wait(id, Duration::ZERO, false),
            Err(CommandError::WaitRejected)
        );
        assert_eq!(queue.try_take(id), Ok(None));
    }

    #[test]
    fn shutdown_completes_queued_and_in_flight_receipts() {
        let queue = CommandQueue::<u8, ()>::new();
        let queued = queue.submit(1).unwrap();
        let in_flight = queue.submit(2).unwrap();
        let snapshot = queue.take_tick_snapshot();
        assert_eq!(snapshot.len(), 2);

        queue.shutdown();

        assert_eq!(
            queue.try_take(queued),
            Ok(Some(Err(CommandError::ShuttingDown)))
        );
        assert_eq!(
            queue.try_take(in_flight),
            Ok(Some(Err(CommandError::ShuttingDown)))
        );
        assert_eq!(queue.submit(3), Err(CommandError::ShuttingDown));
    }
}
