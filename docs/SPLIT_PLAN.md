# Module Split Plan

Status: active; Phase 1 started. Rust API paths may change when a clearer
module boundary justifies it. ABI, native layouts, and runtime behavior remain
unchanged unless a separate change explicitly says otherwise.

## 1. Motivation

Several files mix state ownership, native access, protocol handling, ABI
plumbing, and tests. Current approximate sizes are:

| File | Lines | Main concerns mixed together |
| --- | ---: | --- |
| `src/platform/win32.rs` | 6,670 | Backend state, hooks, commands, request queues, caches, packet dispatch |
| `src/platform/win32/r1_client.rs` | 3,480 | RVAs, native reads and writes, memory validation, layout tests |
| `src/host_api.rs` | 3,790 | Host lifecycle, ABI table, entry points, conversions |
| `sdk/src/lib.rs` | 6,070 | Public types, ABI types/table, host wrapper, subscriptions, resolution |
| `sdk/src/facade.rs` | 1,930 | Safe subsystem views and handle newtypes |

The goal is easier navigation and stronger module ownership. File size is a
signal, not a hard design constraint: cohesive data-heavy modules may remain
larger than the preferred range of roughly 900 lines.

Splitting Rust source files does not create independent crate compilation
units and is not expected to improve build speed by itself.

## 2. Guardrails

- Keep this an in-crate reorganization under `src/` and `sdk/src/`.
- Prefer cohesive public modules over compatibility re-exports. Rust API path
  breakage is acceptable when intentional, documented, and updated throughout
  the workspace; do not retain aliases solely for compatibility.
- Do not change `SampClientSdkApiV1`, any `repr(C)` layout, fixed offset,
  exported symbol, command ordering, queue bound, or cache publication rule.
- Preserve the coherent dialog snapshot, including its 255-byte exported
  list-item limit and 256-byte native text-field scan bound.
- Preserve all typed object, pickup, vehicle, and player handle queues,
  caches, reverse lookups, and connection-boundary invalidation.
- Keep `value` plus `value_ready` atomic pairs as the existing lock-free
  publication mechanism. A wrapper may be considered separately only if it
  proves the same ordering and boundary semantics.
- Do not combine a source move with state decomposition or behavior changes.
- Move implementation first. Redistribute tests only after the implementation
  boundary is stable, so moved tests still guard each mechanical step.
- Avoid widening visibility beyond the narrowest `pub(super)` or `pub(crate)`
  boundary required by the new module structure.

## 3. Baseline and verification

The pre-split baseline is merge commit `cecc2c8`. At that commit:

- `cargo fmt --all --check` passed.
- `cargo test --workspace` passed all 160 unit tests and the remaining
  workspace/doc-test targets.
- `cargo clippy --workspace -- -D warnings` passed.
- `cargo build --workspace` passed for the configured Windows x86 target.
- The independent C++ fixture passed the R1 offset and native-layout tests.
- The host DLL exported exactly `DllMain` and `SampClientSdk_GetApiV1`.

Every slice must:

1. Keep moved definitions byte-for-byte equivalent where practical; review
   moves with whitespace-insensitive and moved-line-aware diffs.
2. Run formatting, workspace tests, strict Clippy, and a workspace build.
3. Run the existing SDK ABI size/offset tests and R1 C++ layout fixture as
   part of the workspace suite.
4. Compare host DLL exports before and after changes that touch host module
   wiring or ABI entry points.
5. Add an external compile fixture before moving broad public SDK type groups;
   it should verify the intended new surface, not preserve obsolete paths.

`git diff --stat` is useful for review but is not evidence of API or ABI
compatibility by itself.

## 4. Target boundaries

These names describe intended ownership, not a requirement to create every
module immediately.

### Host crate

- `platform/win32/`: composition root plus focused context, hooks, packets,
  commands, requests, caches, reads, and memory modules.
- `platform/win32/r1_client/`: R1 operations, grouped RVA/offset constants,
  memory helpers, and native layout definitions/tests.
- `host_api/`: host state and monitoring, the complete ABI table literal,
  event/network/local/direct/raw entry points, and ABI conversion helpers.
  Keep the `host_api` name because bootstrap and monitoring are not ABI data.
- `runtime/`: the `Runtime` composition root plus types, errors, and send
  options.

`BackendState` remains intact during mechanical method moves. State is split
only later, if the resulting module dependency graph demonstrates cohesive
groups with narrow interactions.

### SDK crate

- `limits.rs`: the public namespace for capacity constants.
- `types.rs`: safe public value types and enums.
- `abi/`: `repr(C)` types and the complete `SampClientSdkApiV1` definition.
- `host_api/`: safe wrapper methods grouped into networking, local state,
  players, pools, UI, and conversion helpers.
- `subscriptions.rs` and `resolve.rs`: callback/receipt ownership and host
  discovery.
- `facade/`: safe subsystem views grouped by domain.
- `events/`: codecs split by protocol direction and gameplay domain only
  where a resulting module remains cohesive.

## 5. Migration phases

