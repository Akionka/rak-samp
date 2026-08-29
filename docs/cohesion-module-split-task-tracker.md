# Cohesion-Oriented Module Split Task Tracker

Status: complete.

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
| P5 | Split/domainize game-thread commands | P0 | [x] |
| P6 | Reduce Win32 composition root | P5 recommended | [x] |
| P7 | Split native-client players/UI | P0 + live-validation decision | [x] |
| P8 | Split SDK ABI source | P0 | [x] |
| P9 | Redistribute large tests | relevant production phase | [x] |
| P10 | Split common network probe | P7 recommended | [x] |
| P11 | Documentation and final acceptance | P2–P10 as selected | [x] |

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

- [x] **P5-01 — Convert `commands.rs` to `commands/mod.rs`.**
  - Move the existing flat `GameCommand` enum from `win32/mod.rs` into `commands/mod.rs` unchanged.
  - Update imports only.
  - Keep `CommandQueue<GameCommand, ()>` exactly as-is.

- [x] **P5-02 — Extract network command producers.**
  - Create `commands/network.rs`.
  - Move packet/RPC send/emulation producer methods and send-rate submission.

- [x] **P5-03 — Extract connection command producers.**
  - Create `commands/connection.rs` for game-state/connect/disconnect submission.

- [x] **P5-04 — Extract UI command producers.**
  - Create `commands/ui.rs` for dialog/chat/death/chat-input/chat-command/cursor/scoreboard producer methods.

- [x] **P5-05 — Extract text-label command producers/results.**
  - Create `commands/text_labels.rs`.
  - Move create/delete/update/auto-create and take/wait/forget helpers.
  - Preserve `auto_text_label_creates` semantics.

- [x] **P5-06 — Extract textdraw command producers.**
  - Create `commands/textdraws.rs` for create/delete/style/string/model mutations.

- [x] **P5-07 — Extract player command producers.**
  - Create `commands/players.rs` for spawn/special-action/name/colour and force-sync submissions.

- [x] **P5-08 — Validate Stage A before enum redesign.**
  - `GameCommand` variants and the execution `match` must still be byte-for-byte/semantically equivalent.
  - Run command queue, tick, receipt, text-label, network command, and connection-boundary tests.

  Evidence (2026-08-29): command ownership move `b234f4c`
  (`b234f4ca72bd4360f03affb9fac6b9398e6ae205`); producer domain split
  `1e6478a` (`1e6478a85dce8275c78868184f410b901e78b043`). The flat
  53-variant `GameCommand`, its execution match, the single
  `CommandQueue<GameCommand, ()>`, and tick orchestration remain unchanged.
  All 173 host tests and host Clippy with warnings denied passed. Format check
  and `git diff --check` passed.

### Stage B — domain enums and execution

- [x] **P5-09 — Introduce `NetworkCommand` wrapper.**
  - Move only network/send-rate variants from flat `GameCommand` into `GameCommand::Network(NetworkCommand)`.
  - Add private execution returning `Result<(), CommandError>`.
  - Keep outer queue completion centralized.
  - Validate before proceeding.

  Evidence (2026-08-29): `06a9e9c`. All 173 host tests and host Clippy
  with warnings denied passed. The network payload-copy/detached-receipt test
  now inspects the domain wrapper.

- [x] **P5-10 — Introduce `ConnectionCommand` wrapper.**
  - Move set-state/connect/disconnect variants and execution.
  - Validate before proceeding.

  Evidence (2026-08-29): `e0ef006`. All 6 connection-filtered tests and
  all 4 game-tick-filtered tests passed; host Clippy with warnings denied
  passed. The full host run passed 172 of 173 tests; the unrelated
  `r3_death_window_requires_a_readable_singleton` memory-probe test failed
  because address `0x17aa70` was readable in this process, and the isolated
  rerun reproduced the environment condition.

- [x] **P5-11 — Introduce `UiCommand` wrapper.**
  - Move UI/chat/input/dialog/cursor/scoreboard variants and execution.
  - Keep payload-free logging.
  - Validate before proceeding.

  Evidence (2026-08-29): `58984b2` (`58984b2a8cf0a8aeec69deef8bb6caf166401999`).
  All 62 Win32 vtable tests and host Clippy passed.

- [x] **P5-12 — Introduce `TextLabelCommand` wrapper.**
  - Move text-label variants and execution.
  - Preserve auto-create selected ID publication and command receipt ordering.
  - Validate before proceeding.

  Evidence (2026-08-29): `5b09113` (`5b091135a82da9a214f51df111133960dca152a1`).
  All 7 text-label tests, the shared queue test, and host Clippy passed.

- [x] **P5-13 — Introduce `TextdrawCommand` wrapper.**
  - Move textdraw variants and cache invalidation behavior unchanged.
  - Validate before proceeding.

  Evidence (2026-08-29): `aad2cc5` (`aad2cc5e607df9bbccfac308062658aa774dc979`).
  All 8 textdraw tests, all 4 game-tick tests, and host Clippy passed.

