# Cohesion-Oriented Module Split Task Tracker

Status: in progress.

Handoff: [cohesion-module-split-handoff.md](cohesion-module-split-handoff.md).

Baseline reviewed: `master` at `f3fa5e5`, 2026-08-29.

## Tracker rules

- `[ ]` not started
- `[-]` in progress
- `[x]` complete and validated
- `[!]` blocked; add the blocker and evidence directly under the task

A task is not complete merely because the code moved. Mark it `[x]` only after its stated validation/evidence is captured.

Do not batch unrelated task IDs into one commit unless the first task cannot compile independently without the second. Prefer Conventional Commit messages scoped to the actual boundary moved.

## Phase summary

| Phase | Scope | Depends on | Status |
| --- | --- | --- | --- |
| P0 | Baseline and safety freeze | — | [x] |
| P1 | Protocol descriptor-helper cleanup | P0 | [x] |
| P2 | Split incoming `common` RPCs | P1 | [x] |
| P3 | Split incoming `r1` RPCs | P1 | [x] |
| P4 | Split `Runtime` | P0 | [x] |
| P5 | Split/domainize game-thread commands | P0 | [ ] |
| P6 | Reduce Win32 composition root | P5 recommended | [ ] |
| P7 | Split native-client players/UI | P0 + live-validation decision | [ ] |
| P8 | Split SDK ABI source | P0 | [ ] |
| P9 | Redistribute large tests | relevant production phase | [ ] |
| P10 | Split common network probe | P7 recommended | [ ] |
| P11 | Documentation and final acceptance | P2–P10 as selected | [ ] |

---

## P0 — Baseline and safety freeze

- [x] **P0-01 — Record working baseline.**
  - Record branch, full `HEAD`, and `git status --short`.
  - Record current line counts for all files listed in the handoff baseline.
  - Note any user-owned dirty files that must remain untouched.
  - Evidence: tracker note with date, branch, hash, dirty files.

  Evidence (2026-08-29): branch `cohesion-module-split`, HEAD
  `77edf87119f051bb9853f27e1c02b305e67e068f`. User-owned dirty files:
  `docs/cohesion-module-split-handoff.md` and this tracker. `repo.bundle` was
  visible during the first status read, disappeared without an agent action,
  and was not modified. Baseline physical/nonblank LOC:

  | File | Physical | Nonblank |
  | --- | ---: | ---: |
  | `crates/samp-protocol/src/rpc/incoming/common.rs` | 2,405 | 2,205 |
  | `crates/samp-protocol/src/rpc/incoming/r1.rs` | 1,662 | 1,551 |
  | `src/platform/win32/mod.rs` | 1,730 | 1,626 |
  | `src/platform/win32/commands.rs` | 1,640 | 1,568 |
  | `src/platform/win32/native_client/players.rs` | 1,677 | 1,611 |
  | `src/platform/win32/native_client/ui.rs` | 1,223 | 1,185 |
  | `src/runtime.rs` | 1,271 | 1,114 |
  | `sdk/src/abi.rs` | 1,252 | 1,205 |
  | `sdk/src/tests.rs` | 2,565 | 2,417 |
  | `src/platform/win32/vtable_tests.rs` | 1,999 | 1,843 |
  | `examples/network_probe_common/src/lib.rs` | 2,784 | 2,657 |

  The `sdk/src/tests.rs` count differs from the planned handoff baseline because
  HEAD contains the later `77edf87` commit.

- [x] **P0-02 — Run the pre-refactor static gate.**
  - Run `cargo make format-check`.
  - Run `cargo make check`.
  - Run `cargo make test`.
  - Run `cargo make clippy`.
  - Run `cargo make release-hygiene`.
  - Run `cargo make package-published`.
  - Run `cargo make build-release`.
  - Run `git diff --check`.
  - Evidence: pass/fail summary and any environment limitation.

  Evidence (2026-08-29): all commands passed on HEAD `77edf87`:
  `cargo make format-check`, `check`, `test`, `clippy`, `release-hygiene`,
  `package-published`, `build-release`, and `git diff --check`.

- [x] **P0-03 — Freeze observable API/ABI expectations.**
  - Confirm Protocol tests import from `rpc::incoming::common::*` and `r1::*` today.
  - Confirm SDK exports ABI names from the crate root via `pub use abi::*`.
  - Record current host DLL exports if the environment permits artifact inspection.
  - Confirm ABI/layout tests currently pass.

  Evidence (2026-08-29): existing Protocol integration tests import the flat
  `rpc::incoming::common::*` and `rpc::incoming::r1::*` paths. `sdk/src/lib.rs`
  retains private `mod abi;` and crate-root `pub use abi::*;`. Full workspace
  tests, including ABI and independent native layout tests, passed. `dumpbin
  /exports target/i686-pc-windows-msvc/release/samp_client_sdk.dll` reported
  exactly `DllMain` and `SampClientSdk_GetApiV1`.

