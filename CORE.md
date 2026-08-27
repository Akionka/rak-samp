# Core Features

`samp-client-sdk` has two pillars:

- `sdk/` is the public Rust package imported as `samp_client_sdk` by ASI
  plugins; its public paths follow cohesive modules rather than compatibility
  aliases.
- `crates/samp-protocol/` is the platform-independent Rust package for owned
  Protocol bitstreams, exact-bit payload values, directional typed Wire
  descriptors with explicit trailing policies, Packet/RPC name catalogs, outgoing chat,
  slash-command, profile-neutral byte-aligned RPC codecs, and common
  byte-aligned Packet codecs. Directional marker traits make an invalid typed
  subscription or send fail to compile. It also owns four fixed incoming RPC batches:
  server messages through vehicle stream-out, vehicle position through player name tags, the
  26 descriptors from `CLIENT_CHECK` through `SET_CAMERA_BEHIND`, and the 29 descriptors from
  `ATTACH_CAMERA_TO_OBJECT` through `PLAYER_EXIT_VEHICLE`. `SHOW_DIALOG` remains in the SDK until
  the Native encoded-string extension boundary moves.
  Plugins that need these values depend on it directly; the legacy SDK does
  not re-export it.
- `samp-client-sdk-host` owns the Windows x86 bridge and produces
  `samp_client_sdk.asi`; its runtime keeps failure types and send policy
  separate from lifecycle control, while the R1 bridge isolates approved
  native addresses, object layouts, and guarded memory access from operation
  sequencing. The Windows backend also separates bounded producer-side command
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
  synchronous listener dispatch. The game-process detour forwards into the
  root-owned tick executor, and the constructor detour forwards into
  root-owned client-hook setup. Hook installation and restoration remain
  root-owned, and every enabled MinHook target logs its name, target, detour,
  and trampoline. Host ABI entry points and the ordered V1 table remain in
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

The SDK root re-exports safe types, ABI declarations, wrapper glue, resolution,
and subscription ownership from dedicated modules. The Win32 root retains only
shared state and tick ordering; backend forwarding, command execution, queue
draining, refresh publication, native bitstream/string work, and hook patching
live in dedicated child modules. One version-selected `NativeClientProfile`
gates every direct bridge. Four equal profile specifications contain explicit
per-version RVAs, layouts, and narrow strategies for R1, R3-1, R5-1, and
DL-R1. The common game-tick pump and cache refresh paths consume only
`NativeClientProfile`; shared modules own singleton lookup, native aliases,
textdraws, UI, player/pool reads, and handle lookups. The Host API root retains
its export and ordered ABI
table while `listeners.rs` owns listener lifecycle and dispatch.

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
3. RakClient construction installs the owned networking hooks. The bootstrap
   worker publishes ready state only after those hooks report ready.
4. Plugin workers resolve `SampClientSdk_GetApiV1`, validate the table, and
   register owned subscriptions.
5. Plugin workers unregister and wait for callbacks before their DLL unloads.
