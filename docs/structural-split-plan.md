# Structural Split Follow-up Plan

Status: complete (verified 2026-08-11).

Historical note: references to `events/rpc/incoming/fixed.rs` below describe
the layout reviewed in this completed split. Current production code uses the
semantic `common` and `r1` taxonomy; the historical path is not a supported
public namespace.

This is a follow-up to the completed [module split plan](SPLIT_PLAN.md). The
first pass established domain modules but left several composition roots and
native bridges very large. This pass moves stable implementation and test
groups without decomposing shared runtime state or changing behavior.

## Goals and guardrails

- Use 1,500 nonblank lines as a soft ceiling for actively maintained source
  files. Document exceptions where one cohesive catalog is clearer intact.
- Preserve public SDK paths through root re-exports.
- Preserve every `repr(C)` layout, discriminant, ABI-table field order,
  exported symbol, fixed offset, and native calling convention.
- Preserve error mapping, validation order, command and lock ordering, queue
  bounds, cache generations, connection invalidation, and hook lifecycle.
- Keep `BackendState` fields and game-tick orchestration centralized. Move
  concern-specific `impl` blocks rather than splitting shared state.
- Make every extraction an independently buildable, reviewable change.
- Do not combine source movement with deduplication or behavior changes.

## Baseline

Current counts use nonblank lines; physical lines are included for comparison.

| File | Nonblank | Physical | Disposition |
| --- | ---: | ---: | --- |
| `src/platform/win32/mod.rs` | 4,827 | 5,223 | Split implementation and tests; retain state/tick root |
| `sdk/src/lib.rs` | 3,717 | 3,950 | Split types, ABI, API ownership, subscriptions, resolution, tests |
| `src/platform/win32/r1_client.rs` | 3,019 | 3,135 | Split fixed-offset operations by native domain |
| `sdk/src/events/test_support.rs` | 1,784 | 1,916 | Split mock state, API table, callbacks, fixture builders |
| `src/host_api/mod.rs` | 1,217 | 1,267 | Extract shared helpers, listeners, and tests |
| `sdk/src/events/rpc/incoming/fixed.rs` | 1,163 | 1,297 | Keep intact by default; protocol-catalog exception |

Before moving code, record the current test/build results and host DLL exports.
After each slice run formatting, workspace tests, strict Clippy, and the
workspace release build.

## Latest LOC review (2026-08-11)

All actively maintained Rust files now meet the 1,500-nonblank-line soft
ceiling. The largest are `sdk/src/events/test_support.rs` (1,440 nonblank,
1,550 physical), `sdk/src/tests.rs` (1,403/1,470), and
`src/platform/win32/mod.rs` (1,278/1,363). `sdk/src/events/rpc/incoming/
fixed.rs` (1,163/1,297) remains intact: its descriptor/encoder/decoder catalog
is a cohesive protocol exception, not a generic implementation root.

The 2026-08-11 release artifact export audit reports exactly `DllMain` and
`SampClientSdk_GetApiV1`; both declarations remain at their original host
composition roots.

## Target boundaries

### Tests first

The implementation boundaries from the first split are now stable, so extract
large inline test modules before further production moves:

- SDK root tests: ABI/layout, defaults, host API, subscriptions, and resolution.
- Win32 tests: backend/cache queues, native packets/layout, hooks, and lifecycle.
- R1 tests: memory/layout fixture and pure parsing helpers.
- Host API tests: ABI table, result mapping, listeners, and command receipts.

Keep cross-module invariant tests at the nearest composition root. Do not turn
unit tests into external integration tests when they require private state.

### SDK root

Keep `sdk/src/lib.rs` as the public facade and re-export layer.

- `types.rs`: safe public value types and enums.
- `abi.rs`: result/status enums, `repr(C)` structures, callback types, and the
  complete `SampClientSdkApiV1` declaration.
- `api.rs`: `HostApi`, `CommandReceipt`, shared ABI-to-safe conversions, and
  typed wrapper glue not owned by existing `host_api/` domain modules.
- `subscriptions.rs`: callback state, subscription ownership, sets, and errors.
- `resolve.rs`: host discovery, wait functions, and `ResolveError`.

Move declarations without renaming them. Root re-exports must keep current
consumer paths compiling.

### R1 client bridge

Keep `R1ClientProfile`, profile verification, and cross-domain entry points in
`r1_client.rs`. Extract in increasing dependency order:

1. `singletons.rs`: validated dialog/chat/scoreboard/input/death-window lookup.
2. `native_types.rs`: narrow native function-pointer aliases and shared mirrors.
3. `textdraws.rs`: textdraw pool reads and mutations.
4. `ui.rs`: dialog, chat, input, scoreboard, cursor, and death-window operations.
5. `players.rs`: player reads/mutations and local-player operations.
6. `pools.rs`: vehicle, label, object, pickup, and gangzone pool operations.
7. `handles.rs`: forward and reverse GTA-handle lookups.