- [x] **P0-04 — Choose the native live-validation policy.**
  - Record either `split-before-live-validation` or `revalidate-after-split`.
  - Link the decision to `docs/native-profile-unification-handoff.md`.
  - Do not start P7 without this decision.

  Decision (2026-08-29): `split-before-live-validation`. The separate live
  validation stage in `docs/native-profile-unification-handoff.md` is still
  pending. R1/R3/R5/DL evidence must use the final native-facing code after P7
  and any selected P10 work.

### P0 gate

- [x] Baseline evidence is recorded and no unrelated file has been modified.

---

## P1 — Protocol descriptor-helper cleanup

Goal: remove the global descriptor-to-value synchronization map before distributing descriptors across files.

- [x] **P1-01 — Make `common` descriptor value types explicit.**
  - Replace `descriptor_value!` use in `incoming/common.rs` with a descriptor helper that takes the value type directly.
  - Preserve descriptor type names, constants, IDs, capabilities, and `ExactBytesPolicy`.
  - Do not move descriptors to new files in this task.

- [x] **P1-02 — Make `r1` descriptor value types explicit.**
  - Apply the same pattern to the plain R1 descriptor helper.
  - Preserve every existing `ExactBitsPolicy` / `ExactBytesPolicy` selection exactly.
  - Leave encoded-string descriptor capability behavior unchanged.

- [x] **P1-03 — Verify no descriptor/value mapping table remains.**
  - Search both incoming modules for `descriptor_value!` or equivalent centralized mappings.
  - Confirm each descriptor invocation names its value type directly or uses a self-contained encoded-string descriptor declaration.

- [x] **P1-04 — Validate Protocol helper cleanup.**
  - `cargo test -p samp-protocol --all-targets --locked`
  - `cargo clippy -p samp-protocol --all-targets --locked -- -D warnings`
  - `cargo fmt --all -- --check`
  - Evidence: command results and commit hash.

  Evidence (2026-08-29): commit `23f85114e1e5475890837603951c5d7f90d4b985`. All 80
  `samp-protocol` tests passed, including common/R1 framing and encoded-string
  vectors. Protocol Clippy with warnings denied, format check, and
  `git diff --check` passed. No `descriptor_value` mapping remains.

### P1 gate

- [x] Both incoming files still behave identically and are ready for mechanical domain extraction.

---

## P2 — Split profile-neutral incoming RPCs (`common`)

Target tree: `crates/samp-protocol/src/rpc/incoming/common/`.

### Module shell

- [x] **P2-01 — Convert `common.rs` to a directory module.**
  - Create `common/mod.rs` and private `common/wire.rs`.
  - Move only shared scalar/empty/bool/vector/fixed/tuple codec mechanics to `wire.rs`.
  - Keep all semantic descriptors in `mod.rs` temporarily if needed for the first compiling step.
  - Preserve flat public paths with explicit re-exports only.

### Semantic extraction slices

- [x] **P2-02 — Extract `session.rs`.**
  - Move join/quit, spawn/class/session/control descriptors assigned by the handoff.
  - Move each payload + semantic codec + descriptor + read/write functions together.
  - Keep current IDs and framing unchanged.

- [x] **P2-03 — Extract `player.rs`.**
  - Move player state, attributes, weapons, death, animation/special-action/fighting/velocity families.
  - Keep player-only wire helpers local unless another domain genuinely uses them.

- [x] **P2-04 — Extract `vehicle.rs`.**
  - Move vehicle stream/state/params/damage/trailer and player-vehicle transition descriptors.
  - Preserve exact field order and scalar widths.

- [x] **P2-05 — Extract `world.rs`.**
  - Move checkpoints, time/weather/clock, audio, map icons, pickups, explosion, gangzones, gravity, building/label removal, and other world-state descriptors.

- [x] **P2-06 — Extract `object.rs`.**
  - Move object position/rotation/destroy/edit/select/move/stop/attachment descriptors.

- [x] **P2-07 — Extract `ui.rs`.**
  - Move `SHOW_DIALOG` including its encoded-string codec/capability as one unit.
  - Move server/game text, chat/bubble, menu, widescreen, and textdraw-string descriptors.

- [x] **P2-08 — Extract `camera.rs`.**
  - Move camera position/look-at/behind, attach-camera, and spectate descriptors.

- [x] **P2-09 — Extract `actor.rs`.**
  - Move actor create/destroy/animation/position/angle/health descriptors.

### API and cleanup

- [x] **P2-10 — Make `common/mod.rs` an ownership map, not a second catalog.**
  - Keep module declarations, shared macro wiring if still needed, and explicit `pub use` lists.
  - Remove stale duplicated imports/helpers from the old file conversion.
  - Do not use `pub use module::*` in production.

