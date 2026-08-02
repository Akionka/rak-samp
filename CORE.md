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
`rak-rs.log`. Payloads are never logged, and logger failure does not stop the
host.

## Traffic and dispatch

Plugins can subscribe to incoming or outgoing packets and RPCs, inspect their
bounded [`BitStream`](src/bitstream.rs) payloads, replace byte-aligned payloads,
block traffic, send raw traffic, or emulate incoming traffic. Callback-local
events expire when the callback returns.

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

[`plugin_api/src/events.rs`](plugin_api/src/events.rs) decodes common incoming
and outgoing RPCs over the same raw subscription. `RpcAction` can continue,
block, or atomically replace a complete byte-aligned payload. Text stays as
`Vec<u8>`, and `string32` reads are capped at 4096 bytes. The
[sample plugin](examples/sample_plugin) demonstrates discovery, typed dispatch,
and synchronized shutdown.

The [validation plugin](examples/validation_plugin) keeps server-bound sends and
coordinated shutdown behind marker files. It can replay one observed stats
packet, send one scoreboard RPC, then stop its workers and synchronize all six
subscriptions without doing work under `DllMain`. The separate
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

RakNet Huffman strings, bit-length-preserving replacement, `onShowDialog`,
remaining complex or bit-packed event schemas, broader game-state APIs, and
live verification of every supported build are not yet implemented.
