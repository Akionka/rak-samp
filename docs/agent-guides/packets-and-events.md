# Packets and typed events

- Invoke listeners in registration order. Preserve continue, block, and
  exact-bit atomic replacement outcomes.
- Send each emulated packet through incoming listeners exactly once.
- Keep nested same-thread dispatch non-blocking.
- Explicit sends bypass outgoing listeners.
- Keep callback-local replacement synchronous and serialize it before the
  atomic ABI replacement call.
- Typed decode and replacement-encode failures return `Continue` without
  invoking a malformed typed value. Diagnose source reads, malformed Protocol
  payloads, and replacement encoding separately without logging payload data.
  If replacement encoding fails, leave the original payload untouched.
- Classify callback failures as `DecodeSource`, `DecodeMalformed`,
  `ReplacementEncode`, or `ReplacementHost` before logging metadata and mapping
  them to `Continue`. Fully encode and validate replacement framing before one
  Host replacement call; never mutate Host payload incrementally during encode.
- Keep Host replacement failure-atomic: validate pointers, bit lengths, offsets,
  and capacity before mutation, with no recoverable error after mutation starts.
  Test an injected Host replacement rejection and assert that the original
  payload remains unchanged; cover incoming Packet as well as RPC paths.
- Typed helpers must reuse the single host subscription.
- Keep public `TypedCallbackDescriptor<Direction, Kind>` sealed and limited to
  `Value` and `ID`. It is a metadata projection for public generic bounds, not a
  second owner of Protocol identity or callback behavior. Keep decode, encode,
  `HostApi`, callback events, and ABI actions in its private adapter supertrait.
- Parameterize private callback sealing by the same direction and kind markers.
  Forward Protocol `Value` and `ID` through one blanket implementation per
  Protocol directional trait; do not duplicate built-in IDs in the SDK. Give
  only remaining legacy SDK descriptors explicit metadata implementations.
- Preserve direction and kind through type-level `Incoming`, `Outgoing`,
  `PacketKind`, and `RpcKind` bounds. Do not use runtime adapter switching, and
  do not let custom-message authors implement the SDK metadata trait directly.
- Give every typed Packet/RPC descriptor one direction. Use separate descriptor
  types when one ID has different incoming and outgoing layouts. Keep raw-ID
  subscriptions as the explicit runtime-direction escape hatch.
- Give every built-in Packet/RPC a nominal semantic descriptor whose public
  identity does not expose its codec implementation. Reserve generic directional
  descriptors for custom or ad-hoc message composition.
- Require every generic directional descriptor to name one sealed trailing-policy
  marker explicitly. Do not default framing policy, and allow one codec to be
  reused by descriptors with different policies.
- Keep `TrailingPolicyMarker` behaviorless and externally sealed. Its only
  implementations are `ExactBitsPolicy`, `ExactBytesPolicy`, and
  `TerminalAlignmentPaddingPolicy`, each mapping directly to the corresponding
  `TrailingPolicy` variant. Keep acceptance logic and canonical documentation on
  `TrailingPolicy`; do not add permissive aliases, defaults, or codec inference.
- Apply descriptor framing validation in canonical `encode_bits`, not low-level
  `encode_to`. Preserve exact bits for `ExactBits`, reject non-byte-aligned
  `ExactBytes` output with `EncodeError::NonByteAlignedPayload`, and keep
  `TerminalAlignmentPadding` canonical output unpadded. Its decoder accepts only
  the exact number of terminal bits needed to reach the next byte boundary and
  rejects a wrong-length suffix or a full trailing byte.
- For terminal padding, derive meaningful length from this descriptor payload and
  compute `required_bits = (8 - (meaningful_bit_len % 8)) % 8`. Always accept no
  tail. If a tail exists, require exactly `required_bits`; report a wrong length
  as `InvalidTerminalPaddingLength`. Require zero content and add
  `NonZeroTerminalPadding` only after an independent wire fixture, captured
  traffic, or other authoritative observation verifies that semantic. Otherwise
  preserve acceptance of arbitrary values in the exact padding region. Preserve
  a reader failure while consuming padding as `DecodeError::Source`, and never
  include padding contents in an error.
- Test terminal policy generically for aligned canonical input, rejected extra
  byte, unaligned canonical input, exact required padding, one-bit-short and
  one-bit-long padding, and source failure while consuming padding. Preserve
  established padding-content acceptance vectors until independent evidence
  justifies a stricter zero-content test. Keep `UnexpectedTrailingBits` limited
  to `ExactBits` and `ExactBytes`.
- Keep Protocol directional descriptor traits sealed. Direction belongs to the
  descriptor type, never the codec. External custom messages implement
  `WireCodec` and select a generic directional wrapper; do not add a second
  external direction escape hatch without a demonstrated need.
- Keep public Wire read/write extensions limited to stable neutral primitives.
  Keep RakNet compression, Native encoded strings, profile-specific encodings,
  and message semantics in specialized layers.
- Add neutral Wire primitives only when migrated codecs require them. Include
  direct scalar operations such as `read_u8` and `write_u8` when used; do not add
  speculative operations for symmetry.
- Name length-prefixed raw-byte helpers by prefix representation, such as
  `read_len_prefixed_bytes_u16_le(max_len)`. Treat the prefix as a byte count,
  reject it before allocation when it exceeds `max_len`, verify readable payload
  length where practical, and return bytes without text interpretation. Writers
  must reject limits or unrepresentable lengths without truncation, saturation,
  or wrapping.
- Preserve source failures separately from Protocol length and encoding
  validation errors in neutral Wire helpers. A limit violation is not a source
  error.
- Operate every neutral primitive from the current bit cursor and consume its
  exact wire bit length. An endian suffix describes byte significance, not an
  alignment requirement; never align implicitly unless the helper contract says
  so. Test scalar and length-prefixed helpers at non-byte-aligned offsets.
- Define neutral `Vector2` and `Vector3` helpers only as composition over LE
  `f32` fields. Keep normalization, compression, coordinate rules, and
  profile-specific vector behavior outside the neutral layer.
- Keep text with uncertain encoding as bytes.
- Bound length-prefixed allocations.
- Preserve Protocol `EncodeError` on Protocol descriptor sends. Distinguish it
  from synchronous Host submission failure; queued submission does not report a
  later asynchronous transport failure.
- Use `common` only for profile-neutral wire semantics and `r1` only for explicit
  R1 semantics. Do not use migration phases, batches, or status as production
  module taxonomy, and do not hide ownership with broad re-exports.
- Keep Host-backed descriptor construction, encoding, payload writers, and
  encoded payload representations internal when they have no independent public
  use. A legacy descriptor may remain public for typed registration without
  exposing its Host encoding machinery.
- Never retain callback-local events.
- Keep SDK protocol codecs and bit streams independent of the native host so
  they remain testable without a live client.
- Route `Local` protocol actions through the same typed `Net` operations; do
  not imply that a server-bound send also performs a local GTA state change.
