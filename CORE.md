# Core Features

`samp-client-sdk` is split into host, service, facade, and Protocol layers:

- `sdk/` is the public Rust package imported as `samp_client_sdk` by ASI
  plugins; its public paths follow cohesive modules rather than compatibility
  aliases. `Net` projects every descriptor through one sealed typed callback
  model. The SDK owns no legacy Protocol descriptor, codec, payload writer, or
  encoded payload value. Direction and Packet/RPC kind stay
  type-level, while decode, replacement, callback lifetime, and Host mutation
  remain private SDK adaptation. Typed failures are classified and diagnosed
  without payload data before fail-open conversion. Protocol replacements use
  canonical owned exact-bit payloads before one failure-atomic Host mutation.
  Protocol-backed sends return `ProtocolSendError`, which preserves the exact
  Protocol encode failure separately from synchronous Host submission or
  enqueue rejection. Queued execution remains observable through its receipt.
- `crates/samp-protocol/` is the platform-independent Rust package for owned
  Protocol bitstreams, exact-bit payload values, public neutral `types` and
  `limits`, one canonical set of cursor-preserving Wire read/write primitives,
  including single-bit booleans with Protocol-owned bounds/source error mapping,
  and nominal built-in Wire descriptors whose identity hides private codec
  implementations. `IncomingPacket`, `OutgoingPacket`, `IncomingRpc`, and
  `OutgoingRpc` are the only public generic descriptor wrappers; every custom or
  ad-hoc descriptor therefore carries direction and an explicit trailing policy.
  The sealed direction-neutral `WireDescriptor` remains their common capability.
  Packet/RPC name catalogs,
  outgoing chat, slash-command, profile-neutral outgoing RPC codecs, common
  byte-aligned Packet codecs, and R1 exact-bit remote synchronization Packet
  codecs remain Protocol-owned. Directional marker traits make an invalid typed
  subscription or send fail to compile. Profile-neutral incoming RPCs live under
  `rpc::incoming::common`; R1 player, session, world, UI, vehicle, actor, and
  object-material RPCs live under `rpc::incoming::r1`. `SHOW_DIALOG`,
  `CREATE_3D_TEXT`, `CREATE_OBJECT`, and `SET_OBJECT_MATERIAL` use the narrow
  `EncodedStringRead` and `EncodedStringWrite` contracts. The SDK injects the
  Host compressor without moving Native code into Protocol. Their canonical
  descriptor encoding consumes the injected writer and returns owned
  `EncodedBits` with the exact bit length.
  Plugins that need these values depend on it directly; the legacy SDK does
  not re-export it.
- `crates/modkit-abi/` is the platform-independent Rust package for the stable
  C ABI primitives shared by host and plugin crates: the `ModResult` newtype and
  its numeric constants, fixed-width `ServiceId`/`SubscriptionId`/
  `CommandReceiptId`, opaque game-context token and execution constraints,
  `ServiceHeader`, the `ModHostApiV1` bootstrap table, `CoreServiceV1`,
  `GtaSaServiceV1`, the small `SampServiceV1` and `SampNetServiceV1` tables,
  and the migration-only `LegacySampServiceV1` wrapper.
  It has no Windows, MinHook, GTA, or SA-MP native dependency.
- `crates/modkit-sdk/` is the plugin-side safe connection to the host. It
  resolves only `GtaModHost_GetApiV1` through `Host::connect`/`connect_to`,
  performs exact-version `query_service`, and exposes validated Core, GTA,
  SA-MP, and SA-MP network service views plus callback-scoped `GameContext`.
  It never falls back to `SampClientSdk_GetApiV1`.
- `crates/samp/` is the safe service-backed SA-MP facade for new plugins. It
  resolves Core, `SampServiceV1`, and `SampNetServiceV1` through `modkit-sdk`,
  returns owned snapshots and Core-backed receipts/subscriptions, and adapts
  `samp-protocol` descriptors to callback-local exact-bit events. It contains no
  native addresses and has no dependency on the legacy SDK package.
- `crates/modkit-win32/` owns the generic Windows x86 implementation primitives
  reused by the host backends: per-page guarded native-memory reads/writes,
  validated `ReadableRegion`/`WritableRegion` views, checked PE/module helpers,
  and the host-internal unsafe MinHook `InlineHook` wrapper. Protected writes
  restore their original page protection. The crate contains no GTA/SA-MP
  addresses or profile constants, and only it and the native backends depend on
  `windows-sys`/MinHook.
- `crates/gta-sa-native/` owns the GTA-native implementation: the exact
  SHA-256 and image-base gated GTA SA 1.0 US profile; evidence-bearing typed
  symbol, RVA, field, size, and vtable specifications; guarded typed x86 call
  helpers; fixture-backed native matrix/entity/ped layouts; local-ped
  handle/snapshot reads; verified virtual ped teleport; the `CGame::Process`
  tick hook lifecycle; and `CPools` targets. The SA-MP backend remains a tick
  participant and uses the GTA `cpool_ref` wrapper.
- `crates/gta-sa/` owns pointer-free math, positive typed GTA handles, owned
  snapshots, callback-scoped `Gta`/`Player` access, and Core-backed queued
  snapshot/teleport receipts. It contains no fixed native addresses, native
  memory references, or public native pointers.
- `crates/samp-native/` owns direct-client profiles, guarded SA-MP operations,
  backend request/snapshot values, and RakClient hook installation, detour entry
  points, captured-original metadata, and deterministic vtable restoration for
  R1, R3-1, R5-1, and DL-R1. Detours forward through one process-lifetime host
  callback table without depending on host discovery or service adapters.