- [x] **P5-14 — Introduce `PlayerCommand` wrapper.**
  - Move local-player/player-colour/force-sync variants and execution.
  - Preserve native profile calls exactly.
  - Validate before proceeding.

  Evidence (2026-08-29): `2ec8fcb` (`2ec8fcbefbdafa04f144bcbfcaf8773f102dd214`).
  All 62 Win32 vtable tests and host Clippy passed.

- [x] **P5-15 — Shrink the outer `execute_game_commands`.**
  - Outer loop dispatches to domain execution and remains sole owner of queue completion/logging.
  - No domain executor may directly complete a queued command.

- [x] **P5-16 — Audit command invariants.**
  - FIFO remains unchanged.
  - Queue bound remains 256.
  - Tick snapshot semantics unchanged.
  - Wait rejection on game thread/callback unchanged.
  - Shutdown completes pending receipts.
  - Failure mapping remains equivalent.

  Evidence (2026-08-29): `commands/mod.rs` is 87 physical/81 nonblank
  lines. `GameCommand` has exactly 6 domain variants. Only the outer executor
  calls `game_commands.complete`; domain executors return
  `Result<(), CommandError>`. The single queue field remains
  `CommandQueue<GameCommand, ()>`. Workspace check, tests with the known
  environment-sensitive singleton test skipped, Clippy with warnings denied,
  format check, and `git diff --check` passed.

- [x] **P5-17 — Run full host/workspace gate.**
  - Record Stage A and Stage B commit hashes separately.

  Evidence (2026-08-29): Stage A final `1e6478a`; Stage B final `2ec8fcb`.
  The environment-sensitive singleton condition cleared without a test change.
  The unskipped host gate passed all 173 tests, and the full workspace format,
  check, test, and Clippy gates passed.

  Current physical/nonblank LOC: `mod.rs` 87/81, `network.rs` 206/193,
  `connection.rs` 108/103, `ui.rs` 511/486, `text_labels.rs` 314/303,
  `textdraws.rs` 455/441, `players.rs` 271/259.

### P5 gate

- [x] Command ownership is domain-based, but queue/tick/completion invariants still have one owner.

---

## P6 — Reduce `src/platform/win32/mod.rs`

Keep `BackendState`, `BackendContext`, and cache-entry types root-owned in this phase.

- [x] **P6-01 — Extract `lifecycle.rs`.**
  - Move `attach` and attach-time construction.
  - Move game-process/dialog/constructor/client hook installation orchestration.
  - Move active-backend publication/clearing and loaded-module lookup where appropriate.
  - Keep exported/root API path stable through a narrow re-export.

  Evidence (2026-08-29): `ecf2c97`
  (`ecf2c97f7a5acc50bad3f8cf9c380e948e34dcf4`).

- [x] **P6-02 — Extract `tick.rs`.**
  - Move `prepare_game_tick`, `pump_game_tick`, game-thread publication/checking, and tick orchestration helpers.
  - Preserve original-call exactly-once behavior and command snapshot ordering.

  Evidence (2026-08-29): `a404e7c`
  (`a404e7cd7e8c3c141d8a3a2b7118d3b57cd12fe8`).

- [x] **P6-03 — Extract `cache_lifecycle.rs`.**
  - Move local snapshot publication, every cache-clear helper, disconnect invalidation, connection-boundary invalidation, cache publication/generation helpers, and boundary predicate.
  - Do not move state fields.

  Evidence (2026-08-29): `1b93510`
  (`1b93510701d37b63d318268af07dcbe74f8264c5`).

- [x] **P6-04 — Extract `native_abi.rs`.**
  - Move raw packet/player-ID mirrors, native function aliases, and priority/reliability conversion helpers.
  - Use only the visibility needed by `hooks.rs`/`packets.rs`/root.
  - Do not merge ABI aliases as part of the move.

  Evidence (2026-08-29): `bac46be`
  (`bac46be12fc54732760ce77c81ba03f939bc9ff1`). Independent C++ ABI layout
  fixtures passed without layout or alias changes.

- [x] **P6-05 — Audit root ownership.**
  - `mod.rs` should retain shared constants, `Backend`, `BackendContext`, `BackendState`, cache-entry enums, and module wiring.
  - Do not create `state.rs` if it requires broad field visibility changes.

  Evidence (2026-08-29): `mod.rs` retains `Backend`, `BackendContext`,
  `BackendState`, cache-entry enums, shared constants, module wiring, and
  narrow imports/re-exports. State fields remain private and root-owned.

- [x] **P6-06 — Re-run lifecycle/tick/cache/hook tests.**
  - Explicitly run tests covering original call count, game-thread ID, disconnect invalidation, captured client/reconnect, hook replacement/restore, and incoming-emulation readiness.

  Evidence (2026-08-29): the unskipped 173-test host suite passed, including
  the named game-tick, disconnect, active-backend, vtable replacement/restore,
  client-hook failure, and incoming-emulation readiness tests.

