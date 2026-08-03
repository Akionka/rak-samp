# Review Evidence

Keep only unresolved findings and evidence that defines a native boundary here.
Implementation history belongs in Git; planned work belongs in [TODO.md](TODO.md).

## Open findings

None.

## Windows x86 evidence

- **RakNet packet layout:** `RawPacket` and its embedded `PacketPlayerId` use
  packed offsets. The by-value incoming-RPC `RpcPlayerId` is a distinct aligned
  layout. The independent C++ fixture and an SA-MP 0.3.7 R1 live run support
  this split; do not infer native packing from serialized sizes.
- **Incoming queue:** packet emulation uses the RakPeer receiver captured by
  the incoming-RPC detour, not RakClient pointer arithmetic. R1 loopback
  observed emulated packet and RPC events once each through normal dispatch.
- **Encoded strings:** the host calls the detected client's StringCompressor
  reader and writer as x86 `thiscall` functions. An R1 live test encoded,
  decoded, replaced, and blocked a private dialog without instability.
- **Hook and unload checks:** fixture tests cover owned-slot restoration and
  original calls. An R1 validation run completed callback quiescence before an
  external manager released the validation ASI.

## Typed protocol evidence

R1 RPC and packet codecs are field-by-field serializers; they do not cast
callback memory to Rust structs. Their catalog and fixture coverage were
checked against the public [SAMP.Lua event catalog](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/events.lua),
[synchronization definitions](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/synchronization.lua),
and [SA-MP RPC list](https://github.com/Brunoo16/samp-packet-list/wiki/RPC-List).
This is not live compatibility evidence for non-R1 clients.

When native behavior changes, record the client build, the observed layout or
behavior, the supporting fixture/live result, and any remaining limitation.
