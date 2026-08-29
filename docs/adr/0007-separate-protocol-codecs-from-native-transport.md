# Separate Protocol codecs from Native transport

`samp-protocol` owns the SDK-derived owned Protocol bitstream, Wire descriptors,
minimal transport-neutral bit-reader/bit-writer contracts, pure codecs, and
explicitly labelled R1 wire codecs. The Host transport stream remains separate
because its Native capacity, partial-bit alignment, cursor, and error semantics
differ. Raw trait bit chunks are explicitly left-aligned; associated source
errors preserve SDK Host/FFI failures while protocol-owned outer errors preserve
wire failures. Concrete zero-sized nominal Wire descriptor types own built-in
message identity, encoding and decoding, and explicit trailing policy without
exposing private implementation strategies. Their cursor-free Encoded bits have
minimal storage, exact bounded length, and canonical unused bits, while SDK
callback adapters retain callback lifetime and action behavior. The three Native compressed-string
fields use reader/writer extension contracts that preserve limits, exact cursor
advancement, and Host source failures. Extraction proceeds through independently
green foundation, chat/command, common, R1, and encoded-string slices.
`samp-client-sdk` does not re-export the Protocol crate. Plugins import
Protocol values directly and use `Samp` plus its subsystem facades for Host
operations. Separate non-exhaustive Packet and RPC name catalogs preserve
unknown raw IDs, and a Protocol-only Linux CI job proves the platform boundary
without changing Cargo's target directory.

## Consequences

- Built-in messages use nominal semantic Wire descriptor types. Their public
  identity does not expose or depend on private helper functions or internal
  codec types.
- Generic directional descriptors remain a public extension and composition
  facility for custom or ad-hoc messages. They are not the canonical identity
  representation for built-in messages. Each descriptor explicitly selects one
  public sealed behaviorless marker for the finite `TrailingPolicy` semantics;
  there is no implicit default, alias, or codec-derived policy. The markers
  `ExactBitsPolicy`, `ExactBytesPolicy`, and
  `TerminalAlignmentPaddingPolicy` map one-to-one to the matching enum variants,
  whose implementation remains the sole framing model. Directional descriptor
  traits remain sealed: external codecs describe only value encoding and
  decoding, then use generic wrappers to select ID, Packet/RPC kind, direction,
  and framing. One codec may therefore be reused across different descriptor
  identities and policies.
- The public layer above `BitRead` and `BitWrite` contains only stable neutral
  binary primitives required by real codecs. These primitives operate at the
  current bit cursor, define byte order and explicit allocation bounds where
  needed, and do not align implicitly. RakNet compression, Native encoded
  strings, profile-specific layouts, text semantics, and message validation
  remain in specialized layers.
- Typed callbacks fail open at the ABI boundary, but internal diagnostics retain
  the distinction between source reads, malformed Protocol payloads, and
  replacement encoding failures. A replacement is fully encoded before the
  atomic Host operation; failure leaves the original payload untouched and
  returns `Continue`.
- Public `TypedCallbackDescriptor<Direction, Kind>` is a sealed SDK metadata
  projection required for nameable generic bounds. It exposes only `Value` and
  `ID`; Protocol identity and wire/framing semantics remain Protocol-owned, and
  Host adaptation remains in a private supertrait. Direction and kind stay
  type-level, with no runtime adapter switching or external implementations.
  Parameterized private sealing projects Protocol directional traits directly,
  so SDK forwards Protocol `Value` and `ID` without copying built-in metadata or
  retaining legacy SDK descriptor implementations.
- Canonical descriptor encoding returns exact codec bits for `ExactBits`, rejects
  non-byte-aligned `ExactBytes` output, and returns meaningful unpadded bits for
  `TerminalAlignmentPadding`. The latter decoder accepts only the exact number
  of terminal bits needed to align this message's meaningful representation to
  the next byte boundary and never ignores a full trailing byte. Canonical
  no-tail input is always valid. The [terminal-padding evidence record](../evidence/terminal-alignment-padding.md)
  concludes that zero-valued tail bits are not independently verified for R1
  `ID_MARKERS_SYNC`; exact-length content therefore remains unrestricted and
  `NonZeroTerminalPadding` is not introduced. A future rule for zero content
  needs independent wire evidence and must remain separate from structural
  length failures.
- Callback failure diagnostics distinguish `DecodeSource`, `DecodeMalformed`,
  `ReplacementEncode`, and `ReplacementHost` before mapping each failure to ABI
  `Continue`. Replacement encoding and descriptor framing validation complete
  before the single Host replacement operation begins. The Host replacement
  implementation validates every returned-error condition before mutation and
  has no recoverable failure after mutation begins, so a returned failure leaves
  the original payload unchanged.
- Protocol descriptor sends preserve the original Protocol `EncodeError`
  separately from synchronous Host submission failures. For queued sends, a
  Host error describes only failure observed while enqueueing; it does not claim
  to report later asynchronous transport execution.
- Stable production modules describe Protocol semantics. `common` owns only
  profile-neutral wire semantics, `r1` owns explicit R1 semantics, and semantic
  feature modules such as `chat` never encode migration history.