- [x] **P6-07 — Verify DLL exports after module wiring.**
  - Compare against P0 export baseline where available.

  Evidence (2026-08-29): `dumpbin /exports` on local
  `target/i686-pc-windows-msvc/release/samp_client_sdk.dll` reported exactly
  `DllMain` and `SampClientSdk_GetApiV1`, matching P0.

- [x] **P6-08 — Run full host/workspace gate and record LOC.**

  Evidence (2026-08-29): `cargo make format-check`, `check`, `test`, `clippy`,
  `release-hygiene`, `package-published`, and `build-release` passed. Release
  hygiene was updated in `af758bb` for the directory-module Protocol layout.
  Physical/nonblank LOC: `mod.rs` 502/458, `lifecycle.rs` 522/503, `tick.rs`
  94/89, `cache_lifecycle.rs` 415/383, `native_abi.rs` 68/61.

### P6 gate

- [x] Win32 root remains the state composition root without carrying unrelated implementation bodies.

---

## P7 — Split native-client player and UI operations

Dependency: **P0-04 live-validation policy must be decided first.**

### Players

- [x] **P7-01 — Convert `players.rs` to a directory module.**
  - Create `players/mod.rs` plus child modules without changing behavior.

  Evidence (2026-08-29): `3f96554`
  (`3f965542e5c3daae901bdd7f8969a28424c950a0`).

- [x] **P7-02 — Extract `players/animation.rs`.**
  - Move animation catalog parsing/read and associated profile-boundary tests.

  Evidence (2026-08-29): `35c3bd5`
  (`35c3bd55285ba36305870d0ef70efeb8f238200c`).

- [x] **P7-03 — Extract `players/pool.rs`.**
  - Move counts/max/local/remote/player-state/player-stat operations and tightly coupled pool helpers.
  - Keep player ID/count validation behavior unchanged.

  Evidence (2026-08-29): `3212c00`
  (`3212c009e9697aaeefde077fc068127d37313237`).

- [x] **P7-04 — Extract `players/sync.rs`.**
  - Move on-foot/in-car/passenger/trailer/aim reads.
  - Move sync-record address/write/reset helpers and all force-sync operations.
  - Preserve `ForceSyncReset` strategy behavior.

  Evidence (2026-08-29): `d04b30b`
  (`d04b30baed5dac5e9093d9fea81531384e9ecbb2`).

- [x] **P7-05 — Extract `players/control.rs`.**
  - Move spawn, special action, local name, player colour, send rate, and their native aliases/call helpers.

  Evidence (2026-08-29): `19b6f83`
  (`19b6f83a8bd09d8cd2939b599d0927a14f4f84f8`).

- [x] **P7-06 — Audit native aliases and profile dispatch.**
  - Operation-specific aliases live with call sites.
  - No new R1/R3/R5/DL runtime branch exists.
  - No guessed RVA/offset/value introduced.

  Evidence (2026-08-29): `9d5f616`
  (`9d5f616a0fb0689c8458c0a3a267554c6d0d58a3`). Pool, sync, and control
  aliases are colocated with call sites; shared pool aliases have narrow
  sibling visibility. Existing `R1`/`Classic` dispatch remains unchanged.

### UI

- [x] **P7-07 — Convert `ui.rs` to a directory module.**
  - Create `ui/mod.rs`, `dialog.rs`, `chat.rs`, `input.rs`, `display.rs`.

  Evidence (2026-08-29): `063191a`
  (`063191a4c1b070a4f55be0be648685ff500cd328`).

- [x] **P7-08 — Extract `ui/dialog.rs`.**
  - Dialog show/close/response/client-side/selection/editbox/state/list-item behavior.
  - Preserve `ListItemTextLayout` strategy.

  Evidence (2026-08-29): `88bcac7`
  (`88bcac740af0e24a0fce06965d124f00a348eafb`).

- [x] **P7-09 — Extract `ui/chat.rs`.**
  - Chat/death-window messages, chat entry/history, display mode.

  Evidence (2026-08-29): `14f7894`
  (`14f78945b864373e0891012b0432591ecd5e6fa0`).

- [x] **P7-10 — Extract `ui/input.rs`.**
  - Chat input read/write/enable/process and chat-command registration lifecycle.
  - Preserve input bounds and callback/trampoline behavior.

  Evidence (2026-08-29): `9a3c2da`
  (`9a3c2da456d7f6cee5acc4fca5a5aa39ad74390a`).

- [x] **P7-11 — Extract `ui/display.rs`.**
  - Cursor and scoreboard operations.

  Evidence (2026-08-29): `6079814`
  (`60798146537bd6795d7b6a43ae2752f5b67d25ff`).

- [x] **P7-12 — Run all native profile parity/layout unit tests.**
  - Every supported profile remains covered.
  - Record unit-test count/results and commit hashes.

  Evidence (2026-08-29): the unskipped host suite passed 173/173, including
  all native profile and independent C++ layout tests. Full workspace format,
  check, test, Clippy, release hygiene, package, and release-build gates passed.
  The local release DLL still exports exactly `DllMain` and
  `SampClientSdk_GetApiV1`.

