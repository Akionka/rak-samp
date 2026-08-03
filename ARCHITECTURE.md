# Architecture

## Process model

```text
GTA ASI loader
 ├─ rak_samp.asi (host and native hooks)
 ├─ feature-a.asi ─┐
 └─ feature-b.asi ─┴─> rak_samp_plugin_api ─> RakSamp_GetApiV1 ─> host
```

Each ASI loads independently. Only the host links the native hook backend;
plugins use the versioned ABI client crate.

## Components

| Area | Location | Purpose |
| --- | --- | --- |
| Bootstrap and API | `src/lib.rs`, `src/host_api.rs`, `src/logging.rs` | Start safely, publish attached-versus-ready host state, and log lifecycle events. |
| Runtime | `src/runtime.rs`, `src/event.rs`, `src/bitstream.rs` | Dispatch events, enforce bounded exact-bit payloads, and send exact typed protocol actions (including SCM events) and sync actions through original RakClient calls. |
| Native backend | `src/platform/win32.rs`, `src/platform/win32/r1_client.rs`, `src/client.rs` | Detect and retain the recognized SA-MP version, manage hooks, and cross the RakNet boundary. The R1 profile gates direct local and cached CNetGame helpers. |
| Plugin API | `plugin_api/src/lib.rs`, `plugin_api/src/raknet.rs` | Define the versioned C-compatible ABI, safe filtered/typed callbacks, static protocol-name catalogs, owned bounded BitStreams, bounded native StringCompressor copies, and grouped subscription shutdown. Alpha releases may make explicit compatibility breaks. |
| Typed events | `plugin_api/src/events/` | Provide R1 packet and RPC codecs, with shared mock ABI test support. |
| Examples | `examples/` | Demonstrate one typed callback, grouped typed handlers, and validation lifecycle/self-tests. |

## Lifecycle

1. `DllMain` starts a bootstrap worker and returns immediately.
2. The worker waits for `samp.dll`, detects the client, and installs the
   constructor hook.
3. RakClient construction completes the client hooks; the host reports ready
   or failed state.
4. Plugin workers resolve the host, validate the API table, and register
   callbacks, optionally as a `SubscriptionSet`.
5. Before a plugin unloads, its worker unregisters and waits for every callback
   to quiesce.

## Event flow

```text
RakNet traffic -> native hook -> bounded event -> listeners
                                      |             |
                                      |       continue/block/replace
                                      v
                              original client path
```

Listeners run in registration order. A listener may inspect or replace an
exact-bit payload; unmatched traffic is passed through. Typed descriptors first
decode and validate the payload, then serialize a replacement before the host
applies it atomically. Explicit sends intentionally bypass outgoing listeners.
Incoming packet emulation is queued through the captured RakPeer receiver;
incoming RPC emulation dispatches before the native receiver.

The incoming-packet vtable detour also performs a separate direct-client pump
before packet handling. For a verified SA-MP R1 plus GTA SA 1.0 US profile it
begins refreshing an owned local-player snapshot only after its `INIT_GAME`
server assignment matches the pool ID on two game-thread refreshes, then keeps
it fresh from the verified player pool. The same entry caches R1's opaque
`CNetGame` state scalar, copied current-server metadata, and the validated
three-value local chat display mode, five-value local cursor mode, and
scoreboard-open, dialog-active, and chat-input-active flags. It also copies the
active dialog's fixed core—ID, typed style, bounded caption, and server-side
flag—as an owned optional snapshot; it deliberately does not follow dynamic
dialog text, button, edit-box, or list pointers. None of those caches drives
snapshot readiness. It also makes one owned copy of the fingerprinted R1
animation table. Demand-refreshed remote player-directory reads are drained
from a separate bounded queue (at most four R1 accessor sequences per pump)
and copied into a host-owned cache. It releases the dialog, chat, and death-window
queue locks, then calls `CDialog::Show`, `CChat::AddEntry`, and
`CDeathWindow::AddMessage` for no more than four copied requests from each
queue. It clears the cache
while the ID is still the provisional zero or SA-MP's unassigned `0xFFFF`
sentinel. It neither consumes, creates, nor redispatches
packet/RPC events. Non-R1 and failed GTA fingerprints do not dereference
direct-client layouts and report `UnsupportedVersion` through the ABI.
The plugin API's local-player convenience queries, including score and ping,
are projections of this one host-owned cache; they do not create additional
native reads. `HostApi::player_info` is the separate bounded directory cache:
local IDs project that snapshot, remote IDs are demand-refreshed, and no native
player/ped/pool/GTA handle reaches a plugin.
`HostApi::player_count` is a second scalar cache populated by the exact R1
`CPlayerPool::GetCount` accessor in both NPC modes; it does not inspect the
GTA world or expose streamed-ped counting.
`HostApi::player_max_id` is a third cache containing only the bounded R1
non-streamed player-pool maximum ID, refreshed by the same game-thread pump
after the profile's `UpdateLargestId` signature and fixture-backed field offset
have passed. It likewise does not inspect GTA peds.
`HostApi::is_vehicle_defined` is a separate 2,000-slot boolean cache. Plugin
threads enqueue unknown vehicle IDs; the pump drains at most four R1 accessor
calls per entry and publishes only owned booleans, never a vehicle/pool/GTA
pointer.
`HostApi::is_text_label_defined` is a separate 2,048-slot boolean cache.
Plugin threads enqueue unknown label IDs; the pump drains at most four bounded
R1 pool-flag reads per entry and publishes only owned booleans, never label
text or a label/pool pointer.
`HostApi::is_textdraw_defined` is a separate 2,304-slot boolean cache, in the
R1 raw order of 2,048 global followed by 256 local slots. Plugin threads enqueue
unknown pool indexes; the pump drains at most four bounded pool-flag reads per
entry and publishes only owned booleans, never textdraw data or a pool pointer.
`HostApi::is_object_defined` is a separate 1,000-slot boolean cache. Plugin
threads enqueue unknown object IDs; the pump drains at most four bounded pool-
flag reads per entry and publishes only owned booleans, never object data, a
pool pointer, or a GTA handle.
`HostApi::gangzone` is a separate 1,024-slot cache of owned rectangle and colour
records. Plugin threads enqueue unknown gangzone IDs; the pump drains at most
four bounded pool reads per entry and publishes only scalars, never a gangzone
or pool pointer.

## Native and ABI boundaries

Native pointers, client addresses, vtables, and StringCompressor calls remain
inside the host. The host patches only its RakClient slots and restores a slot
only when it still owns it. The ABI passes copied bytes, capacities, statuses,
and bit lengths, never Rust-owned values or client pointers. Owned string
decoding also returns a scalar read cursor; the plugin API applies it to its
own `BitStream` only after the host reports success.

Typed local-player protocol actions (`send_request_class`,
`send_interior_change`, `send_spawn`, `send_enter_vehicle`, and
`send_exit_vehicle`) follow that same boundary: they serialize the exact R1
outbound RPC and call the original RakClient path, but never invoke native
local-player methods or mutate client state.

The packet/local-profile layout fixture and live validation protect the Windows
x86 boundary. R1 provides the authoritative typed layouts; detected non-R1
clients are raw event targets pending validation. See [REVIEW.md](REVIEW.md)
for evidence and [VALIDATION.md](VALIDATION.md) for the test procedure.

`tests/e2e/` provides a separate fixture host, plugin, and runner. The runner
loads the host as `rak_samp.asi`, loads the plugin independently, dispatches a
test RPC, then verifies that synchronized shutdown removes the callback before
the plugin DLL is released. CI runs it after the release build.
