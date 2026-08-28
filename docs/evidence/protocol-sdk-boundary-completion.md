# Protocol / SDK boundary completion

**Status: GREEN.** Issues #22 and #39 are complete on the Rust 1.98
acceptance baseline. The blocking work from #37 and #38 is included. The
[P0 architecture gate](p0-architecture-gate.md) stayed green while the P1/P2
semantic namespace, facade, package, and documentation work landed.

## Published boundary

- `samp-protocol` owns platform-independent values, Wire primitives,
  descriptors, framing, and structured codec errors. Stable built-ins use
  semantic `common`, `r1`, and feature namespaces.
- `samp-client-sdk` projects Protocol and remaining Host-backed descriptors
  into the typed `Net` API. Plugins resolve `Samp` and use subsystem facades;
  the Host wrapper and payload encoders remain private.
- The SDK depends on the workspace Protocol crate at exactly
  `=0.1.0-alpha.4`. Package verification checks the normalized SDK manifest
  and requires exactly one `samp-protocol` entry in its packaged lockfile.
- Public examples compile through `Samp`, `Net`, and Protocol descriptor paths,
  including Host-injected encoded-string descriptors.

## Contextual vocabulary and API audit

The executable audit is `scripts/check-release-hygiene.ps1`. It scans tracked
and staged repository text, classifies explicit historical/evidence records,
and inspects the generated SDK public item index.

| Term or surface | Result |
| --- | --- |
| `on_*_protocol_*` | Absent from production source. The four typed `Net` methods are the public registration surface. |
| `fixed`, `phase15` | Absent from current Packet/RPC production taxonomy. Historical split documents label removed `fixed` paths as historical. |
| Direction-neutral descriptor paths | Protocol exposes only the generic `IncomingPacket`, `OutgoingPacket`, `IncomingRpc`, and `OutgoingRpc` wrappers. The old neutral `Packet`/`Rpc` descriptors and the SDK `events::Packet` alias are absent; the Host-backed `Rpc` carrier is crate-private. |
| `HostApi` | Private; absent from the generated public item index. The crate-level `compile_fail` example remains a privacy guard. |
| `RpcEncoder`, `PayloadWriter`, `EncodedPayload` | Private; absent from the generated public item index. |
| Incoming RPC ownership | Protocol uses `rpc::incoming::common` and `rpc::incoming::r1`. The SDK injects Host encoded-string operations and owns no duplicate incoming RPC descriptors or payloads. |
| Neutral bit booleans | `WireReadExt::read_bit_bool` and `WireWriteExt::write_bit_bool` are the single canonical MSB-first implementation used by R1 Packet and RPC codecs. Protocol bounds failures stay distinct from reader/writer source failures. |

The `Animations` and `Textdraw` aliases are unrelated facade naming
compatibility and expose no Protocol migration status, codec strategy, or Host
ownership. Historical ADRs and implementation plans retain old terms only when
they describe removed architecture.

## Automated gates

Windows CI and tagged releases use Rust 1.98 and run public documentation,
the contextual public-surface audit, and both package checks. Protocol CI keeps
its explicit `x86_64-unknown-linux-gnu` tests and Clippy run. Cargo's configured
target and target-directory behavior are unchanged.

The completion validation passed with:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo check --workspace --all-targets --locked` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass |
| `cargo test -p samp-client-sdk --doc --locked` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo build --workspace --release --locked` | Pass |
| `cargo doc --workspace --exclude samp-client-sdk-host --no-deps --locked` | Pass |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-release-hygiene.ps1` | Pass |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-published-crates.ps1` | Pass |
| `cargo test -p samp-protocol --target x86_64-unknown-linux-gnu --locked` | Pass in WSL Linux |
| `cargo clippy -p samp-protocol --target x86_64-unknown-linux-gnu --all-targets --locked -- -D warnings` | Pass in WSL Linux |

This closes the P1/P2 Definition of Done for #22. Future message migrations use
directional Protocol wrappers and semantic SDK paths without new registration
methods, macro forms, public codec carriers, primitive mini-codecs, or
migration-named modules.
