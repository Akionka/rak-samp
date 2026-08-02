# Review Notes

Keep unresolved findings and durable compatibility evidence here. Resolved
implementation history belongs in Git; pending work belongs in [TODO.md](TODO.md).

## Open findings

None.

## Resolved findings — 2026-08-02

### P1 — Runtime lock held during synchronous RPC emulation

[`src/host_api.rs`](src/host_api.rs) retains the `host().runtime` mutex guard
while `Runtime::emulate_incoming_rpc` synchronously invokes plugin callbacks. A
callback that registers a listener or calls send/emulation tries to lock the
same mutex and deadlocks. Obtain a callback-safe runtime handle and release the
host mutex before dispatch.

Resolved by publishing an `Arc<Runtime>` through `OnceLock` and cloning it
before registration, send, or emulation calls. A regression test exercises a
nested lookup while an outer runtime handle remains alive.

### P1 — Inline hooks enabled before trampoline publication

[`src/platform/win32.rs`](src/platform/win32.rs) enables an `InlineHook` inside
`InlineHook::install`, but callers publish the returned trampoline afterward. A
detour can therefore run with a zero trampoline during constructor or incoming
RPC hook installation. Separate hook creation from enabling: create the hook,
publish its trampoline, then enable it.

Resolved by splitting `InlineHook::create` from `enable`; the constructor and
incoming-RPC paths now publish the trampoline first. The MinHook fixture checks
that creation alone leaves the target disabled.

### P2 — Client-hook installation failures are discarded

The RakClient constructor detour ignores failures from incoming-RPC or vtable
hook installation after the host has reported `Ready`. Because construction is
normally one-shot, plugins can remain attached to a nonfunctional host. Report
the failure and transition the host to `Failed`, or implement a reliable retry.

Resolved with an observable client-hook state. The bootstrap monitor logs the
successful deferred installation or moves the public host state to `Failed`.

### P2 — Packet timestamp option is ignored

Packet sending forwards priority, reliability, and ordering channel but does
nothing with `SendOptions::timestamp`. Either wrap timestamped packets in the
RakNet `ID_TIMESTAMP` envelope or reject that option instead of reporting a
successful ordinary send.

Resolved by explicitly rejecting timestamped packet sends as
`InvalidArgument`; the runtime reports a dedicated internal error.

### P2 — Native bit lengths can overflow `i32`

`NativeBitStream` converts Rust bit lengths to `i32` without validation. Values
above `i32::MAX` wrap negative at the native boundary. Reject them with
`PayloadTooLarge` before constructing the native stream.

Resolved with checked conversions at native packet, RPC-envelope, and
`NativeBitStream` boundaries plus an overflow regression test.

## Authoritative Windows x86 evidence

### RakNet packet layout

`RawPacket` and its embedded `PacketPlayerId` in
[`src/platform/win32.rs`](src/platform/win32.rs) use packed offsets. The
by-value incoming-RPC argument uses the distinct aligned `RpcPlayerId` layout.

This choice supersedes the default-alignment recommendation from the native
fixture. On SA-MP 0.3.7 R1 (2026-08-01), aligned reads produced a plausible
`length` of 152 but read the packed data pointer as `bit_size`. After switching
to the packed layout, an R1 run (2026-08-02) read the first packet as 18
bytes/144 bits and delivered three packet plus 2,282 RPC callbacks without null
events or metadata rejections. The independent C++ fixture now specifies the
same packing explicitly.

### Incoming packet queue owner

Packet emulation must use the RakPeer receiver captured by the native
incoming-RPC detour; RakClient pointer arithmetic is not authoritative. The
2026-08-02 R1 loopback observed locally emulated packet 254 and RPC 255 exactly
once after this correction, with both replacements verified and blocked.

When new live evidence changes a native boundary, record the client build,
observed fields or offsets, fix, and validation result here. Keep
[CORE.md](CORE.md) and [ARCHITECTURE.md](ARCHITECTURE.md) current.

### Explicit send and coordinated shutdown

An SA-MP 0.3.7 R1 release run at commit `7b704d2` on 2026-08-02 reported the
deferred packet/RPC hooks ready, passed both local rewrite-and-block tests, and
returned `Ok` for one captured `ID_STATS_UPDATE` packet send and one
`RPC_UPDATE_SCORES_AND_PINGS` send. The session observed no null events or
timestamp decode errors.

The validation shutdown worker then synchronized all six subscriptions and
returned success. This proves callback detachment and worker coordination; it
does not yet prove that an external manager can safely call `FreeLibrary`.
