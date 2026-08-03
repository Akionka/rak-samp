# Core Features

`rak-samp` builds one Windows x86 host, `rak_samp.asi`. Independent plugins link
only `rak_samp_plugin_api`; the host owns SA-MP detection and native hooks.

## Runtime

Bootstrap runs outside `DllMain`, waits for `samp.dll`, and exposes a ready or
failed host state. Lifecycle and ABI diagnostics go to `rak-samp.log`; packet and
RPC payloads are never logged.

Plugins subscribe to incoming or outgoing packets and RPCs. ID-filtered and
typed-descriptor helpers target one protocol message; `register_handlers!`
groups related registrations. A matching listener can continue, block, or
atomically replace an exact-bit payload. Listeners run in registration order;
nested dispatch on the same thread remains non-blocking. Events are
callback-local. Explicit sends bypass outgoing listeners, while incoming
emulation follows the normal incoming dispatch path exactly once.

## ABI and plugin safety

`RakSampApiV1` is C-compatible and versioned: Rust references, trait objects,
allocations, and native pointers do not cross the DLL boundary. During the
ALPHA stage its contract may intentionally break and does not have to remain
append-only. Payload sizes and bit counts are checked before they reach RakNet.

`HostApi::show_local_dialog` copies a NUL-free R1 dialog into a bounded
32-request host queue. `HostApi::local_player` returns an owned clone of a
host-owned cache and never waits for the game thread. Both APIs return
`UnsupportedVersion` unless the R1 SA-MP and GTA SA 1.0 US fingerprints pass;
they never expose client pointers or use RPC emulation. Snapshot publication
begins only after the server's R1 `INIT_GAME` assignment matches the
local-player ID on two game-thread refreshes, then refreshes from the verified
R1 player pool. It returns `NotReady` rather than publishing a provisional zero
or SA-MP's `0xFFFF` sentinel.

The local-player ID, nickname, colour, spawned, health, armour, special-action,
animation, score, and ping convenience methods are projections of that same
snapshot. They never add a native call or make remote-player data available.

`HostApi::samp_game_state` returns a cached, opaque `i32` copied from R1
`CNetGame` on the same game-thread pump. It is not a snapshot-readiness gate:
the native state may change during normal play. It instead reports `NotReady`
until a state has been published and keeps no client pointer in the ABI.

`HostApi::samp_version` exposes the recognized `samp.dll` build identity that
the host already verified during attach. `HostApi::is_samp_available` reports
host/hook readiness. Both are safe across every recognized build and need no
native-layout access or live gameplay validation.

`rak_samp_plugin_api::raknet` contains pure SF.lua-compatible RPC and packet
name catalogs. These helpers are available without resolving the host and do
not interpret or inspect network payloads.

The same module provides an owned, bounded `BitStream` with checked cursors and
MSB-first RakNet bit order. Stream sends reuse the host's existing exact-bit
packet/RPC ABI. It deliberately has no data-pointer escape hatch, no native
allocation, and no native bitstream lifetime. `HostApi::decode_string` is the
bounded exception to callback-local StringCompressor decoding: it copies an
owned stream through the ABI, returns at most 4,095 owned bytes, and applies
the resulting read cursor only after a successful native decode.

A plugin must keep its `Subscription` values or a `SubscriptionSet` and, before
runtime unload, call `unregister_and_wait` from a worker thread. Batch failures
retain the callbacks that need a retry. Waiting in `DllMain` or a callback is
invalid because callbacks may still be active.

## Typed events

`events::rpc` and `events::packet` add named R1 codecs over the raw API. They
validate full payload consumption, preserve protocol bit layout, bound dynamic
data, and leave uncertain text as bytes. Encoded SA-MP strings use the client's
StringCompressor rather than a Rust reimplementation.

SA-MP 0.3.7 R1 is the typed-layout authority. Other recognized clients may use
raw callbacks but are not typed-layout compatible until live validation is
recorded.

## Native boundary

The Windows backend owns client addresses, detours, vtable changes, native
string-codec calls, and the private R1 client profile. Its incoming-packet
detour is also the game-thread pump: it refreshes the local snapshot and cached
game-state scalar, then drains at most four copied dialog requests after
releasing queue locks, without touching packet or RPC dispatch. It restores
only hooks it owns and keeps captured
backend state valid for in-flight original calls. Native layouts are covered by
the C++ fixture and live evidence in [REVIEW.md](REVIEW.md).

The Windows x86 end-to-end fixture loads a minimal ABI host and an independent
plugin ASI, then verifies discovery, registration, callback delivery, shutdown,
and unload. Run it with `cargo make test-e2e`.

See [ARCHITECTURE.md](ARCHITECTURE.md) for component ownership and
[VALIDATION.md](VALIDATION.md) for the live check.
