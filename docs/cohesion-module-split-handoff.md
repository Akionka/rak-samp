# Cohesion-Oriented Module Split Handoff

Status: planned.

Baseline reviewed: `master` at `f3fa5e5` (`refactor(protocol-sdk)!: inject encoded-string codecs`), 2026-08-29.

Companion tracker: [cohesion-module-split-task-tracker.md](cohesion-module-split-task-tracker.md).

## 1. Objective

Reduce the current large implementation files by restoring **locality of change** rather than by chasing a line-count target.

A successful split should make one semantic change land in one obvious module. For example, changing an incoming vehicle RPC should not require editing a payload declaration near the top of a file, a codec marker in the middle, a descriptor table elsewhere, and read/write functions at the bottom. Likewise, adding one game-thread UI command should not require extending one global enum and a several-hundred-line central `match` in unrelated code.

This is a structural follow-up to the already-completed `docs/SPLIT_PLAN.md` and `docs/structural-split-plan.md`. Those documents established the current coarse domains. This handoff addresses the next layer of growth inside those domains after the protocol boundary and native-profile work landed.

## 2. Current baseline and pressure points

Current counts below use both physical and nonblank lines so they can be compared with older split documents.

| File | Physical | Nonblank | Main reason to split |
| --- | ---: | ---: | --- |
| `crates/samp-protocol/src/rpc/incoming/common.rs` | 2,405 | 2,205 | Semantic RPCs are separated internally into value/codec/descriptor/read-write regions instead of being colocated by domain. |
| `crates/samp-protocol/src/rpc/incoming/r1.rs` | 1,662 | 1,551 | Same locality problem plus encoded-string RPCs and several independent R1 gameplay domains. |
| `src/platform/win32/mod.rs` | 1,730 | 1,626 | State ownership, attach/hook lifecycle, tick orchestration, connection invalidation, native ABI mirrors, and global helpers coexist in the composition root. |
| `src/platform/win32/commands.rs` | 1,640 | 1,568 | Producer methods for all command domains and one very large execution `match` share one file. |
| `src/platform/win32/native_client/players.rs` | 1,677 | 1,611 | Player-pool reads, local/remote snapshots, sync snapshots, force-sync writes, mutations, animation catalog, and native aliases share one module. |
| `src/platform/win32/native_client/ui.rs` | 1,223 | 1,185 | Dialog, chat/death window, chat input/commands, cursor, scoreboard, and native aliases are coupled by file only. |
| `src/runtime.rs` | 1,271 | 1,114 | Request/snapshot types, network facade methods, game-command facade methods, reads, lifecycle, and a small test share the root. |
| `sdk/src/abi.rs` | 1,252 | 1,205 | Many independent `repr(C)` value types plus one necessarily-large flat API table. |
| `sdk/src/tests.rs` | 2,538 | 2,395 | ABI, facade, callback, replacement, subscription, and resolution tests are mixed. |
| `src/platform/win32/vtable_tests.rs` | 1,999 | 1,843 | Cross-domain backend, queues, cache, hooks, and native-lifecycle tests are mixed. |
| `examples/network_probe_common/src/lib.rs` | 2,784 | 2,657 | One live-validation harness owns state, status reporting, network probes, entity checks, UI checks, sync checks, reconnect handling, and profile markers. |

Line count remains a **signal only**. A cohesive table or ABI declaration may remain large. The actual acceptance test is whether a reviewer can identify the owner of a change without scanning unrelated regions.

## 3. Relationship to existing plans

The following completed documents remain authoritative for constraints they already established:

- `docs/SPLIT_PLAN.md`: ABI/native-layout/runtime-behavior guardrails and the rule not to combine mechanical source moves with state decomposition.
- `docs/structural-split-plan.md`: the earlier composition-root split and the 1,500-nonblank-line soft ceiling.
- `docs/native-profile-unification-handoff.md`: the one-profile runtime model and the pending live-validation stage.
- `docs/agent-guides/abi-and-runtime-safety.md`: native boundary rules.
- `docs/agent-guides/game-thread-commands.md`: command queue and game-tick invariants.
- `docs/agent-guides/packets-and-events.md`: descriptor identity, framing, typed callback, and Protocol ownership rules.

