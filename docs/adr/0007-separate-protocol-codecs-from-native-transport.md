# Separate Protocol codecs from Native transport

`samp-protocol` owns the SDK-derived owned Protocol bitstream, Wire descriptors,
minimal transport-neutral bit-reader/bit-writer contracts, pure codecs, and
explicitly labelled R1 wire codecs. The Host transport stream remains separate
because its Native capacity, partial-bit alignment, cursor, and error semantics
differ. Raw trait bit chunks are explicitly left-aligned; associated source
errors preserve SDK Host/FFI failures while protocol-owned outer errors preserve
wire failures. Concrete zero-sized Wire descriptor types own message identity,
generic codecs, and explicit trailing policy without runtime trait objects or
transport-specific duplication. Their cursor-free Encoded bits have minimal
storage, exact bounded length, and canonical unused bits, while SDK callback
descriptors retain callback lifetime and action behavior. The three Native compressed-string
fields use reader/writer extension contracts that preserve limits, exact cursor
advancement, and Host source failures. Extraction proceeds through independently
green foundation, chat/command, common, R1, and encoded-string slices. The
legacy `samp-client-sdk` does not re-export the Protocol crate; the future
`samp` facade decides its own policy in Phase 7. Separate non-exhaustive Packet
and RPC name catalogs preserve unknown raw IDs, and a Protocol-only Linux CI
job proves the platform boundary without changing Cargo's target directory.
