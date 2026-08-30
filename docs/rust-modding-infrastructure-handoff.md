# Rust GTA SA / SA-MP Modding Infrastructure — Implementation Handoff

**Status:** implementation specification; individual requirements are marked `CURRENT`, `DECIDED`, or `PROPOSED`  
**Date:** 2026-08-26  
**Canonical path:** `docs/rust-modding-infrastructure-handoff.md`  
**Implementation baseline:** the working tree present when Phase 0 begins / package `samp-client-sdk-host 0.1.0-alpha.4`  
**Primary target:** GTA San Andreas 1.0 US, Windows x86 (`i686-pc-windows-msvc`)  
**Primary SA-MP targets:** 0.3.7 R1, 0.3.7 R3-1, 0.3.7 R5-1, 0.3.DL-R1  
**Audience:** Codex or another coding agent continuing the repository without needing architectural clarification.

**Compatibility policy:** no source, binary, or stable ABI compatibility is promised before release `1.0.0`. Existing behavior remains a migration and regression reference, but pre-1.0 APIs and layouts may break when the new architecture requires it.

### Requirement status markers

Use these markers when reading and updating this document:

- **`CURRENT`** — verified description of the repository snapshot or behavior that exists now and must be preserved unless explicitly migrated.
- **`DECIDED`** — normative architecture/ABI decision. Codex must implement it as written unless the user explicitly changes the decision or new evidence makes it impossible.
- **`PROPOSED`** — a suggested API shape or later-stage design whose exact form may change without violating the architecture. If implementation needs to choose among alternatives, record the choice in an ADR before crossing crate/ABI boundaries.

Do not silently reinterpret a `CURRENT` fact as a desired future design, and do not treat a `PROPOSED` sketch as a published ABI commitment.

---

## 0. Executive directive

The repository is no longer to be treated as only a RakNet bridge or only a SA-MP helper library. The long-term product is a **native Rust modding infrastructure for classic GTA San Andreas and SA-MP**.

The existing code is valuable and must be migrated, not discarded. In particular, preserve and generalize:

- the single native host model;
- host-owned hooks;
- game-thread execution of native mutations;
- bounded command queues and receipts;
- callback/unload synchronization;
- scoped game-thread access and owned values instead of safe APIs exposing game pointers;
- exact-bit RakNet `BitStream` handling;
- typed packet/RPC codecs;
- R1/R3-1/R5-1/DL-R1 data-only native profiles;
- independent C++ layout fixtures;
- explicit unsafe raw-address escape hatches.

The target architecture must **not** be a line-by-line Rust transcription of `plugin-sdk` or `SAMPFUNCS`. Those projects are sources of verified reverse-engineering knowledge and behavioral reference. The public Rust API must be designed around one coherent Rust model.

### 0.1 Non-negotiable invariants — `DECIDED`

These are the one-page rules Codex should check before every architectural change:

1. **Never guess native contracts.** Addresses, offsets, class sizes, packing, vtable slots, signatures, calling conventions, enum values, and version parity require evidence. Unsupported is preferable to guessed.
2. **One host owns core hooks and native process state.** Ordinary plugins subscribe to host services instead of independently detouring the same GTA/SA-MP targets.
3. **GTA is below SA-MP.** GTA engine functionality lives in GTA crates/services; SA-MP may depend on GTA, never the reverse.
4. **Safe APIs never expose borrowed references into game memory.** Use callback-scoped `GameContext`, handles, owned snapshots, copied strings/bytes, or explicitly `unsafe` raw escape hatches.
5. **Safe native access is game-thread confined.** A runtime-validated `GameContext` permits synchronous reads and operations whose Native execution constraint allows the current phase. Off-thread callers submit owned reads/mutations and receive receipts/futures.
6. **FFI boundaries are panic- and allocator-safe.** Rust panics never unwind through ABI calls, and Rust/C++ allocator-owned containers never cross the stable plugin ABI.
7. **Published service tables are exact-version and immutable.** Add a V2 or a new service instead of appending fields to a stable V1.
8. **Legacy behavior is migrated, not rewritten wholesale.** Add abstraction -> route existing behavior -> prove parity -> remove superseded code.
9. **DLL unload is synchronized.** A plugin is never freed while callbacks into it or host work owned by it may still execute.
10. **Protocol code remains platform-independent where possible.** RakNet bitstream/codecs and other pure data logic must be testable without Windows, GTA, or SA-MP loaded.
11. **plugin-sdk and SAMPFUNCS are evidence/reference sources, not the public Rust object model.** Do not copy their C++ abstractions mechanically.
12. **Current proven R1/R3-1/R5-1/DL-R1 behavior must survive foundational refactors.** Compatibility is measured per Capability, not by profile label. Removing proven behavior requires a separate explicit decision.

The core rule is:

> **One host owns native process state and hooks. Plugins consume stable, versioned C ABI services through safe Rust facades.**

Do not make `cxx.rs` the runtime foundation. Direct Rust x86 ABI calls are already used successfully in the repository. C++ may be used only for test/oracle code or narrowly scoped compatibility adapters.

---

# 1. Agent execution contract

These rules are normative for implementation.

## 1.1 Work incrementally

Do not perform a repository-wide rewrite. Every migration phase must leave the workspace buildable and the existing SA-MP behavior intact unless the phase explicitly replaces it and has parity tests.

For each phase:

1. add the replacement abstraction;
2. route existing behavior through it;
3. add/retain tests proving equivalence;
4. only then delete superseded code;
5. update `ARCHITECTURE.md`, `CORE.md`, `TODO.md`, and relevant `docs/agent-guides/*` in the same change.

## 1.2 Never guess native data — `DECIDED`

Never invent:

- addresses;
- RVAs;
- field offsets;
- class sizes;
- vtable slots;
- calling conventions;
- packing;
- enum values;
- SA-MP version parity.

If a value cannot be verified, do not promote it into a production profile or public capability. Record it in an `*-unverified-values.md` file, mark the feature unsupported/not-ready, and continue with other verified work. A missing implementation is acceptable. A guessed native contract is not.

## 1.3 Native evidence taxonomy — `DECIDED`

Every new native fact added to a central profile/symbol/layout database must record provenance and an evidence grade. A "source" means an independently attributable observation; two repositories that copied the same constant are not two independent sources.

### Grade A — directly verified

At least one direct observation against the target binary/build, for example:

- independent C++ layout/oracle fixture compiled against the reference SDK and checked against expected target layout;
- live runtime verification on the exact supported executable/module version;
- static disassembly/decompilation of the exact target binary with enough surrounding instructions to establish the symbol/offset/signature;
- deterministic signature/pattern resolution tied to a known module hash plus a regression fixture.

Grade A is acceptable for production support.

### Grade B — corroborated

No direct local oracle is available yet, but the fact has at least two genuinely independent matching sources, or one trusted external source plus independent disassembly/signature/layout corroboration. Examples include a plugin-sdk value confirmed independently by SAMPFUNCS and by surrounding binary behavior, provided provenance shows they are not simply copied from each other.

Grade B is acceptable for production support, but add a regression test/fixture as soon as practical.

### Grade C — single-source / inherited / weakly verified

Examples:

- one external header/repository with no independent confirmation;
- an undocumented constant inherited from legacy code whose provenance is unknown;
- a forum post or snippet without reproducible evidence;
- a value that "works on my machine" but is not tied to a verified target build.

Grade C is **not** sufficient for a newly claimed production capability. Keep it in an unverified/reference dataset or behind an explicitly experimental feature.

### Grade U — unknown

No trustworthy provenance. Do not use for production behavior.

For existing working `rak_rs` values whose provenance is not currently documented: do not delete working support merely to satisfy this taxonomy. Keep the current backend intact, then assign A/B evidence while migrating each value into the new centralized database. Do not duplicate an undocumented legacy constant into a new profile as if it were newly verified.

`docs/native-profile-unverified-values.md` is the authoritative Evidence
register until its contents are migrated into a replacement with equal or
better provenance. A dated smoke report upgrades only the exact facts and
Capabilities that it exercises; it does not verify an entire profile. Before
moving profile data, create a feature-by-profile matrix that separates profile
recognition, runtime readiness, and individual Capabilities.

Recommended metadata shape:

```text
symbol = "CGame::Process"
value = 0x53BEE0
target = "gta-sa-1.0-us"
evidence_grade = "A"
source = "tests/fixtures/... + target disassembly note"
verified_at = "2026-..."
notes = "..."
```

The exact storage format is `PROPOSED`; the evidence requirement is `DECIDED`.

## 1.4 Preserve loader and FFI safety — `DECIDED`

- Keep heavy initialization out of `DllMain`.
- Do not let Rust panics unwind across FFI.
- Do not let Rust references, trait objects, `String`, `Vec`, allocator-owned objects, or borrowed game objects cross the stable DLL ABI.
- Do not free a plugin DLL while a callback into it may still run.
- Do not wait for receipts/subscription drains from the game thread, `DllMain`, or a host callback.
- Safe plugin APIs must not construct Rust references into GTA/SA-MP memory.
- Native pointers stay host-internal unless surfaced through an explicitly `unsafe` API.

## 1.5 Plugin trust model — `DECIDED`

Third-party native plugins are **trusted in the operating-system/process-security sense**. They are DLL/ASI code executing inside the GTA process and can bypass the SDK, call Win32 APIs, patch memory, or crash the process. The host does not and cannot provide a meaningful security sandbox for malicious native plugins.

The host **does** defend against accidental misuse and lifecycle bugs at its ABI boundary:

- validate null pointers, lengths, enum/range inputs, service versions, and handle/ID existence;
- copy borrowed input bytes/strings before retaining them asynchronously;
- never retain a plugin pointer beyond the lifetime promised by the ABI contract;
- reject blocking waits from contexts where they would deadlock;
- synchronize callback removal and plugin unload;
- isolate Rust panics at callback/FFI boundaries;
- fail closed (`InvalidArgument`, `NotReady`, `UnsupportedVersion`, etc.) instead of dereferencing invalid inputs when the host can detect them cheaply.

Do not spend foundational milestones building pseudo-sandboxing, SEH-based containment of arbitrary plugin faults, or adversarial memory isolation. Those cannot make in-process native plugins untrusted.

## 1.6 Preserve privacy of plugin payloads — `DECIDED`

Current logging intentionally avoids logging arbitrary chat/dialog/RPC payloads. Keep that policy. Diagnostic logs may contain IDs, sizes, versions, addresses of host-owned targets, status codes, and queue metadata, but should not dump plugin-owned text/network payloads by default.

## 1.7 Validation commands — `CURRENT` / `DECIDED`

The migration target toolchain and MSRV are Rust 1.98. Before Rust 1.98.0 is
stable, development that requires this target may use the matching beta
toolchain. Once stable is available, pin CI and `rust-version` to `1.98` and do
not leave the release build on a floating `stable` channel.

At the end of each behavior-changing phase, run from the workspace root:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
```

Where a newly extracted crate is platform-independent, also ensure it can be tested independently of GTA/SA-MP and does not contain the workspace-wide Windows x86 `compile_error!` restriction.

Do not mark a phase complete if the relevant commands fail unless the failure is an already-recorded environment limitation and the code has been verified by CI or the target Windows toolchain.

---

# 2. Current repository baseline — `CURRENT`

This section records the state that the migration must preserve.

## 2.1 Workspace/package baseline

Current root package:

```text
name        = samp-client-sdk-host
version     = 0.1.0-alpha.4
edition     = 2024
rust-version= 1.87
crate type  = cdylib
host output = samp_client_sdk.asi
```

The `rust-version = 1.87` value above describes the current working-tree
baseline. It must move to the decided Rust 1.98 target during the migration.

Current public plugin SDK package:

```text
sdk/
package = samp-client-sdk
lib     = samp_client_sdk
```

Target defaults to `i686-pc-windows-msvc`.

Current host dependencies include `minhook`, `windows-sys`, `log`, and `simplelog`.

## 2.2 Current host/plugin model

Current architecture:

```text
ASI loader
   |
   +-- samp_client_sdk.asi     <-- owns hooks/native state
   |
   +-- plugin-a.asi ----------- C ABI --> host
   +-- plugin-b.asi ----------- C ABI --> host
   +-- plugin-c.asi ----------- C ABI --> host
