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
`CNetGame` state scalar and copied current-server metadata, neither of which
drives snapshot readiness. It releases the dialog, chat, and death-window
queue locks, then calls `CDialog::Show`, `CChat::AddEntry`, and
`CDeathWindow::AddMessage` for no more than four copied requests from each
queue. It clears the cache
while the ID is still the provisional zero or SA-MP's unassigned `0xFFFF`
sentinel. It neither consumes, creates, nor redispatches
packet/RPC events. Non-R1 and failed GTA fingerprints do not dereference
direct-client layouts and report `UnsupportedVersion` through the ABI.
The plugin API's local-player convenience queries, including score and ping,
are projections of this one host-owned cache; they do not create additional
native reads.

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