- [x] **P7-13 — Satisfy the live-validation evidence rule.**
  - If split-before-live-validation: record this phase's final hash as the binary source for later R1/R3/R5/DL validation.
  - If revalidate-after-split: repeat all four live validators and record hashes/status/failure/log evidence.

  Evidence (2026-08-29): under the decided `split-before-live-validation`
  policy, record `60798146537bd6795d7b6a43ae2752f5b67d25ff` as the P7 final
  native-facing source hash. R1/R3/R5/DL live validation remains deferred to
  the final native-facing binary.

  Physical/nonblank LOC: `players/mod.rs` 138/128, `animation.rs` 45/42,
  `pool.rs` 833/811, `sync.rs` 504/472, `control.rs` 200/193; `ui/mod.rs`
  149/140, `dialog.rs` 436/423, `chat.rs` 272/264, `input.rs` 309/299,
  `display.rs` 90/83.

### P7 gate

- [x] One shared `NativeClientProfile` implementation model remains and live evidence applies to the final native-facing code.

---

## P8 — Split SDK ABI source organization

- [x] **P8-01 — Convert `sdk/src/abi.rs` to `sdk/src/abi/mod.rs`.**
  - Preserve `mod abi;` in `sdk/src/lib.rs` and `pub use abi::*` behavior.

  Evidence (2026-08-29): `3855585`
  (`3855585ab5c93fcd15f66e511bc7d00281f5f4ce`).

- [x] **P8-02 — Extract `abi/values.rs`.**
  - Move owned/output `repr(C)` snapshot/value structs without changing fields/derives/defaults.

  Evidence (2026-08-29): `e826dc5`
  (`e826dc5bf5c80b5d607af7d7f4afb6579553a51b`). Zero-default ABI tests
  passed without field, derive, or default changes.

- [x] **P8-03 — Extract `abi/control.rs`.**
  - Move result/status/version/direction/action enums, subscription/receipt types, send options, encoded-string/event/callback ABI declarations.

  Evidence (2026-08-29): `4c215a1`
  (`4c215a1f395b70d3c2bba8c5c811ae237bd259ca`). Send-option defaults and
  the `Busy = 14` discriminant tests passed.

- [x] **P8-04 — Extract `abi/table.rs`.**
  - Move the entire `SampClientSdkApiV1` declaration as one uninterrupted struct.
  - Move `SampClientSdkGetApiV1` with it.
  - Do not macro-generate or nest API-table groups.

  Evidence (2026-08-29): `f70352b`
  (`f70352b0bd53becee2a0b4743059a7f69061f6ee`). The complete 145-field
  table and getter moved together as one flat declaration.

- [x] **P8-05 — Explicitly re-export ABI items from `abi/mod.rs`.**
  - Preserve all crate-root public names.
  - Avoid adding a new public `abi` namespace unless separately intended; `abi` remains private as today.

  Evidence (2026-08-29): `abi/mod.rs` explicitly re-exports all control,
  value, table, and getter names. `sdk/src/lib.rs` remains unchanged with
  private `mod abi;` and crate-root `pub use abi::*`.

- [x] **P8-06 — Run ABI/default/layout tests immediately.**
  - Include append-order and offset/size assertions.
  - Confirm `SampClientSdkApiV1` is unchanged.

  Evidence (2026-08-29): all 100 SDK tests passed. The append-order test
  confirms 145 x86 four-byte fields, 4-byte alignment, 580-byte total size,
  exact `index * 4` offsets, and final `incoming_emulation_ready` position.

- [x] **P8-07 — Run SDK/workspace gate and public API hygiene.**
  - `cargo make release-hygiene`
  - `cargo make package-published`
  - Record commit hash and any generated public API diff; expected semantic diff is none.

  Evidence (2026-08-29): workspace format, check, tests, Clippy, release
  hygiene, and published-package checks passed. Existing crate-root imports
  compiled unchanged in all workspace consumers; no semantic public API diff
  was introduced. Physical/nonblank LOC: `mod.rs` 28/24, `values.rs` 427/403,
  `control.rs` 196/175, `table.rs` 632/629.

### P8 gate

- [x] ABI source is navigable while the ABI table remains flat, complete, and byte-layout equivalent.

---

## P9 — Redistribute large tests

Do this only after the production owner for each test is stable.

### SDK tests

- [x] **P9-01 — Create `sdk/src/tests/` module shell.**
  - Keep existing private test access.

  Evidence (2026-08-29): `7735b9a`; `tests/mod.rs` remains private and
  owns only shared codecs, synchronization, and fixtures.

- [x] **P9-02 — Extract ABI/default/layout tests.**
  - `tests/abi.rs`.

  Evidence (2026-08-29): `fa9ddeb`.

