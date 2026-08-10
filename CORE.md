# Core Features

`samp-client-sdk` has two pillars:

- `sdk/` is the public Rust package imported as `samp_client_sdk` by ASI
  plugins; its public paths follow cohesive modules rather than compatibility
  aliases.
- `samp-client-sdk-host` owns the Windows x86 bridge and produces
  `samp_client_sdk.asi`; its runtime keeps failure types and send policy
  separate from lifecycle control, while the R1 bridge isolates approved
  native addresses, object layouts, and guarded memory access from operation
  sequencing. The Windows backend also separates bounded producer-side command
  and cache-refresh requests from game-thread execution, with scalar and owned
  snapshot/catalog, chat-history, gangzone, object, player, text-label, textdraw,
  vehicle, and forward/reverse handle-cache reads gated on completed game-thread
  publication. Native outbound packet and RPC sends execute from the game-thread
  command pump through the captured original RakClient functions. Incoming
  packet emulation transfers a native allocation into the captured receive queue;
  incoming RPC emulation dispatches listeners once before calling the captured
  trampoline directly. RPC envelope helpers preserve optional timestamps and
  exact payload bit lengths across listener replacement. Native outgoing stream
  dispatch publishes listener mutations synchronously and fails open on rewrite
  errors. Raw incoming packet dispatch validates packed metadata before copying
  owned callback data; the incoming detour retains return/deallocate/retry
  ownership through captured native-call wrappers. Incoming network detours
  keep packet ownership and RPC receiver/player publication in `hooks.rs`;
  malformed RPC envelopes fail open to the captured trampoline. Outgoing
  detours call captured originals through non-owning ABI wrappers after
  synchronous listener dispatch. The game-process detour forwards into the
  root-owned tick executor, and the constructor detour forwards into
  root-owned client-hook setup. Hook installation and restoration remain
  root-owned. Host ABI entry points and the ordered V1 table remain in
  `host_api/mod.rs`; `host_api/conversions.rs` converts owned runtime snapshots
  into fixed C-compatible output storage, while `host_api/raw.rs` owns the
  opaque native-address entry points and `host_api/events.rs` owns event
  bitstream operations and native string codecs. `host_api/network.rs` owns
  direct and queued packet/RPC sends and incoming emulation;
  `host_api/animations.rs` owns cached local animation-table reads and lookups;
  `host_api/chat_input.rs` owns the cached local chat-input text read;
  `host_api/dialog.rs` owns the detailed local-dialog snapshot and scalar
  selection reads;
  `host_api/environment.rs` owns game-state, server-info, and version reads;
  `host_api/local_state.rs` owns local UI-state and active-dialog-core reads;
  `host_api/commands.rs` owns receipt polling, waiting, and release, and
  `host_api/handles.rs` owns forward/reverse native-handle lookups.
  `host_api/players.rs` owns local-player snapshots and player-pool reads.
  `host_api/pools.rs` owns bounded pool-existence reads, and
  `host_api/snapshots.rs` owns pooled gangzone, text-label, textdraw, and chat
  entry snapshots.

## Process model

```text
GTA ASI loader
 ├─ samp_client_sdk.asi (samp-client-sdk-host)
 ├─ feature-a.asi ─┐
 └─ feature-b.asi ─┴─> samp-client-sdk ─> SampClientSdk_GetApiV1 ─> host
```

## Lifecycle

1. `DllMain` starts a bootstrap worker and returns.
2. The worker waits for `samp.dll`, recognizes the client build, and installs
   the owned RakClient constructor hook.
3. RakClient construction installs the owned networking hooks and publishes
   ready state.
4. Plugin workers resolve `SampClientSdk_GetApiV1`, validate the table, and
   register owned subscriptions.
5. Plugin workers unregister and wait for callbacks before their DLL unloads.