This handoff does **not** reopen those architectural decisions.

### Native live-validation dependency

`docs/native-profile-unification-handoff.md` still marks R1/R3/R5/DL live validation as pending. Therefore any source reorganization that touches `src/platform/win32/native_client/`, hook/tick execution, or command execution must follow one of these two policies:

1. Preferable: finish this structural split first, then perform the four-profile live validation against the final code; or
2. If live validation is completed before this split, repeat all affected live validation after the final native-facing structural change.

Do not cite live evidence from a pre-split binary as final acceptance evidence for a post-split native implementation.

## 4. Non-negotiable guardrails

### Behavior and ABI

- Do not change `SampClientSdkApiV1` field order, field types, append-only semantics, `repr(C)` layouts, discriminants, calling conventions, exported symbols, fixed offsets, RVAs, queue capacities, request pump limits, or cache-generation semantics.
- Preserve the exact game-tick invariant: snapshot accepted commands before the native game-process call, call the original exactly once, execute only the captured snapshot afterward, then refresh/publish caches according to the current ordering.
- Preserve FIFO ordering and the single bounded `GameCommand` queue.
- Preserve command receipt behavior, including detach-on-drop, retryable timed waits, game-thread/callback wait rejection, shutdown completion, and the special text-label auto-create result path.
- Preserve listener ordering, exact-bit replacement behavior, Host replacement atomicity, descriptor direction/kind, and trailing-policy behavior.
- Preserve `common` for profile-neutral protocol semantics and `r1` for explicitly R1-only semantics.
- Preserve all current public Protocol paths under `samp_protocol::rpc::incoming::common::*` and `...::r1::*` unless an independently approved breaking API change explicitly says otherwise.
- Preserve the SDK root exports. `sdk/src/abi.rs` may become `sdk/src/abi/`, but `pub use abi::*` must continue to expose the same names from the crate root.

### Refactor discipline

- Do not combine a source move with behavior normalization, deduplication, new feature work, native-layout corrections, or cache/state redesign.
- First create an equivalent module boundary. Only after it is green should an internal model such as `GameCommand` be reshaped.
- Avoid broad wildcard production re-exports. Preserve flat public Protocol paths with explicit re-exports from the domain modules.
- Keep visibility as narrow as possible. In particular, do **not** move `BackendState` into a child module if doing so forces most of its fields to become `pub(super)` merely so existing sibling modules can still reach them.
- Do not split a semantic Protocol descriptor away from its semantic codec. Generic scalar/helper codecs may be shared; descriptor-specific decode/encode code must stay with the descriptor domain.
- Do not split tests before the corresponding production boundary is stable unless a test file itself blocks the production move.

## 5. Definition of a good module boundary

Use these rules when the exact placement of one item is ambiguous:

1. **Same reason to change**: values, descriptor identity, codec, and semantic validation for one RPC family belong together.
2. **One invariant owner**: queue completion remains centralized even when domain command execution moves out.
3. **Dependency direction is visible**: domain modules may depend on shared primitives; shared primitives must not depend back on gameplay domains.
4. **No filename-only grouping**: do not create `misc.rs`, `types.rs`, or `helpers.rs` as dumping grounds unless the contents genuinely share a stable lower-level invariant.
5. **Prefer a slightly larger cohesive module over two mutually dependent small modules.**
6. **Move aliases with their unsafe call sites** when they are not truly shared ABI definitions. This keeps native signatures auditable next to the operation that casts/calls them.

## 6. Target: incoming Protocol RPCs

### 6.1 `common.rs`

Convert:

```text
crates/samp-protocol/src/rpc/incoming/common.rs
```

into:

```text
crates/samp-protocol/src/rpc/incoming/common/
  mod.rs
  wire.rs
  session.rs
  player.rs
  vehicle.rs
  world.rs
  object.rs
  ui.rs
  camera.rs
  actor.rs
```

`mod.rs` should contain module declarations, the small descriptor macro if still useful, and **explicit** public re-exports preserving the current flat `common::*` API.

`wire.rs` should contain only genuinely shared wire mechanics: scalar/empty/bool/vector/fixed-string/tuple codecs and any lower-level helper that is reused by multiple semantic domains. It must not become a second descriptor catalog.

