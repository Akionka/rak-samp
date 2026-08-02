# Core Features

`rak-rs` is a process-wide SA-MP networking host for Rust ASI plugins. The root
crate produces only `rak_rs.asi`; plugins link the separate
[`rak_rs_plugin_api`](plugin_api/src/lib.rs) crate, so only the host installs
native hooks.

## Host and hooks

`DllMain` starts a worker without waiting under the loader lock. The worker
initializes file logging, waits for `samp.dll`, identifies SA-MP 0.3.7 R1, R2,
R3.1, R4.2, R5.1, or DL, and attaches [`Runtime`](src/runtime.rs). Host state then
becomes `Ready` or `Failed`. [`src/host_api.rs`](src/host_api.rs) publishes the
runtime as an `Arc` through `OnceLock`, so synchronous plugin callbacks never
run while a host runtime mutex is held. A monitor reports whether the deferred
RakClient hooks become ready or fail during construction.

The Windows x86 backend in [`src/platform/win32.rs`](src/platform/win32.rs) owns
the constructor and incoming-RPC detours and patches RakClient vtable slots 6,
8, and 25 in place. Teardown restores only slots still owned by the host and
keeps captured backend state alive for in-flight original calls. Each inline
hook is created disabled, its original trampoline is published, and only then
is its detour enabled.

[`src/logging.rs`](src/logging.rs) writes lifecycle and ABI diagnostics to
`rak-rs.log` through `simplelog`, with RFC 3339 UTC timestamps. A new host or
validation ASI session truncates its previous log. Payloads are never logged,
and logger failure does not stop the host.

## Traffic and dispatch

Plugins can subscribe to incoming or outgoing packets and RPCs, inspect their
bounded [`BitStream`](src/bitstream.rs) payloads, replace exact-bit payloads,
block traffic, send raw traffic, or emulate incoming traffic. Callback-local
events expire when the callback returns. `BitStream::replace_bits` retains a
partial final byte, resets the read cursor, and enforces the native stream's
original capacity.

Explicit sends call the captured original RakClient methods and intentionally
bypass outgoing listeners to avoid recursion. Native bit lengths above
`i32::MAX` and timestamped packet requests are rejected instead of being
truncated or silently downgraded.

[`src/event.rs`](src/event.rs) dispatches listeners in registration order. It
serializes different threads, permits nested dispatch on the current thread,
and skips only the currently executing `FnMut` listener during re-entry.
Unmatched native traffic passes through unchanged. A panicking host-side Rust
listener is removed; plugins must never unwind across the C ABI.

Emulated packets enter the native receive queue and cross incoming listeners
once when dequeued. Emulated RPCs dispatch synchronously before the native
receiver. Either may be rewritten or blocked. Incoming packet metadata is
validated against its byte length, bit length, and a 16 MiB bound before native
memory is read; invalid metadata is logged once and passed through untouched.

## Plugin ABI and typed events

The append-only `RakRsApiV1` table uses C layouts and `extern "system"` function
pointers; no Rust references, trait objects, or allocations cross DLLs.
[`HostApi::unregister_and_wait`](plugin_api/src/lib.rs) removes a subscription
and waits for callbacks on other threads. It returns `CallbackInProgress` on
the active callback thread, so unload coordination must run on a worker before
`FreeLibrary`. Safe [`HostApi::send_packet`](plugin_api/src/lib.rs) and
[`HostApi::send_rpc`](plugin_api/src/lib.rs) wrappers keep ordinary plugin code
away from raw function pointers.

[`plugin_api/src/events/`](plugin_api/src/events/) decodes common incoming
and outgoing RPCs over the same raw subscription. The public RPC catalogs are
`events::rpc::incoming` and `events::rpc::outgoing`; packet directions remain
under `events::packet`. `RpcAction` can continue,
block, or atomically replace a complete exact-bit payload. The fixed-layout
incoming set includes checkpoints, audio streams, object updates, death and map
UI notifications, vehicle interiors, and player colors. Text stays as
`Vec<u8>`, `string32` reads are capped at 4096 bytes, and `onShowDialog`
supports SA-MP's compressed dialog text. `onSetPlayerSkin` exposes RPC 153 as
two byte-aligned signed IDs (`player_id`, then `skin_id`). `Rpc::encode` returns
bytes plus the meaningful bit length, which can be passed directly to emulation
or send APIs.

