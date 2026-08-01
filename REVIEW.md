# Review Notes

Keep unresolved findings and durable compatibility evidence here. Resolved
implementation history belongs in Git; pending work belongs in [TODO.md](TODO.md).

## Open findings

None.

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
