# Core Features

## Purpose

`rak-rs` is a process-wide SA-MP hook host for Rust plugins. The root crate emits only a 32-bit Windows `cdylib`; install its result as `rak_rs.asi` and let GTA's ASI loader load it alongside independently compiled plugin ASIs. Plugins must depend on the separate `rak_rs_plugin_api` crate in `plugin_api/`, not the host crate. Only `rak_rs.asi` installs SA-MP hooks.

## Host Lifecycle

The exported `DllMain` starts a background bootstrap without waiting under the Windows loader lock. The bootstrap waits until `samp.dll` is present, then calls [`Runtime::attach`](src/runtime.rs) before RakClient construction. On success the host reports `Ready`; unsupported clients or failed hooks set `Failed`.

`Runtime` identifies SA-MP 0.3.7 R1, R2, R3.1, R4.2, R5.1, and DL through [`SampVersion` and `AddressSet`](src/client.rs). It owns the only constructor detour, inbound-RPC detour, and in-place RakClient vtable patches for slots 6, 8, and 25. Dropping it restores only patches that it still owns.

The Windows x86 tests in [`src/platform/win32.rs`](src/platform/win32.rs) compare the packed RakNet packet layout against the independently compiled C++ declarations in [`tests/fixtures/raknet_layout.cpp`](tests/fixtures/raknet_layout.cpp), then read a packet initialized by C++. The packed offsets are backed by an R1 live diagnostic in which aligned reads observed the packet's bit count at the presumed length offset and its data pointer at the presumed bit-size offset. Tests also use a 55-slot fake RakClient vtable to prove that only the three owned slots change and that teardown preserves a later hook. A separate MinHook fixture validates create, trampoline call, enable, disable, removal, and recreation against a test-only function.

## Logging

[`src/logging.rs`](src/logging.rs) configures `simplelog::WriteLogger` once from the bootstrap worker, after `DllMain` returns. It appends `Debug`-and-higher host lifecycle and ABI-registration messages to `rak-rs.log` in the GTA process working directory. It deliberately does not log packet or RPC payloads. If the log file or global logger cannot be initialized, the reason is written to standard error and the host continues without logging.

## Plugin ABI

Plugin ASIs depend on [`rak_rs_plugin_api`](plugin_api/src/lib.rs), never on the root `rak_rs` crate. From a worker thread, a plugin calls `wait_for_default_host`, which waits for the already-loaded `rak_rs.asi`, resolves `RakRs_GetApiV1`, and waits for `Ready`.

The versioned `RakRsApiV1` table registers packet or RPC callbacks, removes subscriptions, reads and writes an opaque event's payload, atomically replaces byte-aligned callback payloads, sends raw traffic, and locally emulates incoming traffic. [`HostApi::emulate_incoming_packet`](plugin_api/src/lib.rs) queues a packet for the native receive path through the RakPeer pointer captured by the first native incoming RPC; it returns `NotReady` until that pointer is known. [`HostApi::emulate_incoming_rpc`](plugin_api/src/lib.rs) synchronously enters incoming RPC dispatch. Their payload slices exclude the event ID. A listener may rewrite or block an emulated event, and a block is considered successful emulation. Callbacks receive an opaque `RakRsEventV1` valid only during that callback and return `Continue` or `Block`. ABI values use `#[repr(C)]` and `extern "system"`; no Rust trait objects, references, or heap allocations cross the DLL boundary.

For runtime unload, [`HostApi::unregister_and_wait`](plugin_api/src/lib.rs) removes a subscription and waits until callbacks already executing on other threads finish. It returns `CallbackInProgress` when called by the active callback thread, avoiding a self-deadlock. An unload manager must run this barrier from a plugin shutdown worker before calling `FreeLibrary`; process termination needs no explicit shutdown. [`examples/sample_plugin`](examples/sample_plugin) demonstrates discovery, registration, typed dispatch, and an exported shutdown handshake that also quiesces its discovery worker without linking the host runtime.

[`examples/validation_plugin`](examples/validation_plugin) is an in-game diagnostic consumer. Four observers update fixed atomic incoming/outgoing packet/RPC ID histograms; for packet ID 40 (`ID_TIMESTAMP`), they also read the timestamp and logical inner ID, then restore the event cursor. Two earlier listeners recognize only private 16-byte self-test markers. A worker emulates packet 254 and RPC 255 locally; each first listener atomically rewrites its marker, and the observer verifies the replacement and returns `Block` before the synthetic event reaches SA-MP. A reporter writes aggregate counts, named nonzero histograms, and self-test status—not payloads—to `rak-rs-validation.log` every five seconds. Its shutdown export joins all workers and synchronizes all six subscriptions before allowing runtime unload.

## Typed Event Helpers

[`plugin_api/src/events.rs`](plugin_api/src/events.rs) adds a Rust-native layer modeled after MoonLoader's `samp.events`. It does **not** register hooks or callbacks itself: a plugin still registers one raw RPC callback with `RakRsApiV1`, then calls a named helper such as `events::incoming::on_server_message` or `events::outgoing::on_send_chat` from that callback. This keeps `rak_rs.asi` as the only hook owner and avoids retaining plugin closure state across the DLL boundary.

Each [`events::Rpc`](plugin_api/src/events.rs) descriptor matches one RPC ID, resets the callback-local read cursor, decodes a concrete Rust value, and maps `RpcAction::Continue`, `Block`, or `Replace(value)` to the raw ABI. `Replace` serializes the complete byte-aligned payload locally, then atomically swaps it into the host `BitStream`; an invalid or oversized replacement leaves the original traffic intact. Named helpers now cover common messages and game text; player join/quit, position, health, armour, team, time, money, weapons, checkpoints, and vehicle state; and outgoing chat, commands, dialog responses, class/spawn requests, menu/textdraw clicks, vehicle events, deaths, and map markers. Text is `Vec<u8>` to preserve SA-MP's non-UTF-8 payloads, and `string32` reads are limited to 4096 bytes.

## Event Processing

The host stores every plugin subscription in [`src/host_api.rs`](src/host_api.rs). When an SA-MP hook fires, [`src/event.rs`](src/event.rs) invokes callbacks in registration order over a bounded [`BitStream`](src/bitstream.rs). Its owner-aware dispatch gate serializes separate network threads, permits same-thread nested dispatch, and acts as the unload synchronization barrier. The currently executing `FnMut` listener is skipped for its nested event while other matching listeners still run. Incoming packets use the packed x86 RakNet `Packet` and embedded `PlayerID` layout verified in [`src/platform/win32.rs`](src/platform/win32.rs); the independently working by-value incoming-RPC player argument remains a separate aligned `RpcPlayerId`. Incoming RPC lengths use RakNet-compatible compressed `u32` encoding, including its 4-bit low-value form. Emulated packets are queued without pre-dispatch and pass through incoming listeners exactly once when the receive detour dequeues them. If no matching listener exists, native traffic passes through unchanged. Before a listener reads incoming data, the backend checks that the bit count fits the declared byte length and a 16 MiB safety bound. Impossible metadata is logged once and passed through unchanged instead of dereferencing its data pointer. A callback can inspect, replace, or block its packet/RPC. The core registry removes a Rust callback that panics; plugins must not unwind across the C ABI.

## Current Limits

The ABI remains the foundation for packet work and for event types that are not yet modeled. RakNet Huffman encoded strings, bit-length-preserving atomic replacement, `onShowDialog`, the remaining complex/bit-packed MoonLoader-style definitions, chat/command state APIs, and live verification against every supported SA-MP build remain future work.
