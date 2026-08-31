# Keep native string decoding outside frozen network V1

Phase 11 publishes arbitrary plugin-owned `BitStream` compressed-string decoding under exact-version service ID `0x0000_2008`. `SampCodecServiceV1` preserves read-cursor-on-success semantics and keeps native codec state in the Host without extending frozen `SampNetServiceV1`; packet/RPC event-local decoding remains in Network V1.
