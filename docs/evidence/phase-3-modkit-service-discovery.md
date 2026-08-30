# Phase 3 Modkit Service Discovery Evidence

Date: 2026-08-30

Phase 3 introduces the exact-version modkit bootstrap beside the unchanged
legacy export. The correction pass verified these properties:

- service and bootstrap tables require exact ABI sizes before typed access;
- unknown host result codes and nonzero consumer-reserved fields are preserved;
- Core timeouts support immediate, finite, and explicit infinite waits;
- subscription and receipt IDs do not wrap or alias stale handles;
- callback drains retain enough state to retry after timeout;
- log input is bounded before FFI memory is read;
- the SDK resolves the deployed `samp_client_sdk.asi` artifact;
- `GameContext` tokens reject invalid, stale, cross-thread, wrong-phase, and
  shutdown use;
- the example has an explicit worker-thread shutdown handshake before unload.
- safe SDK services hide raw ABI tables; the Legacy migration pointer requires
  an explicit unsafe access;
- the x86 Rust ABI layouts match an independent C++ fixture.

Quality gate:

- `cargo fmt --all -- --check` — passed;
- `cargo check --workspace --all-targets --locked` — passed;
- `cargo test --workspace --locked` — passed;
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed;
- `cargo build --workspace --release --locked` — passed;
- `cargo test -p modkit-abi -p modkit-runtime -p modkit-sdk --target
  x86_64-pc-windows-msvc --locked` — 51 tests passed;
- `cargo doc --workspace --exclude samp-client-sdk-host --no-deps --locked` —
  passed;
- `scripts/check-release-hygiene.ps1` — passed;
- independent review — Standards and Spec findings were applied; registry
  `NotReady` is modeled but unobservable in this host because the static
  registry is published before its bootstrap table can be called.

Phase 9 remains responsible for native callback sources, host construction of
SDK `GameContext` values, and operation-specific token validation. Phase 3
contains only the stable token transport and generic runtime validator.
