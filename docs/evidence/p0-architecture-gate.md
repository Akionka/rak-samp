# P0 Protocol / SDK architecture gate

**Status: GREEN.** Issue #36 verifies the P0 architecture defined by #22. The
blocking work from #34 and #35 is present in the acceptance baseline.

## Acceptance seams

| Seam | Evidence |
| --- | --- |
| Public Protocol descriptor API | [`wire_io.rs`](../../crates/samp-protocol/tests/wire_io.rs) covers neutral, bounded, unaligned Wire primitives and distinct source errors. [`wire.rs`](../../crates/samp-protocol/tests/wire.rs) covers nominal built-ins, custom wrappers, explicit policies, canonical encoding, and structured framing errors. Packet and RPC integration tests preserve exact vectors. [`terminal-alignment-padding.md`](terminal-alignment-padding.md) fixes the structural-only padding decision. |
| Public SDK facade over mock ABI | [`sdk/src/tests.rs`](../../sdk/src/tests.rs) proves unified Protocol and Host-backed registration, one Host subscription, action parity, failure diagnostics, exact-bit replacement, and replacement atomicity. The key tests are `normal_typed_methods_accept_all_descriptor_sources`, `malformed_typed_packet_is_diagnosed_before_fail_open`, `typed_source_failure_is_warned_before_fail_open`, `replacement_encode_failure_preserves_payload_without_host_mutation`, `host_rejection_preserves_incoming_rpc_and_packet_payloads`, and `successful_non_byte_aligned_replacement_uses_one_host_call`. |
| External packaged and documented consumer | The crate-level examples in [`sdk/src/lib.rs`](../../sdk/src/lib.rs) compile as an external consumer through `Samp` and `Net` for one Protocol descriptor and one Host-backed descriptor. A minimal `compile_fail` example proves that `HostApi` is not nameable. Both published crates package with the synchronized Protocol version. |

## Failure boundaries

Typed callbacks follow `classify -> diagnose -> Continue`. The private adapter
keeps `DecodeSource`, `DecodeMalformed`, `ReplacementEncode`, and
`ReplacementHost` distinct before [`callback.rs`](../../sdk/src/events/callback.rs)
emits metadata-only diagnostics. Tests prove that malformed values do not reach
typed handlers and that replacement failures do not mutate the original bits.

Protocol sends return `ProtocolSendError::Encode(original)` or
`ProtocolSendError::Host(result)`. Immediate and queued tests preserve the
structured encode error, Host error, ID, bytes, meaningful bit length, and send
options. [`Runtime::send_packet_with_options` and
`Runtime::send_rpc_with_options`](../../src/runtime.rs) submit directly to the
backend, so explicit sends do not dispatch outgoing listeners.

## Public surface and dependency audits

- `HostApi` and its resolvers are crate-private. The crate root exports only
  `CommandReceipt`, `TextLabelCreateReceipt`, and `ResolveError` from their
  implementation modules. Legacy descriptor construction, `EncodedPayload`,
  and Host-backed manual encoding are also crate-private.
- Generated public item indexes contain no `HostApi`, Host resolver,
  `EncodedPayload`, or Host-backed manual encoder item. Rendered references to
  those names are deliberate privacy `compile_fail` examples.
- `samp-client-sdk` pins `samp-protocol` to exact version
  `=0.1.0-alpha.4` while retaining the workspace path.
- `samp-protocol` has no dependencies and no SDK, Host, ABI, Native profile,
  hook, Windows, or Host encoded-string implementation.
- The four normal typed registration methods and typed-only
  `register_handlers!` forms accept the existing Protocol and Host-backed
  descriptors. A later message migration needs no new public registration
  method, macro form, SDK helper family, codec identity, Wire primitive, or
  framing policy.

The later semantic taxonomy cut exposes profile-neutral incoming RPCs only
through `rpc::incoming::common` and profile-specific RPCs through
`rpc::incoming::r1`. It adds no second callback, descriptor, primitive,
framing, or send boundary.

## Validation

Validation uses Rust and Cargo 1.98 on the repository's default Windows x86
target. No Cargo target-directory setting changed.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo test -p samp-protocol --tests --locked` | Pass: 62 tests |
| `cargo test -p samp-client-sdk --lib --locked` | Pass: 103 tests |
| `cargo test -p samp-client-sdk --doc --locked` | Pass: 7 external-context doctests |
| `cargo test --workspace --all-targets --locked` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo build --workspace --release --locked` | Pass |
| `cargo doc --workspace --exclude samp-client-sdk-host --no-deps --locked` | Pass with no warnings |
| `cargo package -p samp-protocol --allow-dirty --locked` | Pass and verify |
| `cargo package -p samp-client-sdk --allow-dirty --locked --config 'patch.crates-io.samp-protocol.path="C:/Development/rak_rs/crates/samp-protocol"'` | Pass and verify against the local synchronized crate |

All three acceptance seams are green. P0 is green.
