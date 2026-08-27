# Terminal alignment padding evidence

## Decision

`TerminalAlignmentPadding` is structural-only. A decoder accepts either no
terminal tail or exactly the number of bits needed to reach the next byte
boundary. The content of that exact-length tail is not constrained.

Do not add `NonZeroTerminalPadding`. Do not require zero-valued terminal bits.

## Scope

This decision applies to the R1 `ID_MARKERS_SYNC` payload (packet 208). It
does not change its decoder, framing errors, or acceptance vectors.

## Evidence

The primary evidence is the SA-MP-compatible legacy sender in open.mp, pinned
at commit [`91a38854f02b05f35bcd3d1162d4fc205d9a4cb9`](https://github.com/openmultiplayer/open.mp/tree/91a38854f02b05f35bcd3d1162d4fc205d9a4cb9).

- [`PlayerMarkersSync::write`](https://github.com/openmultiplayer/open.mp/blob/91a38854f02b05f35bcd3d1162d4fc205d9a4cb9/Shared/NetCode/core.hpp#L1856-L1910)
  writes packet ID 208, the player count, each player ID, a one-bit marker
  flag, and optional coordinates. It writes no terminal padding and makes no
  padding-content assertion. With one inactive marker, the payload after the
  packet ID is exactly 49 meaningful bits: 32 count bits, 16 player-ID bits,
  and one flag bit.
- [`RakNetLegacyNetwork::broadcastPacket`](https://github.com/openmultiplayer/open.mp/blob/91a38854f02b05f35bcd3d1162d4fc205d9a4cb9/Server/Components/LegacyNetwork/legacy_network_impl.hpp#L113-L157)
  explicitly preserves the exact bit length and passes
  `GetNumberOfBitsUsed()` to RakNet. Its input byte capacity is therefore not
  a declaration that the unused bits in its last byte belong to the packet.

This is authoritative evidence for the bit framing emitted by open.mp's
SA-MP-compatible legacy path. It establishes that the message ends after its
meaningful bits. It does not establish a protocol rule for the values of bits
outside that bit length. Therefore a zero-content requirement is unverified.

## Consequence

When a byte-oriented Host exposes the final storage byte, the bits after the
meaningful payload are an alignment boundary only. Validate their structural
length, but accept arbitrary values. The existing MARKERS_SYNC regression that
accepts non-zero exact-length terminal bits remains required behavior.