- [x] **P2-11 — Audit descriptor colocation.**
  - For every semantic descriptor, verify payload type, descriptor declaration, semantic codec marker/impl, and decode/encode helpers live in the same domain module.
  - Exceptions allowed only for generic `wire.rs` primitives.

- [x] **P2-12 — Validate all existing common API paths.**
  - Existing `incoming_common.rs` and `incoming_common_world.rs` must compile unchanged unless only import formatting changes.
  - `incoming_encoded_strings.rs` must continue exercising `SHOW_DIALOG` through the same public path.

- [x] **P2-13 — Run Protocol gate and record final common LOC.**
  - Run Protocol tests/Clippy/format.
  - Record LOC per new module and note any module still unusually large with justification.
  - Evidence: commit hashes per slice plus final phase hash.

  Evidence (2026-08-29): module shell `bdc1a6a`; wire `56befc5`;
  actor `8a257dd`; session `f5be84a`; camera `d446395`; object
  `6bb45dc`; vehicle `be91f04`; player `be30baa`; world `76c08e1`;
  UI `7b6c11d`; final ownership/import cleanup `49795a9`
  (`49795a9c0fa74d5c54eecca168b0f798963b9df7`). All 80 Protocol tests, Clippy with warnings
  denied, format check, and `git diff --check` passed. Existing flat imports
  remain unchanged. The nominal-identity test now expects the physical private
  domain declaration path reported by `type_name`; this does not change the
  public re-export path.

  Final physical/nonblank LOC:

  | Module | Physical | Nonblank |
  | --- | ---: | ---: |
  | `common/mod.rs` | 112 | 101 |
  | `common/wire.rs` | 156 | 136 |
  | `common/session.rs` | 141 | 118 |
  | `common/player.rs` | 487 | 407 |
  | `common/vehicle.rs` | 669 | 575 |
  | `common/world.rs` | 477 | 395 |
  | `common/object.rs` | 193 | 163 |
  | `common/ui.rs` | 289 | 242 |
  | `common/camera.rs` | 97 | 81 |
  | `common/actor.rs` | 159 | 133 |

  `vehicle.rs` is the largest remaining module because its 19 descriptor
  families share the same vehicle-state reason to change; it is still cohesive.

### P2 gate

- [x] No semantic common descriptor is still spread across independent file regions.
- [x] Flat `samp_protocol::rpc::incoming::common::*` paths remain valid through explicit re-exports.

---

## P3 — Split R1 incoming RPCs

Target tree: `crates/samp-protocol/src/rpc/incoming/r1/`.

### Module shell

- [x] **P3-01 — Convert `r1.rs` to a directory module.**
  - Create `r1/mod.rs` and private `r1/wire.rs` for only genuinely shared R1 primitive helpers.
  - Preserve all public flat `r1::*` paths with explicit re-exports.

### Semantic extraction slices

- [x] **P3-02 — Extract `session.rs`.**
  - `INIT_GAME`, `REQUEST_CLASS_RESPONSE`, `SET_SPAWN_INFO`, `UPDATE_SCORES_AND_PINGS`, `ENABLE_STUNT_BONUS` and supporting values.
  - Preserve bit-level field ordering and current trailing policies.

- [x] **P3-03 — Extract `player.rs`.**
  - `PLAYER_STREAM_IN`, `APPLY_PLAYER_ANIMATION`, `PLAY_CRIME_REPORT`, `SET_PLAYER_ATTACHED_OBJECT`, `TOGGLE_PLAYER_SPECTATING`.

- [x] **P3-04 — Extract `vehicle.rs`.**
  - `VEHICLE_STREAM_IN`, `DISABLE_VEHICLE_COLLISIONS` and their supporting values.

- [x] **P3-05 — Extract `object.rs`.**
  - `ENTER_EDIT_OBJECT`, encoded `CREATE_OBJECT`, encoded `SET_OBJECT_MATERIAL`.
  - Keep object material limits and encoded-string behavior local to this domain where possible.

- [x] **P3-06 — Extract `text_labels.rs`.**
  - encoded `CREATE_3D_TEXT` and `TextLabel3D`.

- [x] **P3-07 — Extract `ui.rs`.**
  - `INIT_MENU`, `TOGGLE_SELECT_TEXT_DRAW`, `SHOW_TEXT_DRAW`, `TEXT_DRAW_HIDE` and supporting menu/textdraw values.

- [x] **P3-08 — Extract `camera.rs`.**
  - `INTERPOLATE_CAMERA`, `TOGGLE_CAMERA_TARGET_NOTIFYING`.

- [x] **P3-09 — Extract `actor.rs`.**
  - `APPLY_ACTOR_ANIMATION`.

### Framing/capability audit

- [x] **P3-10 — Audit trailing policies one descriptor at a time.**
  - Compare every R1 descriptor against the pre-split version.
  - No `ExactBitsPolicy`/`ExactBytesPolicy` drift is allowed.