Before splitting domains, remove the global `descriptor_value!` mapping. Change the private descriptor helper to accept the value type explicitly, conceptually:

```rust
// Shape only; preserve the existing nominal descriptor implementation.
descriptor!(
    SetVehiclePosition,
    SET_VEHICLE_POSITION,
    159,
    VehiclePositionCodec,
    VehiclePosition,
);
```

That eliminates one cross-file synchronization table and lets each domain own its descriptor/value relationship locally. This helper cleanup should be its own green refactor before source movement.

Suggested ownership:

| Domain | Representative/current RPC ownership |
| --- | --- |
| `session.rs` | join/quit, spawn-response/class-selection, game-mode restart, connection rejection, server-statistics response, client check |
| `player.rs` | player position/health/armour/facing/control, name/money/weapons/skin/interior/wanted/team, player stream-out, reset state, drunk state, colour/skill/name-tag, animation/special-action/fighting/velocity/ammo, player death notifications |
| `vehicle.rs` | put/remove/enter/exit vehicle, vehicle stream-out, position/angle/health, component/interior/params/tires/damage/velocity/numberplate, trailer attach/detach |
| `world.rs` | checkpoint/race checkpoint, world/player time, weather, clock, sound/audio, map icons, pickups, building removal, explosion, global timer, shop name, gangzones, gravity, 3D-label removal |
| `object.rs` | object position/rotation/destroy, attach object to player, attached-object editing, object selection, no-camera-collision, move/stop object |
| `ui.rs` | encoded `SHOW_DIALOG`, server message, game text, chat/chat bubble, menu show/hide, widescreen, textdraw string |
| `camera.rs` | camera behind/position/look-at, attach camera to object, player/vehicle spectate |
| `actor.rs` | actor create/destroy, clear animation, angle/position/health |

The exact assignment of a borderline RPC may change during implementation, but the descriptor, payload value, semantic codec, and its read/write functions must move together.

### 6.2 `r1.rs`

Convert:

```text
crates/samp-protocol/src/rpc/incoming/r1.rs
```

into:

```text
crates/samp-protocol/src/rpc/incoming/r1/
  mod.rs
  wire.rs
  session.rs
  player.rs
  vehicle.rs
  object.rs
  text_labels.rs
  ui.rs
  camera.rs
  actor.rs
```

Suggested ownership:

| Domain | R1 descriptors/types |
| --- | --- |
| `session.rs` | `INIT_GAME`, `REQUEST_CLASS_RESPONSE`, `SET_SPAWN_INFO`, `UPDATE_SCORES_AND_PINGS`, `ENABLE_STUNT_BONUS` and their supporting settings/spawn/score values |
| `player.rs` | `PLAYER_STREAM_IN`, `APPLY_PLAYER_ANIMATION`, `PLAY_CRIME_REPORT`, `SET_PLAYER_ATTACHED_OBJECT`, `TOGGLE_PLAYER_SPECTATING` |
| `vehicle.rs` | `VEHICLE_STREAM_IN`, `DISABLE_VEHICLE_COLLISIONS` |
| `object.rs` | `ENTER_EDIT_OBJECT`, encoded `CREATE_OBJECT`, encoded `SET_OBJECT_MATERIAL`, object/material/attachment values |
| `text_labels.rs` | encoded `CREATE_3D_TEXT` and `TextLabel3D` |
| `ui.rs` | `INIT_MENU`, `TOGGLE_SELECT_TEXT_DRAW`, `SHOW_TEXT_DRAW`, `TEXT_DRAW_HIDE` |
| `camera.rs` | `INTERPOLATE_CAMERA`, `TOGGLE_CAMERA_TARGET_NOTIFYING` |
| `actor.rs` | `APPLY_ACTOR_ANIMATION` |

Keep R1 trailing policies exactly where they are today. `ExactBitsPolicy` versus `ExactBytesPolicy` is part of descriptor framing and must move with each descriptor unchanged.

Encoded-string descriptors must retain the encoded-string capability boundary. Do not accidentally convert them to plain `WireCodec` descriptors just to make the split easier.

### 6.3 Protocol tests

