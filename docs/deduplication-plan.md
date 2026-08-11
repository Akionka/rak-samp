# Deduplication Plan

Status: complete.

This refactor reduces repeated queue, lock, ABI-submission, default-value, and
R1 singleton plumbing. It intentionally does not generalize cache storage,
protocol codecs, ABI function tables, public facade wrappers, or
runtime/backend forwarding.

## Decisions

- Add `SampClientSdkResult::Busy = 14` and internal
  `DirectClientError::Busy`.
- Map public-path `Mutex::try_lock` failures precisely:
  `TryLockError::WouldBlock` becomes `Busy`; poisoning remains `NotReady`.
- Use `Busy` for all public request-producer and direct-cache-read contention.
  Actual queue-capacity exhaustion remains `QueueFull`; missing or unpublished
  cache data remains `NotReady`.
- Keep game-tick paths nonblocking. Contended internal drains and cache
  publications skip work for the current tick and expose no new result.
- Keep ABI version 1 under the current alpha compatibility policy. Plugins must
  be rebuilt for the new result variant; existing discriminants retain their
  current numeric values.
- Preserve queue FIFO order, duplicate suppression, capacities, per-pump
  budgets, cache refresh ordering, generation publication, and connection
  invalidation.

## Implementation design

### Lock and request helpers

Add one internal public-path try-lock helper that distinguishes `WouldBlock`
from `Poisoned`. Use it in every request producer and direct cache reader that
currently maps all `try_lock` errors to `QueueFull` or `NotReady`.

Replace producer repetition with a generic unique-enqueue helper accepting a
`Mutex<VecDeque<T>>`, capacity, and value. Keep named request methods so fields
and capacity constants remain explicit.

Replace the game-tick drain bodies with a generic helper accepting a queue and
per-pump limit. Keep named drain methods and their request-kind constants.
Failure to acquire the internal drain lock returns an empty snapshot without
removing queued values.

### Host command submissions

Add a private helper for receipt-bearing submissions whose runtime method
returns `Result<u64, DirectClientError>`. It owns initialized-runtime lookup,
submission, successful receipt write, and error mapping.

Exported wrappers must still reject a null receipt before reading other caller
pointers, validate and copy payloads before submission, and retain their exact
ABI signatures. Specialized network send/emulation helpers and command-receipt
poll/wait/release paths stay separate.

### Mechanical cleanup

Centralize `Default` for the zero-only ABI structures currently implementing it
manually: chat input text, dialog list item/snapshot/active dialog, local
player, player info, chat entry, textdraw, text label, server info, and
animation. Rust 1.87 does not implement `Default` for the large fixed arrays in
these types, so use one reviewed zero-validity macro instead of derives. Keep
manual defaults for command results and send options because their defaults are
semantic rather than all-zero.

Add one R1 singleton resolver accepting an RVA and minimum readable length.
Delegate dialog, chat, scoreboard, input, and death-window lookup to it while
preserving pointer reads, null rejection, and readable-range validation.

## Progress checklist

### Error contract

- [x] Add internal `DirectClientError::Busy`.
- [x] Add public `SampClientSdkResult::Busy = 14` and retryability docs.
- [x] Add the `WouldBlock`/poison classification helper.
- [x] Map `Busy` through the host ABI result conversion.
- [x] Update exhaustive direct-client error matches.

### Queue and cache paths

- [x] Introduce the generic unique-enqueue helper.
- [x] Convert all bounded cache-refresh request producers.
- [x] Introduce the generic bounded-drain helper.
- [x] Convert all request-kind drain methods.
- [x] Convert every public direct cache read to the shared lock classifier.
- [x] Keep opportunistic refresh failure from hiding an already-known value.

### Remaining deduplication

- [x] Add the shared host command-submission helper.
- [x] Convert all receipt-bearing `DirectClientError` command wrappers.
- [x] Consolidate zero-only ABI defaults and remove their repeated implementations.
- [x] Add and adopt the R1 singleton resolver.
- [x] Update `CORE.md`, `ARCHITECTURE.md`, and affected SDK API documentation.

### Verification

- [x] Test contended enqueue returns `Busy`.
- [x] Test a full queue still returns `QueueFull`.
- [x] Test poisoned public-path locks return `NotReady`.
- [x] Test contended direct cache reads return `Busy`.
- [x] Test absent cache entries remain `NotReady` and known values survive
      refresh-queue contention.
- [x] Test contended drains preserve queued values, FIFO order, deduplication,
      capacities, and per-pump limits.
- [x] Test host `Busy` mapping, validation order, and receipt IDs.
- [x] Verify zero-only defaults remain all-zero and ABI size/offset tests pass.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets --locked`.
- [x] Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [x] Run `cargo build --workspace --release --locked`.

## Completion criteria

All checklist items pass; public contention is distinguishable from queue
capacity and missing data; no queue, cache-generation, native-pointer, command,
wire-format, or ABI-layout regression is introduced.
