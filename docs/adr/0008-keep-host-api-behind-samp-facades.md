# Keep HostApi behind Samp facades

`samp-client-sdk` exposes Host resolution and supported operations through the
root `Samp` facade and its subsystem facades. `HostApi`, its resolver and
construction paths, and the narrow Host encoded-string adapter remain internal
because exposing them would couple plugins to ABI wrapper details. All Wire
descriptors, Protocol encoding, and exact-bit payload values belong to
`samp-protocol`, while manual transport belongs to explicit raw APIs.

## Verification

An external compile-pass consumer exercises normal facade access. A minimal
`compile_fail` import is only a supplementary guard: source/API audits must also
prove that `HostApi` is private, is not re-exported or aliased, does not occur in
public signatures, and is absent from generated public API documentation.