### Phase 0 — Freeze observable boundaries

- Record the validation and DLL-export baseline above.
- Keep the ABI table literals and their append-order tests complete.
- Before broad SDK moves, add a small external compile fixture covering the
  intended public module paths.

### Phase 1 — Extract true leaf modules

Move definitions that have no state ownership or control-flow responsibility:

1. SDK capacity limits.
2. Runtime errors and send-option value types.
3. R1 RVA/offset constants.
4. R1 memory helpers and native layout mirrors.
5. Other self-contained SDK ABI or safe value types after the external API
   fixture exists.

Each item is an independent validated change. Runtime and R1/native work is
deferred until the preceding leaf establishes a clean pattern.

Progress:

- [x] Extract SDK capacity constants to public `sdk/src/limits.rs`; update
  workspace consumers and remove compatibility re-exports.
- [x] Extract runtime errors/options into private child modules while keeping
  lifecycle status and tests in the runtime composition root.
- [x] Extract R1 executable/function addresses while leaving native layout
  offsets with the memory/layout slice.
- [x] Extract R1 native layouts, fixed offsets, and guarded memory helpers.

### Phase 2 — Move methods while retaining state

- Convert `win32.rs` to a directory module and move hooks, packets, commands,
  queue methods, cache methods, and reads into child modules.
- Child modules may implement methods on the existing `BackendState`.
- Do not decompose fields, alter synchronization, or redesign cross-module
  calls in this phase.

Progress:

- [x] Convert the Windows backend to a directory module and extract
  producer-side game/network command submission without moving state or
  execution.
- [x] Extract bounded cache-refresh request producers without moving drains,
  caches, or invalidation.
- [x] Extract lock-free scalar cache readers without moving refresh writers or
  publication control.
- [x] Extract non-entity published snapshot readers without moving refresh
  writers or invalidation.
- [x] Extract animation catalog reads and lookups without moving lazy refresh
  or shutdown clearing.
- [x] Extract forward/reverse handle cache reads without moving request
  producers, refresh, or invalidation.
- [x] Extract local and remote player reads without moving request producers,
  refresh, or invalidation.
- [x] Extract 3D text-label existence and snapshot reads without moving request
  producers, refresh, or invalidation.
- [x] Extract textdraw existence and snapshot reads without moving request
  producers, refresh, or invalidation.
- [x] Extract vehicle existence reads without moving request producers, refresh,
  or invalidation.
- [x] Extract object existence reads without moving request producers, refresh,
  or invalidation.
- [x] Extract gangzone reads without moving request producers, refresh, or
  invalidation.
- [x] Extract chat-history entry reads without moving request producers, refresh,
  or invalidation.
- [x] Extract outbound native packet/RPC sends without moving command execution
  or hook ownership.
- [x] Extract incoming packet emulation without moving command execution or
  native queue ownership.
- [x] Extract incoming RPC emulation without moving listener or hook ownership.
- [x] Extract pure RPC envelope parsing/building with exact timestamp and bit
  preservation tests.
- [x] Extract native packet/RPC stream dispatch without moving detours or hook
  ownership.
- [ ] Extract remaining hook methods as independent validated slices.

### Phase 3 — Split host and SDK API plumbing

- Split ABI conversions and cohesive entry-point groups first.
- Move each ABI table literal last and as one block.
- Split SDK host-wrapper methods behind unchanged root exports.
- Verify DLL exports around host module-wiring changes.

### Phase 4 — Split facade and codecs

- Group facade views by networking, local/player, pool, and UI domains.
- Split packet/RPC codecs along existing protocol and gameplay boundaries.
- Do not redesign codecs or move their tests in the same change.

### Phase 5 — Evaluate state decomposition

After method moves expose actual dependencies, document which fields are used
by each module. Split `BackendState` only when a proposed sub-state:

- owns a coherent invariant,
- has a narrow dependency direction,
- does not require pervasive visibility widening,
- preserves callback/game-thread synchronization semantics, and
- makes cross-component operations clearer rather than merely relocating
  coupling.

Potential groups such as context, hook state, request queues, and cache store
remain hypotheses until this review.

### Phase 6 — Redistribute tests and finish documentation

- Move narrow unit tests beside stable leaf modules.
- Keep queue-to-pump-to-cache and other cross-module behavior tests in a
  parent integration test module.
- Update `ARCHITECTURE.md` and `CORE.md` after each major boundary becomes
  real, not in anticipation of it.

## 6. Completion criteria

- Large files are reduced into cohesive modules with documented ownership;
  line-count targets remain advisory.
- Public SDK paths follow the documented target modules; intentional breaking
  moves are updated throughout the workspace. Host DLL exports match the
  baseline.
- ABI size/offset tests and the independent native fixture remain unchanged
  and passing unless a separately approved ABI change is made.
- Dialog snapshot and typed-handle behavior retain their exact queue, cache,
  capacity, and connection-boundary guarantees.
- The four workspace validation commands pass after every migration slice.
- No module split leaves temporary duplicate implementations or compatibility
  shims.