During production extraction, keep the existing integration tests (`incoming_common.rs`, `incoming_common_world.rs`, `incoming_r1.rs`, `incoming_r1_world_ui.rs`, `incoming_encoded_strings.rs`) working through the unchanged flat public paths. They are useful evidence that the re-export layer is correct.

Only after both production trees are stable should test files be regrouped by the same semantic domains, and only if doing so improves maintenance. Test regrouping is not required for the production split to be accepted.

## 7. Target: `Runtime`

Convert `src/runtime.rs` to a directory module while leaving the public `Runtime` type and crate-visible snapshot/request names unchanged:

```text
src/runtime/
  mod.rs
  errors.rs
  options.rs
  requests.rs
  snapshots.rs
  network.rs
  commands.rs
  reads.rs
```

The existing `errors.rs` and `options.rs` remain as-is.

Ownership:

- `mod.rs`: `Runtime`, `ClientHookStatus`, `attach`, `Drop`, module declarations, and narrow crate-visible re-exports.
- `requests.rs`: `LocalDialogRequest`, `LocalChatMessageRequest`, `LocalDeathMessageRequest`, `LocalDialogStyle`, `LocalChatMessageStyle`.
- `snapshots.rs`: local/remote player snapshots, all sync snapshots, gangzone/text-label/textdraw/chat/server/animation snapshots, and the small neutral `Vector3` runtime value.
- `network.rs`: listener registration, immediate/queued packet and RPC sends, incoming emulation, hook/readiness status, raw RakClient/RakPeer/pool pointers, SA-MP version, and encoded-string bridge methods.
- `commands.rs`: all local UI, connection, text-label, textdraw, player, force-sync, send-rate, receipt wait/take/release methods.
- `reads.rs`: all published cache/snapshot/handle/animation read methods.

This is primarily a mechanical extraction. Do not redesign `Runtime` into sub-facades in the same change. Existing host API modules already provide the domain-facing layer above it.

## 8. Target: game-thread commands

This area needs **two separate stages**: mechanical extraction first, internal command-domain modeling second.

### 8.1 Stage A: directory module without changing `GameCommand`

Convert:

```text
src/platform/win32/commands.rs
```

into:

```text
src/platform/win32/commands/
  mod.rs
  network.rs
  connection.rs
  ui.rs
  text_labels.rs
  textdraws.rs
  players.rs
```

First move the existing flat `GameCommand` enum out of `win32/mod.rs` into `commands/mod.rs` unchanged. Keep the one `CommandQueue<GameCommand, ()>` and all completion behavior unchanged.

Then move producer methods by domain while keeping the flat enum and central execution `match`. This creates stable source ownership without changing the command model.

Suggested ownership:

- `network.rs`: send packet/RPC, incoming emulation, network submission helpers, send-rate command.
- `connection.rs`: game state, connect, disconnect.
- `ui.rs`: dialog/chat/death-window/chat-input/chat-command/cursor/scoreboard commands.
- `text_labels.rs`: delete/create/create-auto/update text and the created-ID wait/take/forget path.
- `textdraws.rs`: create/delete/all style/string/model mutations.
- `players.rs`: spawn, special action, nickname, colour, force-sync commands.
- `mod.rs`: queue primitives, `GameCommand`, the outer execution loop, logging without payloads, and receipt completion.

### 8.2 Stage B: domain command enums

Only after Stage A is green, replace the monolithic variant set with internal domain enums:

```rust
enum GameCommand {
    Network(NetworkCommand),
    Connection(ConnectionCommand),
    Ui(UiCommand),
    TextLabel(TextLabelCommand),
    Textdraw(TextdrawCommand),
    Player(PlayerCommand),
}
```

Each domain enum owns its variants and a private execution function returning `Result<(), CommandError>`.

The outer queue-drain loop remains centralized and must still own:

- exactly-once completion;
- first-completion diagnostic behavior;
- payload-free failure logging;
- mapping domain execution failure to the current command receipt result;
- the command snapshot boundary established by the game tick.

Do not let a domain executor complete its own queue item. Execution and completion are separate invariants.

The text-label auto-create side channel (`auto_text_label_creates`) may remain in `BackendState`; its producer/result methods belong in `text_labels.rs`.

## 9. Target: Win32 composition root

The earlier split intentionally kept `BackendState` root-owned. Keep that decision for this pass.