- [x] **P9-03 — Extract host API/facade conversion tests.**
  - `tests/host_api.rs`.

  Evidence (2026-08-29): `4453a8b`.

- [x] **P9-04 — Extract Protocol send/convenience tests.**
  - `tests/protocol.rs`.

  Evidence (2026-08-29): `a992c30`.

- [x] **P9-05 — Extract typed/raw callback and replacement tests.**
  - `tests/callbacks.rs`.
  - Preserve exact-bit and failure-atomic replacement coverage.

  Evidence (2026-08-29): `86f9d5b`; exact-bit, malformed-source,
  replacement-encode, and host-rejection cases remain present and pass.

- [x] **P9-06 — Extract subscription lifecycle tests.**
  - `tests/subscriptions.rs`.

  Evidence (2026-08-29): `1e4ee47`.

- [x] **P9-07 — Extract resolution/probe tests if enough material exists.**
  - `tests/resolution.rs`; otherwise leave them in `tests/mod.rs` rather than creating a tiny file.

  Evidence (2026-08-29): no `tests/resolution.rs` was created. The three
  resolution tests already belong to `sdk/src/resolve.rs`; `tests/mod.rs`
  contains no resolution tests.

### Win32 tests

- [x] **P9-08 — Convert `vtable_tests.rs` to a directory module.**
  - Create a narrow shared `fixtures.rs` only for backend/profile construction helpers used across multiple groups.

  Evidence (2026-08-29): `11f62df` and `6986568`. The shared fixture file
  contains only backend/profile construction and reusable owned values.

- [x] **P9-09 — Extract cache/request tests.**
  - Group cache publication/invalidation separately from bounded request queue behavior where practical.

  Evidence (2026-08-29): cache `43bcc0c`; request queues `b7bb9ab`.

- [x] **P9-10 — Extract command/tick tests.**
  - Preserve cross-module queue-to-tick-to-completion tests at the parent test layer if needed.

  Evidence (2026-08-29): `15e8a80`; the group retains queue, native tick,
  completion, reconnect, and wait-rejection coverage.

- [x] **P9-11 — Extract hook/native lifecycle tests.**
  - Hook patch/restore, captured originals, readiness/failure visibility.

  Evidence (2026-08-29): `51d9d4e`; hook-only fake vtable state is local to
  `hooks_native.rs`.

- [x] **P9-12 — Reassess `profile_layout_tests.rs`.**
  - Split only if navigation benefit is real.
  - If split, share assertions and separate profile data without duplication.

  Evidence (2026-08-29): left intact. It is one cohesive independent C++
  layout fixture; a split would duplicate assertion and fixture structure.

- [x] **P9-13 — Run full test suite and compare test count/coverage intent.**
  - No test may disappear merely because of module movement.

  Evidence (2026-08-29): `cargo make quality` passed, including workspace
  format, check, tests, Clippy, docs, release hygiene, and package checks.
  SDK remains 100 tests and host remains 173 tests. `cargo make build-release`
  produced `target/i686-pc-windows-msvc/release/samp_client_sdk.dll`; its only
  exports are `DllMain` and `SampClientSdk_GetApiV1`.

### P9 gate

- [x] Large test roots are easier to navigate and no new monolithic `test_support` dumping ground exists.

  Physical/nonblank LOC: SDK `mod.rs` 114/94, `abi.rs` 719/709,
  `host_api.rs` 514/487, `protocol.rs` 256/243, `callbacks.rs` 849/777,
  `subscriptions.rs` 134/122; Win32 `mod.rs` 10/9, `fixtures.rs` 191/182,
  `cache.rs` 1,053/979, `requests.rs` 386/360, `commands_tick.rs` 237/205,
  `hooks_native.rs` 147/126.

---

## P10 — Split the common live-validation probe (optional/deferred)

Do not start while the live-validation contract is still changing.

- [x] **P10-01 — Confirm probe split is still justified.**
  - If the file remains cohesive enough after native work, document it as an exception and skip P10.

  Evidence (2026-08-29): split remains justified. The 2,784-line common
  harness still combines configuration, state/reporting, network, entity, UI,
  sync, reconnect, and orchestration ownership. The four profile wrappers
  continue to include the same common source.

- [x] **P10-02 — Extract `config.rs`.**
  - Profile marker/status constants, timeouts, entry-point identity, profile-specific expected values.

  Evidence (2026-08-29): `62c30b5`; every profile-selected value and `cfg`
  arm moved without value changes.

- [x] **P10-03 — Extract `state.rs` and `status.rs`.**
  - Shared probe state/observations/phase state and status-file/reporting logic.

  Evidence (2026-08-29): state `9cbdae8`; status `895dd91`. Public
  `MAIN_SUCCESS_STATUS` and `FULL_SUCCESS_STATUS` re-exports remain unchanged.
  R1/R3/R5/DL probe suites each remain 9/9 and all four pass Clippy with
  warnings denied.