Keep fixed-offset checks and readable-range validation explicit. Avoid macros
that hide native safety conditions.

### Host API root

Keep bootstrap, the exported `SampClientSdk_GetApiV1` symbol, and the complete
ABI table literal in `host_api/mod.rs`.

- `helpers.rs`: direct-client result mapping, command submission/completion,
  bounded string copying, and initialized-host access.
- `listeners.rs`: registration, unregistration, waiting, and callback dispatch.
- Test modules grouped by ABI, command, and listener behavior.

Expose extracted helpers only as `pub(super)` where required.

### Win32 backend root

Keep `BackendContext`, `BackendState`, attach/shutdown coordination,
game-thread identification, tick snapshot/pump ordering, cache publication,
and connection-boundary invalidation in `win32/mod.rs`.

Move methods while leaving fields root-owned:

- `backend.rs`: `Backend` forwarding and public backend surface.
- `commands.rs`: command submission and execution around the existing
  `GameCommand` invariant.
- `requests.rs`: producer methods and bounded request draining.
- `refresh.rs`: cache refresh writers and publication helpers.
- `native_bitstream.rs`: raw/native bitstream ownership and packet helpers.
- `hooks.rs`: hook storage, vtable/inline-hook primitives, installation, and
  restoration where their lifecycle remains auditable.

Do not split `BackendState` into cache-domain sub-states during this work.

### Event fixtures and fixed RPC catalog

Split `events/test_support.rs` into `test_support/` modules for mock state, the
test ABI table, callbacks, and fixture builders.

Leave `events/rpc/incoming/fixed.rs` intact unless a later LOC review finds a
cohesive protocol boundary. If split, move each RPC descriptor with its exact
encoder/decoder pair and preserve existing constant paths through re-exports.

## Progress checklist

### Decisions and baseline

- [x] Confirm the 1,500-nonblank-line soft ceiling.
- [x] Confirm `incoming/fixed.rs` may remain a documented exception.
- [x] Confirm public SDK paths and current error behavior remain unchanged.
- [x] Record baseline LOC and workspace checks; retain ABI/layout fixtures.

### Test extraction

- [x] Extract SDK root tests into a child test module.
- [x] Extract Win32 backend/layout/hook tests into child test modules.
- [x] Extract R1 bridge tests into a child test module.
- [x] Extract host API tests into a child test module.
- [x] Verify private test access and coverage remain equivalent.

### SDK root

- [x] Extract safe public types and enums.
- [x] Extract ABI declarations and the API-table type.
- [x] Extract `HostApi`, receipts, and shared conversions.
- [x] Extract subscriptions and callback ownership.
- [x] Extract host resolution.
- [x] Verify all existing root imports still compile.

### R1 bridge

- [x] Extract singleton lookup helpers.
- [x] Extract native aliases and mirrors.
- [x] Extract textdraw operations.
- [x] Extract UI operations.
- [x] Extract player-directory and local-player snapshot operations; retain
  game/network-coupled player mutations with their sequencing root.
- [x] Extract remaining pool operations.
- [x] Extract handle lookup operations.
- [x] Run the independent x86 memory/layout fixture after native moves.

### Host API

- [x] Extract shared helpers.
- [x] Extract listener lifecycle and dispatch.
- [x] Keep the exported entry point and ABI table together.
- [x] Compare DLL exports before and after the split.

### Win32 backend

- [x] Extract `Backend` forwarding methods.
- [x] Consolidate command submission/execution in `commands.rs`.
- [x] Consolidate request production/draining in `requests.rs`.
- [x] Extract cache refresh writers without moving state fields.
- [x] Extract native bitstream and packet-layout helpers.
- [x] Consolidate hook primitives and patch/restore lifecycle in `hooks.rs`;
  retain attach/install/shutdown sequencing in the Win32 root.
- [x] Re-audit tick, lock, queue, invalidation, and shutdown ordering.

### Event support and completion

- [x] Split event test support into ABI-table and raw-callback modules.
- [x] Reassess the fixed incoming-RPC catalog; retain it as the documented protocol-catalog exception.
- [x] Recount tracked Rust files and record remaining exceptions.
- [x] Update `ARCHITECTURE.md`, `CORE.md`, and `TODO.md` as boundaries land.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets --locked`.
- [x] Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [x] Run `cargo build --workspace --release --locked`.

## Completion criteria

The large roots have cohesive ownership, actively maintained files meet the
soft ceiling or carry a documented exception, current SDK imports still
compile, ABI/native layouts and exports are unchanged, and every verification
item passes.
