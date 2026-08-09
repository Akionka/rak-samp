# Module Split Plan

Status: proposed. No code changed yet.

## 1. Motivation

Two files in the host crate and several in the SDK crate have grown into
god-files: they mix three or four different layers (state, native access,
protocol codecs, ABI plumbing) in a single file of thousands of lines. The
biggest offenders:

| File | Lines | Mixed concerns |
| --- | --- | --- |
| `src/platform/win32.rs` | ~6,800 | `BackendState` (~90 fields), `GameCommand` dispatch, 17 request queues, ~35 cache refreshers, hooks/detours, memory helpers |
| `src/platform/win32/r1_client.rs` | ~3,300 | ~400 RVA constants, `R1ClientProfile`, all pointer-based reads, layout tests |
| `src/host_api.rs` | ~3,570 | `HostState`, `monitor_client_hooks`, ABI table, ~150 ABI functions, `*_to_abi` converters |
| `sdk/src/lib.rs` | ~5,670 | Public types, all `repr(C)` ABI structs, `SampClientSdkApiV1` table, `HostApi` (~2,500 lines), subscription machinery, host resolution |
| `sdk/src/facade.rs` | ~1,690 | 20+ facade view structs |
| `sdk/src/events/` | ~2,300 | Codec registries, ~160 `decode_*`/`encode_*` pairs, ~80 RPC structs |

Goals:

- One file — one responsibility; each module becomes a natural boundary that
  enforces encapsulation at compile time.
- Files sized for navigation (target: ≤ ~900 lines, with data-heavy
  constant/struct modules as the accepted exception).
- Faster incremental builds (rustc compiles files in parallel).
- Layout/fixture tests live beside their code instead of at the end of a
  6,800-line file.
- Smaller, reviewable diffs when working on one layer.

## 2. Non-goals and guardrails

- **No code moves out of `src/` / `sdk/src/`.** This is an in-crate
  re-organization only: single files become directories with submodules.
- **No public ABI change.** The layout of `SampClientSdkApiV1`, all `repr(C)`
  structs, fixed offsets, and the exported symbol stay untouched.
- **No behavior change.** The retained unit and C++ layout fixture tests are
  the guard. Each migration step must pass
  `cargo build --workspace`, `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`, and `cargo fmt --check`.
- `command.rs`, `event.rs`, `client.rs`, `bitstream.rs` (host) and
  `events/core.rs`, `events/rpc/incoming/mod.rs` registries are already
  focused; they are not split further.
- Do not split the exported ABI table into multiple statements; keep the
  table as one literal block per module (see 3.4).
- The stale snapshot at `.codex-rewrite-98faec8/` is ignored by this plan
  (candidate for deletion in a separate change).

## 3. Host crate (`src/`)

### 3.1 `src/platform/win32.rs` → `src/platform/win32/`

| New module | Contents | ~Lines |
| --- | --- | --- |
| `mod.rs` | `Backend` (thin composition root, ~7 fields), `attach`, `run_game_process_tick`, `shutdown`, `dispatch_packet_stream`/`dispatch_rpc_stream`/`dispatch_raw_packet`, `active_state`/`clear_active_backend` | 900 |
| `context.rs` | `ClientContext`: `module_base`, `version`, `addresses`, `r1_client`, `rak_client`, `rpc_receiver`, connection (`player_address`/`player_port`), `string_codec` | 200 |
| `hooks.rs` | `HookState` (trampolines, captured originals, `client_hook_status`, `game_thread_id`, `hooks`), `VtableHook`, `InlineHook`, `HookStorage`, the five detours, `install_*_hook` | 800 |
| `packets.rs` | `RawPacket`, `RawBitStream`, `NativeBitStream`, `packet_stream`, `validated_packet_byte_len`, `parse/build_rpc_envelope`, `call_outgoing_packet`/`call_incoming_packet`/`deallocate_packet`/`call_outgoing_rpc` | 700 |
| `commands.rs` | `GameCommand`, all `submit_*`, `execute_game_commands`, `submit_game_command`/`queue_network_command` helpers | 1,100 |
| `requests.rs` | `RequestQueues` (17 bounded queues), request-queue capacity constants, all `queue_*`/`take_*` | 700 |
| `caches.rs` | `CacheStore` (all `*CacheEntry` maps, local-player snapshot/candidate, scalar snapshot + `*_ready` pairs, `server_info_snapshot`, `animation_catalog`, `cache_generation`), `clear_*`, `refresh_*`, `invalidate_connection_state` | 1,000 |
| `reads.rs` | Direct reads (`local_player`, `player_info`, `server_info`, `textdraw`, ...), `cached_direct_client_value` | 600 |
| `mem.rs` | `write_protected`, `loaded_samp_module`, `pe_entry_point` | 150 |