A good target is:

```text
src/platform/win32/
  mod.rs
  lifecycle.rs
  tick.rs
  cache_lifecycle.rs
  native_abi.rs
  backend.rs
  commands/
  requests.rs
  refresh.rs
  reads.rs
  hooks.rs
  packets.rs
  ... existing domain modules ...
```

Ownership:

- `mod.rs`: constants shared by child modules, `Backend`, `BackendContext`, `BackendState`, cache-entry enums, module wiring, and only genuinely composition-root helpers.
- `lifecycle.rs`: `attach`, hook-install sequencing, constructor/client-hook installation orchestration, active-backend publication/clearing, loaded-module lookup, and shutdown sequencing.
- `tick.rs`: `prepare_game_tick`, `pump_game_tick`, game-thread publication/checks, and only the orchestration that must preserve the tick snapshot invariant.
- `cache_lifecycle.rs`: local-player snapshot publication, all cache clear helpers, disconnect/connection-boundary invalidation, publication-generation helpers, and `crosses_connection_boundary`.
- `native_abi.rs`: `RpcPlayerId`, `PacketPlayerId`, `RawPacket`, raw native function-pointer aliases, and small priority/reliability ABI conversion helpers used by packet/hook code.

Why `BackendState` stays in `mod.rs`: child modules can access private items declared in their parent. Moving the struct into `state.rs` would either force widespread `pub(super)` field visibility or require a simultaneous state API redesign. That is exactly the kind of source move plus state decomposition this handoff forbids.

Likewise, do not split `BackendState` into cache/hook/command substates as part of this work. The previous split already concluded that those fields currently share ordering and synchronization invariants.

## 10. Target: native client player/UI operations

These are source-locality improvements inside the already unified `NativeClientProfile`. They must not reintroduce profile-specific runtime dispatch.

### 10.1 `native_client/players.rs`

Convert to:

```text
src/platform/win32/native_client/players/
  mod.rs
  animation.rs
  pool.rs
  sync.rs
  control.rs
```

Ownership:

- `animation.rs`: animation-catalog read/parse and its profile-boundary tests.
- `pool.rs`: player counts/max ID, local-player address/snapshot, remote-player directory/state, streamed-out status, player stats, and player/ped lookup helpers tightly coupled to the player pool.
- `sync.rs`: on-foot/in-car/passenger/trailer/aim snapshots; sync-record addressing; reset/write helpers; all force-sync native calls.
- `control.rs`: spawn, special action, local name, player colour, send-rate mutation, and operation-specific native aliases.
- `mod.rs`: only shared player-operation helpers that truly span two or more child modules; avoid turning it into the old monolith again.

Move native function aliases into the child that performs the cast/call unless an alias is truly shared. Keep R1/classic aliases distinct unless ABI equality has already been proven and encoded elsewhere.

### 10.2 `native_client/ui.rs`

Convert to:

```text
src/platform/win32/native_client/ui/
  mod.rs
  dialog.rs
  chat.rs
  input.rs
  display.rs
```

Ownership:

- `dialog.rs`: dialog show/close/response, client-side flag, selected item, editbox, dialog snapshot/list item text.
- `chat.rs`: chat messages, death-window messages, chat entries/history, display mode.
- `input.rs`: chat-input text/enabled/process, native chat-command register/unregister/list/read helpers.
- `display.rs`: cursor and scoreboard operations.

Again, native aliases should live beside their unsafe call sites.

### 10.3 Native-profile evidence rule

Do not remove or weaken any four-profile spec/parity tests while moving these methods. If a helper is shared across all profiles today, the split must keep one shared implementation. A new `match` on R1/R3/R5/DL introduced solely by the split is a regression.

Because live validation is still pending, record the final commit hash used for each live probe after these moves are complete.

## 11. Target: SDK ABI source organization

`abi.rs` is large but its flat API table is intentionally cohesive. Split declarations without changing the ABI surface:

```text
sdk/src/abi/
  mod.rs
  values.rs
  control.rs
  table.rs
```

Ownership:

- `values.rs`: all owned/output `repr(C)` snapshot/value structs (`LocalPlayer`, dialog, player state/sync, gangzone, text-label, textdraw, chat entry, server info, animation, etc.).
- `control.rs`: result/status/version/direction/action enums, subscription and receipt handles/results, send options, encoded-string/event/callback ABI types.
- `table.rs`: the **entire** `SampClientSdkApiV1` declaration in its current field order plus `SampClientSdkGetApiV1`.
- `mod.rs`: explicit internal re-exports so `sdk/src/lib.rs` can continue `pub use abi::*` with the same crate-root names.

Do not split `SampClientSdkApiV1` itself across macros, nested structs, or extension blocks. Its boring flat declaration is a feature because layout review must remain straightforward.

Run all ABI size/offset/default tests immediately after this source move.

## 12. Tests and probe harnesses

These are lower priority than production locality and should be done after the relevant production modules stabilize.

### `sdk/src/tests.rs`

Suggested child test modules:

```text
sdk/src/tests/
  mod.rs
  abi.rs
  host_api.rs
  protocol.rs
  callbacks.rs
  subscriptions.rs
  resolution.rs
```

Keep helpers in the narrowest common parent needed by two or more test modules. Do not build a giant `test_support.rs` merely to relocate the same coupling.

### `src/platform/win32/vtable_tests.rs`

Suggested grouping:

```text
src/platform/win32/vtable_tests/
  mod.rs
  fixtures.rs
  cache.rs
  commands.rs
  requests.rs
  tick.rs
  hooks.rs
```

Cross-module tests such as queue-to-tick-to-cache behavior stay at this parent test layer rather than being forced beside one leaf module.

### `src/platform/win32/profile_layout_tests.rs`

This file is below the current soft ceiling and is highly data/layout-oriented. Split only if navigation remains painful after the production changes. If split, group by shared assertions plus `r1`, `r3`, `r5`, and `dl` data; do not duplicate the assertion logic per profile.

### `examples/network_probe_common/src/lib.rs`

Defer until native live validation logic is stable. When split, prefer probe capability domains rather than profile files:

```text
examples/network_probe_common/src/
  lib.rs
  config.rs
  state.rs
  status.rs
  network.rs
  entities.rs
  ui.rs
  sync.rs
  reconnect.rs
```

The common probe must remain common. Do not fork duplicated R1/R3/R5/DL implementations; keep profile differences in data/constants/macros as they are today.

## 13. Migration order

Recommended order minimizes risk and review complexity:

1. Baseline and evidence capture.
2. Simplify Protocol descriptor helper (`descriptor_value!` removal) with no source movement.
3. Split `incoming/common` by semantic domain.
4. Split `incoming/r1` by semantic domain.
5. Split `Runtime` mechanically.
6. Split game commands Stage A mechanically.
7. Introduce domain command enums Stage B.
8. Extract Win32 lifecycle/tick/cache/native-ABI code while keeping `BackendState` root-owned.
9. Split native-client players/UI, coordinated with the pending live-validation stage.
10. Split SDK ABI declarations while preserving the flat table.
11. Redistribute large tests.
12. Split the shared network probe only if it remains a maintenance problem.
13. Update architecture/agent documentation and run final acceptance gates.

Protocol and SDK-ABI work are technically independent of Win32 work, but keeping this order makes each review narrative simple.

## 14. Validation strategy

### Per-slice minimum

Run the narrow relevant package tests first, then formatting and Clippy for that package where practical. Before merging any phase, run the complete repository gate.

Protocol slices should at minimum exercise:

```powershell
cargo test -p samp-protocol --all-targets --locked
cargo clippy -p samp-protocol --all-targets --locked -- -D warnings
```

Host/native/runtime slices should at minimum exercise host unit tests in addition to the final workspace gate.

SDK ABI/test slices should exercise the SDK test target and all ABI layout tests.

### Phase/final gate

Use the repository-defined quality workflow rather than inventing a parallel one:

```powershell
cargo make format-check
cargo make check
cargo make test
cargo make clippy
cargo make release-hygiene
cargo make package-published
cargo make build-release
git diff --check
```

`cargo make quality` covers the first six quality tasks and can be used when available.

For changes touching native client, hooks, game tick, or command execution, final acceptance also requires the R1/R3/R5/DL live-validation policy described above.

### API/ABI checks