- [x] **P10-04 — Extract `network.rs`.**
  - Listener registration, exact-bit/codec checks, inbound/outbound checks.

  Evidence (2026-08-29): `d598682`; listener registration preserves callback
  order and retains partial subscriptions on failure. R1-only network state was
  localized in `1181e54`.

- [x] **P10-05 — Extract `entities.rs`.**
  - Entity ID parsing, handles, player pools, object/pickup/vehicle checks.

  Evidence (2026-08-29): `c29f133`.

- [x] **P10-06 — Extract `ui.rs`.**
  - Dialog/chat/input/scoreboard/cursor/text-label/textdraw validation.

  Evidence (2026-08-29): `7894e92`.

- [x] **P10-07 — Extract `sync.rs`.**
  - Sync snapshots, force-sync receipts, vehicle phases.

  Evidence (2026-08-29): `41f9240`.

- [x] **P10-08 — Extract `reconnect.rs`.**
  - Disconnect invalidation, reconnect request, restored-state checks.

  Evidence (2026-08-29): `691bb6b`.

- [x] **P10-09 — Keep `lib.rs` as orchestration.**
  - Plugin entry/init, `run_probe`, ordered phase execution, final status.
  - Do not duplicate one file per client profile.

  Evidence (2026-08-29): `lib.rs` now owns module wiring, DLL lifecycle,
  initialization, ordered `run_probe`, shared retry orchestration, exports, and
  nine small cross-capability tests. All four profile crates use the same
  capability modules.

- [x] **P10-10 — Re-run every required live validator after the final probe refactor.**
  - Record final binaries/hashes/status/failure results.

  Final evidence (2026-08-29): host commit `f80fc8f` pins the independent DL
  pool function RVAs; probe commit `a0e36ea` decodes DL connection states
  through the Classic codec. `cargo make build-release` produced the final
  repository-local artifacts. SHA-256 values: host
  `F511FE9695DF11182892A0D8ED51EF9A6FB114E0CBD36C5B732AF4D6586F2E0F`,
  R1 `D59D992C04608CF706809B9A6B32A10F2D7A99DD2885EE3C2431C1220DB74377`,
  R3 `DC8F30B87DEA3DEF2BBE7702A49A625680DA6009AF8BDC51E184EB68B67C48F6`,
  R5 `D25036C8536E064871A75DBB7A7581E92B2FD454A1B1761AC0383AF42CADF2EB`,
  and DL `8965A48E73079890603185CC6776EF257EB1E67724D2D3136F821364CCDB255D`.

  Each installed artifact matched its repository-local hash. R1, R3, R5, and
  DL each completed the connected and reconnect run with
  `status=0x3FFFFFFF`, `failure=0`, `reconnect_server_ready=true`,
  `reconnect_local_ready=true`, `reconnect_spawned=Some(true)`, and
  `reconnect_incoming_ready=true`. R1/R3/R5/DL reconnect game state was
  `Some(14)`.

  The first R1 attempt exposed an unrelated `gta_sa_fsr.asi` crash:
  `fastman92limitAdjuster.log` recorded `EXCEPTION_ACCESS_VIOLATION` at
  `gta_sa.exe+0x3F98DF` while dereferencing a null object, with FSR frames and
  no SDK/probe frame in the captured stack. The same final host/probe pair
  completed after only `gta_sa_fsr.asi` was disabled.

  The DL investigation found a native regression introduced by inherited R3
  pool RVAs: DL requires player getter `0x1170`, vehicle getter `0x1180`, and
  vehicle-exists `0x1150`. Commit `f80fc8f` pins those values and adds the
  four-profile RVA matrix test. The corrected host completed the DL live run.

### P10 gate

- [x] The probe remains one common capability-oriented harness and final live evidence is current.

---

## P11 — Documentation and final acceptance

- [x] **P11-01 — Recount production/test LOC and document intentional exceptions.**
  - Use nonblank and physical counts.
  - Do not fail the project solely because a cohesive file remains above a soft ceiling.

  Evidence (2026-08-29), physical/nonblank totals: common incoming RPCs
  2,780/2,351; R1 incoming RPCs 1,793/1,608; Runtime 1,436/1,269;
  game-thread commands 1,952/1,866; P6 composition files 1,601/1,494;
  native players 1,720/1,646; native UI 1,256/1,209; SDK ABI 1,283/1,231;
  SDK tests 2,586/2,432; Win32 vtable tests 2,024/1,861; common probe
  2,846/2,704. Intentional cohesive exceptions include the flat 632/629-line
  ABI table, the 1,384/1,335-line independent profile layout oracle, the
  1,053/979-line cache publication/invalidation test matrix, and focused
  vehicle, player-pool, callback, probe-orchestration, and probe-UI modules.

- [x] **P11-02 — Update `ARCHITECTURE.md`.**
  - Replace obsolete single-file paths with the final module trees and ownership descriptions.

- [x] **P11-03 — Update `CORE.md`.**
  - Reflect final ownership where the core implementation map references moved modules.