**State decomposition (decision D1).** The plan splits the state, not just
the methods. Each submodule owns the state type it operates on:

```rust
struct BackendState {          // mod.rs, thin
    registry: Registry,
    context: ClientContext,    // context.rs
    hooks: HookState,          // hooks.rs
    commands: CommandQueue<GameCommand, ()>, // command.rs (already exists)
    requests: RequestQueues,   // requests.rs
    caches: CacheStore,        // caches.rs
}
```

Consequences:

- Method impls move with their state: `queue_*`/`take_*` live beside
  `RequestQueues`, `refresh_*`/`clear_*` beside `CacheStore`. No method
  operates on a 90-field struct anymore.
- Cross-module operations (e.g. `execute_game_commands` touching both
  `commands` and `caches`) take `&mut` borrows of both substructs from
  `mod.rs` call sites, or the submodule re-imports the peer struct — the
  dependency direction becomes visible instead of implicit.
- `Backend`/`BackendState` remains the only externally visible name; all
  substructs stay `pub(crate)`/`pub(super)`.

`src/platform/mod.rs` keeps `mod win32;` unchanged (directory resolves to
`win32/mod.rs`).

### 3.2 `src/platform/win32/r1_client.rs` → `src/platform/win32/r1_client/`

| New module | Contents | ~Lines |
| --- | --- | --- |
| `mod.rs` | `R1ClientProfile` and its methods (singleton/pool lookups) | 1,100 |
| `rva.rs` | All ~400 RVA/offset constants, grouped by native object (dialog, chat, input, netgame, pools, textdraw, label, ...) | 1,300 |
| `memory.rs` | `read_pointer`, `read_unaligned`, `read_vector3`, `read_r1_bool`, `bounded_c_string`, `nul_terminated`, `readable_range`, `writable_range` | 150 |
| `layout.rs` | `NativeDxutComboBoxItem`, `NativeVector3`, fixture-backed layout tests | 700 |

### 3.3 `src/host_api.rs` → `src/abi/`

| New module | Contents | ~Lines |
| --- | --- | --- |
| `mod.rs` | `HostState`, `SampClientSdk_GetApiV1`, `begin_bootstrap`, `host()`, `clone_initialized`, `ListenerKind` | 400 |
| `monitor.rs` | `monitor_client_hooks` | 550 |
| `table.rs` | The `SampClientSdkApiV1` table literal + status constants | 550 |
| `events.rs` | `register_packet`/`register_rpc`/`unregister`/`unregister_and_wait`, `event_*` read/write/replace helpers | 400 |
| `net.rs` | `send_packet`/`send_rpc`, `submit_*`, `emulate_incoming_*`, string codec entry points | 450 |
| `local.rs` | `submit_local_*` (dialog, chat, death, cursor, scoreboard, game state, player, textdraw, label, chat input) | 400 |
| `direct.rs` | Read entry points (`local_player`, `player_info`, `server_info`, `gangzone`, handle reads, ...) | 300 |
| `raw.rs` | `raw_rakclient`, `raw_rakpeer`, `raw_player_pool`, `raw_vehicle_pool`, `raw_local_player`, `raw_native_address` | 200 |
| `convert.rs` | All `*_to_abi`, `send_options`, `samp_version_to_abi`, error mapping helpers | 350 |

### 3.4 `src/runtime.rs` → `src/runtime/`

| New module | Contents |
| --- | --- |
| `mod.rs` | `Runtime`, `attach`, `Drop` |
| `types.rs` | All snapshot/request structs (`LocalDialogSnapshot`, `PlayerInfoSnapshot`, `TextdrawSnapshot`, ...) and `Vector3` |
| `errors.rs` | `AttachError`, `SendError`, `DirectClientError`, `CodecError`, `ClientHookStatus` |
| `options.rs` | `SendOptions`, `PacketPriority`, `PacketReliability`, `validate_packet_options` |