- `samp-client-sdk-host` owns the Windows x86 bridge and produces
  `samp_client_sdk.asi`; its runtime keeps failure types and send policy
  separate from lifecycle control. The transitional root backend consumes one
  `samp-native` profile for approved R1, R3-1, R5-1, and DL-R1 direct
  operations and guarded memory access.
  The Windows backend also separates bounded producer-side command
  and cache-refresh requests from game-thread execution, with scalar and owned
  snapshot/catalog, chat-history, gangzone, object, player, owned on-foot, in-car, passenger, trailer, and aim sync, text-label, textdraw,
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
  synchronous listener dispatch. The GTA-native game-process runtime marks the
  game thread, drives the SA-MP participant snapshot, calls the captured
  original exactly once, and then drives the SA-MP command/cache pump. The
  constructor detour still forwards into root-owned client-hook setup. Every
  enabled MinHook target logs its name, target, detour, and trampoline. Host ABI
  entry points and the ordered V1 table remain in
  `host_api/mod.rs`; `host_api/conversions.rs` converts owned runtime snapshots
  into fixed C-compatible output storage, while `host_api/raw.rs` owns the
  opaque native-address entry points and `host_api/events.rs` owns event
  bitstream operations and native string codecs. `host_api/network.rs` owns
  direct and queued packet/RPC sends and incoming emulation;
  `host_api/connection.rs` owns connection and disconnection command producers;
  `host_api/animations.rs` owns cached local animation-table reads and lookups;
  `host_api/chat_input.rs` owns cached local chat-input text and command-name
  reads plus its command producers; `host_api/chat_commands.rs` owns fixed
  native command trampolines and callback-safe command-subscription lifetime;
  `host_api/dialog.rs` owns detailed local-dialog reads, one-shot owned
  close-response reads, and command producers;
  `host_api/environment.rs` owns game-state, server-info, and version reads;
  `host_api/local_state.rs` owns local UI-state and active-dialog-core reads;
  `host_api/local_commands.rs` owns local cursor, scoreboard, cursor-toggle,
  and chat-display command producers;
  `host_api/messages.rs` owns local chat and death-message command producers;
  `host_api/commands.rs` owns receipt polling, waiting, and release, and
  `host_api/handles.rs` owns forward/reverse native-handle lookups.
  `host_api/players.rs` owns local-player snapshots and player-pool reads.
  `host_api/player_commands.rs` owns local-player action, colour, and name
  command producers.
  `host_api/sampfuncs.rs` owns optional loaded-SAMPFUNCS detection and direct
  console logging through its exported implementation.
  `host_api/pools.rs` owns bounded pool-existence reads, and
  `host_api/snapshots.rs` owns pooled gangzone, text-label, textdraw, and chat
  entry snapshots; `host_api/text_labels.rs` owns text-label command producers,
  including typed receipts for game-thread-selected free label IDs and validated
  text replacement.
  `host_api/textdraws.rs` owns textdraw command producers, including
  explicit-slot native creation.

Public nonblocking cache reads and refresh-request producers report `Busy` when
another thread currently owns their mutex. `QueueFull` remains a real bounded
queue-capacity result, while unpublished, absent, disconnected, and poisoned
cache state remains `NotReady`; callers may retry `Busy` later.

Leaf tests are colocated with their stable codecs and facade views. Parent test
modules retain mock-ABI behavior, native ABI/layout checks, and cross-module
Win32 queue, pump, cache-publication, invalidation, and hook-lifecycle tests.

The SDK root re-exports safe types, ABI declarations from `abi/`, wrapper glue,
resolution, and subscription ownership from dedicated modules. `runtime/`
keeps lifecycle/composition separate from requests, snapshots, network,
commands, and reads. The Win32 root retains host composition state; lifecycle,
tick ordering, cache invalidation, callback forwarding, domain command
execution, refresh publication, and native bitstream/string work live in
dedicated child modules. One version-selected profile from
`crates/samp-native/` gates every direct bridge. The crate owns all verified
direct singleton, connection, pool, GTA-handle mapping, player/synchronization,
UI, text-label, and textdraw operations plus RakClient detour/vtable ownership.
Root host composition retains coherent cache scheduling/publication, command
receipts, service tables, captured-original invocation, and listener dispatch.

The Host API root retains
its export and ordered ABI
table while `listeners.rs` owns listener lifecycle and dispatch.

## Process model

```text
GTA ASI loader
 ├─ samp_client_sdk.asi (samp-client-sdk-host)
 ├─ feature-a.asi ─┐
 └─ feature-b.asi ─┴─> samp-client-sdk ─> SampClientSdk_GetApiV1 ─> host
```

New plugins may instead connect through the modkit bootstrap:

```text
GTA ASI loader
 ├─ samp_client_sdk.asi (samp-client-sdk-host)
 └─ modkit-plugin.asi ─> modkit-sdk ─> GtaModHost_GetApiV1 ─> query_service
```

## Lifecycle

1. `DllMain` starts a bootstrap worker and returns.
2. The worker waits for `samp.dll`, recognizes the client build, and installs
   the owned RakClient constructor hook.
3. RakClient construction installs the owned networking hooks. The bootstrap
   worker publishes ready state only after those hooks report ready.
4. Plugin workers resolve `SampClientSdk_GetApiV1`, validate the table, and
   register owned subscriptions. New plugins may instead resolve
   `GtaModHost_GetApiV1` and query exact-version service tables.
5. Plugin workers unregister and wait for callbacks before their DLL unloads.