- [x] **P11-04 — Update relevant agent guides.**
  - `repository-layout.md` must match directory modules.
  - Update `game-thread-commands.md` if internal ownership wording is now stale, without changing its invariants.
  - Update `packets-and-events.md` if Protocol ownership wording needs the new semantic child paths.

  Evidence (2026-08-29): `repository-layout.md` now names the directory
  modules and Protocol crate. `game-thread-commands.md` and
  `packets-and-events.md` were reviewed; their invariant-based wording remains
  current and required no change.

- [x] **P11-05 — Mark historical split plans appropriately.**
  - Keep completed plans as history.
  - Do not rewrite old baselines to pretend the new structure existed earlier.
  - Link this handoff as the follow-up where useful.

  Evidence (2026-08-29): `SPLIT_PLAN.md` now labels its paths as a historical
  baseline and links this handoff. `structural-split-plan.md` already carries
  the required historical note. Historical baseline tables were not rewritten.

- [x] **P11-06 — Update this tracker with final evidence.**
  - Every completed phase has commit hashes and validation results.
  - Every skipped optional phase has a reason.
  - Every remaining blocker has an owner/next action.

  Evidence (2026-08-29): architecture and ownership documentation commit
  `0a05e85` records the final tree, LOC, P10 hashes, and live results. No
  blocker remains.

- [x] **P11-07 — Run complete static acceptance.**
  - `cargo make format-check`
  - `cargo make check`
  - `cargo make test`
  - `cargo make clippy`
  - `cargo make release-hygiene`
  - `cargo make package-published`
  - `cargo make build-release`
  - `git diff --check`

  Evidence (2026-08-29): `cargo make quality` passed all six repository tasks:
  format, workspace check/tests, strict Clippy, documentation, release hygiene,
  and published-package checks. `cargo make build-release` and
  `git diff --check` passed. The suites included 100 SDK tests, 174 host tests,
  and 11 tests for each profile probe.

- [x] **P11-08 — Verify public/ABI/native invariants.**
  - Protocol flat incoming paths preserved.
  - SDK crate-root exports preserved.
  - ABI table size/offset/order preserved.
  - Native layout fixtures preserved.
  - Host DLL exports preserved.
  - No new version dispatcher in `NativeClientProfile` operations.

  Evidence (2026-08-29): Protocol integration tests compile through the flat
  explicit `common::*` and `r1::*` surfaces; SDK root and 580-byte, 145-field
  ABI table tests pass; independent four-profile layout tests pass. The final
  x86 host exports exactly `DllMain` and `SampClientSdk_GetApiV1`. Source audit
  found only the intended profile selection match and no operation dispatcher.

- [x] **P11-09 — Verify game-thread invariants.**
  - Single bounded queue.
  - FIFO.
  - Tick snapshot before original call.
  - Original called once.
  - Snapshot executed after original.
  - Cache refresh/invalidation ordering unchanged.
  - Receipt lifecycle unchanged.

  Evidence (2026-08-29): the full host suite and the focused
  `vtable_tests::commands_tick` suite (11/11) pass. Source/test audit confirms
  the single bounded FIFO queue, pre-original snapshot, one original call,
  post-original execution, one cache-generation bracket, connection-boundary
  invalidation, and unchanged receipt wait/timeout/detach/shutdown behavior.

- [x] **P11-10 — Verify final live evidence where required.**
  - R1 pass recorded.
  - R3 pass recorded.
  - R5 pass recorded.
  - DL pass recorded.
  - Evidence hashes correspond to the final native-facing code.

  Evidence (2026-08-29): P10-10 records R1/R3/R5/DL full passes against the
  live-deployed host `F511FE...F2E0F`. A later post-quality host rebuild has
  SHA-256 `F6C3F3BC95053E65BF496F8CFF65B8942EE9F7AF88816F130F1DE3D84F16DF9D`;
  byte and PE-section comparison proves that it differs only in COFF/debug
  timestamps and the PDB GUID. `.text`, `.data`, and `.reloc` are identical.

- [x] **P11-11 — Final repository hygiene.**
  - No temporary compatibility modules.
  - No wildcard ownership-hiding re-exports added.
  - No duplicate old/new implementation remains.
  - No unrelated user file modified or staged.
  - Conventional Commit history is reviewable by semantic slice.

  Evidence (2026-08-29): old single-file roots and duplicate implementations
  are absent; only intentional SDK crate-root wildcard exports remain. The
  temporary DL bisect worktree and stale review worktree metadata were removed.
  DL `SAMPFUNCS.asi` was restored. R1 `gta_sa_fsr.asi` remains recoverably
  disabled because it caused the independently recorded crash.

### Final acceptance