## 4. SDK crate (`sdk/src/`)

### 4.1 `sdk/src/lib.rs` → split by layer

| New module | Contents | ~Lines |
| --- | --- | --- |
| `lib.rs` | Re-exports, crate-level constants, module wiring | 100 |
| `limits.rs` | `MAX_SAMP_*` constants | 50 |
| `types.rs` | Public safe types (`LocalDialog`, `LocalPlayer`, `PlayerInfo`, `TextDraw`, ...) and their enums | 600 |
| `abi/types.rs` | All `SampClientSdk*V1` `repr(C)` structs + `Default` impls | 600 |
| `abi/table.rs` | `SampClientSdkApiV1` struct + `CallbackState` | 550 |
| `host_api/mod.rs` | `HostApi` shell, `CallbackState` handling, `register_handlers` | 300 |
| `host_api/net.rs` | Send/emulate/typed-RPC/packet helpers, string codec | 400 |
| `host_api/local.rs` | `submit_local_*`, local reads, `local_animation*` | 500 |
| `host_api/players.rs` | Player/remote-state reads, player count/max-id | 350 |
| `host_api/pools.rs` | Textdraw/label/object/pickup/vehicle/gangzone reads and submissions, handle reads | 500 |
| `host_api/ui.rs` | Dialog/chat/chat-input/cursor/scoreboard reads and submissions | 450 |
| `host_api/convert.rs` | All `*_from_abi` converters, `cached_boolean`, `valid_bounded_bytes` | 300 |
| `subscriptions.rs` | `CommandReceipt`, `Subscription`, `SubscriptionSet`, their error types | 450 |
| `resolve.rs` | `resolve_host`, `wait_for_host`, `wait_for_default_host`, `ResolveError` | 150 |

Note: `impl HostApi` may be spread over multiple files via multiple
`impl` blocks in submodules (each file declares
`impl HostApi { ... }`); this is idiomatic Rust and keeps every
`impl` block in the module it belongs to.

### 4.2 `sdk/src/facade.rs` → `sdk/src/facade/`

| New module | Contents |
| --- | --- |
| `mod.rs` | `Samp`, `Probe`, `bounded_id`, `gta_handle` |
| `net.rs` | `Net` |
| `server.rs` | `Server` |
| `local.rs` | `Local`, `LocalPlayer` |
| `players.rs` | `Players`, `Player` |
| `pools.rs` | `Textdraws`, `Labels`, `Objects`, `Pickups`, `Vehicles`, `Gangzones` + handle newtypes |
| `ui.rs` | `Dialogs`, `Chat`, `DeathWindow`, `ChatInput`, `Cursor`, `Scoreboard`, `Anim`, `ChatEntry` |

### 4.3 `sdk/src/events/` — codecs split by domain

- `rpc/incoming/fixed.rs` (~1,160) and `rpc/incoming/types.rs` (~690) are
  both split along the same domain axis:
  `player.rs`, `vehicle.rs`, `object.rs`, `world.rs`, `textdraw.rs`,
  `label.rs`, `dialog.rs`, `misc.rs` under `fixed/` and `types/`
  respectively; shared primitives (`decode_vector3`, `decode_f32`, ...) go to
  `fixed/common.rs`.
- `rpc/incoming/r1.rs` (~900) → `rpc/incoming/r1/` with `spawn.rs`,
  `world.rs`, `object.rs`, `misc.rs`; bit-level helpers stay in `r1/common.rs`.
- `rpc/outgoing.rs` (~780) → `rpc/outgoing/`: `chat.rs`, `player.rs`,
  `vehicle.rs`, `dialog.rs`.
- `packet/mod.rs` (~1,000) → `packet/`: `ids.rs` (ID constants),
  `common.rs` (vector/quaternion/compressed-float helpers), `player.rs`,
  `vehicle.rs`, `connection.rs`, `stats.rs`.
- RPC ID constants that are duplicated between `fixed.rs` and `outgoing.rs`
  move to `rpc/ids.rs` and are `pub(crate)`-re-exported.