- [x] **P3-11 — Audit encoded-string capability ownership.**
  - `CREATE_3D_TEXT`, `CREATE_OBJECT`, and `SET_OBJECT_MATERIAL` must remain encoded-string descriptors.
  - No host/native string codec dependency may enter Protocol.

- [x] **P3-12 — Run R1 Protocol vectors and exact-bit tests.**
  - Run the existing R1/world/UI/encoded-string Protocol tests.
  - Run full Protocol tests/Clippy/format.
  - Record final R1 module LOC.

  Evidence (2026-08-29): module shell `329d4ce`; wire `d816faf`;
  player `9276cb7`; actor `17b2817`; session `4d7efb9`; vehicle
  `8b6174b`; UI `8b7c3b1`; camera `853bf87`; object `9e6c7d2`;
  text labels `236460b`; final ownership/import cleanup `4102db7`
  (`4102db7110f17f1d6bafa174bdf31b734511be8e`). All 80 Protocol
  tests, Clippy with warnings denied, format check, and `git diff --check`
  passed. Existing flat imports remain unchanged. The descriptor audit found
  20 plain descriptors: 16 use `ExactBitsPolicy` and 4 use
  `ExactBytesPolicy`. All 3 encoded-string descriptors retain their encoded
  capability. No wildcard imports or re-exports remain in the R1 module.

  Final physical/nonblank LOC:

  | Module | Physical | Nonblank |
  | --- | ---: | ---: |
  | `r1/mod.rs` | 127 | 105 |
  | `r1/wire.rs` | 60 | 50 |
  | `r1/session.rs` | 357 | 327 |
  | `r1/player.rs` | 323 | 290 |
  | `r1/vehicle.rs` | 117 | 107 |
  | `r1/object.rs` | 318 | 288 |
  | `r1/text_labels.rs` | 62 | 56 |
  | `r1/ui.rs` | 313 | 284 |
  | `r1/camera.rs` | 72 | 63 |
  | `r1/actor.rs` | 44 | 38 |

### P3 gate

- [x] Flat `samp_protocol::rpc::incoming::r1::*` paths remain valid and every descriptor retains its exact framing policy.

---

## P4 — Split `Runtime`

Target tree: `src/runtime/` with existing `errors.rs` and `options.rs` retained.

- [x] **P4-01 — Convert request/value declarations.**
  - Create `requests.rs` for local dialog/chat/death request types and styles.
  - Create `snapshots.rs` for all owned cache/sync/server/UI snapshots and `Vector3`.
  - Re-export crate-visible names from `runtime/mod.rs` so existing callers do not need broad edits.

- [x] **P4-02 — Extract `network.rs`.**
  - Move packet/RPC listeners, sends, submissions, incoming emulation, hook status/readiness, raw pointer reads, version, and string codec bridge methods.
  - Keep signatures unchanged.

- [x] **P4-03 — Extract `commands.rs`.**
  - Move every local UI/connection/text-label/textdraw/player/force-sync/send-rate producer and receipt helper.
  - Do not redesign the host-facing command surface.

- [x] **P4-04 — Extract `reads.rs`.**
  - Move all cache/snapshot/handle/animation read forwarding methods.

- [x] **P4-05 — Reduce `runtime/mod.rs` to lifecycle/composition.**
  - Keep `Runtime`, `ClientHookStatus`, `attach`, `Drop`, module declarations, and narrow re-exports.
  - Move the existing narrow inline test beside the owner if its ownership becomes obvious; otherwise keep a small parent test module.

- [x] **P4-06 — Audit crate-visible paths.**
  - Search all host modules for `crate::runtime::...` imports and confirm they still resolve without unnecessary visibility widening.

- [x] **P4-07 — Validate Runtime split.**
  - Host unit tests.
  - Workspace check/test/Clippy at phase close.
  - Record final module LOC and commit hash.

  Evidence (2026-08-29): request/snapshot values `6fc4176`; network
  facade `87657e1`; command facade `3d02e5b`; read facade `6bad80a`;
  composition-root and test ownership cleanup `19e3698`
  (`19e36987c41fa7a48d6b637505dd5acc318a89b4`). All 173 host tests
  passed. Workspace check, tests, and Clippy with warnings denied passed.
  Format check and `git diff --check` passed. All existing
  `crate::runtime::...` consumers compile through explicit crate-private
  re-exports; no wildcard re-export was added.

  Final physical/nonblank LOC:

  | Module | Physical | Nonblank |
  | --- | ---: | ---: |
  | `runtime/mod.rs` | 58 | 53 |
  | `runtime/requests.rs` | 125 | 114 |
  | `runtime/snapshots.rs` | 249 | 232 |
  | `runtime/network.rs` | 177 | 152 |
  | `runtime/commands.rs` | 462 | 408 |
  | `runtime/reads.rs` | 232 | 190 |
  | `runtime/errors.rs` | 85 | 77 |
  | `runtime/options.rs` | 48 | 43 |

  `commands.rs` remains the largest module because it owns the unchanged
  host-facing command producer and receipt surface. P5 will split the
  game-thread command implementation by domain without changing this facade.

