# Handoff: zero-copy bit reading in `samp-protocol`

## Purpose

Review and, if accepted, implement a `samp-protocol` bit-reading API that can decode borrowed network buffers without copying the complete payload and without allocating for every primitive read.

This arose while implementing the Whetstone Rust transport prototype. The prototype should reuse `samp-protocol` bit primitives instead of maintaining separate readers and writers, but the current API makes that substitution unnecessarily expensive on the UDP receive hot path.

## Existing artifacts

Do not reproduce the transport prototype or its replay specification.

- Consumer implementation: Whetstone commit `936b1e3` (`feat(transport): add 4057 replay prototype`).
- Consumer bit reader: `prototypes/rust-transport/src/bitstream.rs` at that commit.
- Consumer bit writer: `prototypes/rust-transport/src/bitwriter.rs` at that commit.
- Consumer call site: `prototypes/rust-transport/src/reliability.rs` at that commit.
- Originating validation ticket: <https://github.com/Akionka/whetstone/issues/14>.
- `samp-protocol` baseline reviewed here: commit `92dc6b07b3a67996588c88de89c33be301f73cf4`.
- Relevant upstream files: `crates/samp-protocol/src/bitstream.rs`, `wire_io.rs`, and `lib.rs`.

## Current API constraints

`BitStream` owns `Vec<u8>`. Both constructors consume owned storage:

```rust
BitStream::from_bytes(bytes: impl Into<Vec<u8>>)
BitStream::from_bits(bytes: impl Into<Vec<u8>>, bit_len: usize)
```

A UDP receiver starts with `&[u8]`. Constructing `BitStream` therefore copies the complete datagram before decoding. The current Whetstone reader borrows that slice and allocates only when it extracts the final owned application payload.

There is a second allocation boundary in the abstraction:

```rust
pub trait BitRead {
    type Error;
    fn remaining_bits(&self) -> usize;
    fn read_left_aligned_bits(&mut self, bit_len: usize)
        -> Result<Vec<u8>, Self::Error>;
}
```

`WireReadExt` builds primitive reads on this method. Consequently even fixed-width values pass through a newly allocated `Vec<u8>`. Adding only a borrowed `BitStream` constructor would remove the full-datagram copy but would not make primitive decoding allocation-free.

The writer does not have the same ownership problem. `samp_protocol::BitStream::new()` is a suitable owned writer, and `BitWrite` already accepts borrowed input buffers.

## Requested design decision

Prefer a boring split between borrowed reading and owned writing:

- Keep `BitStream` as the owned, growable writer and cursor-compatible owned stream.
- Add a read-only borrowed cursor, tentatively `BitSlice<'a>` or `BitReader<'a>`, constructed from `&'a [u8]` plus an exact meaningful bit length.
- Make fixed-width reads write into caller-provided storage rather than return a fresh `Vec`.
- Keep an allocating convenience method only for callers that need an owned variable-length result.
- Preserve the existing MSB-first bit order, little-endian numeric representation, exact bit-length validation, and left-aligned versus SDK-compatible right-aligned partial-byte distinction.

Avoid making the principal type generic over `Vec`, slice, `Cow`, or a storage trait unless a concrete use case requires mutable borrowed storage. A dedicated borrowed reader has a smaller API and prevents invalid write operations by construction.

## Candidate API shape

Names are provisional; compatibility and ergonomics should be reviewed against existing codecs.

```rust
pub trait BitRead {
    type Error;

    fn remaining_bits(&self) -> usize;

    fn read_left_aligned_bits_into(
        &mut self,
        output: &mut [u8],
        bit_len: usize,
    ) -> Result<(), Self::Error>;

    fn read_left_aligned_bits(
        &mut self,
        bit_len: usize,
    ) -> Result<Vec<u8>, Self::Error> {
        // checked allocation followed by `read_left_aligned_bits_into`
    }
}

pub struct BitSlice<'a> {
    bytes: &'a [u8],
    bit_len: usize,
    read_offset: usize,
}

impl<'a> BitSlice<'a> {
    pub fn from_bits(bytes: &'a [u8], bit_len: usize)
        -> Result<Self, BitStreamError>;
}
```

`WireReadExt::read_fixed` should use a stack `[u8; N]` and `read_left_aligned_bits_into`. Variable-length byte/string reads may still allocate exactly once for their result.

An alternative is to make the non-allocating operation the only required trait method and retain the current allocating method as a default helper. This preserves most codec call sites while letting both `BitStream` and `BitSlice` implement efficient reads.

## Consumer migration boundary

After the upstream API exists, Whetstone can:

1. Replace its local `BitReader<'a>` with the borrowed upstream reader.
2. Replace its local `BitWriter` with owned `samp_protocol::BitStream`.
3. Keep RakNet transport-specific compressed `u16`/`u32` and alignment helpers in `reliability.rs`; these are transport framing, not SA-MP payload semantics.
4. Map `BitStreamError`/wire errors to `TransportError` at the transport boundary.
5. Delete `prototypes/rust-transport/src/bitstream.rs` and `bitwriter.rs`.

Raw application payload extraction must remain left-aligned and preserve exact `bit_len`. Partial transport fields currently represented as low-order values must continue using the right-aligned partial-bit contract or an explicit scalar helper.

## Acceptance criteria

- Constructing a reader over an incoming `&[u8]` does not copy or retain owned storage.
- Reading bools and fixed-width integers performs no heap allocation.
- Reading an owned variable-length payload performs at most the allocation needed for that result.
- Invalid `bit_len`, cursor overflow, and output-size mismatch are checked errors; no panic and no unsafe code.
- Existing `samp-protocol` codec vectors remain byte-for-byte and bit-for-bit unchanged.
- New tests cover non-byte-aligned input, zero bits, terminal partial bytes, cursor exhaustion, exact last-bit reads, and unchanged left/right alignment semantics.
- The Whetstone reliability replay tests pass after deleting both local bitstream implementations.
- Benchmark or allocation-count evidence demonstrates removal of the full-datagram copy and per-primitive allocations; do not add a benchmark framework if the repository already has a simpler allocation-testing convention.

## Risks to review

- Adding a required trait method is a breaking API change. A default compatibility implementation may reduce migration cost, but it must not make the allocation-free path optional for in-tree implementations.
- `BitStreamError` currently describes owned-stream operations. Confirm whether the name remains appropriate for `BitSlice` errors or whether a neutral `BitError` rename is justified; avoid churn without concrete benefit.
- `read_left_aligned_bits_into` must define output padding precisely. The final partial byte should remain left-aligned and unused low bits should be zeroed.
- Do not expose unchecked cursor mutation to avoid bounds checks. Correctness is more important than micro-optimization; optimize repeated per-bit dispatch only after preserving wire vectors.

## Suggested skills

- `tdd`: implement the API and consumer migration test-first, preserving exact wire contracts.
- `code-review`: review the resulting `samp-protocol` change against repository standards and the zero-copy/no-per-primitive-allocation requirements.

## Recommended next steps

1. Review this API boundary with the `samp-protocol` maintainer.
2. Add failing borrowed-reader and allocation-behavior tests in `samp-protocol`.
3. Implement the minimal non-allocating `BitRead` primitive and borrowed reader.
4. Migrate existing `WireReadExt` fixed-width reads to stack buffers.
5. Migrate the Whetstone prototype and remove its duplicate bitstream files.
6. Run focused codec/replay tests, then each repository's full suite once.