- Existing Protocol integration tests must compile against the same `common::*` and `r1::*` paths.
- Existing SDK root imports must compile unchanged.
- Host DLL exports must remain exactly the expected set after host module-wiring changes.
- `SampClientSdkApiV1` size/offset tests and independent native layout fixtures must remain unchanged and passing.

## 15. Commit and review strategy

Prefer one independently green semantic move per commit. Example sequence:

```text
refactor(protocol): make incoming descriptor value explicit
refactor(protocol): split common player RPCs
refactor(protocol): split common vehicle RPCs
refactor(protocol): split common world and UI RPCs
refactor(protocol): split R1 incoming RPC domains
refactor(runtime): split runtime facade implementation
refactor(win32): move game command ownership into module
refactor(win32): split game command producer domains
refactor(win32): split game command execution domains
refactor(win32): extract backend lifecycle and tick modules
refactor(native-client): split player operation domains
refactor(native-client): split UI operation domains
refactor(sdk): split ABI declarations from API table
refactor(tests): split SDK and Win32 test roots
refactor(probes): split common live-validation harness
```

Do not use a single huge "split large files" commit. The entire point of this work is to make ownership and regressions reviewable.

For pure source moves, inspect diffs with rename/move detection and whitespace-insensitive review. For the command enum reshape and descriptor-macro cleanup, review as actual code changes and require full behavioral tests.

## 16. Known traps

- **Rust privacy trap:** moving `BackendState` to a child module causes sibling modules to lose access to its private fields. Do not solve this by making the whole state `pub(super)` during a structural split.
- **Macro ownership trap:** a shared descriptor macro can silently recreate a centralized descriptor table. The macro may define boilerplate, but each invocation must stay next to its domain codec/value.
- **Re-export trap:** `pub use domain::*` hides ownership and violates the current Protocol guidance. Use explicit re-exports for the flat compatibility surface.
- **Command completion trap:** domainizing `GameCommand` must not decentralize queue completion or change receipt semantics.
- **Native alias trap:** deduplicating R1/classic function aliases while moving them is an ABI claim. Move aliases first; merge them only under separate evidence-backed work.
- **Live evidence trap:** a successful live probe on an earlier binary is not evidence for later native-facing source changes.
- **Test-support trap:** splitting tests by moving every shared helper into one giant fixture file simply recreates the monolith under another name.
- **Line-count trap:** do not split `SampClientSdkApiV1` or cohesive static layout/spec tables just to satisfy a numeric threshold.

## 17. Completion criteria

This handoff is complete when:

- `incoming/common` and `incoming/r1` are semantic directory modules with descriptors colocated with their semantic codecs and payload values;
- `descriptor_value!` no longer acts as a global descriptor-to-value synchronization table;
- `Runtime` is a small composition root with network/commands/reads and request/snapshot ownership in focused child modules;
- `GameCommand` ownership lives under `commands/`, domain producers/executors are separated, and queue completion/tick semantics remain centralized;
- `win32/mod.rs` retains state ownership but no longer owns attach/hook lifecycle, tick implementation, cache invalidation implementation, and raw native ABI declarations all at once;
- native player/UI operations are split by capability without reintroducing profile dispatch or changing native behavior;
- SDK ABI declarations are organized while the complete flat `SampClientSdkApiV1` remains auditable and unchanged;
- large test/probe files are split only where the resulting ownership is clearer;
- no compatibility wildcard re-export, duplicate implementation, temporary forwarding layer, guessed native value, or behavior normalization remains from the split;
- all static quality/ABI/layout gates pass;
- any required four-profile live validation is recorded against the final native-facing commit;
- `ARCHITECTURE.md`, `CORE.md`, relevant agent guides, this handoff, and the companion tracker describe the code that actually landed.

## 18. Start-here checklist for the next implementer

1. Read this handoff and the companion tracker.
2. Read `AGENTS.md` plus the relevant agent guide for the phase being changed.
3. Verify branch, `HEAD`, and dirty files; do not overwrite unrelated work.
4. Record a fresh baseline before the first source move.
5. Start with tracker task `P0-01`, then take only one independently green slice at a time.
6. When a task changes the intended boundary, update the tracker immediately rather than letting the document drift from the code.