- `events/tests.rs` (~980) and the bulk of `events/test_support.rs` (~1,780)
  are distributed as `#[cfg(test)] mod tests` submodules beside their codecs;
  the mock ABI stays in `test_support.rs` (trimmed to the ABI-mock surface,
  `test_*` helpers unchanged in behavior).

## 5. Migration order

Each step is one independent change; the guardrails from §2 apply to every
step.

1. **Host, mechanical first:** `win32/mem.rs` → `win32/requests.rs` →
   `win32/hooks.rs` → `win32/packets.rs`. These are pure moves with no logic
   change and no `pub` surface change.
2. **Host, stateful:** `win32/caches.rs` → `win32/commands.rs` →
   `win32/reads.rs`. Higher risk (large bodies share `BackendState`); review
   diffs carefully.
3. **Host ABI:** `abi/convert.rs` → `abi/net.rs`/`abi/local.rs`/`abi/direct.rs`
   → `abi/monitor.rs` → `abi/table.rs`. The table literal moves last, in one
   piece.
4. **Host runtime:** `runtime/types.rs` + `errors.rs` + `options.rs`.
5. **SDK:** `limits.rs`/`types.rs`/`abi/` → `subscriptions.rs`/`resolve.rs` →
   `host_api/` (net, local, players, pools, ui, convert) → `facade/`.
6. **SDK events:** `rpc/ids.rs` → `packet/` → `rpc/outgoing/` → `rpc/incoming/`
   (types, fixed, r1) → test distribution.
7. **Docs:** update `ARCHITECTURE.md` (module map table) and `CORE.md`
   (per-pillar references) after each major step, per `AGENTS.md`.

## 6. Completion criteria

- No source file above ~900 lines except: data-heavy constant modules
  (`r1_client/rva.rs`, `abi/table.rs`, `abi/types.rs`) and codec registries
  (`rpc/incoming/*/mod.rs`), capped at ~1,300 lines.
- Every module has one documented responsibility; `cargo doc` shows a
  navigable module tree.
- All four guardrail commands from §2 pass.
- Public API, ABI layout, and exported symbols are byte-identical (verified by
  the existing test suite and `git diff --stat` on non-module files).

## 7. Decisions and open questions

Decisions locked during the interview:

- **D1 — State is decomposed, not just methods.** `BackendState` splits into
  `ClientContext` / `HookState` / `RequestQueues` / `CacheStore`; each module
  owns its state type (see §3.1).
- **D2 — `value` + `value_ready` pairs are kept as-is; no `Mutex<Option<T>>`
  merge.** The pairs are a deliberate lock-free publication pattern: readers
  do two `Acquire` loads with no lock (win32.rs:2547-2553), writers do
  `swap(AcqRel)` on value then flag, and `crosses_r1_connection_boundary`
  consumes the *previous* (value, ready) pair as a transaction
  (win32.rs:4347-4351). `Mutex<Option<T>>` would put a lock on every plugin
  read and cannot express the boundary-detection semantics. The flag is a
  generation-publication marker cleared by `invalidate_connection_state`
  independently of the data; even the already-`Mutex<Option<T>>` heavy
  snapshots carry a separate `AtomicBool`. Optional follow-up (not part of
  this plan): wrap the idiom in a `PubScalar<T>` newtype exposing
  `get() -> Option<T>` / `set(T)` with the same orderings, so `CacheStore`
  declares one field per scalar instead of two.

Open questions:

- **Q1 — Test placement.** `win32.rs`'s tests build a full `test_backend_state()`
  and exercise cross-module flows (queue → pump → cache). After decomposition,
  unit tests of `RequestQueues`/`CacheStore` move beside their modules, but the
  cross-module tests stay in `win32/tests.rs`. How do we keep the moved unit
  tests from needing the full state (per-substruct test builders)?
- **Q2 — "Mechanical" verification.** The guardrail "no behavior change" is
  currently enforced by the test suite only. Tests are also being *moved* in
  step 6, weakening the guard. Should each pure-move step additionally require
  a diff review with `git diff -w` (whitespace-ignored, pure cut-paste, zero
  edits inside moved blocks)?
