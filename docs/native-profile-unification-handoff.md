# Native Profile Unification Handoff

Status: complete. Structural migration, four-profile live validation, and
final acceptance finished on 2026-08-29; detailed artifact evidence is in the
[cohesion split tracker](cohesion-module-split-task-tracker.md#p10--split-the-common-live-validation-probe-optionaldeferred).

## Objective

Replace the version-shaped native backends with one data-driven native client:

```rust
struct NativeClientProfile {
    module_base: usize,
    spec: &'static ProfileSpec,
}
```

R1, R3-1, R5-1, and DL-R1 must be equal profiles. Shared operations must have
one implementation. Version modules must contain only verified identity data,
layouts, RVAs, limits, and narrowly scoped ABI or behavior strategies.

This document preserves the implementation plan and completion evidence. Do
not interpret its historical starting paths as the current repository layout.

## Confirmed decisions

- Use one `NativeClientProfile` and four static `ProfileSpec` values.
- Do not keep a version enum as the operation dispatcher.
- Do not create R2 or R4 profiles.
- Complete the structural refactor and static verification before any live
  validation.
- Run R1, R3, R5, and DL live validation as a separate final stage.
- When an RVA, offset, ABI, or behavior is not verified, record it explicitly,
  return `NotReady` for the affected operation, and research it after the
  structural migration. Never add a guessed value or silent fallback.

## Final state

`src/platform/win32/native_client/profile.rs` defines the single
`NativeClientProfile`. Attach-time selection chooses exactly one of four static
specifications from `native_client/profiles/`: R1, R3-1, R5-1, or DL-R1.
Shared connection, singleton, pool, player, sync, UI, text-label, textdraw,
handle, colour, and memory operations consume the selected specification
without a version-shaped operation dispatcher. The later cohesion split moved
player and UI operations into capability-oriented directory modules without
changing this model.

The historical starting baseline included:

- `448a8bc fix(r1): correct label and textdraw mutations`
- `3e03ca8 fix(probes): report profile-specific validation`
- R1 live result: `status=0x3FFFFFFF`, `failure=0`

## Work log

| Date | Branch | HEAD | Starting dirty files | Baseline checks |
| --- | --- | --- | --- | --- |
| 2026-08-16 | `feature/helpers` | `3e03ca846cab383e2cc172dab2f1675d4d91d792` | `TODO.md`; `.serena/memories/dl_binary_layout_corrections.md`; `.serena/memories/dl_exact_rva_reports.md`; `.serena/memories/dl_implementation_checkpoint.md`; `.serena/memories/dl_player_rvas.md`; `.serena/memories/dl_support_handoff.md`; `docs/native-profile-unification-handoff.md`; `docs/r1-native-profile-unification-handoff.md` | `cargo test --workspace` passed; `cargo clippy --workspace -- -D warnings` passed |

## Final module structure

```text
src/platform/win32/native_client/
  mod.rs
  profile.rs
  memory.rs
  connection.rs
  colours.rs
  singletons.rs
  players/
    mod.rs
    animation.rs
    pool.rs
    sync.rs
    control.rs
  pools.rs
  ui/
    mod.rs
    dialog.rs
    chat.rs
    input.rs
    display.rs
  text_labels.rs
  textdraws.rs
  profiles/
    mod.rs
    r1.rs
    r3.rs
    r5.rs
    dl.rs
```

Responsibilities:

- `profile.rs`: `NativeClientProfile`, nested spec types, strategies, and
  selection.
- `profiles/*.rs`: static verified data only. A profile module must not own a
  duplicate lifecycle implementation.
- Operation modules: one implementation per public/native operation.
- `memory.rs`: guarded reads/writes and bounded string helpers that are valid
  for all profiles.
- Native function aliases remain beside their checked call sites. Distinct
  aliases stay separate when ABI equality is not proven.

Do not create a single flat `ProfileSpec` with hundreds of unrelated fields.
Use nested subsystem specs so ownership and fixture coverage remain visible.

## Proposed data model

```rust
struct ProfileSpec {
    identity: ProfileIdentity,
    net_game: NetGameSpec,
    pools: PoolSpec,
    players: PlayerSpec,
    sync: SyncSpec,
    ui: UiSpec,
    text_labels: TextLabelSpec,
    textdraws: TextdrawSpec,
    handles: HandleSpec,
    strategies: ProfileStrategies,
}

struct ProfileIdentity {
    name: &'static str,
    version: SampVersion,
    entry_point: u32,
}

struct NativeRva(usize);
struct FieldOffset(usize);
struct NativeSize(usize);
```

Use small newtypes where they prevent mixing RVAs, offsets, sizes, and limits.
All specs must be immutable `'static` data.

Selection should be data-driven:

```rust
fn select(
    module_base: usize,
    version: SampVersion,
    entry_point: u32,
) -> Option<NativeClientProfile> {
    let spec = profiles::for_identity(version, entry_point)?;
    (module_base != 0).then_some(NativeClientProfile { module_base, spec })
}
```

Do not expose version-specific constructors such as `verify_r5` or
`verify_dl`.

## Source of profile data

Use these sources before moving values:

- R1 RVAs: `src/platform/win32/r1_client/addresses.rs`.
- R1 layouts and limits: `src/platform/win32/r1_client/memory.rs`.
- R1 ABI aliases: `src/platform/win32/r1_client/native_types.rs`.
- R3 base values and R3/R5/DL overrides:
  `src/platform/win32/r3_client.rs`.
- R3/R5/DL overrides are currently expressed by the `build_value` accessors
  on `ClassicClientProfile`.
- Independent layouts: `tests/fixtures/raknet_layout.cpp` and
  `src/platform/win32/profile_layout_tests.rs`.
- R1 fixture checks: `src/platform/win32/r1_client/tests.rs`.
- Entry-point identities: `src/client.rs`.

Pinned entry points:

| Profile | Entry point |
| --- | ---: |
| R1 | `0x31DF13` |
| R3-1 | `0x0CC4D0` |
| R5-1 | `0x0CBC90` |
| DL-R1 | `0x0FDB60` |

Every current `build_value(r3, r5, dl)` call must become a named spec field.
Add a spec-value test before deleting the corresponding old accessor.

## Required strategies

Offsets and RVAs are insufficient for the following confirmed differences.
Model each difference explicitly. Do not put broad version checks inside shared
operations.

### Game-state codec

R1 uses the public state values directly and accepts `0, 9, 13, 14, 15, 18`.
The classic implementation maps those states to native `0, 1, 2, 5, 6, 11`.

Use an explicit codec strategy:

```rust
enum GameStateCodec {
    Identity,
    Classic,
}
```

### Local-player source and cached address

R1 calls `CPlayerPool::GetLocalPlayer`. Classic profiles read a profile-specific
pointer field. Model this as a `LocalPlayerSource` strategy.

The cache currently publishes an R1-only `raw_r1_address`. Rename it to a
profile-neutral concept. Before declaring parity complete, either verify and
publish the equivalent address for every profile or remove the profile-specific
contract. Do not retain an R1-named field in the unified design.

### Native boolean representation

R1 uses `read_r1_bool`; classic operations validate integer boolean fields at
their call sites. Define the actual representation and validation policy in
the relevant layout/spec. Do not assume all fields share one representation.

### Force-sync behavior

Every profile clears `last_any_update` and writes records through its selected
layout before the native send call. Keep one shared force-sync flow
parameterized by `SyncSpec` and an explicit reset strategy. The public
unoccupied-sync seat contract is `u8` for every profile; native calls receive
that value widened to their `i32` ABI parameter.

### Dialog list-item layout

R1 follows a `DXUTComboBoxItem` and reads its embedded text. The classic path
treats an item as a direct text pointer. Use a narrow `ListItemTextLayout`
strategy.

### String validation

R1 and classic direct methods currently differ on embedded NUL handling and
empty server hostnames. The unified public behavior must be identical.
Normalize to the safer bounded policy unless an existing public test requires
another result:

- reject embedded NUL input;
- reject unterminated or oversized native strings;
- treat an empty hostname consistently as not ready.

Do this as an explicit behavior-normalization change with tests, not as an
accidental side effect of moving code.

### Textdraw native calls

Create/delete/set-text RVAs differ. R1 and classic profiles also use different
pool/layout values. Keep one operation flow with typed RVAs and layout data.
The verified R1 text setter is `0xAC870` and must remain a native call.

### Native function signatures

R1 and classic currently define separate aliases. Merge an alias only after
confirming parameters, return type, and calling convention for all four
profiles. Otherwise store a typed strategy or retain separate aliases behind a
shared operation.

### Server metadata

R1 currently accepts an empty hostname while classic rejects it. Normalize the
observable result and cover it with parity tests.

### GTA handle conversion

The GTA `CPools::GetPedRef` and `CPools::GetVehicleRef` targets are shared GTA
addresses, not SA-MP module RVAs. Keep them outside the per-SA-MP profile spec
unless evidence proves a profile dependency.

## Migration sequence

Each phase must compile and pass static tests. Do not combine all moves into one
commit.

### Phase 1: Freeze the profile data

1. Add nested spec types and four static specs without changing runtime
   selection.
2. Copy every existing R1 constant and every R3/R5/DL `build_value` result into
   a named field.
3. Add tests comparing new specs with the existing constants/accessors and C++
   fixtures.
4. Create `docs/native-profile-unverified-values.md` for every value without
   independent evidence. Record profile, operation, current source, required
   evidence, and status.

### Phase 2: Replace selection and identity dispatch

1. Introduce `NativeClientProfile { module_base, spec }`.
2. Return it from one selector for R1/R3/R5/DL.
3. Replace enum-pattern logging with `spec.identity.name`.
4. Keep the old operation implementations temporarily behind the new profile
   only if needed for an incremental build. Do not add a new permanent enum
   dispatcher.

### Phase 3: Move shared primitives

1. Consolidate guarded memory reads/writes and bounded strings.
2. Consolidate native type aliases only where ABI equality is proven.
3. Move singleton lookup and profile-neutral address calculation.
4. Add tests for overflow, invalid ranges, invalid booleans, embedded NULs, and
   unterminated strings.

### Phase 4: Migrate operations by subsystem

Use this order to limit coupling:

1. connection and server metadata;
2. singleton lookup and UI readiness;
3. players and player pools;
4. sync reads and force-sync mutations;
5. handles, vehicles, objects, pickups, and gangzones;
6. chat, input, dialog, scoreboard, cursor, and death window;
7. text labels;
8. textdraws;
9. animation catalog and remaining helpers.

For each subsystem:

1. add the required spec fields and strategies;
2. port one shared implementation;
3. run fixture and parity tests for all four specs;
4. delete the superseded R1/classic implementation immediately;
5. commit the completed subsystem separately.

### Phase 5: Remove the forwarding architecture

1. Replace the forwarding methods in `native_profile.rs` with direct
   `NativeClientProfile` methods.
2. Update `BackendContext`, refresh functions, commands, reads, and tests to use
   the new type.
3. Remove `NativeProfile`, `R1ClientProfile`, `ClassicClientProfile`,
   `ClassicVersion`, and `build_value`.
4. Remove `r1_client.rs`, `r1_client/`, and `r3_client.rs` only after their last
   operation and test have moved.

### Phase 6: Restore equal observable behavior

1. Replace `raw_r1_address` with the resolved profile-neutral contract.
2. Apply the shared string and server-metadata policy.
3. Ensure every operation exposed through `NativeClientProfile` has the same
   result semantics for all four profiles.
4. Add a compile-time or table-driven parity test that exercises the complete
   operation surface for all specs.

### Phase 7: Documentation and static gates

Update:

- `ARCHITECTURE.md`
- `CORE.md`
- relevant agent guides if module ownership changes
- fixture documentation and the unverified-value register

Run:

```powershell
cargo fmt --all -- --check
$env:CARGO_TARGET_DIR='target'; cargo test --workspace
$env:CARGO_TARGET_DIR='target'; cargo clippy --workspace -- -D warnings
$env:CARGO_TARGET_DIR='target'; cargo build --workspace
git diff --check
```

## Static test requirements

- Selector tests identify exactly R1, R3, R5, and DL specs.
- Every layout field is fixture-backed or listed in the unverified-value
  register.
- Every strategy has tests for all applicable variants.
- All four profiles traverse the same backend refresh and command paths.
- No test helper constructs a version-specific backend type.
- No shared operation contains `if version == ...` or a match over SA-MP
  versions. Version selection belongs in the spec and narrow strategies.
- Invalid pointers, ranges, enums, booleans, strings, and IDs retain current
  safe failures.
- No native pointer or borrowed native string crosses the host ABI.

## Known verification gaps

Record these before migration; do not silently treat current constants as
independent proof:

- Most native function and singleton RVAs are not checked against shipped
  client binaries.
- R1 RVAs are centralized but only the textdraw setter has a direct pin test.
- Classic R3/R5/DL non-layout RVAs are mostly tested only through constants or
  synthetic memory.
- GTA handle-conversion targets lack an executable fixture.
- Native ABI signatures and calling conventions are not fixture-verified.
- Some R1 remote-player fixture fields exist without individual Rust
  assertions.
- `NET_GAME_SERVER_SETTINGS_OFFSET` is fixture-backed but unused.

Use the agreed missing-value workflow:

1. add an entry to `docs/native-profile-unverified-values.md`;
2. make the affected spec field optional or use an explicit unavailable
   strategy;
3. return `DirectClientError::NotReady` for that operation;
4. add a test proving there is no fallback or guessed call;
5. research and pin the value after the structural migration;
6. remove the register entry only when fixture, binary, or live evidence is
   recorded.

## Separate live-validation stage

Do not perform live validation during the structural phases. After the full
refactor and all static gates pass, prepare a separate run for each profile:

1. R1
2. R3
3. R5
4. DL

Use the shared validators and require the complete status mask, zero failure,
all lifecycle phases complete, codec round trip, and exact packet/RPC bits.
Do not declare the unification complete from R1 evidence alone.

## Completion criteria

- One runtime type: `NativeClientProfile`.
- Four equal static specs: R1, R3, R5, and DL.
- One implementation of every shared operation.
- Version-specific modules contain data and narrow verified strategies only.
- No `ClassicVersion`, `build_value`, or version forwarding matches remain.
- No R1-specific cache field or public behavior remains without an equivalent
  profile-neutral contract.
- All static gates pass.
- The unverified-value register contains every remaining evidence gap.
- The later live-validation stage passes independently for all four profiles.

## First implementation task

Define the nested `ProfileSpec` model and four static specs, then add tests that
compare each spec field with the current constants, `build_value` results, and
C++ fixtures. Do not move native operations until this data-freeze phase is
green.

## Progress checklist

### Tracking rules

- Check an item only after its stated verification passes.
- Add the implementing commit hash after each completed subsection.
- Keep at most one subsystem migration in progress.
- Record every unknown value in
  `docs/native-profile-unverified-values.md` before continuing.
- Do not mark a value verified from an existing Rust constant alone.
- Do not start live validation until every structural and static item is
  complete.
- Preserve unrelated working-tree changes.

### 0. Baseline and ownership

- [x] Select Variant B: one `NativeClientProfile` plus static specs.
- [x] Complete read-only architecture research.
- [x] Create this implementation handoff.
- [x] Record the starting branch, HEAD, and dirty files in the work log.
- [x] Confirm the baseline workspace tests pass before refactoring.
- [x] Confirm the baseline Clippy check passes.
- [x] Create `docs/native-profile-unverified-values.md`.
- [x] Add the known verification gaps from this handoff to that register.

Evidence/commit: `a1f438a` (`feat(native-client): add profile data model`)

### 1. Profile data model

- [x] Add `native_client/profile.rs`.
- [x] Define `NativeClientProfile`.
- [x] Define nested `ProfileSpec` subsystem types.
- [x] Add RVA, offset, and size newtypes where they prevent category errors.
- [x] Define narrow strategy types for confirmed behavioral differences.
- [x] Keep the new model unused by runtime selection during this phase.
- [x] Add construction tests for every spec type.
- [x] Run formatting and targeted tests.

Evidence/commit: `a1f438a` (`feat(native-client): add profile data model`)

### 2. Freeze R1 data

- [x] Create `native_client/profiles/r1.rs`.
- [x] Move R1 identity and entry point into `R1_SPEC`.
- [x] Copy all R1 native RVAs into named nested specs.
- [x] Copy all R1 layouts, sizes, capacities, and limits.
- [x] Record the R1 game-state codec.
- [x] Record the R1 local-player getter strategy.
- [x] Record the R1 native-boolean rules.
- [x] Record the R1 force-sync reset strategy.
- [x] Record the R1 list-item text layout.
- [x] Record the R1 textdraw native-call strategy.
- [x] Compare every moved layout value with the C++ fixture.
- [x] Register each unverified RVA or ABI instead of assuming proof.
- [x] Add exhaustive R1 spec-value tests.

Evidence/commit: `9470327` (`feat(native-client): freeze r1 profile data`)

### 3. Freeze R3 data

- [x] Create `native_client/profiles/r3.rs`.
- [x] Move R3 identity and entry point into `R3_SPEC`.
- [x] Copy every R3 base layout, RVA, size, capacity, and limit.
- [x] Replace each R3 `build_value` input with a named spec field.
- [x] Record the classic game-state codec and applicable strategies.
- [x] Compare every layout value with the R3 fixture.
- [x] Register each unverified RVA or ABI.
- [x] Add exhaustive R3 spec-value tests.

Evidence/commit: `e7a0002` (`feat(native-client): freeze r3 profile data`)

### 4. Freeze R5 data

- [x] Create `native_client/profiles/r5.rs`.
- [x] Move R5 identity and entry point into `R5_SPEC`.
- [x] Materialize every R5 `build_value` override as a named field.
- [x] Copy shared classic values explicitly through reusable spec constants,
  without inheriting an R3 profile object.
- [x] Compare every layout value with the R5 fixture.
- [x] Register each unverified RVA or ABI.
- [x] Add exhaustive R5 spec-value tests.

Evidence/commit: `bbb1858` (`feat(native-client): freeze r5 profile data`)

### 5. Freeze DL data

- [x] Create `native_client/profiles/dl.rs`.
- [x] Move DL identity and entry point into `DL_SPEC`.
- [x] Materialize every DL `build_value` override as a named field.
- [x] Record DL limits, extended object pool, sync layouts, and RVAs.
- [x] Compare every layout value with the DL fixture.
- [x] Register each unverified RVA or ABI.
- [x] Add exhaustive DL spec-value tests.

Evidence/commit: `9fcc749` (`feat(native-client): freeze dl profile data`)

### 6. Profile selection

- [x] Add `native_client/profiles/mod.rs`.
- [x] Implement one identity-to-spec selector.
- [x] Select only R1, R3, R5, and DL.
- [x] Reject mismatched entry points.
- [x] Reject a zero module base.
- [x] Replace version-specific verification constructors.
- [x] Replace enum-pattern logging with profile identity data.
- [x] Add selector tests for all profiles and mismatch cases.
- [x] Keep runtime behavior otherwise unchanged.

Evidence/commit: `11ddec9` (`feat(native-client): select immutable profiles`)

### 7. Shared memory and ABI primitives

- [x] Move guarded read/write helpers into `native_client/memory.rs`.
- [x] Move bounded native-string helpers.
- [x] Add overflow and invalid-range tests.
- [x] Add embedded-NUL and unterminated-string tests.
- [x] Define field-specific native boolean handling.
- [x] Audit every function-pointer alias across all four profiles.
- [x] Merge only aliases with proven identical signatures.
- [x] Record unverified signatures in the register.
- [x] Ensure no native pointer or borrowed string crosses the host ABI.

Evidence/commit: `55e73eb` (`refactor(native-client): share guarded memory primitives`)

### 8. Connection and server metadata

- [x] Migrate RakPeer address resolution.
- [x] Migrate disconnect handling.
- [x] Migrate game-state reads and writes through `GameStateCodec`.
- [x] Migrate reconnect/server-address mutation.
- [x] Migrate server metadata snapshots.
- [x] Normalize empty-hostname behavior.
- [x] Add cross-profile result-parity tests.
- [x] Remove the superseded R1/classic connection implementations.

Evidence/commit: `0637ae4` (`refactor(native-client): share connection operations`)

### 9. Singletons and readiness

- [x] Migrate NetGame and pool-root lookup.
- [x] Migrate chat, input, dialog, scoreboard, death-window, and game lookup.
- [x] Migrate readiness checks.
- [x] Preserve checked ranges for every singleton.
- [x] Add synthetic-memory tests for every profile.
- [x] Remove the superseded singleton implementations.

Evidence/commit: `321d598` (`refactor(native-client): share singleton resolution`)

### 10. Players and player pools

- [x] Migrate player-pool lookup and counts.
- [x] Migrate local-player lookup through `LocalPlayerSource`.
- [x] Migrate local-player snapshots.
- [x] Migrate remote-player directory and snapshots.
- [x] Migrate score, ping, name, colour, health, and armour reads.
- [x] Migrate player mutations.
- [x] Resolve the profile-neutral raw local-player address contract.
- [x] Add parity tests for R1/R3/R5/DL layouts and results.
- [x] Remove superseded player implementations.

Evidence/commits: `24988d8` (`refactor(native-client): share player pool scalars`), `c57008a` (`refactor(native-client): share local player snapshot`), `6bc9d89` (`refactor(native-client): share remote player reads`), `51b3c9a` (`refactor(native-client): share player mutations`)

### 11. Sync records and force-send operations

- [x] Migrate on-foot sync reads.
- [x] Migrate in-car sync reads.
- [x] Migrate passenger sync reads.
- [x] Migrate trailer sync reads.
- [x] Migrate aim sync reads.
- [x] Migrate last-update reset logic through `SyncSpec`.
- [x] Migrate every force-send operation.
- [x] Add exact-layout and mutation tests for every profile.
- [x] Remove superseded sync runtime dispatch implementations.

Evidence/commit: `24848ad` (`feat(native-client): share sync operations`); `cargo clippy --workspace -- -D warnings` and `cargo test -p samp-client-sdk-host --lib` (192 passed).

### 12. Handles and entity pools

- [x] Migrate ped and vehicle handle conversion.
- [x] Keep GTA addresses outside SA-MP specs unless evidence requires otherwise.
- [x] Migrate vehicle existence and game-object lookup.
- [x] Migrate object and pickup pools.
- [x] Migrate gangzones.
- [x] Add limit and invalid-ID tests for every profile.
- [x] Cover DL extended object limits.
- [x] Remove superseded handle/entity runtime dispatch implementations.

Evidence/commits: `ff6bfa9` (`refactor(native-client): share entity pool reads`) and
`df8b8ae` (`refactor(native-client): share gta handle conversion`); `cargo clippy
--workspace -- -D warnings` and `cargo test -p samp-client-sdk-host --lib` (194
passed).

### 13. UI operations

- [x] Migrate chat entries and messages.
- [x] Migrate death-window messages.
- [x] Migrate chat input and command registration.
- [x] Migrate dialog show, close, response, and edit-box operations.
- [x] Migrate list items through `ListItemTextLayout`.
- [x] Migrate scoreboard operations.
- [x] Migrate cursor operations.
- [x] Apply one bounded input-string policy to all profiles.
- [x] Add cross-profile behavior-parity tests.
- [x] Remove superseded UI runtime dispatch implementations.

Evidence/commits: `c03ecfe` (`refactor(native-client): share ui cache reads`),
`1cc6360` (`refactor(native-client): share chat command cache`), `0650e54`
(`refactor(native-client): share dialog cache reads`), `b45d8c6`
(`refactor(native-client): share ui memory writes`), `8279b26`
(`refactor(native-client): share ui native calls`), `e07530c`
(`refactor(native-client): share ui input operations`), `d3e0185`
(`refactor(native-client): share ui command routes`), and `3e4d2e4`
(`test(native-client): cover ui profile boundaries`); `cargo clippy --workspace --
-D warnings` and `cargo test -p samp-client-sdk-host --lib` (196 passed).

### 14. Text labels

- [x] Migrate label existence and snapshots.
- [x] Migrate label creation, mutation, and deletion.
- [x] Preserve the shared ARGB-to-native conversion.
- [x] Add layout, colour, capacity, and lifecycle tests for all profiles.
- [x] Remove superseded label runtime dispatch implementations.

Evidence/commit: `0bc9dfd` (`refactor(native-client): share text label operations`);
`cargo clippy --workspace -- -D warnings` and `cargo test -p
samp-client-sdk-host --lib -- --test-threads=1` (198 passed).

### 15. Textdraws

- [x] Migrate textdraw existence and snapshots.
- [x] Migrate create and delete native calls.
- [x] Migrate all numeric/style mutations.
- [x] Migrate text mutation through the verified native setter strategy.
- [x] Preserve R1 setter RVA `0xAC870` and its 799-byte input bound.
- [x] Add layout, capacity, setter, and lifecycle tests for all profiles.
- [x] Remove superseded textdraw implementations.

Evidence/commit: `575aa9f` (`refactor(native-client): share textdraw
operations`); strict workspace Clippy and 200 host tests passed at the phase
checkpoint.

### 16. Remaining shared operations

- [x] Migrate the animation catalog.
- [x] Migrate send-rate globals and mutations.
- [x] Migrate any operation not covered by earlier sections.
- [x] Audit the complete old forwarding surface for omissions.
- [x] Add operation-surface parity coverage.

Evidence/commit: `9111794` (`refactor(native-client): share animation
catalog`); strict workspace Clippy and 199 host tests passed at the phase
checkpoint.

### 17. Remove the old architecture

- [x] Replace all `NativeProfile` type references.
- [x] Replace `BackendContext` and refresh-path references.
- [x] Replace command/read/test constructors.
- [x] Delete the forwarding match block.
- [x] Delete `ClassicVersion` and `build_value`.
- [x] Delete `R1ClientProfile` and `ClassicClientProfile`.
- [x] Delete `r1_client.rs`, `r1_client/`, and `r3_client.rs` after they are empty.
- [x] Confirm no R1/R3/R5/DL version branch remains in shared operations.
- [x] Confirm no version-specific cache field remains.

Evidence/commit: `c39022b` (`refactor(native-client): remove legacy profiles`).

### 18. Documentation and static completion

- [x] Update `ARCHITECTURE.md`.
- [x] Update `CORE.md`.
- [x] Update relevant agent guides.
- [x] Update fixture documentation.
- [x] Review the unverified-value register for completeness.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace`.
- [x] Run `cargo clippy --workspace -- -D warnings`.
- [x] Run `cargo build --workspace`.
- [x] Run `git diff --check`.
- [x] Confirm unrelated user files remain untouched.

Evidence/commit: `6ff9d8c` (`docs: record cohesion split acceptance`); the
final quality, release, export, ABI/native, and repository-hygiene results are
recorded in tracker P11.

### 19. Separate live-validation stage

- [x] Confirm the structural refactor is complete before starting this stage.
- [x] Build and deploy the shared host and R1 validator.
- [x] Complete R1 live validation with the full status mask and zero failure.
- [x] Build and deploy the R3 validator.
- [x] Complete R3 live validation with the full status mask and zero failure.
- [x] Build and deploy the R5 validator.
- [x] Complete R5 live validation with the full status mask and zero failure.
- [x] Build and deploy the DL validator.
- [x] Complete DL live validation with the full status mask and zero failure.
- [x] Record hashes, status records, and relevant logs for all four profiles.
- [x] Resolve every live regression before declaring completion.

Evidence/commits: `f80fc8f` pins the independent DL pool function RVAs;
`a0e36ea` corrects DL public connection-state expectations; `6ff9d8c` records
final acceptance. Exact host/probe hashes and all four `0x3FFFFFFF`, failure 0
results are in tracker P10-10. The unrelated R1 FSR crash is isolated in the
same evidence record.

### 20. Final acceptance

- [x] Exactly one runtime native-profile type remains.
- [x] Exactly four equal static profile specs remain.
- [x] Every shared operation has one implementation.
- [x] Profile modules contain only data and narrow verified strategies.
- [x] No unregistered guessed RVA, offset, ABI, or fallback exists.
- [x] Every remaining evidence gap is registered.
- [x] All static gates pass.
- [x] All four live validators pass independently for their named probe
  surfaces.
- [x] Final commits follow Conventional Commits.
- [x] The handoff, checklist, capability matrix, and architecture documentation
  match the code and recorded evidence.

Final evidence/commits: native completion through `c39022b`; final DL
correction `f80fc8f`; probe correction `a0e36ea`; accepted documentation and
quality/live evidence `6ff9d8c`.