### P4 gate

- [x] `runtime/mod.rs` is a composition root; no behavior or public/host contract changed.

---

## P5 — Split and domainize game-thread commands

### Stage A — mechanical module split

- [ ] **P5-01 — Convert `commands.rs` to `commands/mod.rs`.**
  - Move the existing flat `GameCommand` enum from `win32/mod.rs` into `commands/mod.rs` unchanged.
  - Update imports only.
  - Keep `CommandQueue<GameCommand, ()>` exactly as-is.

- [ ] **P5-02 — Extract network command producers.**
  - Create `commands/network.rs`.
  - Move packet/RPC send/emulation producer methods and send-rate submission.

- [ ] **P5-03 — Extract connection command producers.**
  - Create `commands/connection.rs` for game-state/connect/disconnect submission.

- [ ] **P5-04 — Extract UI command producers.**
  - Create `commands/ui.rs` for dialog/chat/death/chat-input/chat-command/cursor/scoreboard producer methods.

- [ ] **P5-05 — Extract text-label command producers/results.**
  - Create `commands/text_labels.rs`.
  - Move create/delete/update/auto-create and take/wait/forget helpers.
  - Preserve `auto_text_label_creates` semantics.

- [ ] **P5-06 — Extract textdraw command producers.**
  - Create `commands/textdraws.rs` for create/delete/style/string/model mutations.

- [ ] **P5-07 — Extract player command producers.**
  - Create `commands/players.rs` for spawn/special-action/name/colour and force-sync submissions.

- [ ] **P5-08 — Validate Stage A before enum redesign.**
  - `GameCommand` variants and the execution `match` must still be byte-for-byte/semantically equivalent.
  - Run command queue, tick, receipt, text-label, network command, and connection-boundary tests.

### Stage B — domain enums and execution

- [ ] **P5-09 — Introduce `NetworkCommand` wrapper.**
  - Move only network/send-rate variants from flat `GameCommand` into `GameCommand::Network(NetworkCommand)`.
  - Add private execution returning `Result<(), CommandError>`.
  - Keep outer queue completion centralized.
  - Validate before proceeding.

- [ ] **P5-10 — Introduce `ConnectionCommand` wrapper.**
  - Move set-state/connect/disconnect variants and execution.
  - Validate before proceeding.

- [ ] **P5-11 — Introduce `UiCommand` wrapper.**
  - Move UI/chat/input/dialog/cursor/scoreboard variants and execution.
  - Keep payload-free logging.
  - Validate before proceeding.

- [ ] **P5-12 — Introduce `TextLabelCommand` wrapper.**
  - Move text-label variants and execution.
  - Preserve auto-create selected ID publication and command receipt ordering.
  - Validate before proceeding.

- [ ] **P5-13 — Introduce `TextdrawCommand` wrapper.**
  - Move textdraw variants and cache invalidation behavior unchanged.
  - Validate before proceeding.

- [ ] **P5-14 — Introduce `PlayerCommand` wrapper.**
  - Move local-player/player-colour/force-sync variants and execution.
  - Preserve native profile calls exactly.
  - Validate before proceeding.

- [ ] **P5-15 — Shrink the outer `execute_game_commands`.**
  - Outer loop dispatches to domain execution and remains sole owner of queue completion/logging.
  - No domain executor may directly complete a queued command.

- [ ] **P5-16 — Audit command invariants.**
  - FIFO remains unchanged.
  - Queue bound remains 256.
  - Tick snapshot semantics unchanged.
  - Wait rejection on game thread/callback unchanged.
  - Shutdown completes pending receipts.
  - Failure mapping remains equivalent.

- [ ] **P5-17 — Run full host/workspace gate.**
  - Record Stage A and Stage B commit hashes separately.

### P5 gate

- [ ] Command ownership is domain-based, but queue/tick/completion invariants still have one owner.

---

## P6 — Reduce `src/platform/win32/mod.rs`

Keep `BackendState`, `BackendContext`, and cache-entry types root-owned in this phase.

- [ ] **P6-01 — Extract `lifecycle.rs`.**
  - Move `attach` and attach-time construction.
  - Move game-process/dialog/constructor/client hook installation orchestration.
  - Move active-backend publication/clearing and loaded-module lookup where appropriate.
  - Keep exported/root API path stable through a narrow re-export.

- [ ] **P6-02 — Extract `tick.rs`.**
  - Move `prepare_game_tick`, `pump_game_tick`, game-thread publication/checking, and tick orchestration helpers.
  - Preserve original-call exactly-once behavior and command snapshot ordering.

