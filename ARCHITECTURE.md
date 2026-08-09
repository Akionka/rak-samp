# Architecture

## Process model

```text
GTA ASI loader
 ├─ samp_client_sdk.asi (samp-client-sdk-host)
 ├─ feature-a.asi ─┐
 └─ feature-b.asi ─┴─> samp-client-sdk ─> SampClientSdk_GetApiV1 ─> host
```

Each ASI loads independently. Only the host links the native hook backend;
plugins use the public, versioned SDK package.

## Components

| Area | Location | Purpose |
| --- | --- | --- |
| Bootstrap and ABI | `src/lib.rs`, `src/host_api.rs`, `src/logging.rs` | Start outside `DllMain`, publish status, and export `SampClientSdkApiV1`. |
| Runtime | `src/runtime.rs`, `src/event.rs`, `src/bitstream.rs` | Own bounded event data, dispatch listeners, and send exact RakNet payloads. |
| Native backend | `src/platform/win32.rs`, `src/platform/win32/r1_client.rs`, `src/client.rs` | Select networking offsets, own hooks, and apply the fixed R1 bridge offsets with local validation. |
| Public SDK | `sdk/src/lib.rs`, `sdk/src/facade.rs`, `sdk/src/raknet.rs`, `sdk/src/raw.rs` | Resolve the host through `Samp`, provide safe subsystem views, callback helpers, owned BitStreams, protocol catalogs, and explicit unsafe native-address access. |
| Typed events | `sdk/src/events/` | Provide bounded R1 packet/RPC codecs and mock ABI support. |
| Examples | `examples/` | Show a minimal subscription and a chat-command plugin. |
| Layout fixture | `tests/fixtures/raknet_layout.cpp` | Independently verify native packing for boundary layouts. |

## Lifecycle

1. `DllMain` starts a bootstrap worker and returns.
2. The worker waits for `samp.dll`, recognizes the client build, and installs
   the owned RakClient constructor hook.
3. RakClient construction installs the owned networking hooks and publishes
   ready state.
4. Plugin workers resolve `SampClientSdk_GetApiV1`, validate the table, and
   register owned subscriptions.
5. Plugin workers unregister and wait for callbacks before their DLL unloads.

## Event and native boundaries

```text
RakNet traffic -> owned hook -> bounded event -> listeners -> original client path
```

Listener replacements are serialized before the atomic ABI replacement call.
Callback values are never retained. The host captures the original hook state
before it enables a detour and restores only its own vtable slots during
shutdown.

The R1 direct bridge never exports native pointers. It reads only after local
pointer/range/capacity validation and publishes owned snapshots. Local UI
requests are copied to the shared, bounded `GameCommand` queue; its receipt
core preserves FIFO snapshots, retryable timeouts, detached waiters, and
shutdown results. `SampClientSdkApiV1` appends typed UI submissions and common
poll/wait/release slots using fixed receipt/result storage; `CommandReceipt<T>`
owns release-on-drop on the Rust side. The owned `CGame::Process` detour at
GTA SA 1.0 US address `0x53E4B0` retains its trampoline, calls the original
once, and then drains the pre-tick command snapshot and refreshes snapshots.
Incoming packet detours remain networking-only. The same post-tick dispatcher
invokes the verified R1 `SCLocalPlayer::{Spawn, SetSpecialAction, SetColor,
SendUnoccupiedData}` and `SCPlayerPool::SetLocalPlayerName` entry points,
invokes connected `SCRemotePlayer::SetColor`, and writes the validated R1
send-rate globals; none of those operations can run from a plugin thread. It
also runs R1 `CGame::SetCursorMode` and, when
hiding the cursor, `CGame::ProcessInputEnabling`, preserving the native input
side effects instead of mutating the cursor field directly. Chat display-mode
changes use the same dispatcher to write the validated `CChat::m_nMode` scalar,
and dialog closes invoke R1 `CDialog::Close` with a validated response button.
R1 connection control copies a bounded host and port before setting
`GAME_MODE_WAITCONNECT`; disconnect validates and invokes RakClient's slot-2
`Disconnect` with channel zero, then calls `CNetGame::ShutdownForRestart` and
invalidates all captured connection state.
Textdraw deletion validates the R1 pool and identifier before dispatching to
`CTextDrawPool::Delete`; position updates require a present textdraw plus
writable fixture-backed finite `x` and `y` fields. Letter-style updates use the
same presence checks before writing the fixture-backed dimensions and colour;
the proportional flag is likewise written only through its validated byte, and
shadow or outline updates validate both their byte and background-colour fields.
Box updates similarly validate finite fixture-backed dimensions, colour, and
the native boolean byte before queuing their write.
Alignment accepts only the three source-defined values and updates its three
fixture-backed flags together on the dispatcher.
Display-string updates own a NUL-free payload bounded by the independently
mapped R1 `m_szString` capacity before clearing and copying the validated
native storage.
Reads copy the same fixture-backed fixed buffer into the cached textdraw ABI
record, never exposing its native address.
Model-style writes validate finite rotation and zoom values, then update the
contiguous fixture-backed rotation, zoom, and vehicle-colour fields.
Chat-entry writes own bounded text and prefix bytes before updating the
validated fixed R1 history record on the dispatcher.
Chat-entry reads demand-refresh one fixture-backed fixed history record on the
dispatcher and return only its owned text, prefix, and colours from the cache.
Selected-ID 3D label creation owns and NUL-terminates bounded input before
calling `CLabelPool::Create` on the dispatcher with validated geometry.
3D text-label deletion validates its pool pointer and source-documented
`CLabelPool::Delete` result on the dispatcher before completing its receipt.
Dialog list selection is cached from the validated DXUT listbox and queued
writes validate the same selected-index field before mutation.
The list-item count is likewise copied only after its fixture-backed signed
field is readable and non-negative.
The active dialog snapshot also copies the fixture-anchored dynamic dialog
text, the DXUT editbox text through the native GetText path, and each bounded
listbox item string; the editbox replacement queues a bounded NUL-free write
through the same dispatcher.
Chat-input reads are copied into the game-thread cache and commands own their
bounded text before invoking R1 DXUT edit-box and `CInput` methods on the
dispatcher.

