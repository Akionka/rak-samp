# Repository Guidelines

## Layout

This Rust 2024 workspace builds the Windows x86 host
`samp_client_sdk.asi`. Host runtime and ABI implementation live in `src/`;
independently loaded ASI plugins depend only on the public SDK in `sdk/`.
Typed RPC helpers belong in `sdk/src/events/`. The examples contain a minimal
plugin and a `/sampclientsdk` chat-command plugin. Keep SA-MP networking
offsets in `src/client.rs`, native hook code in `src/platform/win32.rs`, and
independent ABI oracles in `tests/fixtures/`.

## Commands

Run from the repository root:

- `cargo build --workspace` (add `--release` for distribution builds)
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check` (`cargo fmt` applies formatting)
- `cargo make deploy` to copy the release host to `$env:GTA_DIR`
- `cargo make deploy-chat-command-example` to deploy the chat-command example

Close GTA before deployment because Windows locks loaded ASIs.

## Code and ABI rules

- Follow `rustfmt` and Rust naming conventions. Prefer focused modules and
  explicit errors over `unwrap()` or `expect()` outside tests.
- Use the `log` facade; `src/logging.rs` owns setup. Never log packet or RPC
  payloads.
- Keep the plugin ABI C-compatible and versioned. During ALPHA, its contract
  may intentionally break; no Rust references, trait objects, or allocations
  may cross the DLL boundary.
- Preserve native layouts using the independent C++ fixture. Serialized sizes
  alone do not prove in-memory packing.
- The R1 bridge uses approved fixed offsets and validates every pointer, range,
  capacity, and enum at the operation boundary.
- Patch and restore only the RakClient vtable slots owned by the host. Detours
  call originals through their captured backend state.
- Emulated packets cross incoming listeners exactly once. Nested same-thread
  dispatch remains non-blocking.
- Typed helpers reuse the single host subscription, keep uncertain text as
  bytes, bound length-prefixed allocations, and serialize replacements before
  the atomic replacement ABI call. Never retain callback-local events.
- Before runtime plugin unload, remove every subscription with
  `HostApi::unregister_and_wait` from a worker thread. Never wait in `DllMain`,
  a callback, or the game tick; never `FreeLibrary` while callbacks can run.

## Tests and documentation

Put unit tests beside their modules and use observable behavior names. Run the
workspace tests for behavior changes. Wire-format and native-boundary changes
need exact vectors and the C++ layout fixture.

Keep dependencies minimal and never commit `target/`, secrets, machine paths,
or proprietary clients and headers. Update [CORE.md](CORE.md) and
[ARCHITECTURE.md](ARCHITECTURE.md) whenever a feature or module changes.
[README.md](README.md) covers usage and [TODO.md](TODO.md) tracks planned
work.
