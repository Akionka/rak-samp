# Review Record

Findings and resolutions from the latest code review. These are deliberately
kept outside [`TODO.md`](TODO.md) so they retain the reviewer’s context,
priority, and validation evidence.

## Superseded — Default-aligned RakNet packet layout

The review recommendation to use default C alignment passed the `/Zp8` fixture
but failed against the shipped SA-MP 0.3.7 R1 client. During live validation on
2026-08-01, the aligned `length` read returned the plausible packet bit count
`152`, while the aligned `bit_size` read returned `0x1B980DB0`, the pointer at
the packed `data` offset. The metadata guard passed the packet through and
prevented the prior crash.

[`src/platform/win32.rs`](src/platform/win32.rs) now models `RawPacket` and its
embedded `PacketPlayerId` with packed offsets while retaining a distinct,
aligned `RpcPlayerId` for the call boundary that successfully delivered 2,613
RPC callbacks. The C++ oracle now explicitly uses packing instead of assuming
the compiler default. Automated layout validation passes. A second R1 live run
on 2026-08-02 accepted the first packet as 18 bytes/144 bits and delivered
three packet callbacks plus 2,282 RPC callbacks with no null events or metadata
rejections.

## Resolved — Compilation failure in typed events

[`plugin_api/src/events.rs`](plugin_api/src/events.rs) now reads the `string8`
length into a local before calling `read_bytes`, removing the overlapping
mutable borrow that produced E0499. Validated with
`cargo clippy --workspace -- -D warnings` and `cargo make deploy` on 2026-08-01.

## Resolved — Emulated incoming packets dispatched twice

[`src/platform/win32.rs`](src/platform/win32.rs) now queues emulated packets
without pre-dispatch. `incoming_packet_detour` performs their single incoming
dispatch when SA-MP dequeues them, so each listener can rewrite or block once.
The workspace test suite and Clippy passed on 2026-08-01.

## Resolved — Nested emulation deadlocked event dispatch

[`src/event.rs`](src/event.rs) now uses an owner-aware reentrant dispatch gate.
Other threads remain serialized, while nested dispatch on the callback thread
increments the gate depth. The active `FnMut` callback is skipped during its
own nested event and other matching listeners still execute. The
`permits_nested_dispatch_on_the_callback_thread` regression test passed on
2026-08-01.

## Resolved — In-flight detours lost original calls during teardown

Every vtable detour in [`src/platform/win32.rs`](src/platform/win32.rs) now
passes its captured `Arc<BackendState>` to original-call and deallocation
helpers. Those helpers no longer perform a second `ACTIVE_BACKEND` lookup. The
`captured_state_calls_original_after_active_slot_is_cleared` regression test
passed on 2026-08-01.

## Resolved — Packet emulation used the wrong queue owner

The 2026-08-02 R1 loopback run showed packet emulation returning `Ok` while
packet 254 never reached the incoming callback. The backend derived RakPeer by
subtracting `0xDDE` from RakClient, but the working reference queues packets
through the receiver passed to the native incoming-RPC handler. The backend now
uses its captured `rpc_receiver` and returns `NotReady` until it exists. The
`packet_emulation_requires_the_captured_rpc_receiver` regression test prevents
RakClient readiness from being treated as sufficient. The repeat R1 loopback
on 2026-08-02 observed packet 254 and RPC 255 exactly once, reported both
self-tests as `passed`, and retained zero null events and timestamp decode
errors while GTA remained responsive.

## Resolution Rule

When addressing a finding, update this file with the fix and validation, then
remove or mark the resolved entry. Keep [`CORE.md`](CORE.md) and
[`ARCHITECTURE.md`](ARCHITECTURE.md) current when the fix changes behavior or
module interactions.