The explicit `raw` module exposes only opaque native R1 singleton/pool
addresses plus host-captured RakClient, validated RakPeer, player-pool, and
vehicle-pool addresses, including the 26 known RakClient vtable slots. Its accessors are
unsafe, do not manufacture Rust references, and remain valid only while the
matching client remains loaded.

The same bounded queue owns copied UI requests, explicit RakClient sends, and
incoming emulation. Public `Net` send and emulation methods return owned
`CommandReceipt<()>` values, allowing a plugin worker to observe native
success or failure on a later game tick; documentation-hidden legacy ABI calls
remain submission-only. Callback-local event replacement remains synchronous.
Typed `Net` helpers encode their owned RPC or packet payload before submitting
through that same receipt-bearing path.
The R1 cursor-mode, scoreboard, and dialog-client-side mutations validate their
input and writable fixed-layout field ranges inside that queue before storing
the new values. The dialog bridge maps its public client-side flag to the
inverse native `server_side` field.
R1 game-state writes are constrained to the established native enum values and
the independently fixture-anchored `CNetGame` field.
The server-settings pointer remains raw and opaque; its R1 packed layout is
checked against the independent C++ fixture before it is exposed.
The pickup-pool raw address uses the same fixture-verified R1 pool layout and
is never converted into a Rust reference.
The local-player raw address is captured only during the game-thread refresh
and cleared at connection boundaries, so plugin threads never resolve it by
calling client code.
Object, pickup, vehicle, and player-ped handle reads use per-pool bounded
request queues with per-tick pump limits and first-read `NotReady`. The native
profile reads object `SCEntity::m_handle`, pickup `m_handle` GTAREF slots, and
converts validated vehicle/ped pointers through the fixture-anchored GTA SA
`CPools::GetVehicleRef`/`GetPedRef` targets; reverse lookups drain bounded pool
scans. All handle caches and pending requests are cleared at connection
boundaries, and the SDK exposes typed non-null handle newtypes with
`to_id` conversions.
The raw bit-stream data address is a borrow of the SDK-owned bounded buffer,
so it never exposes native RakNet allocation ownership across the plugin ABI.
`Local` protocol actions are thin typed routes to the exact same queued `Net`
operations, preserving the distinction between a server-bound send and a local
native state change.

R1 cache refreshes are bracketed by a monotonic generation after each native
game tick. A connection-state boundary invalidates player, vehicle, label,
textdraw, object, and gangzone caches together with their pending heavy reads.

The `sdk/` crate keeps protocol codecs and BitStreams independent of the native
host. This lets exact wire-vector and callback behavior be tested without a
live client, while the retained C++ fixture checks C++↔Rust packing at the
native boundary.
