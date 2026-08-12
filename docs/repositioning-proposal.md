# Repositioning Proposal: `rak-samp` → `samp-client-sdk`

Status: approved. Decisions locked 2026-08-04; work has not started yet.

## 1. Motivation

The project drifted away from its original identity. The docs frame `rak-samp`
as "a networking host", but the actual scope is a two-pillar SDK: RakNet
middleware plus a large read-only game-state bridge (the TODO backlog mirrors
SF.lua's 207 globals). The evidence-driven posture (fingerprints, fixtures as
offset proof, opt-in live validators, `[~]` provisional statuses, E2E fixture
runs) was built to compensate for unverified offsets, but the offsets are
well-known community knowledge (plugin-sdk, SAMPFUNCS, SAMPGDX ship the same
tables) and time-proven by the mods built on them. The regime added maintenance
cost without value.

The API shape is also wrong: a C-style procedural surface inherited from
SAMPFUNCS/SF.lua, with mutations and pointer access excluded as a policy
matter, and native calls crossing from plugin threads (races against the game
thread).

This proposal repositions the project and restructures the code accordingly.

## 2. New identity

`samp-client-sdk` is a safe-by-default, exhaustive SA-MP client SDK for Rust
ASI plugins: RakNet networking middleware, game-state access, UI mutations,
and raw access where explicitly opted into — running on known-stable offsets,
with all mutation synchronized to the game thread.

Two pillars, one invariant:

- **Net/middleware** — intercept, inspect, block, replace, send, and emulate
  packets/RPCs; typed codecs; `BitStream`.
- **State/bridge** — cached, game-thread-refreshed game-state reads and
  game-thread-executed mutations; SF.lua-compatible where sensible.
- Invariant: cached state is refreshed and mutations execute on the game
  thread, never on plugin threads.

Gone: the provisional `[~]` system, fingerprint gates, "pending live
validation" language, and every "excluded from the safe ABI" permanent
exclusion. Every SAMPFUNCS/SF.lua function is implemented in some form — as a
struct method, a projection, a queued mutation, a handle operation, or an
explicit `unsafe` escape hatch. No function is left unclassified.

## 3. What gets removed

| Artifact | Disposition |
| --- | --- |
| `tests/e2e/` (fixture host, plugin, runner) | Delete |
| `examples/validation_plugin/`, `examples/validation_unloader/` | Delete |
| `REVIEW.md`, `VALIDATION.md` | Delete |
| Fingerprint/PE-verification gates (`r1_client.rs`, `client.rs`) | Replace with plain offset constants |
| Provisional `[~]` statuses and live-evidence language in `TODO.md` and elsewhere | Rewrite |
| ABI self-tests and doc sections tied to validation lifecycle | Rewrite |

### What stays

| Artifact | Disposition |
| --- | --- |
| `tests/fixtures/raknet_layout.cpp` + `layout_tests` + `build.rs` cc wiring | Keep — C++↔Rust layout agreement checks |
| Unit tests and the mock-ABI test support in `sdk` | Keep |
| Hooks, dispatch, codecs, cache machinery, bounded C ABI | Keep (restructured) |

Rationale for the C++ layout fixture: it verifies that Rust `repr(C)` structs
agree with the C++ native layouts — a pure build-time check that catches
packing mistakes the offset constants never would. It needs no live client and
has no maintenance cost beyond keeping fixture structs in sync with the Rust
`repr(C)` types they mirror.

## 4. Game-thread hook (plugin-sdk style)

Current design pumps state inside the incoming-packet detour — per-packet, not
per-frame — and mutations are only queued UI draws. Plugin threads calling
native mutations race against the game thread.

New design:

- Detour **`CGame::Process`** (SA 1.0 US `0x53BEE0`, the same hook plugin-sdk
  uses). This is the game tick:
  - refresh all caches (local snapshot, player directory, textdraws, labels,
    objects, gangzones, UI flags, server info, animation table),
  - drain the **command queue**: mutation commands submitted by plugin threads
    execute here, on the game thread, atomically per frame.
- Keep the RakClient packet detour for the networking pillar (unchanged).
- Plugin threads call into the ABI as today; **mutating calls are queued and
  execute on the tick**. Reads stay non-blocking cached reads. A queued
  command can be awaited when the plugin needs its result.

This gives the primitive for everything plugin-sdk does, including arbitrary
queued closures later.

## 5. API design: struct facade over the C ABI

The C ABI stays as internal plumbing (versioned, bounded, C-compatible). The
public API is a Rust struct hierarchy living in `sdk`:

```rust
samp.net()               // subscriptions, send_rpc/packet, bitstream, codecs
samp.game_state()        // opaque CNetGame state
samp.server()            // .address(), .hostname(), .port()
samp.local()             // LocalPlayer facade
    .id(), .nickname(), .colour(), .spawned()
    .health(), .armour(), .special_action(), .animation_id()
    .set_health(98.0)            // queued mutation
    .set_colour(0xFF00FF00)      // queued mutation
    .spawn(...), .request_spawn(), .request_class(...)
samp.players()
    .count(), .max_id()
    .get(id) -> Player   // .info(), .health(), .is_defined(), .is_paused(),
                         // .set_colour(...), .force_sync(...)
samp.textdraws()         // .get(idx), .exists(idx), .create(...), .set_string(...), .delete()
samp.labels()            // .get(id), .exists(id), .create(...), .set_text(...), .destroy()
samp.objects()           // .get(id), .exists(id), .create(...), .delete(), .set_pos(...)
samp.vehicles()          // .get(id), .exists(id), .create(...), .set_health(...), .repair()
samp.gangzones()         // .get(id), .create(...), .set_colour(...), .delete()
samp.dialogs()           // .show(...), .active(), .current_id(), .type()
samp.chat()              // .add(...), .display_mode(), .set_display_mode(...), .input()...
samp.cursor()            // .mode(), .set_mode(...), .toggle(...)
samp.scoreboard()        // .is_open(), .toggle(...)
samp.anim()              // .name(id), .id(name, file)
samp.version()           // build identity
```

Every SF.lua global maps onto this hierarchy. Exact method names and the
mapping table are settled during implementation, not frozen here.

### Implementation tiers

1. **Safe reads** — cached, non-blocking (today's surface, minus provisional
   gates).
2. **Safe mutations** — queued to the game tick, atomic by construction. All
   the `sampSet*` / `sampTextdrawSet*` / `sampForce*` family.
3. **Unsafe raw access** — explicit `unsafe` module for pointer-returning
   functions:

   ```rust
   pub mod raw {
       pub unsafe fn base() -> usize
       pub unsafe fn rakclient() -> *mut c_void
       pub unsafe fn rakpeer() -> *mut c_void
       // + raw netgame/pools accessors, callbacks
   }
   ```

### Handles and IDs

Handles returned by SAMPFUNCS/SF.lua (`sampGetObjectHandleBySampId` and
friends) become typed wrappers, not integers:

```rust
pub struct VehicleHandle(u32);  // GTA vehicle handle
pub struct ObjectHandle(u32);
pub struct PedHandle(u32);
impl ObjectHandle { pub fn to_id(self) -> Option<ObjectId> }
```

SA-MP IDs also get newtypes (`PlayerId`, `VehicleId`, `ObjectId`,
`TextdrawIndex`, ...) instead of bare integers. Functions that accept handles
take the wrappers.

## 6. Rename

- Project/crates: `rak-samp` → `samp-client-sdk`; plugin API crate follows
  (exact crate names settled during the rename pass).
- ASI output renamed accordingly (from `rak_samp.asi`).
- Repo/docs/naming updated in one pass. `samp-sdk` is taken on crates.io by an
  unrelated abandoned crate; `samp-client-sdk` is free and chosen.
- Git history will be rewritten as part of the work; the old artifacts are
  deleted outright, not archived.

## 7. Migration plan

### Phase 1 — cleanup, rename, docs (no behavior change in what remains)

1. Delete `tests/e2e/`, `examples/validation_plugin/`,
   `examples/validation_unloader/`, `REVIEW.md`, `VALIDATION.md`.
2. Replace fingerprint/PE-verification gates with plain offset constants;
   strip provisional `[~]`/live-evidence language.
3. Rename crates, ASI, workspace members, CI, Makefile tasks.
4. Rewrite docs: `README.md` (new identity, two-pillar overview, compatibility
   notes, install, plugins), `CORE.md` (per-pillar sections), `ARCHITECTURE.md`
   (module map), `TODO.md` (feature checklist of the 207 mapped items with
   their tier), `AGENTS.md` (layout: fixture rule only, no live evidence).
5. Full workspace build + tests green (including the retained C++ layout
   fixture tests and unit tests).

### Phase 2 — game-thread tick

1. Add the `CGame::Process` detour; move the cache pump there.
2. Introduce the command queue with blocking completion for mutations.
3. Migrate existing queued UI drains (dialog/chat/death-window) into it.

### Phase 3 — struct facade

1. Newtype ID and handle types.
2. Build the `samp.*` facade over the ABI; move reads and mutations behind it.
3. Implement the mutation tier (dialog/chat/death window, textdraw/label/
   object/vehicle/gangzone setters, cursor/scoreboard, force-sync sends).
4. Add the `raw` unsafe module.
5. Finish the remaining SF.lua mapping (create/create variants, streamed
   reads, remaining sends) and delete nothing from the catalog — everything
   ends up classified into one of the tiers.

## 8. Guardrails

- No behavior change in surviving code during Phase 1; the retained unit and
  layout tests are the guard.
- The C ABI stays C-compatible and versioned; the struct facade is Rust sugar
  over it.
- Nothing is "excluded forever": any function without a safe form gets an
  `unsafe` form, and every function is classified in `TODO.md`.
- Over-engineering is rejected: trivial things stay trivial. The three tiers
  exist to say where each function lands, not to grow ceremony.