```

Keep this mode working during migration.

A future host-managed `mods/*.dll` loader is allowed, but it is a later milestone and must not block the foundational work.

## 2.3 Current ABI

The host exports:

```text
SampClientSdk_GetApiV1
```

The current `SampClientSdkApiV1` in `sdk/src/abi.rs` contains **145 fields**, including its `abi_version` and `size` fields.

This table has grown beyond the size that should be used as the permanent architecture.

Important current compatibility flaw:

`HostApi::from_raw` in `sdk/src/api.rs` requires:

```rust
raw.size >= size_of::<SampClientSdkApiV1>()
```

Therefore, despite comments about append-only v1 compatibility, a plugin compiled against a newer larger `SampClientSdkApiV1` cannot safely consume an older shorter v1 table. The next architecture must fix this by using **immutable exact-version service tables**, not by continuing to append fields to one monolithic structure.

Do not add new fields to `SampClientSdkApiV1` unless required to fix a critical alpha regression before the new service ABI is available.

## 2.4 Current command execution invariant

`src/command.rs` contains a generic bounded `CommandQueue<C, R>`.

Current important semantics:

- queue capacity: **256**;
- FIFO;
- one tick snapshot drains commands accepted before snapshot acquisition;
- commands submitted after the snapshot wait for the next tick;
- receipt timeout does not consume the receipt;
- dropping/releasing a receipt does not cancel the command;
- shutdown rejects new submissions and completes retained receipts;
- waiting can be rejected for game-thread/callback callers.

These semantics are part of the desired generic runtime and must be preserved.

## 2.5 Current game-thread hook

`src/platform/win32/mod.rs` currently owns the absolute GTA SA address:

```text
CGame::Process = 0x53BEE0
```

It installs a MinHook detour and uses it to:

1. identify the game thread;
2. snapshot queued commands before the original;
3. call the original exactly once;
4. execute the captured commands after the original;
5. refresh/publish coherent caches.

This is a **GTA runtime responsibility**, not a SA-MP responsibility, and must be extracted as such.

## 2.6 Current GTA helpers incorrectly living under SA-MP

Current direct-client code includes GTA pool-ref calls:

```text
CPools::GetPedRef     = 0x54FF60
CPools::GetVehicleRef = 0x54FFC0
```

They currently live in SA-MP profile operation modules. They must move into the GTA native backend.

Current public SA-MP facade defines `PedHandle`, `VehicleHandle`, `ObjectHandle`, and `PickupHandle`. Those are GTA concepts and must move to the GTA-facing crate/types. SA-MP may return them when mapping an SA-MP pool entity to a GTA entity.

## 2.7 Current SA-MP native profiles

The repository already has a good unified profile model under:

```text
src/platform/win32/native_client/
  profile.rs
  profiles/
    r1.rs
    r3.rs
    r5.rs
    dl.rs
```

`NativeClientProfile` contains:

```rust
module_base: usize
spec: &'static ProfileSpec
```

`ProfileSpec` is already decomposed into nested data-only subsystem specs such as:

- identity;
- net game;
- pools;
- players;
- sync;
- UI;
- text labels;
- textdraws;
- handles;
- strategies.

This design is good. Preserve it and move it into the future SA-MP native crate with minimal behavioral change.

Recognized entry-point identities currently include:

```text
R1    0x31DF13
R3-1  0x0CC4D0
R5-1  0x0CBC90
DL-R1 0x0FDB60
```

Do not broaden recognized identities or advertised Capabilities until verified.

## 2.8 Current RakNet/protocol strengths

`sdk/src/raknet.rs` contains an owned `BitStream` with:

```rust
bytes: Vec<u8>
bit_len: usize
read_offset: usize
```

Important properties to preserve:

- exact bit length;
- bounded payload sizes;
- checked cursors;
- exact-bit read/write;
- byte helpers;
- native string codec integration;
- no native RakNet pointer lifetime exposed to plugins.

`sdk/src/events/` already contains typed packet/RPC codecs and replacement semantics. The protocol/codec portion must become platform-independent.

## 2.9 Current subscription strengths

Current callback API already provides:

- owned `Subscription` objects;
- unregister;
- `unregister_and_wait`;
- callback lifetime synchronization;
- `catch_unwind` around plugin callbacks;
- registration-order dispatch;
- callback-local event lifetime;
- exact-bit atomic replacement;
- continue/block/replace semantics.

Preserve this behavior and generalize the lifetime mechanism so GTA events and future render/input services can use the same subscription model.

## 2.10 Current native memory safety helpers

`src/platform/win32/native_client/memory.rs` uses `VirtualQuery` before reads/writes and provides bounded string helpers.

Keep the validation policy, but do not make per-scalar `VirtualQuery` calls the long-term hot-path API for plugin-sdk-scale entity reads. Introduce a validated memory-region/view abstraction later so one validated range can serve multiple field accesses within a native operation.

---

# 3. North-star architecture — `DECIDED`

The final logical architecture should be:

```text
                            +-----------------------+
                            |      Rust plugin      |
                            |  safe Rust facades    |
                            +-----------+-----------+
                                        |
                               stable C ABI services
                                        |
                            +-----------v-----------+
                            |     gta_mod_host      |
                            |   single ASI host     |
                            +-----+-----------+-----+
                                  |           |
                      +-----------+           +----------------+
                      |                                        |
             +--------v---------+                     +--------v---------+
             | GTA SA runtime   |                     | SA-MP service    |
             |/native backend   |<--------------------|/native backend   |
             +--------+---------+                     +--------+---------+
                      |                                        |
              GTA memory/calls                         samp.dll/RakNet
                      |
                gta_sa.exe
```

Public plugin code must not know whether a SA-MP operation was historically implemented by SAMPFUNCS, SF.lua, MoonLoader, or direct reverse-engineered offsets.

## 3.1 Target workspace

Directory names below are normative for the migration. Published crate package names may receive a project prefix later if crates.io naming requires it, but do not churn names during the initial split.

```text
crates/
  modkit-abi/
  modkit-runtime/
  modkit-sdk/
  modkit-win32/
  gta-sa/
  gta-sa-native/
  samp-protocol/
  samp/
  samp-native/
  sampfuncs-compat/        # later/optional

host/
  gta-mod-host/            # final location of the cdylib/ASI host

examples/
  gta-basic-plugin/
  samp-chat-plugin/
  samp-network-plugin/
```

Migration does **not** begin by physically moving the root package into `host/`. Add crates beside the current root first. Move/rename the root host only after service routing is stable.

## 3.2 Dependency rules

These are hard architectural boundaries.

### `modkit-abi`

Purpose: stable C ABI primitives shared by host and plugin-side crates.

May depend on:

- `core`/standard primitive types only;
- optionally `std` only if unavoidable, but prefer no allocator dependency.

Must not depend on:

- `windows-sys`;
- MinHook;
- GTA/SA-MP native crates;
- `Vec`, `String`, Rust trait objects in ABI declarations.

### `modkit-runtime`

Purpose: platform-neutral host runtime primitives.

Own:

- command queue/receipt state;
- subscription lifecycle primitives;
- callback activity tracking;
- generic tick phase scheduling abstractions where possible.

Must not contain GTA/SA-MP addresses.

### `modkit-win32`

Purpose: Windows x86 implementation primitives.

Own:

- PE/module inspection;
- guarded memory regions;
- page protection helpers;
- hook wrappers/broker internals;
- loader utility functions;
- thread identity helpers.

Only this layer and the native backends should need `windows-sys`/MinHook.

### `modkit-sdk`

Purpose: plugin-side safe connection to the host and service discovery.

Own:

- `Host::connect` / host discovery;
- resolution of `GtaModHost_GetApiV1` without fallback to the legacy SA-MP export;
- service query wrapper;
- common subscription wrapper;
- common command receipt wrapper;
- callback-scoped `GameContext` wrapper and token-validation plumbing;
- plugin-side panic boundaries/macros later.

Must not depend on native backends.

### `gta-sa`

Purpose: plugin-side GTA SA value types and safe facade.

Examples:

- `Vector2`, `Vector3`, matrices where appropriate;
- typed GTA handles;
- owned entity snapshots;
- `Gta`, `World`, `Peds`, `Vehicles`, `Camera`, etc.;
- game event subscriptions.

Must not contain fixed native addresses or direct memory dereferences.

### `gta-sa-native`

Purpose: host-only direct GTA SA backend.

Own:

- GTA executable detection/profile;
- raw verified layouts;
- typed native function calls;
- GTA-specific memory access;
- GTA handle conversion;
- CGame tick integration;
- entity snapshot/mutation implementations.

### `samp-protocol`

Purpose: pure protocol library.

Own:

- `BitStream`;
- packet/RPC identifiers;
- typed codecs;
- pure serialization/deserialization;
- protocol test vectors.

Must build/test without a loaded GTA process and without Windows x86-only restrictions.

### `samp`

Purpose: plugin-side safe SA-MP facade.

Own:

- SA-MP IDs;
- server/chat/dialog/player/pool facades;
- network event facade built using `samp-protocol`;
- conversions that return GTA types/handles from `gta-sa`.

Must not directly access `samp.dll`.

### `samp-native`

Purpose: host-only SA-MP direct backend.

Own:

- R1/R3/R5/DL profiles;
- SA-MP singleton/pool access;
- RakClient hooks;
- native SA-MP UI/player/pool operations;
- sync state operations;
- mapping of SA-MP entities to GTA backend handles.

May depend on `gta-sa-native` for GTA-specific operations instead of reimplementing them.

### `sampfuncs-compat`

Purpose: optional later compatibility layer only.

It must not be required by the core host. It may provide adapters for selected SAMPFUNCS behavior or use C++/`cxx` where unavoidable.

---

# 4. Stable service ABI specification — `DECIDED`

The current monolithic `SampClientSdkApiV1` must stop growing. Introduce a small host bootstrap ABI whose primary job is **exact-version service discovery**. The definitions in this section are normative unless explicitly marked `PROPOSED`.

## 4.1 ABI-level scalar types and result codes

Do not expose a Rust `enum` directly as the extensible result type. Use an integer-compatible transparent newtype so future hosts can return a code that an older plugin does not know without creating an invalid Rust enum discriminant.

```rust
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModResult(pub i32);

pub const MOD_OK: ModResult                  = ModResult(0);
pub const MOD_NOT_READY: ModResult           = ModResult(1);
pub const MOD_INVALID_ARGUMENT: ModResult    = ModResult(2);
pub const MOD_UNSUPPORTED_VERSION: ModResult = ModResult(3);
pub const MOD_NOT_FOUND: ModResult           = ModResult(4);
pub const MOD_OUT_OF_BOUNDS: ModResult       = ModResult(5);
pub const MOD_PAYLOAD_TOO_LARGE: ModResult   = ModResult(6);
pub const MOD_NATIVE_CALL_FAILED: ModResult  = ModResult(7);
pub const MOD_CALLBACK_IN_PROGRESS: ModResult= ModResult(8);
pub const MOD_QUEUE_FULL: ModResult          = ModResult(9);
pub const MOD_PENDING: ModResult             = ModResult(10);
pub const MOD_TIMED_OUT: ModResult           = ModResult(11);
pub const MOD_WAIT_REJECTED: ModResult       = ModResult(12);
pub const MOD_SHUTTING_DOWN: ModResult       = ModResult(13);
pub const MOD_BUSY: ModResult                = ModResult(14);
pub const MOD_UNSUPPORTED: ModResult         = ModResult(15);
pub const MOD_BUFFER_TOO_SMALL: ModResult    = ModResult(16);
```

Rules:

- `0` means success; non-zero means non-success.
- Existing numeric assignments are immutable after first publication.
- Unknown positive codes must be treated by older wrappers as a generic host/service error while preserving the raw integer for diagnostics.
- Do not use negative values in V1. Reserve them for possible transport/system failures in a future ABI.
- A function that returns `MOD_PENDING` must document which receipt/operation ID becomes authoritative for completion.

IDs use transparent fixed-width integer types:

```rust
#[repr(transparent)]
pub struct ServiceId(pub u32);

#[repr(transparent)]
pub struct SubscriptionId(pub u64);

#[repr(transparent)]
pub struct CommandReceiptId(pub u64);
```

`0` is invalid/reserved for all host-issued IDs. Host-issued IDs must not be reused during the lifetime of a host process in a way that could make a stale plugin ID refer to a different live object. The simplest valid implementation is monotonic allocation with checked overflow; on exhaustion return `MOD_BUSY`/`MOD_NATIVE_CALL_FAILED` rather than wrapping.

The callback-scoped native execution proof uses these fixed scalar values:

```rust
#[repr(transparent)]
pub struct GameContextTokenV1(u64);

#[repr(transparent)]
pub struct NativeExecutionConstraintV1(u32);
```

Token `0` is invalid. Plugins must not synthesize tokens. The host validates
token lifetime, owner thread, shutdown state, and operation constraint on each
direct native call. Constraint values are `0` = game thread at any phase, `1`
= post-game-process only, `2` = render phase only, and `3` = queued only.

## 4.2 New host export

Add the new export while preserving the existing export:

```text
GtaModHost_GetApiV1
```

Do not delete `SampClientSdk_GetApiV1` during the migration.

Normative bootstrap ABI:

```rust
#[repr(C)]
pub struct ServiceHeader {
    pub service_id: ServiceId,
    pub version: u32,
    pub size: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct ModHostApiV1 {
    pub abi_version: u32,
    pub size: u32,
    pub query_service: unsafe extern "system" fn(
        service: ServiceId,
        requested_version: u32,
        out_service: *mut *const ServiceHeader,
    ) -> ModResult,
}

pub type GetModHostApiV1 =
    unsafe extern "system" fn(out_api: *mut *const ModHostApiV1) -> ModResult;
```

Export semantics:

- `GtaModHost_GetApiV1(NULL)` returns `MOD_INVALID_ARGUMENT`.
- On any failure, the host writes null to a non-null `out_api` before returning.
- On success, `*out_api` points to a host-owned immutable `ModHostApiV1` table valid until host shutdown/process termination.
- `abi_version` is exactly `1` for this layout.
- `size` is exactly the published V1 struct size. Do not use it as permission to append fields to V1 after stabilization.
- All reserved fields must be zero when produced and ignored when consumed.

`query_service` semantics:

- `out_service == NULL` -> `MOD_INVALID_ARGUMENT`.
- The host writes null to `*out_service` before performing lookup.
- Requests an **exact** service ID + version pair.
- Exact pair is compiled into the Host and the registry is published -> `MOD_OK` and non-null table pointer, regardless of native backend readiness.
- Known service but requested version unavailable -> `MOD_UNSUPPORTED_VERSION`.
- Unknown service ID -> `MOD_NOT_FOUND`.
- Host registry not yet published -> `MOD_NOT_READY`.
- Host is shutting down -> `MOD_SHUTTING_DOWN`.
- Native backend or operation readiness is reported by the returned Service's status/functions, not by repeated discovery.
- Returned tables are host-owned, immutable, and valid until host shutdown. Plugins never free them.
- Every service table begins with an exact `ServiceHeader` prefix whose `service_id`, `version`, and `size` match the returned table.

All stable exported functions and callbacks use `extern "system"`. On the primary x86 Windows target this is the Windows system ABI (`stdcall`). Do not expose C++ member ABI or Rust ABI across the stable plugin boundary.

## 4.3 Pointer, buffer, and ownership contract

Unless a service function explicitly says otherwise:

- input pointer + length pairs are borrowed only for the duration of the call;
- a null pointer is allowed only when the corresponding length is zero **and** the function documentation explicitly permits an empty value;
- the host must copy input bytes before retaining them after the call returns;
- output pointers are caller-owned writable storage valid for the duration of the call;
- host-returned raw pointers, when a low-level service intentionally exposes one, are borrowed/opaque and must carry explicit unsafe lifetime documentation; the safe facade must not turn them into Rust references;
- no ABI function transfers ownership of memory allocated by the Rust global allocator, C++ allocator, or CRT allocator across module boundaries;
- strings use UTF-8 byte pointer + byte length unless a service explicitly documents an opaque byte encoding;
- text inputs are not required to be NUL-terminated unless the function is explicitly a C-string compatibility function;
- variable-size outputs use either caller-provided buffers or a two-call `required_len` pattern; insufficient capacity returns `MOD_BUFFER_TOO_SMALL` and reports the required size through a fixed-width out parameter;
- structs crossing the ABI are `#[repr(C)]`, contain only fixed-layout ABI-safe fields, and document whether trailing/reserved bytes must be zero.

Do not expose `String`, `Vec<T>`, slices, Rust references, trait objects, `Option<NonNull<T>>`, C++ STL types, or allocator-specific ownership across this boundary.

## 4.4 Callback and context ownership contract

Each service defines event-specific callback signatures, but all callbacks follow these rules:

- callback ABI is `unsafe extern "system"`;
- every registration accepts an opaque plugin context pointer (`*mut c_void`) when per-registration state is required;
- every registration that retains a plugin context also accepts a plugin `unsafe extern "system" fn(*mut c_void)` release callback;
- the host stores the context value unchanged, never dereferences it, and passes it exactly once to the release callback after callback drain;
- the plugin owns the context allocation and must keep it valid until registration has been removed **and** `unregister_and_wait` has completed successfully (or host shutdown has synchronously guaranteed no further callbacks);
- the host catches/contains its own Rust panics before invoking plugin code and the safe Rust plugin wrapper catches plugin panics before returning through FFI;
- callback parameters are borrowed for the callback duration only unless documented as copied/owned snapshots;
- after successful `unregister_and_wait(subscription_id, ...)`, no callback for that subscription may begin or remain in flight;
- ordinary unregister without wait may disable future callback starts but may return while an already-entered callback is still running; callers that will unload/free callback state must use the draining variant;
- safe `Subscription` drop must schedule Deferred reclamation after callback drain instead of permanently leaking plugin callback state during normal operation;
- Deferred reclamation must not block the game thread or a host-to-plugin callback; a process-teardown leak is an allowed fail-safe;
- `unregister_and_wait` completes only after the release callback has returned; successful completion permits immediate plugin DLL unload for that registration;
- the Host must never invoke a release callback after the owning plugin DLL has been unloaded;
- the same subscription callback may not be invoked concurrently with itself unless the service explicitly opts into concurrent delivery. Serialization does not prove Game-thread execution; the Host must validate callback thread provenance before issuing a `GameContext`;
- callback ordering for a single service/event source must preserve the ordering documented by that service.

A callback may submit non-blocking commands, query owned values, log, and perform other operations explicitly documented as callback-safe. A callback holding a valid `GameContext` may also perform synchronous native operations whose Native execution constraint allows the current phase. It must not perform an operation whose contract says `may block game thread`.

## 4.5 Threading, reentrancy, and blocking rules

Every service function must be classified in rustdoc/ABI docs as one of:

```text
ANY_THREAD
GAME_THREAD_ONLY
CALLBACK_SAFE
MAY_BLOCK
```

A function may have more than one applicable property (for example `ANY_THREAD + CALLBACK_SAFE`). Missing classification is a documentation/test failure for newly published services.

Every native operation exposed through a `GameContext` must additionally record
one Native execution constraint:

```text
GAME_THREAD_ANY_PHASE
POST_GAME_PROCESS_ONLY
RENDER_PHASE_ONLY
QUEUED_ONLY
```

Add another constraint only when native evidence requires it. Ecosystem
practice plus project smoke evidence may establish Grade B for
`GAME_THREAD_ANY_PHASE`; do not assume that every callback source runs on the
Game thread.

V1 defaults:

- `GtaModHost_GetApiV1` and `query_service`: `ANY_THREAD`, `CALLBACK_SAFE`, non-blocking.
- host logging: `ANY_THREAD`, `CALLBACK_SAFE`, non-blocking after bounded/copying work.
- command submission: `ANY_THREAD`, `CALLBACK_SAFE`, non-blocking; returns receipt/queue status.
- receipt poll/release: `ANY_THREAD`, `CALLBACK_SAFE`.
- receipt wait: `MAY_BLOCK`; callers must not invoke it from `DllMain`; reject with `MOD_WAIT_REJECTED` from the known game thread or a host->plugin callback context that could deadlock progress.
- subscription unregister: `ANY_THREAD`, `CALLBACK_SAFE`, non-blocking disable operation.
- subscription `unregister_and_wait`: `MAY_BLOCK`; reject from the same unsafe contexts as receipt wait.
- direct native reads/mutations require a valid `GameContext` and a compatible Native execution constraint; ordinary plugin-thread facades submit them to the game-thread executor.

Reentrancy rule: a plugin callback may call back into functions documented `CALLBACK_SAFE`. Host internal locks must be designed so this does not self-deadlock. Do not invoke arbitrary plugin callbacks while holding a lock that callback-safe service functions need to acquire.

## 4.6 Host shutdown semantics

Once shutdown begins:

1. the host enters a monotonic `ShuttingDown` state;
2. new service discovery may return `MOD_SHUTTING_DOWN`;
3. new subscriptions and command submissions are rejected with `MOD_SHUTTING_DOWN`;
4. existing subscriptions are disabled from starting new callbacks;
5. in-flight callbacks are drained according to the host shutdown path;
6. queued/retained receipts are completed with a shutdown result where possible;
7. service table memory remains valid until no plugin code can legitimately call the host again / process teardown reaches the documented final point;
8. all host-issued IDs become invalid after shutdown and are never meaningful across process restarts.

The host must not unload itself while independently loaded plugin DLLs can still hold service table pointers. In the initial architecture the host should remain resident for process lifetime; host hot-unload is a non-goal.

## 4.7 Initial service IDs

Use stable integer service IDs. Reserve ranges by subsystem:

```text
0x0000_0001  Core
0x0000_1000  GTA SA
0x0000_2000  SA-MP
0x0000_2001  SA-MP Network
0x0000_3000  Render            (future)
0x0000_4000  Input             (future)
0x0000_F000  Legacy SA-MP ABI  (migration only)
```

Place constants in `modkit-abi`. Once first published for plugin consumption, do not renumber them.

## 4.8 Core service v1

`CoreServiceV1` owns cross-service lifetime primitives rather than duplicating them in every service.

Minimum responsibilities:

- host status/version;
- common subscription unregister;
- `unregister_and_wait`;
- common command receipt poll/wait/release;
- host logging;
- game-thread/callback-context detection needed to enforce wait rejection.

Normative fixed result carrier:

```rust
#[repr(C)]
pub struct CommandCompletionV1 {
    pub status: ModResult,
    pub reserved: u32,
    pub value0: u64,
    pub value1: u64,
}
```

`value0/value1` are only for compact scalar/handle results. Large owned results use fixed service-specific output structs, snapshots, or caller-provided buffers.

Frozen V1 layout:

```rust
#[repr(C)]
pub struct CoreServiceV1 {
    pub header: ServiceHeader,

    pub host_status: unsafe extern "system" fn(out: *mut HostStatusV1) -> ModResult,
    pub unregister: unsafe extern "system" fn(id: SubscriptionId) -> ModResult,
    pub unregister_and_wait: unsafe extern "system" fn(
        id: SubscriptionId,
        timeout_ms: u32,
    ) -> ModResult,

    pub receipt_poll: unsafe extern "system" fn(
        id: CommandReceiptId,
        out: *mut CommandCompletionV1,
    ) -> ModResult,
    pub receipt_wait: unsafe extern "system" fn(
        id: CommandReceiptId,
        timeout_ms: u32,
        out: *mut CommandCompletionV1,
    ) -> ModResult,
    pub receipt_release: unsafe extern "system" fn(id: CommandReceiptId) -> ModResult,

    pub log_utf8: unsafe extern "system" fn(
        level: u32,
        ptr: *const u8,
        len: u32,
    ) -> ModResult,
}
```

`HostStatusV1.state` is `0` = waiting, `1` = ready, `2` = failed, and
`3` = shutting down. For every Core timeout, `0` requests an immediate check,
`0xFFFF_FFFF` requests an unbounded wait, and other values are finite
milliseconds. Log levels are `0` = error, `1` = warn, `2` = info, and `3` =
debug. Core V1 accepts at most 4096 log-message bytes. Unknown log levels and
larger messages return `MOD_INVALID_ARGUMENT`. This layout is frozen; do not
append fields.

## 4.9 Service table immutability

Rules:

- `GtaSaServiceV1` is immutable once declared stable.
- `SampServiceV1` is immutable once declared stable.
- `SampNetServiceV1` is immutable once declared stable.
- Adding functionality after stabilization means V2 or a new service.
- Plugin-side wrappers request the highest exact version they understand and may fall back explicitly.
- Never cast a shorter table to a larger Rust struct.
- `size` is primarily a defensive validation field, not an extension mechanism.

Before project release `1.0.0`, including alpha and beta releases, any service API or ABI may break. When a layout changes, bump its pre-release ABI version/constant, update every workspace consumer, and record the break so mismatched binaries fail closed. At `1.0.0`, every table published as stable becomes immutable.

## 4.10 Legacy bridge service

During migration, register service ID `0x0000_F000` as a normal header-prefixed wrapper around the existing `SampClientSdkApiV1`:

```rust
#[repr(C)]
pub struct LegacySampServiceV1 {
    pub header: ServiceHeader,
    pub api: *const c_void,
}
```

The pointer is opaque here so `modkit-abi` does not depend on the legacy SDK
crate. An explicitly unsafe migration adapter may cast it to the exact legacy
V1 table type. The safe `modkit-sdk` service view does not expose raw tables.

This exists only so the new host/service discovery path can be introduced without rewriting the whole current SDK in the same commit. The legacy endpoint may return a pointer to the existing static table for exact legacy-v1 consumers.

The old `SampClientSdk_GetApiV1` export may temporarily return the same nested
API pointer. Do not add new features to this legacy Service. Replace or remove
it before `1.0.0` when workspace consumers use normal Services; do not preserve
its special shape as a permanent exception.

# 5. Plugin-side API model — `DECIDED`

The intended user experience is Rust-first and safe by default.

Conceptual usage:

```rust
use modkit::prelude::*;

fn init(host: Host) -> Result<()> {
    let gta = host.gta_sa()?;
    let samp = host.samp()?;

    gta.events().game_process(|| {
        // safe callback body
    })?;

    samp.chat_input().register_command("hello", |args| {
        // copied args
    })?;

    Ok(())
}
```

Do not expose C++ class names as the primary public model merely because plugin-sdk or SAMPFUNCS uses them.

The first supported consumer API is the safe Rust SDK. The C ABI is the stable
transport boundary, but a supported C/C++ SDK and generated headers are later
work.

## 5.1 Handles, not borrowed native references

Do not expose:

```rust
&'static mut CPed
&mut CVehicle
&CEntity
```

Instead expose typed identities:

```rust
PedHandle
VehicleHandle
ObjectHandle
PickupHandle
```

and owned snapshots:

```rust
PedSnapshot
VehicleSnapshot
EntitySnapshot
```

A handle is an identity/token, not a Rust borrow and not a guarantee that the underlying native object still exists.

Every operation that uses a handle must validate it against the current game state/profile before native use.

Safe Rust GTA handle wrappers store their positive signed raw token as
`NonZero<i32>` (equivalently `NonZeroI32`) behind a private field. `NonZero`
rejects only zero, so the checked constructor must reject every `raw <= 0`
before constructing the wrapper. Stable C ABI tables continue to exchange raw
`i32` values; the safe facade validates them before creating a Rust handle.
Never accept a possibly zero FFI value directly as `NonZero<i32>` because an
invalid Rust value exists before validation can run.

The first release does not require entity-destruction hooks merely to keep a
handle wrapper alive. Handles are ephemeral native references, and each use
performs live native validation. Add lifecycle hooks only if a later API makes
a stronger identity or liveness guarantee than the native reference provides.

SA-MP typed ID constructors validate against the largest supported bound for
that ID category, so constructing an ID does not require an active Native
profile. Every backend operation then validates the ID against the active
profile's narrower bound and the current pool state. Out-of-profile IDs fail
before any native pool access.

SA-MP-to-GTA conversions use SF/MoonLoader-compatible Current pool mapping.
Within a valid `GameContext`, forward and reverse conversions inspect the live
SA-MP pool at the instant of the call. They do not use a Persistent cache and
do not promise that the returned ID or handle remains live after return.

The safe Rust facade returns `Result<Option<T>>` for these conversions:

- `Ok(Some(value))`: a current mapping exists;
- `Ok(None)`: the input is valid for the active profile, but the current slot,
  streamed entity, or GTA mapping is absent;
- `Err(OutOfBounds)`: the input exceeds the active profile's bound;
- `Err(NotReady)`: the SA-MP backend is not ready;
- other errors: native/backend failure.

At the C Service ABI, an absent current mapping returns `MOD_NOT_FOUND` and
zeros the output parameter. The safe facade maps this status to `Ok(None)`.
Do not encode absence as `MOD_OK` plus a zero sentinel.

Reverse lookup returns the first active matching slot in ascending SA-MP ID
order. This deliberately preserves SF-like behavior if corrupt or transitional
native state contains duplicate mappings. It does not guarantee uniqueness.

Off-thread conversions use the global Host queue. Their receipt/future reports
the mapping observed when the command executes on the Game thread, not when it
was submitted. A later native operation independently revalidates the handle.

## 5.2 Callback-scoped game-thread access

Snapshot publication is not a synchronization mechanism for native game state.
GTA does not acquire Host locks, so a mutex cannot make arbitrary plugin-thread
access safe. The primary safety boundary is Game-thread confinement.

The safe SDK exposes a callback-scoped `GameContext<'scope>`. The Host issues a
context only after validating that the callback currently executes on the
thread observed running `CGame::Process`. The context lifetime prevents safe
Rust code from retaining access after the callback; an opaque Host token also
validates the thread and active callback scope across the C ABI.

`GameContext` proves Game-thread execution, not one universal engine phase.
Each native operation must satisfy its documented Native execution constraint.
Packet/RPC, render, chat, and other hook callbacks do not receive a context
merely because they are synchronous; the Host must validate their actual
thread at runtime.

Native pointers remain Host-internal. A `GameContext` never exposes a Rust
reference into native memory.

## 5.3 Reads and owned results

Reads through a valid `GameContext` execute synchronously and return copied
scalars or owned compound values. An Owned snapshot is useful when a compound
read must survive the callback or cross a thread boundary; it is not required
for every safe scalar read.

Off-thread callers submit an owned read request to the Game-thread executor and
receive the copied result through a receipt/future or service-specific output
buffer. Reads and mutations that require relative ordering must share the Host
command sequence.

Persistent eager or demand-refreshed caches are optional optimizations. Keep
the current caches during migration where they preserve existing behavior, but
do not make cache generations or delayed refresh semantics the foundation of
new safe APIs. Introduce or retain a cache only for a measured use case with an
explicit invalidation policy.

## 5.4 Mutations

Mutations through a valid `GameContext` may execute synchronously when their
Native execution constraint permits the current phase. Their safe API returns
the immediate result.

Off-thread callers submit owned/copied command payloads and receive a
`CommandReceipt` where completion matters. Keep synchronous context methods and
queued ordinary facade methods explicit; do not silently change execution mode
based on the caller's thread.

---

# 6. Runtime and hook ownership specification — `DECIDED`

## 6.1 Extract `CommandQueue`

Moved `src/command.rs` into `modkit-runtime` with behavior unchanged; the host
re-exports it through `src/command.rs`. The queue was not redesigned during the
extraction.

Required tests preserved (all in `crates/modkit-runtime/src/command.rs`):

- FIFO snapshot;
- new commands deferred to next snapshot;
- capacity 256 for the current default queue;
- timeout retryability;
- detach does not cancel;
- wait rejection does not consume receipt;
- shutdown completes receipts;
- ID wraparound never reuses an active receipt.

After behavior-preserving extraction, use one Host work sequence for all new
off-thread native reads and mutations. One tick snapshot executes that work in
global FIFO order across GTA and SA-MP Services. Separate bounded,
deduplicated domain queues may remain only for optional Persistent cache
refreshes; their order is not an API ordering guarantee.

## 6.2 GTA tick runtime owns `CGame::Process`

The `CGame::Process` hook must move out of SA-MP backend ownership.

Target ownership:

```text
gta-sa-native / host runtime
  -> installs CGame::Process detour
  -> marks game-thread ID
  -> captures command snapshot(s)
  -> calls original exactly once
  -> runs post-process pumps
  -> opens/closes validated Game-context scopes at documented phases
  -> publishes legacy/optimized caches where retained
```

SA-MP backend must register/use the GTA tick runtime instead of owning that hook.

## 6.3 Tick phase ordering

Preserve existing observable ordering during the first extraction:

```text
before original:
  mark game thread
  take accepted command snapshot

call original CGame::Process exactly once

after original:
  execute captured commands
  refresh/publish caches
```

Only change ordering in a dedicated change with tests and a written reason.
This ordering preserves the current backend during extraction; it does not make
cache publication mandatory for new safe reads. Every callback source that can
issue a `GameContext` must document its engine phase, and every context operation
must check the matching Native execution constraint.

## 6.4 Hook broker

Create an internal host hook broker in/around `modkit-win32`.

Initial goals:

- one owner per target;
- named hook registration;
- captured trampoline/original storage;
- deterministic disable/restore;
- diagnostics with target/detour/trampoline addresses;
- no duplicate MinHook creation for the same target by two services.

Do **not** expose arbitrary inline-hook creation to third-party safe plugins in v1.

Current SAMP RakClient hooks can continue using their existing specialized storage initially. Migrate them to the broker after GTA tick extraction proves the abstraction.

Coexistence with third-party hook owners is best-effort in the first release.
Do not claim compatibility with SAMPFUNCS, CLEO, MoonLoader, or arbitrary ASI
detours without a dedicated compatibility fixture.

---

# 7. Windows/native memory layer

Extract generic Windows memory utilities out of `samp-native` concerns.

## 7.1 Preserve guarded access

Retain `VirtualQuery` validation for untrusted/volatile native pointers and ranges.

## 7.2 Add validated region/view

Introduce an internal abstraction similar to:

```rust
pub(crate) struct ReadableRegion {
    start: NonZeroUsize,
    len: usize,
}

impl ReadableRegion {
    pub unsafe fn validate(address: usize, len: usize) -> Option<Self>;
    pub unsafe fn read_unaligned<T: Copy>(&self, offset: usize) -> Option<T>;
    pub unsafe fn subregion(&self, offset: usize, len: usize) -> Option<Self>;
}
```

and an equivalent writable view if needed.

Goal: one range validation for a native object snapshot, followed by multiple checked offset reads, instead of repeated `VirtualQuery` for every scalar field.

Do not weaken pointer/range validation to gain speed.

## 7.3 Separate address kinds

Continue the good direction already present in `NativeRva`, `FieldOffset`, `NativeSize`, `NativeLimit`.

For GTA add similarly distinct types:

```rust
AbsoluteAddress
ImageRva
FieldOffset
VtableSlot
NativeSize
NativeLimit
```

Avoid APIs taking undifferentiated `usize` where mixing concepts is plausible.

---

# 8. GTA SA native backend specification

This is the plugin-sdk replacement layer.

## 8.1 Do not bind plugin-sdk at runtime

Do not link plugin-sdk into production host code and do not reproduce its C++ ownership model.

Use plugin-sdk as:

- a source of class layouts;
- a source of addresses/signatures;
- a source of behavior/reference examples;
- a C++ compile-time oracle for selected `sizeof`/`offsetof`/signature checks.

The production runtime should be direct Rust x86 calls and verified layouts.

## 8.2 Initial GTA profile

Create `GtaProfile` / `GtaProfileSpec` analogous to the existing SA-MP `NativeClientProfile`.

Do not claim support for multiple GTA executables before exact detection is implemented and verified. A production GTA profile must identify one exact executable build by a stable hash; `GTA SA 1.0 US` alone is not a sufficient identity.

The first profile may represent the exact GTA SA 1.0 US executable currently used by the repository/live tests. Record any Compact/Hoodlum distinction as unverified until confirmed.

Suggested shape:

```rust
pub(crate) struct GtaProfile {
    module_base: usize,
    spec: &'static GtaProfileSpec,
}

pub(crate) struct GtaProfileSpec {
    identity: GtaIdentity,
    game: GameSpec,
    pools: PoolSpec,
    world: WorldSpec,
    streaming: StreamingSpec,
    camera: CameraSpec,
    // add only when implemented/verified
}
```

Do not create a single flat hundreds-field spec.

## 8.3 First GTA symbols to migrate

The first migration is not a broad plugin-sdk port. Move the already-used GTA symbols first:

- `CGame::Process` at current verified target `0x53BEE0`;
- `CPools::GetPedRef` at current verified target `0x54FF60`;
- `CPools::GetVehicleRef` at current verified target `0x54FFC0`.

After moving them, SA-MP code must call GTA backend helpers rather than owning these constants.

## 8.4 plugin-sdk port order

Port in dependency order:

### Tier A — foundations

- vector/math primitives;
- basic matrices/transforms;
- raw handle types;
- module/profile detection;
- typed native call helpers;
- basic vtable helper for verified virtual methods.

### Tier B — entity foundations

- `CPlaceable` knowledge needed by public features;
- `CEntity`;
- `CPhysical`;
- entity type/model/position snapshot subsets;
- world add/remove only when signatures are verified.

### Tier C — pools and actors

- GTA pools;
- ped lookup/ref validation;
- vehicle lookup/ref validation;
- `CPed` snapshot subset;
- `CPlayerPed` subset;
- `CVehicle` subset.

### Tier D — common gameplay systems

- `CWorld`;
- `CStreaming`;
- `CTimer`;
- `CCamera`;
- model loading/request/release;
- spawn/create/delete workflows needed by examples.

### Tier E — advanced engine surface

Only after the above is stable:

- tasks/AI;
- RenderWare internals;
- D3D9/render service;
- audio;
- script/CLEO interop;
- advanced vehicle subclasses;
- animation/task internals.

Do not attempt 100% plugin-sdk parity before the common gameplay layer is usable.

## 8.5 Raw layout rules

A raw native struct may be defined only when its size/fields used by the implementation are verified.

Prefer partial opaque structures plus field offsets when full layout is not required.

Examples:

```rust
#[repr(C)]
pub(crate) struct CVectorRaw {
    x: f32,
    y: f32,
    z: f32,
}
```

For a large class where only selected fields are known, do not fabricate padding for every unknown member unless layout oracle tests prove the whole structure.

## 8.6 Virtual methods

Do not model C++ virtual classes as Rust traits for native objects.

For verified virtual calls, use a host-internal typed vtable call helper:

```rust
unsafe fn call_vmethod<F>(object: NonNull<c_void>, slot: VtableSlot) -> Option<F>
where
    F: Copy;
```

Then invoke the typed function pointer with the correct x86 ABI.

Every vtable call requires:

- readable object pointer;
- readable vtable pointer;
- verified slot;
- verified function signature;
- function target validation as appropriate.

## 8.7 Public GTA examples as acceptance targets

The first useful GTA-facing release should support examples equivalent to:

1. read local player state synchronously through a callback-scoped `GameContext`;
2. retain an owned ped snapshot outside the callback;
3. teleport locally through `GameContext` and off-thread through a queued mutation;
4. subscribe to a game-process/tick event;
5. create/spawn a vehicle only after model streaming and creation signatures are verified;
6. inspect basic vehicle state.

These examples are more important than broad but untested API surface.

---

# 9. SA-MP backend/facade migration specification

The current SA-MP implementation is not to be rewritten from scratch.

## 9.1 Move existing native profile code

Eventually move:

```text
src/platform/win32/native_client/**
```

into:

```text
crates/samp-native/src/**
```

while preserving:

- `NativeClientProfile` data model;
- R1/R3/R5/DL specs;
- strategy enums;
- current tests/fixtures;
- guarded memory access behavior;
- exact calling conventions;
- current profile selection behavior.

Do not inherit a native fact from another profile through a broad struct update
unless the Evidence register records Grade A or B support for that fact on both
builds. Extract an explicitly named shared spec only for proven parity.

Do the move subsystem by subsystem, not as one unreviewable file relocation plus redesign.

## 9.2 Remove GTA ownership from SA-MP

SA-MP code must stop defining/owning:

- `CGame::Process` hook;
- GTA CPools functions;
- GTA handle type definitions;
- future generic GTA entity operations.

SA-MP may request GTA conversions through a host-internal GTA backend interface.

Example desired flow:

```text
SA-MP PlayerId
   -> native SA-MP player/ped pointer
   -> GTA backend validates/converts
   -> gta_sa::PedHandle
```

This flow uses Current pool mapping. Player, vehicle, and object mappings return
their corresponding GTA pool handle types. Pickup mapping returns the distinct
`PickupHandle`; it must not reinterpret a pickup reference as `ObjectHandle`.
Forward and reverse safe facade operations return `Result<Option<T>>`, and a
queued off-thread conversion observes state at execution time.

## 9.3 Preserve safe SA-MP facade semantics

The current `Samp` facade is a good direction. Preserve:

- bounded SA-MP ID newtypes;
- SF/MoonLoader-compatible live forward/reverse pool mappings;
- owned strings/byte buffers;
- owned player/dialog/textdraw/etc snapshots;
- command receipts;
- no safe pointer exposure;
- explicit `raw` escape hatch only.

As the new service ABI comes online, rewrite the facade backend to use `SampServiceV1` / `SampNetServiceV1`, not to expose service tables directly to users.

## 9.4 Split networking from general SA-MP service

`SampNetServiceV1` should own:

- incoming/outgoing packet subscription;
- incoming/outgoing RPC subscription;
- exact-bit event payload operations;
- send packet/RPC;
- incoming emulation;
- native encoded string helper where direct RakNet compressor compatibility is required.

`SampServiceV1` should own:

- version/server state;
- player/pool/UI/chat/dialog/textdraw/text-label APIs;
- connection actions;
- local-player SA-MP operations.

This avoids recreating one 145+ field service.

---

# 10. `samp-protocol` extraction specification

This should be one of the earliest implementation phases because it is low risk and immediately improves architecture/testability.

## 10.1 Move pure `BitStream`

Move the owned bitstream implementation from `sdk/src/raknet.rs` into `crates/samp-protocol`.

The new crate must not have:

```rust
compile_error!("... only Windows x86 ...")
```

and must not depend on host discovery or native pointers.

## 10.2 Split codec from callback transport

Current `sdk/src/events/` mixes protocol knowledge with callback/host ABI behavior.

Separate:

```text
samp-protocol
  -> IDs/descriptors
  -> encode/decode
  -> packet/RPC owned value types
  -> protocol constants

samp plugin facade
  -> subscription registration
  -> callback-local Event wrapper
  -> converts event BitStream to typed protocol values
```

A codec must be testable by giving it a `BitStream` and receiving a Rust value, with no `HostApi` available.

## 10.3 Preserve exact protocol semantics

Keep tests for:

- exact bit lengths;
- partial final bytes;
- exact replacement vectors;
- bounded allocations;
- encoded strings;
- all existing common RPC vectors;
- malformed input rejection;
- read cursor behavior.

Text with uncertain encoding remains bytes unless a protocol field is definitively encoded/decoded as text.

---

# 11. SAMPFUNCS migration strategy — `DECIDED`

Do not port the SAMPFUNCS C++ class hierarchy as the final public API.

Map its responsibilities into the new architecture.

## 11.1 Responsibility mapping

```text
SAMPFUNCS plugin bootstrap/lifecycle
  -> gta_mod_host + modkit-sdk plugin lifecycle

SAMPFUNCS logging/console
  -> Core service logging

SFSAMP
  -> SampService facade/backend

SFRakNet
  -> SampNetService + samp-protocol

SFGame / game_api
  -> gta-sa + gta-sa-native

SFRender
  -> future Render service

SFCLEO
  -> optional future CLEO compatibility service

raw SA-MP structures/version layouts
  -> samp-native profiles/layout fixtures
```

## 11.2 SAMPFUNCS source policy

The previously inspected SAMPFUNCS SDK includes C++ ABI types and versioned SA-MP layouts. Use it as reference material, but do not make production Rust plugins depend on `SAMPFUNCS.lib` or `SAMPFUNCS.asi`.

Because the supplied archive did not present a clearly established redistribution license in the previous audit, do not vendor SAMPFUNCS headers/source into the published repository until licensing is explicitly resolved.

Store derived verified numeric metadata and independently written Rust code only where legally appropriate.

## 11.3 `cxx` policy

`cxx` is allowed only for:

- temporary compatibility adapters;
- a development-only oracle for C++ calls/layout behavior;
- integrations with third-party C++ libraries where a direct C/Rust ABI is unreasonable.

Core GTA/SA-MP native operation code should remain direct Rust and typed x86 function pointers.

---

# 12. Compatibility strategy

## 12.1 Legacy plugin SDK compatibility

There are no known external consumers of `samp-client-sdk 0.1.0-alpha.4`, and
pre-1.0 source or binary compatibility is not a release requirement. The
legacy export and facade may remain temporarily when they provide useful
regression coverage or reduce migration risk, but they must not constrain the
new architecture.

When legacy behavior is retained during migration, do not change its meaning
merely to match new API naming. Remove or break it explicitly when doing so
simplifies the target architecture, and record the break.

## 12.2 New safe API

New plugins should progressively migrate to:

```text
modkit-sdk
+ gta-sa
+ samp
+ samp-protocol
```

Do not force new plugins to depend on the legacy `samp-client-sdk` monolithic ABI after the new services reach parity for their used functionality.

## 12.3 Raw API

The existing `sdk/src/raw.rs` contains R1-specific offsets and helpers. Do not copy this wholesale into the new generic API.

Split raw escape hatches by ownership:

```text
gta_sa::raw      # GTA-specific unsafe addresses/operations
samp::raw        # SA-MP-specific unsafe addresses/operations
```

Every raw function must state:

- exact supported profile(s);
- pointer/address lifetime;
- whether the value is cached or live;
- what validation remains the caller's responsibility.

Raw APIs are explicitly not stable-safe contracts.

---

# 13. Plugin lifecycle roadmap — mixed `DECIDED` / `PROPOSED`

Do not block the native/backend split on a new plugin loader.

## 13.1 Stage A — preserve independent ASI plugins

Keep current model:

```text
plugin.asi -> GetModuleHandle/GetProcAddress -> host service API
```

Add `modkit-sdk::Host::connect` that resolves `GtaModHost_GetApiV1` similarly to
current `Samp::connect`, but never falls back to `SampClientSdk_GetApiV1`.

## 13.2 Stage B — add plugin macro/runtime helper

After service ABI is stable, provide a helper/proc macro that removes boilerplate around:

- `DllMain`;
- worker-thread initialization;
- panic boundary;
- host connection;
- owned `SubscriptionSet`;
- orderly shutdown/drain.

Conceptual API:

```rust
#[mod_plugin]
fn init(ctx: PluginContext) -> Result<()> {
    // ...
    Ok(())
}
```

Do not introduce this macro before the underlying lifecycle API is explicit and testable without the macro.

## 13.3 Stage C — optional host-managed mods directory

Later, the host may load:

```text
mods/foo.dll
mods/bar.dll
```

through a dedicated plugin entrypoint.

Required before enabling unload/hot reload:

- host tracks plugin identity;
- subscriptions are attributable to a plugin;
- host can prevent new callbacks during teardown;
- all in-flight callbacks drain;
- pending command waiters are detached/completed;
- plugin shutdown returns;
- only then `FreeLibrary` occurs.

Do not implement hot reload by simply calling `FreeLibrary` on arbitrary independently loaded ASIs.

---

# 14. Testing and verification architecture

Testing is part of the reverse-engineering contract, not optional cleanup.

## 14.1 Preserve current C++ layout fixture

Keep `tests/fixtures/raknet_layout.cpp` and existing Rust layout tests.

## 14.2 Add GTA/plugin-sdk layout oracle

Create a development/test fixture such as:

```text
tests/fixtures/gta_sa_layout.cpp
```

It should compile against an explicitly chosen plugin-sdk reference/version and print or static-assert selected values:

- `sizeof`;
- `alignof`;
- `offsetof`;
- enum values where relevant.

Do not require plugin-sdk as a production runtime dependency.

If repository policy avoids vendoring plugin-sdk, CI/test setup may use a separately provided checkout/path. Document this clearly.

## 14.3 Profile data tests

Every address/offset/spec migration should have a test that proves the new data equals the old verified source before the old source is removed.

For new GTA profile values, add explicit tests against oracle metadata or known verified values.

## 14.4 ABI tests

Add tests for:

- exact numeric `ModResult` constants and `0 == success`;
- sizes and alignment of every public `repr(C)` / `repr(transparent)` ABI type;
- `ServiceHeader` prefix, reserved-zero behavior, exact service ID/version/size validation;
- `GtaModHost_GetApiV1(NULL)` and every documented null-pointer error path;
- exact service version lookup and distinction between unknown service vs unsupported version;
- failure paths zero/null the documented out-pointer before returning;
- service pointers remain stable/immutable for host lifetime;
- `GameContext` tokens are rejected on the wrong thread, outside their callback scope, and under an incompatible Native execution constraint;
- input pointer+length and caller-buffer `MOD_BUFFER_TOO_SMALL` behavior;
- callback context is returned untouched and no callback runs after successful drain;
- each retained callback context is released exactly once through its plugin-provided release callback after drain;
- `unregister_and_wait` does not complete before the release callback returns;
- receipt/subscription ID `0` is rejected and live IDs are not accidentally reused;
- blocking waits are rejected from game-thread/callback test contexts;
- old legacy export remains valid during compatibility period;
- no Rust/C++ allocator-owned values or Rust references exist in public service structs.

## 14.5 Runtime tests

Preserve/add:

- command FIFO/frame boundary;
- original `CGame::Process` called exactly once;
- game-thread identification before callback re-entry;
- wait rejection from game thread;
- callback registration order;
- unregister-and-wait behavior;
- shutdown with in-flight callback;
- hook disable/restore ordering;
- retained cache generation consistency while a Persistent cache exists;
- connection invalidation behavior.

## 14.6 Protocol tests

Run `samp-protocol` tests without a GTA process.

Add fuzz/property tests later for:

- arbitrary bit lengths;
- cursor operations;
- malformed length-prefixed RPCs;
- roundtrip of supported encode/decode pairs.

Fuzzing is a later enhancement, not a blocker for the initial extraction.

## 14.7 Live smoke matrix

Maintain a manual/live smoke checklist for:

```text
SA-MP 0.3.7 R1
SA-MP 0.3.7 R3-1
SA-MP 0.3.7 R5-1
SA-MP 0.3.DL-R1
```

For each recognized build verify at minimum:

- host attaches;
- correct profile is reported;
- RakClient hooks install;
- one outgoing network event is observed;
- one queued command crosses the game-thread boundary;
- retained cache generation publishes where that optimization remains enabled;
- shutdown disables hooks cleanly.

For GTA-only functionality add a smoke that runs without requiring SAMP service readiness where technically possible.

---

# 15. Migration map from current files

Use this map when moving code.

| Current path | Target ownership | Notes |
| --- | --- | --- |
| `src/command.rs` | `modkit-runtime` | Moved to `crates/modkit-runtime/src/command.rs`; host re-exports via `src/command.rs`. |
| `sdk/src/subscriptions.rs` | `modkit-sdk` + generic runtime support | Keep safe RAII wrapper; generalize host-side lifecycle separately. |
| `sdk/src/raknet.rs` | `samp-protocol` | Pure owned bitstream. |
| `sdk/src/events/**` pure codecs | `samp-protocol` | Separate from callback transport. |
| `sdk/src/events/**` callback wrappers | `samp` | Use new `SampNetServiceV1`. |
| `src/platform/win32/native_client/profile.rs` | `samp-native` | Preserve nested spec design. |
| `src/platform/win32/native_client/profiles/**` | `samp-native` | Preserve exact profile data. |
| `src/platform/win32/native_client/memory.rs` | generic parts -> `modkit-win32`; SA-MP-specific use stays in `samp-native` | Add validated regions. |
| `src/platform/win32/hooks.rs` | hook primitives -> `modkit-win32`; SA-MP detours -> `samp-native`; game tick -> `gta-sa-native` | Do not move all at once. |
| `src/platform/win32/mod.rs` `CGame::Process` ownership | `gta-sa-native`/host runtime | First GTA extraction target. |
| `GTA_CPOOLS_GET_PED_REF` | `gta-sa-native` | Remove from SA-MP player code. |
| `GTA_CPOOLS_GET_VEHICLE_REF` | `gta-sa-native` | Remove from SA-MP pool code. |
| `sdk/src/facade/*` GTA handles | `gta-sa` | SA-MP facades return these types. |
| `sdk/src/facade/*` SA-MP API | `samp` | Keep owned/safe design. |
| `sdk/src/raw.rs` | split into legacy/raw GTA/raw SA-MP | Do not make R1 constants generic. |
| `src/host_api/**` | service adapters in host | Legacy adapter remains until parity. |
| `tests/fixtures/raknet_layout.cpp` | keep/shared fixture area | Do not delete. |
| `examples/**` | retain, then add new service-based examples | Existing examples become regression tests. |

---

# 16. Detailed implementation phases / TODO

The following order is normative unless a concrete dependency forces a small change. If order changes, update this document with the reason.

---

## Phase 0 — Freeze and document the baseline

**Goal:** make the current working system a measurable compatibility target.

### Tasks

- [x] Create the immutable baseline record at
  `docs/baselines/phase-0-2026-08-26.md`.
- [x] Record the current branch, full HEAD, and complete
  `git status --porcelain=v1 --untracked-files=all` output before Phase 0 edits.
- [x] Treat that working tree, including every pre-existing tracked and
  untracked change, as the implementation baseline; do not reconstruct a
  separate archive snapshot.
- [x] Record the declared `rust-version`, the Rust/Cargo versions actually
  used, target triple, relevant Cargo configuration, and any required local
  build-directory override.
- [x] Run and record the formatting/test/clippy/release-build baseline,
  including exit status and current test counts. Do not copy stale counts from
  older planning documents.
- [x] Record the release DLL path, build profile, target triple, SHA-256, and
  complete PE export list; expected exports include `DllMain` and
  `SampClientSdk_GetApiV1`.
- [x] Add a small test/helper that asserts the current `SampClientSdkApiV1` field/layout baseline while it remains supported.
- [x] Assert and record that the table currently has exactly 145 fields and
  must not grow except for an explicitly approved critical pre-1.0 fix.
- [x] Ensure current R1/R3/R5/DL profile tests are passing.
- [x] Create `docs/native-capability-matrix.md` with separate profile,
  recognition, readiness, Capability, evidence source/grade, and limitation
  data. Do not infer whole-profile support from aggregate smoke results.
- [x] Ensure current `CGame::Process` lifecycle tests are passing.
- [x] Ensure current subscription/unregister-and-wait tests are passing.
- [x] Ensure exact-bit event replacement tests are passing.
- [x] Move and rename this canonical handoff as
  `docs/rust-modding-infrastructure-handoff.md`.

Completed: 2026-08-26. The immutable
[Phase 0 baseline](baselines/phase-0-2026-08-26.md) contains the accepted
validation, artifact, export, ABI, profile, lifecycle, callback, exact-bit,
and final diff-audit evidence. The [Native Capability Matrix](native-capability-matrix.md)
remains the separate living evidence record.

### Acceptance criteria

- No runtime behavior changed.
- Full current workspace checks pass.
- Existing host exports and ABI layout are documented with the release
  artifact identity needed to reproduce the observation.
- Current profile-specific tests are green on the recorded Rust 1.98
  toolchain.
- The complete working tree is the source of truth; the baseline does not
  depend on an external archive.
- Only documentation and test-only guards change during this phase; no
  production refactor begins.
- Baseline is recorded so later parity regressions are obvious.

### Suggested commit

```text
docs(architecture): freeze baseline for gta/samp modkit migration
```

---

## Phase 1 — Extract `samp-protocol`

**Goal:** make protocol code usable/testable independently of Windows and the host.

**Status:** complete (2026-08-29).

`crates/samp-protocol` now owns the platform-independent bitstream, bounded
wire primitives, structured error domains, cursor-free `EncodedBits`, explicit
descriptor framing policies, Packet/RPC catalogs, and pure common/R1 codecs.
The SDK retains callback lifetime, listener actions, Host error mapping, and
the injected Host-backed encoded-string adapter. Protocol tests and the
dedicated Linux CI job run without Host or Windows dependencies.

Completion evidence:

- [Protocol/SDK boundary completion](evidence/protocol-sdk-boundary-completion.md)
  closes issues #6, #21, #22, and #39.
- [P0 architecture gate](evidence/p0-architecture-gate.md) records the public
  surface, error-boundary, package, test, and release checks.
- [Terminal-alignment padding evidence](evidence/terminal-alignment-padding.md)
  records the exact-bit framing decision retained by the extracted codecs.

Native encoded strings intentionally remain Host-backed until a separate
platform-independent extension is approved.

---

## Phase 2 — Extract generic runtime primitives

**Goal:** make command/subscription behavior reusable by GTA and SA-MP services.

### Tasks

- [x] Create `crates/modkit-runtime`.
- [x] Move `CommandQueue<C, R>` with semantics unchanged.
- [x] Move/shared-define command ID and generic queue errors.
- [x] Add/retain all queue behavior tests.
- [x] Identify host-side subscription lifecycle state that is generic across event types.
- [x] Extract generic in-flight callback counting/waiting primitives without changing current public SDK yet.
- [x] Add non-blocking Deferred reclamation for dropped subscription callback state; invoke its plugin-provided release callback exactly once after drain.
- [x] Keep plugin-side existing `Subscription` facade working through compatibility glue.
- [x] Define an internal callback-context guard that marks “inside host callback” for wait rejection.
- [x] Define generic active-scope state for runtime-validated `GameContext` tokens without depending on GTA or SA-MP addresses.

### Acceptance criteria

- Existing game command behavior is byte/semantics equivalent.
- Existing subscription tests pass.
- No GTA/SA-MP address appears in `modkit-runtime`.

### Suggested commit

```text
refactor(runtime): extract reusable command and subscription primitives
```

---

## Phase 3 — Introduce `modkit-abi` and service discovery beside legacy ABI

**Goal:** stop architectural growth of the monolithic ABI without breaking existing plugins.

### Tasks

- [x] Create `crates/modkit-abi`.
- [x] Implement the exact `ModResult` newtype and numeric constants from Section 4.1.
- [x] Define `ServiceId`, `SubscriptionId`, `CommandReceiptId`, `ServiceHeader`, and bootstrap ABI exactly as specified in Section 4.
- [x] Define the opaque ABI token carried by plugin-side `GameContext<'scope>` and test wrong-thread, stale-scope, and wrong-phase rejection.
- [x] Add `GtaModHost_GetApiV1` export in the current host crate with documented out-pointer clearing/error behavior.
- [x] Implement exact-version `query_service` with distinct `NotFound`, `UnsupportedVersion`, and `ShuttingDown` results; model registry-level `NotReady` for compatible hosts. This host publishes its static registry before the bootstrap table is callable, so its discovery path has no observable unpublished-registry state. Discovered tables do not depend on native backend readiness.
- [x] Implement Core service v1 with host status plus shared subscription/receipt/log primitives; freeze its layout only after its null/thread/lifetime tests exist.
- [x] Implement callback-context tracking needed for `CALLBACK_SAFE` and wait-rejection rules.
- [x] Register the migration-only header-prefixed `LegacySampServiceV1` wrapper containing a pointer to the existing API table.
- [x] Keep `SampClientSdk_GetApiV1` unchanged.
- [x] Add ABI size/alignment, numeric-code, nullability, pointer-lifetime, service-lookup, ID-lifecycle, callback-drain, and wait-rejection tests from Section 14.4.
- [x] Create `crates/modkit-sdk` with `Host::connect` / `Host::connect_to` resolution logic based only on the new export; do not fall back to `SampClientSdk_GetApiV1`.
- [x] The safe SDK must preserve unknown raw result codes for diagnostics instead of constructing invalid enums.
- [x] Formalize host lifetime: host API/service pointers remain immutable and valid for process lifetime; host hot-unload is not supported.

Completed: 2026-08-30. Phase 3 now has exact-size service validation, bounded
Core operations, retryable timed drains, monotonic IDs, process-lifetime host
resolution, and the generic `GameContext` token foundation. Actual native
callback delivery remains Phase 9. See
[Phase 3 completion evidence](evidence/phase-3-modkit-service-discovery.md).

### Acceptance criteria

- The old export may remain as a temporary migration path, but the new SDK never resolves it.
- A new test/example connects via `GtaModHost_GetApiV1` and queries Core + Legacy SA-MP service.
- No new functionality was appended to `SampClientSdkApiV1` for this migration.

### Suggested commits

```text
feat(abi): add versioned modkit service discovery
feat(sdk): add plugin-side host and service resolver
```

---

## Phase 4 — Extract `modkit-win32` memory/hook primitives

**Goal:** remove generic Windows implementation details from the SA-MP backend.

### Tasks

- [x] Create `crates/modkit-win32` as Windows x86 host-internal crate.
- [x] Move generic PE/module helpers.
- [x] Move generic `VirtualQuery` range validation.
- [x] Introduce `ReadableRegion`/`WritableRegion` abstraction.
- [x] Move generic inline hook wrapper around MinHook.
- [x] Keep SA-MP-specific detour functions in their current location initially.
- [x] Add tests for null/overflow/page protection and owned-memory reads/writes.
- [x] Ensure public plugin crates do not depend on `modkit-win32`.

### Acceptance criteria

- [x] Current SA-MP host still installs all hooks and reads profiles successfully.
- [x] Memory tests retain existing safety behavior.
- [x] No profile-specific constant moves accidentally into generic Win32 code.

Implementation, automated validation, and R3-1 live attach evidence is recorded
in [Phase 4 evidence](evidence/phase-4-modkit-win32.md).

### Suggested commit

```text
refactor(win32): extract guarded memory and hook primitives
```

---

## Phase 5 — Create GTA native runtime and move `CGame::Process`

**Goal:** establish GTA as the base service beneath SA-MP.

### Tasks

- [x] Create `crates/gta-sa-native`.
- [x] Define initial `GtaProfile`/`GtaProfileSpec` for the currently verified GTA SA 1.0 US target.
- [x] Move ownership of `CGame::Process` target `0x53BEE0` into the GTA profile/runtime.
- [x] Move the game-process detour function and trampoline ownership into GTA runtime or host composition.
- [x] Preserve exact ordering: mark thread -> snapshot -> original once -> post-pump.
- [x] Provide an internal mechanism for SA-MP backend to register/run its post-game-process pump.
- [x] Do not expose a public GTA API yet beyond profile/status if not needed.
- [x] Move game-thread ID ownership into GTA runtime.
- [x] Move generic wait-rejection query (`is_game_thread`) behind runtime/Core service.
- [x] Update tests proving original call count and command frame boundaries.

Complete. `crates/gta-sa-native` owns the SHA-256-gated GTA SA 1.0 US profile,
`CGame::Process` hook/trampoline, detour diagnostics, and game-thread identity.
The existing SA-MP backend registers as a `GameTickParticipant`; the GTA
runtime preserves mark -> snapshot -> original once -> post-pump ordering. The
Core wait path reaches the GTA-owned thread query through the existing host
runtime, without changing the frozen `CoreServiceV1` layout. See
[Phase 5 evidence](evidence/phase-5-gta-native-runtime.md).

### Implementation note

Prefer a small explicit post-tick participant mechanism over a generalized plugin hook system. This is host-internal infrastructure.

A simple first design may be:

```rust
trait GameTickParticipant: Send + Sync {
    fn before_game_process(&self) {}
    fn after_game_process(&self) {}
}
```

or explicit host composition calls. Avoid dynamic complexity if direct composition is clearer.

### Acceptance criteria

- [x] SA-MP behavior remains unchanged in a fresh live attach and clean exit.
- [x] `samp-native` no longer owns the GTA `CGame::Process` constant/hook.
- [x] Existing game-tick and frame-boundary tests pass under new ownership.

### Suggested commit

```text
refactor(gta): move game-process runtime out of samp backend
```

---

## Phase 6 — Move GTA handle conversions and public handle types

**Goal:** establish the first real GTA API boundary.

### Tasks

- [x] Move `CPools::GetPedRef` (`0x54FF60`) to `gta-sa-native` profile/spec.
- [x] Move `CPools::GetVehicleRef` (`0x54FFC0`) to `gta-sa-native` profile/spec.
- [x] Add validated typed wrappers around those calls.
- [x] Create `crates/gta-sa`.
- [x] Move `PedHandle`, `VehicleHandle`, `ObjectHandle`, `PickupHandle` ownership to `gta-sa` or an appropriate GTA value module.
- [x] Store valid positive GTA handle tokens as private `NonZero<i32>` values in the safe Rust types; keep raw `i32` at the C ABI boundary and reject `raw <= 0` before wrapping.
- [x] Keep `PickupHandle` distinct from `ObjectHandle` and use pickup-specific validation.
- [x] Change SA-MP player/vehicle conversions to call the GTA backend internally.
- [x] Ensure SA-MP IDs remain in `samp`, not `gta-sa`.
- [x] Validate typed ID construction against the cross-profile maximum, then validate each operation against the active profile's bound before pool access.
- [x] Add roundtrip/mapping tests using existing mock ABI behavior.

Completed on 2026-08-30. Automated and live evidence is recorded in
[`docs/evidence/phase-6-gta-handles.md`](evidence/phase-6-gta-handles.md).

### Acceptance criteria

- No GTA CPools absolute address remains in SA-MP operation modules.
- New GTA handle types are usable by new service/facade code.
- Invalid zero/negative raw handles, out-of-profile IDs, stale handles, and pickup-specific references have regression tests.
- Existing proven SA-MP-to-GTA mapping behavior remains covered while public pre-1.0 source compatibility may break.

### Suggested commit

```text
refactor(gta): centralize gta pool handles and samp mappings
```

---

## Phase 7 — Create `SampServiceV1` and `SampNetServiceV1`

**Goal:** move new plugin code off the 145-field legacy table.

### Tasks

- [x] Define small initial `SampServiceV1` containing only already-stable operations needed by examples.
- [x] Define `SampNetServiceV1` for packet/RPC subscription/send/emulation.
- [x] Implement both tables as adapters over the existing host/backend first.
- [x] Add `samp` crate safe facade using `modkit-sdk` service discovery.
- [x] Route `samp-protocol` typed event helpers through `SampNetServiceV1`.
- [x] Preserve registration order, block/continue/replace semantics.
- [x] Use Core service for common unregister-and-wait and command receipt operations where possible.
- [x] Port one simple example (`chat command`) to the new service path.
- [x] Port one network example to the new service path.
- [x] Keep legacy examples unchanged as regression coverage.

Completion evidence: [Phase 7 SA-MP services](evidence/phase-7-samp-services.md).

### Suggested minimal initial `SampServiceV1` scope

- version/probe;
- game state;
- server info;
- local player snapshot;
- chat add;
- chat input command register/unregister;
- basic player lookup.

### Suggested minimal initial `SampNetServiceV1` scope

- register packet listener;
- register RPC listener;
- event ID/bit payload access or copied callback payload;
- send packet;
- send RPC;
- emulate incoming packet/RPC.

Do not immediately migrate every current UI/pool API. Prove the service model first.

### Acceptance criteria

- New `samp` examples no longer touch `SampClientSdkApiV1`.
- Legacy examples still work.
- Network vector tests are identical.

### Suggested commits

```text
feat(samp): add versioned samp service facade
feat(samp-net): route typed protocol events through network service
```

---

## Phase 8 — Move current SA-MP native implementation into `samp-native`

**Goal:** finish structural ownership after new services prove the split.

### Tasks

- [ ] Create `crates/samp-native` if not already created during earlier extraction.
- [ ] Move `native_client/profile.rs` and four profile files first.
- [ ] Move shared singleton/layout/player/pool/UI modules subsystem by subsystem.
- [ ] Move SA-MP RakClient hooks/detours.
- [ ] Keep GTA runtime dependencies pointing into `gta-sa-native`.
- [ ] Keep host-specific API table adapters in host composition, not in `samp-native`.
- [ ] Move SA-MP cache state in coherent groups only when tests make ownership clear.
- [ ] Avoid splitting large shared state just to satisfy file size; behavior/invariants take priority.
- [ ] Re-run C++ RakNet layout fixture after native moves.

### Acceptance criteria

- Root host composition no longer contains direct profile implementation logic.
- SA-MP native crate has no plugin-side host discovery code.
- New and legacy API adapters both call the same native backend.

### Suggested series

```text
refactor(samp-native): move profile data
refactor(samp-native): move player and pool operations
refactor(samp-native): move ui and textdraw operations
refactor(samp-native): move rakclient hooks
```

Do not combine these into one commit.

---

## Phase 9 — Begin plugin-sdk knowledge port

**Goal:** make GTA functionality independently useful instead of only supporting SA-MP mappings.

### Tasks — foundations

- [ ] Add `gta_sa_layout.cpp` oracle strategy/documentation.
- [ ] Add GTA profile unverified-values document.
- [ ] Add `Vector2`/`Vector3` and verified matrix representation.
- [ ] Add symbol/address spec infrastructure.
- [ ] Add verified typed call helpers for cdecl/stdcall/thiscall as needed on x86.
- [ ] Add verified vtable-call helper.

### Tasks — first entity slice

- [ ] Implement a minimal `EntitySnapshot` with only verified fields.
- [ ] Implement local player/ped handle resolution.
- [ ] Implement `PedSnapshot` subset: position, health/armour and other fields only when verified.
- [ ] Add callback-scoped `GameContext` access with runtime Game-thread validation and per-operation Native execution constraints.
- [ ] Add a queued off-thread compound read that returns an owned result without requiring a Persistent cache.
- [ ] Implement queued teleport/set-position using a verified native method or safe write path.
- [ ] Implement synchronous teleport/set-position through `GameContext` when its execution constraint permits the callback phase.
- [ ] Add GTA service functions and safe `gta-sa` facade for those operations.
- [ ] Add `gta-basic-plugin` example.

### Acceptance criteria

Example should conceptually be able to:

```rust
let gta = host.gta_sa()?;
let (snapshot_tx, snapshot_rx) = std::sync::mpsc::sync_channel(1);
let _tick = gta.on_tick(move |ctx| {
    let player = ctx.player()?;
    let _ = snapshot_tx.try_send(player.snapshot()?);
    player.teleport(Vector3::new(...))?;
    Ok(())
})?;

let owned_snapshot = snapshot_rx.recv()?;
let receipt = gta.player().teleport(Vector3::new(...))?;
receipt.wait(...)?;
```

without any plugin-side native pointer access.

### Suggested commits

```text
feat(gta): add verified core math and entity snapshot primitives
feat(gta): expose local ped snapshot and queued teleport
```

---

## Phase 10 — Expand GTA common gameplay surface

**Goal:** cover the plugin-sdk subset needed by most basic mods.

### Tasks

- [ ] GTA pools: peds/vehicles/objects as verified.
- [ ] Vehicle snapshots and existence validation.
- [ ] `CWorld` subset.
- [ ] `CStreaming` model request/load/release subset.
- [ ] `CTimer`/frame time subset.
- [ ] Camera snapshot/control subset.
- [ ] Spawn/create vehicle workflow after all required symbols are verified.
- [ ] Deletion/destruction workflow with strict lifetime validation.
- [ ] Add examples equivalent to common plugin-sdk samples.
- [ ] Add live smoke checklist for each feature.

### Non-goal

Do not port every plugin-sdk class before publishing a usable common gameplay layer.

---

## Phase 11 — Finish migration from legacy SA-MP ABI

**Goal:** make new services feature-complete enough that the monolithic alpha API can be deprecated.

### Tasks

- [ ] Build a compatibility matrix mapping every public legacy safe facade method to new service/facade equivalent.
- [ ] Port remaining chat/dialog/UI/player/pool/textdraw/text-label/connection actions.
- [ ] Port command receipts including text-label-create typed result.
- [ ] Port raw unsafe accessors into ownership-correct `gta_sa::raw` / `samp::raw` modules where still desired.
- [ ] Update all examples to new APIs while retaining at least one legacy compatibility smoke until removal.
- [ ] Mark `SampClientSdk_GetApiV1` deprecated in docs.
- [ ] Freeze legacy behavior; no new features.

### Removal gate

Do **not** remove `SampClientSdk_GetApiV1` until:

- all intended safe legacy functionality has a new equivalent or an explicit “will not migrate” decision;
- parity tests pass;
- at least one release cycle has documented deprecation if external users exist.

---

## Phase 12 — SAMPFUNCS compatibility and advanced services

**Goal:** add optional ecosystem compatibility only after native Rust architecture is independent.

### Candidate tasks

- [ ] SAMPFUNCS console/log compatibility adapter.
- [ ] Compatibility mapping document for SFSAMP/SFRakNet/SFGame/SFRender/SFCLEO.
- [ ] Optional `sampfuncs-compat` crate using C++/`cxx` only where justified.
- [ ] Render/D3D9 service.
- [ ] ImGui integration layer.
- [ ] CLEO/script interop service.
- [ ] Host-managed plugin DLL loading.
- [ ] Plugin macro/runtime.
- [ ] Hot reload only after ownership tracking is proven.

These are not blockers for GTA/SA-MP core infrastructure.

---

# 17. First Codex implementation sequence

This is a **dispatch index**, not a second copy of the phase TODOs. Execute the detailed checklists and acceptance criteria in Section 16. Do not implement multiple phases in one broad rewrite unless a small mechanical dependency makes the split impossible.

| Codex task | Execute | Primary deliverable | Hard stop |
|---|---|---|---|
| 1 | Phase 0 | recorded baseline + regression coverage | no production refactor |
| 2 | Phase 1 | `crates/samp-protocol` | no native/hook behavior changes |
| 3 | Phase 2 | `crates/modkit-runtime` | no native address changes |
| 4 | Phase 3 | `modkit-abi`, `modkit-sdk`, `GtaModHost_GetApiV1`, Core + Legacy services | old export remains; Section 4 ABI tests pass |
| 5 | Phase 4 then Phase 5 | generic Win32 layer + GTA-owned tick runtime | `CGame::Process` called exactly once; SA-MP remains post-tick participant |
| 6 | Phase 6 | GTA-owned pool refs/handles + SA-MP mappings | proven mapping behavior retained; no borrowed native pointers escape |

Before Task 1, Codex must read `AGENTS.md`, `ARCHITECTURE.md`, `CORE.md`, the focused `docs/agent-guides/*` files referenced by those documents, and this handoff.

Only after Tasks 1-6 satisfy their acceptance criteria may Codex begin broad service expansion or the plugin-sdk knowledge port. The next default task is Phase 7, not an opportunistic render/UI/hot-reload subsystem.

---

# 18. Definition of done for the architectural migration

The foundational migration is complete when all of the following are true:

- [ ] One host owns native hooks.
- [ ] GTA game-process hook is owned by GTA runtime, not SA-MP.
- [ ] GTA native symbols/layouts live in `gta-sa-native`.
- [ ] SA-MP profiles/layouts live in `samp-native`.
- [ ] SA-MP uses GTA backend for GTA handle/native engine operations.
- [ ] Plugin-side safe GTA API contains no direct native addresses.
- [ ] Plugin-side safe SA-MP API contains no direct native addresses.
- [ ] `samp-protocol` is platform-independent.
- [ ] Command queue/runtime primitives are not SA-MP-specific.
- [ ] Subscription/unload semantics are shared across services.
- [ ] New plugins discover exact-version service tables through `GtaModHost_GetApiV1`.
- [ ] No new features are added to the 145-field monolithic legacy table.
- [ ] `SampServiceV1` and `SampNetServiceV1` cover the actively used safe SA-MP facade.
- [ ] GTA has at least a useful local-player/entity/vehicle/world subset.
- [ ] Layout/address verification exists for every native feature that claims support.
- [ ] R1/R3/R5/DL smoke behavior remains intact.
- [ ] Existing protocol vectors remain exact.
- [ ] Full workspace format/test/clippy/release build passes on the target toolchain.

The project is then ready to grow as a Rust modding SDK rather than as a compatibility wrapper.

---

# 19. Explicit non-goals for the first architecture release

Do not let these expand scope prematurely:

- 100% plugin-sdk class coverage;
- binary compatibility with existing C++ plugin-sdk plugins;
- binary compatibility with arbitrary `.sf` SAMPFUNCS plugins;
- x64 support;
- GTA Definitive Edition;
- SA-MP versions without verified profile data;
- arbitrary third-party inline-hook API;
- safe references into game memory;
- automatic hot reload before callback/command ownership is tracked;
- RenderWare/D3D/ImGui before core GTA/SA-MP service separation is working;
- replacing all existing docs/plans with a new naming scheme in one commit.

---

# 20. Decision index

The normative cross-cutting decisions are consolidated in **Section 0.1 Non-negotiable invariants** and the formal contracts in Sections 1, 4, and 6. Do not maintain a second duplicate decision list here.

For implementation review, verify that a change does not violate those sections. If a new cross-cutting decision is required, record it in an ADR and then update the single canonical section that owns the rule.

---

# 21. Open decisions that may be made later

These are intentionally deferred and must not block the initial phases:

- final project/crates.io naming/prefix;
- whether GTA 1.0 US Compact and Hoodlum become separate profiles or one verified-compatible profile;
- exact public shape of advanced entity iterators;
- render service backend and ImGui ownership;
- plugin manifest format;
- host-managed mod directory format;
- hot reload policy;
- whether to expose an advanced unsafe hook service;
- CLEO opcode compatibility scope;
- whether to implement a SAMPFUNCS-compatible C ABI facade for external consumers;
- code generation format for the eventual large plugin-sdk symbol/layout database.

When one of these becomes necessary, add a short ADR/design note before implementing a cross-cutting choice.

---

# 22. Documentation maintenance

During migration, treat existing repository documents as follows:

- `AGENTS.md`: keep concise; point agents at this handoff and focused guides.
- `ARCHITECTURE.md`: describe the architecture that is actually implemented, not future phases.
- `CORE.md`: describe current shipped capabilities.
- `TODO.md`: track current active phase and near-term incomplete features.
- `docs/SPLIT_PLAN.md` and completed structural/profile plans: keep as historical implementation records; do not rewrite them to match the new vision.
- this handoff: update phase checkboxes and decisions as the large migration proceeds.

When a phase is completed, annotate it with the completion date and commit hash if practical.

---

# 23. Final instruction to Codex

Treat this document as an execution contract. Start with the ordered tasks in Section 17 and stop scope expansion when a phase's acceptance criteria are not yet met.

For every native vertical slice, preserve the canonical flow:

```text
verified native knowledge
        -> host backend
        -> versioned C service
        -> safe Rust facade
        -> test/example
```

When evidence is insufficient, keep the feature unsupported and document the missing evidence instead of guessing. When existing behavior is being replaced, prove parity before deleting the old path.