- [x] Incoming Protocol descriptors are colocated by semantic domain.
- [x] `Runtime` is a composition root rather than a mixed facade implementation file.
- [x] Game-thread command domains are separated while completion/tick semantics remain centralized.
- [x] Win32 state remains root-owned without lifecycle/tick/cache/native-ABI implementation bloat.
- [x] Native player/UI operation files have capability ownership and one profile-independent implementation path.
- [x] SDK ABI source is split without changing the flat API table or public exports.
- [x] Tests/probes are split only where the new boundary improves ownership.
- [x] Static gates pass.
- [x] Required live gates pass against the final code.
- [x] Documentation matches the landed implementation.

## Evidence log

Append one row per completed slice rather than editing historical rows.

| Date | Task | Branch | Commit | Validation/evidence | Notes |
| --- | --- | --- | --- | --- | --- |
| 2026-08-29 | P0 baseline and safety freeze | `cohesion-module-split` | `77edf87119f051bb9853f27e1c02b305e67e068f` | All static gates passed; two DLL exports recorded | Live policy: `split-before-live-validation` |
| 2026-08-29 | P1 descriptor-helper cleanup | `cohesion-module-split` | `23f85114e1e5475890837603951c5d7f90d4b985` | 80 Protocol tests, Clippy, format, diff check passed | Explicit value types; no descriptor movement |
| 2026-08-29 | P2 split common incoming RPCs | `cohesion-module-split` | `49795a9c0fa74d5c54eecca168b0f798963b9df7` | 80 Protocol tests, Clippy, format, diff check passed | Flat public imports preserved through explicit re-exports |
| 2026-08-29 | P3 split R1 incoming RPCs | `cohesion-module-split` | `4102db7110f17f1d6bafa174bdf31b734511be8e` | 80 Protocol tests, Clippy, format, diff check passed | Framing policies and flat public imports preserved |
| 2026-08-29 | P4 split Runtime facade | `cohesion-module-split` | `19e36987c41fa7a48d6b637505dd5acc318a89b4` | 173 host tests; workspace check, tests, Clippy, format, diff check passed | `runtime/mod.rs` reduced to lifecycle and composition |
| 2026-08-29 | P5 domainize game-thread commands | `cohesion-module-split` | `2ec8fcbefbdafa04f144bcbfcaf8773f102dd214` | 173 host tests; full workspace format, check, tests, and Clippy passed | Stage A final `1e6478a`; one queue and outer completion owner preserved |
| 2026-08-29 | P6 reduce Win32 composition root | `cohesion-module-split` | `bac46be12fc54732760ce77c81ba03f939bc9ff1` | Full repository gate and local release DLL export audit passed | State remains private and root-owned; release-hygiene fix `af758bb` |
| 2026-08-29 | P7 split native-client players/UI | `cohesion-module-split` | `60798146537bd6795d7b6a43ae2752f5b67d25ff` | 173 host tests; full repository gate; local DLL export audit passed | Final native-facing source hash recorded for deferred R1/R3/R5/DL live validation |
| 2026-08-29 | P8 split SDK ABI source | `cohesion-module-split` | `f70352b0bd53becee2a0b4743059a7f69061f6ee` | 100 SDK tests; full workspace and public-hygiene gates passed | Flat 145-field, 580-byte x86 API table and crate-root exports preserved |
| 2026-08-29 | P9 redistribute large tests | `cohesion-module-split` | `51d9d4e4cb2d1b75f5c6b815e5a5ea8492b3c188` | 100 SDK and 173 host tests; full quality and release gate passed | SDK and Win32 test roots split by stable behavior domains |
| 2026-08-29 | P10 final native correction | `cohesion-module-split` | `f80fc8f232c418b0732107487dc43f2ef8362304` | 174 host tests, strict host Clippy, four-profile RVA matrix, DL live pass | Pin DL pool getter/existence RVAs independently from R3 |
| 2026-08-29 | P10 final probe and live matrix | `cohesion-module-split` | `a0e36ea84c225628e5a3ba5324a58d0cbbf33fec` | Four probe suites and strict Clippy passed; R1/R3/R5/DL each reported `0x3FFFFFFF`, failure 0 | Final host and probe hashes recorded in P10-10 |
| 2026-08-29 | P11 documentation and acceptance | `cohesion-module-split` | `0a05e85842b8d6c44a132a4cba934314d23fde40` | Full quality/release gate, export audit, focused tick tests, LOC and hygiene audits passed | Architecture, core, agent layout, live evidence, and historical-plan status synchronized |

## Blockers / decisions log

| Date | Task | Decision or blocker | Owner/next action | Status |
| --- | --- | --- | --- | --- |
| 2026-08-29 | P0-04 | Use `split-before-live-validation`; prior live stage was pending | Completed by the final P10 four-profile live matrix | Resolved |
| 2026-08-29 | P5-10 | Unrelated singleton test assumed `0x17aa70` was unreadable, but the address was temporarily readable in the process | The environment condition cleared; the unchanged test passed in the 173-test unskipped host gate | Resolved |
| 2026-08-29 | P10-10 | Final R1/R3/R5/DL live validation required one exclusive probe server on UDP 7777 | All four final probes passed; server sessions were isolated during the matrix | Resolved |