- [ ] **P6-03 — Extract `cache_lifecycle.rs`.**
  - Move local snapshot publication, every cache-clear helper, disconnect invalidation, connection-boundary invalidation, cache publication/generation helpers, and boundary predicate.
  - Do not move state fields.

- [ ] **P6-04 — Extract `native_abi.rs`.**
  - Move raw packet/player-ID mirrors, native function aliases, and priority/reliability conversion helpers.
  - Use only the visibility needed by `hooks.rs`/`packets.rs`/root.
  - Do not merge ABI aliases as part of the move.

- [ ] **P6-05 — Audit root ownership.**
  - `mod.rs` should retain shared constants, `Backend`, `BackendContext`, `BackendState`, cache-entry enums, and module wiring.
  - Do not create `state.rs` if it requires broad field visibility changes.

- [ ] **P6-06 — Re-run lifecycle/tick/cache/hook tests.**
  - Explicitly run tests covering original call count, game-thread ID, disconnect invalidation, captured client/reconnect, hook replacement/restore, and incoming-emulation readiness.

- [ ] **P6-07 — Verify DLL exports after module wiring.**
  - Compare against P0 export baseline where available.

- [ ] **P6-08 — Run full host/workspace gate and record LOC.**

### P6 gate

- [ ] Win32 root remains the state composition root without carrying unrelated implementation bodies.

---

## P7 — Split native-client player and UI operations

Dependency: **P0-04 live-validation policy must be decided first.**

### Players

- [ ] **P7-01 — Convert `players.rs` to a directory module.**
  - Create `players/mod.rs` plus child modules without changing behavior.

- [ ] **P7-02 — Extract `players/animation.rs`.**
  - Move animation catalog parsing/read and associated profile-boundary tests.

- [ ] **P7-03 — Extract `players/pool.rs`.**
  - Move counts/max/local/remote/player-state/player-stat operations and tightly coupled pool helpers.
  - Keep player ID/count validation behavior unchanged.

- [ ] **P7-04 — Extract `players/sync.rs`.**
  - Move on-foot/in-car/passenger/trailer/aim reads.
  - Move sync-record address/write/reset helpers and all force-sync operations.
  - Preserve `ForceSyncReset` strategy behavior.

- [ ] **P7-05 — Extract `players/control.rs`.**
  - Move spawn, special action, local name, player colour, send rate, and their native aliases/call helpers.

- [ ] **P7-06 — Audit native aliases and profile dispatch.**
  - Operation-specific aliases live with call sites.
  - No new R1/R3/R5/DL runtime branch exists.
  - No guessed RVA/offset/value introduced.

### UI

- [ ] **P7-07 — Convert `ui.rs` to a directory module.**
  - Create `ui/mod.rs`, `dialog.rs`, `chat.rs`, `input.rs`, `display.rs`.

- [ ] **P7-08 — Extract `ui/dialog.rs`.**
  - Dialog show/close/response/client-side/selection/editbox/state/list-item behavior.
  - Preserve `ListItemTextLayout` strategy.

- [ ] **P7-09 — Extract `ui/chat.rs`.**
  - Chat/death-window messages, chat entry/history, display mode.

- [ ] **P7-10 — Extract `ui/input.rs`.**
  - Chat input read/write/enable/process and chat-command registration lifecycle.
  - Preserve input bounds and callback/trampoline behavior.

- [ ] **P7-11 — Extract `ui/display.rs`.**
  - Cursor and scoreboard operations.

- [ ] **P7-12 — Run all native profile parity/layout unit tests.**
  - Every supported profile remains covered.
  - Record unit-test count/results and commit hashes.

- [ ] **P7-13 — Satisfy the live-validation evidence rule.**
  - If split-before-live-validation: record this phase's final hash as the binary source for later R1/R3/R5/DL validation.
  - If revalidate-after-split: repeat all four live validators and record hashes/status/failure/log evidence.

### P7 gate

- [ ] One shared `NativeClientProfile` implementation model remains and live evidence applies to the final native-facing code.

---

## P8 — Split SDK ABI source organization

- [ ] **P8-01 — Convert `sdk/src/abi.rs` to `sdk/src/abi/mod.rs`.**
  - Preserve `mod abi;` in `sdk/src/lib.rs` and `pub use abi::*` behavior.

- [ ] **P8-02 — Extract `abi/values.rs`.**
  - Move owned/output `repr(C)` snapshot/value structs without changing fields/derives/defaults.

- [ ] **P8-03 — Extract `abi/control.rs`.**
  - Move result/status/version/direction/action enums, subscription/receipt types, send options, encoded-string/event/callback ABI declarations.

- [ ] **P8-04 — Extract `abi/table.rs`.**
  - Move the entire `SampClientSdkApiV1` declaration as one uninterrupted struct.
  - Move `SampClientSdkGetApiV1` with it.
  - Do not macro-generate or nest API-table groups.