The R1 inbound catalog is complete. Its variable layouts cover game
initialization, class/spawn state, streamed players (including eleven weapon-skill levels) and vehicles, 3D labels,
objects with order-preserving texture/text materials, menus, camera and
textdraw state, animations, attached objects, and score/ping collections.
One-bit flags, fixed 32-byte strings, and encoded text retain their wire
semantics. Every typed descriptor requires complete consumption; malformed or
unsupported payloads return an error for the callback to fail open.

`events::packet` uses the same callback ABI for fixed-size packet layouts. Its
outgoing helpers cover authentication, RCON, stats, weapons, and player,
vehicle, passenger, aim, unoccupied, trailer, bullet, and spectator
synchronization. Incoming helpers decode authentication, connection lifecycle,
and remote-player aim, bullet, unoccupied, trailer, passenger, player, vehicle,
and marker sync. Remote player/vehicle sync uses compressed vectors, normalized
quaternions, packed health/armour, and optional fields; marker coordinates are
signed R1 `i16` values. Decoders check full bit consumption before replacement;
packed key, weapon, seat, and camera state bytes stay opaque protocol values
rather than Rust bitfields.

Encoded strings deliberately use SA-MP's implementation rather than a second
Huffman codec. [`src/client.rs`](src/client.rs) maps each supported build's
StringCompressor object and reader/writer functions. `HostApi::encode_string`
and `Event::read_encoded_string` pass caller-owned buffers through
[`src/host_api.rs`](src/host_api.rs) and [`Runtime`](src/runtime.rs) to x86
`thiscall` wrappers in [`src/platform/win32.rs`](src/platform/win32.rs). Only
bytes, capacities, results, and exact bit counts cross the plugin ABI; native
addresses and pointers stay inside the host.

The [sample plugin](examples/sample_plugin) demonstrates discovery, typed
dispatch, and synchronized shutdown.

The [chat-command example](examples/chat_command_plugin) subscribes once to
outgoing RPCs. It decodes RPC 50 with `rpc::outgoing::on_send_command`, blocks the
local `/rakrs` command, serializes and sends a real RPC 101 through
`rpc::outgoing::SEND_CHAT`, then serializes `rpc::incoming::SHOW_DIALOG` and emulates RPC
61 locally. It also blocks RPC 62 for its reserved dialog ID so closing the fake
dialog does not notify the server. The explicit chat message is intentionally
server-bound.

The [validation plugin](examples/validation_plugin) keeps server-bound sends and
coordinated shutdown behind marker files. Its local tests also encode, decode,
rewrite, verify, and block an `onShowDialog` payload without showing a dialog.
It resolves logs and marker files relative to its own ASI instead of relying on
the process working directory, and waits for the native string compressor to
become ready before running that test.
It can replay one observed stats packet, send one scoreboard RPC, then stop its
workers and synchronize all six subscriptions without doing work under
`DllMain`. The separate
[validation unload manager](examples/validation_unloader) waits for those tests,
calls the shutdown export, and only then releases the plugin's module reference.

## Native validation

Incoming packets use the packed x86 RakNet `Packet` layout verified by the
independent C++ fixture in
[`tests/fixtures/raknet_layout.cpp`](tests/fixtures/raknet_layout.cpp) and an R1
live run. The by-value incoming-RPC player argument remains a distinct aligned
layout. Fake-vtable and MinHook tests cover slot-local patching, restoration,
original calls, removal, and recreation. Durable live-client evidence is kept
in [REVIEW.md](REVIEW.md). An R1 release run also validates explicit packet/RPC
sends, synchronized shutdown of all validation callbacks, and external unload
of the validation ASI while the host remains active.

## Limits

The typed R1 catalog is complete. Broader game-state APIs and live verification
on every detected client build remain pending; native encoded-string helpers and
the new compressed layouts still need an R1 in-game validation run.