- [ ] **P8-05 — Explicitly re-export ABI items from `abi/mod.rs`.**
  - Preserve all crate-root public names.
  - Avoid adding a new public `abi` namespace unless separately intended; `abi` remains private as today.

- [ ] **P8-06 — Run ABI/default/layout tests immediately.**
  - Include append-order and offset/size assertions.
  - Confirm `SampClientSdkApiV1` is unchanged.

- [ ] **P8-07 — Run SDK/workspace gate and public API hygiene.**
  - `cargo make release-hygiene`
  - `cargo make package-published`
  - Record commit hash and any generated public API diff; expected semantic diff is none.

### P8 gate

- [ ] ABI source is navigable while the ABI table remains flat, complete, and byte-layout equivalent.

---

## P9 — Redistribute large tests

Do this only after the production owner for each test is stable.

### SDK tests

- [ ] **P9-01 — Create `sdk/src/tests/` module shell.**
  - Keep existing private test access.

- [ ] **P9-02 — Extract ABI/default/layout tests.**
  - `tests/abi.rs`.

- [ ] **P9-03 — Extract host API/facade conversion tests.**
  - `tests/host_api.rs`.

- [ ] **P9-04 — Extract Protocol send/convenience tests.**
  - `tests/protocol.rs`.

- [ ] **P9-05 — Extract typed/raw callback and replacement tests.**
  - `tests/callbacks.rs`.
  - Preserve exact-bit and failure-atomic replacement coverage.

- [ ] **P9-06 — Extract subscription lifecycle tests.**
  - `tests/subscriptions.rs`.

- [ ] **P9-07 — Extract resolution/probe tests if enough material exists.**
  - `tests/resolution.rs`; otherwise leave them in `tests/mod.rs` rather than creating a tiny file.

### Win32 tests

- [ ] **P9-08 — Convert `vtable_tests.rs` to a directory module.**
  - Create a narrow shared `fixtures.rs` only for backend/profile construction helpers used across multiple groups.

- [ ] **P9-09 — Extract cache/request tests.**
  - Group cache publication/invalidation separately from bounded request queue behavior where practical.

- [ ] **P9-10 — Extract command/tick tests.**
  - Preserve cross-module queue-to-tick-to-completion tests at the parent test layer if needed.

- [ ] **P9-11 — Extract hook/native lifecycle tests.**
  - Hook patch/restore, captured originals, readiness/failure visibility.

- [ ] **P9-12 — Reassess `profile_layout_tests.rs`.**
  - Split only if navigation benefit is real.
  - If split, share assertions and separate profile data without duplication.

- [ ] **P9-13 — Run full test suite and compare test count/coverage intent.**
  - No test may disappear merely because of module movement.

### P9 gate

- [ ] Large test roots are easier to navigate and no new monolithic `test_support` dumping ground exists.

---

## P10 — Split the common live-validation probe (optional/deferred)

Do not start while the live-validation contract is still changing.

- [ ] **P10-01 — Confirm probe split is still justified.**
  - If the file remains cohesive enough after native work, document it as an exception and skip P10.

- [ ] **P10-02 — Extract `config.rs`.**
  - Profile marker/status constants, timeouts, entry-point identity, profile-specific expected values.

- [ ] **P10-03 — Extract `state.rs` and `status.rs`.**
  - Shared probe state/observations/phase state and status-file/reporting logic.

- [ ] **P10-04 — Extract `network.rs`.**
  - Listener registration, exact-bit/codec checks, inbound/outbound checks.

- [ ] **P10-05 — Extract `entities.rs`.**
  - Entity ID parsing, handles, player pools, object/pickup/vehicle checks.

- [ ] **P10-06 — Extract `ui.rs`.**
  - Dialog/chat/input/scoreboard/cursor/text-label/textdraw validation.

- [ ] **P10-07 — Extract `sync.rs`.**
  - Sync snapshots, force-sync receipts, vehicle phases.

- [ ] **P10-08 — Extract `reconnect.rs`.**
  - Disconnect invalidation, reconnect request, restored-state checks.

- [ ] **P10-09 — Keep `lib.rs` as orchestration.**
  - Plugin entry/init, `run_probe`, ordered phase execution, final status.
  - Do not duplicate one file per client profile.

- [ ] **P10-10 — Re-run every required live validator after the final probe refactor.**
  - Record final binaries/hashes/status/failure results.

### P10 gate

- [ ] The probe remains one common capability-oriented harness and final live evidence is current.

---

## P11 — Documentation and final acceptance

- [ ] **P11-01 — Recount production/test LOC and document intentional exceptions.**
  - Use nonblank and physical counts.
  - Do not fail the project solely because a cohesive file remains above a soft ceiling.

- [ ] **P11-02 — Update `ARCHITECTURE.md`.**
  - Replace obsolete single-file paths with the final module trees and ownership descriptions.

- [ ] **P11-03 — Update `CORE.md`.**
  - Reflect final ownership where the core implementation map references moved modules.

- [ ] **P11-04 — Update relevant agent guides.**
  - `repository-layout.md` must match directory modules.
  - Update `game-thread-commands.md` if internal ownership wording is now stale, without changing its invariants.
  - Update `packets-and-events.md` if Protocol ownership wording needs the new semantic child paths.

- [ ] **P11-05 — Mark historical split plans appropriately.**
  - Keep completed plans as history.
  - Do not rewrite old baselines to pretend the new structure existed earlier.
  - Link this handoff as the follow-up where useful.

- [ ] **P11-06 — Update this tracker with final evidence.**
  - Every completed phase has commit hashes and validation results.
  - Every skipped optional phase has a reason.
  - Every remaining blocker has an owner/next action.

- [ ] **P11-07 — Run complete static acceptance.**
  - `cargo make format-check`
  - `cargo make check`
  - `cargo make test`
  - `cargo make clippy`
  - `cargo make release-hygiene`
  - `cargo make package-published`
  - `cargo make build-release`
  - `git diff --check`

- [ ] **P11-08 — Verify public/ABI/native invariants.**
  - Protocol flat incoming paths preserved.
  - SDK crate-root exports preserved.
  - ABI table size/offset/order preserved.
  - Native layout fixtures preserved.
  - Host DLL exports preserved.
  - No new version dispatcher in `NativeClientProfile` operations.

- [ ] **P11-09 — Verify game-thread invariants.**
  - Single bounded queue.
  - FIFO.
  - Tick snapshot before original call.
  - Original called once.
  - Snapshot executed after original.
  - Cache refresh/invalidation ordering unchanged.
  - Receipt lifecycle unchanged.

- [ ] **P11-10 — Verify final live evidence where required.**
  - R1 pass recorded.
  - R3 pass recorded.
  - R5 pass recorded.
  - DL pass recorded.
  - Evidence hashes correspond to the final native-facing code.

- [ ] **P11-11 — Final repository hygiene.**
  - No temporary compatibility modules.
  - No wildcard ownership-hiding re-exports added.
  - No duplicate old/new implementation remains.
  - No unrelated user file modified or staged.
  - Conventional Commit history is reviewable by semantic slice.

### Final acceptance

- [ ] Incoming Protocol descriptors are colocated by semantic domain.
- [ ] `Runtime` is a composition root rather than a mixed facade implementation file.
- [ ] Game-thread command domains are separated while completion/tick semantics remain centralized.
- [ ] Win32 state remains root-owned without lifecycle/tick/cache/native-ABI implementation bloat.
- [ ] Native player/UI operation files have capability ownership and one profile-independent implementation path.
- [ ] SDK ABI source is split without changing the flat API table or public exports.
- [ ] Tests/probes are split only where the new boundary improves ownership.
- [ ] Static gates pass.
- [ ] Required live gates pass against the final code.
- [ ] Documentation matches the landed implementation.

## Evidence log

Append one row per completed slice rather than editing historical rows.

| Date | Task | Branch | Commit | Validation/evidence | Notes |
| --- | --- | --- | --- | --- | --- |
| 2026-08-29 | P0 baseline and safety freeze | `cohesion-module-split` | `77edf87119f051bb9853f27e1c02b305e67e068f` | All static gates passed; two DLL exports recorded | Live policy: `split-before-live-validation` |
| 2026-08-29 | P1 descriptor-helper cleanup | `cohesion-module-split` | `23f85114e1e5475890837603951c5d7f90d4b985` | 80 Protocol tests, Clippy, format, diff check passed | Explicit value types; no descriptor movement |
| 2026-08-29 | P2 split common incoming RPCs | `cohesion-module-split` | `49795a9c0fa74d5c54eecca168b0f798963b9df7` | 80 Protocol tests, Clippy, format, diff check passed | Flat public imports preserved through explicit re-exports |
| 2026-08-29 | P3 split R1 incoming RPCs | `cohesion-module-split` | `4102db7110f17f1d6bafa174bdf31b734511be8e` | 80 Protocol tests, Clippy, format, diff check passed | Framing policies and flat public imports preserved |
| 2026-08-29 | P4 split Runtime facade | `cohesion-module-split` | `19e36987c41fa7a48d6b637505dd5acc318a89b4` | 173 host tests; workspace check, tests, Clippy, format, diff check passed | `runtime/mod.rs` reduced to lifecycle and composition |

## Blockers / decisions log

| Date | Task | Decision or blocker | Owner/next action | Status |
| --- | --- | --- | --- | --- |
| 2026-08-29 | P0-04 | Use `split-before-live-validation`; prior live stage remains pending | Run all four live validators after final native-facing structural changes | Decided |
