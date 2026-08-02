# Repository Guidelines

## Layout

This Rust 2024 workspace builds a process-wide Windows x86 host (`rak_rs.asi`).
Host runtime and ABI implementation live in `src/`; independently loaded ASI
plugins depend only on `plugin_api/`. Typed RPC helpers belong in
`plugin_api/src/events.rs`. The examples contain a minimal plugin, a `/rakrs`
chat-command plugin, and in-game validation tooling. Keep SA-MP offsets in
`src/client.rs`, native hook code in `src/platform/win32.rs`, and independent
ABI oracles in `tests/fixtures/`.

## Commands

Run from the repository root:

- `cargo build --workspace` (add `--release` for distribution builds)
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check` (`cargo fmt` applies formatting)
- `cargo make deploy` to copy the release host to `$env:GTA_DIR`
- `cargo make deploy-validation` to deploy the host and validation plugin
- `cargo make deploy-validation-unload` to include the external unload manager
- `cargo make deploy-chat-command-example` to deploy the `/rakrs` example

Close GTA before deployment because Windows locks loaded ASIs.

## Code and ABI rules

- Follow `rustfmt` and Rust naming conventions. Prefer focused modules and
  explicit errors over `unwrap()` or `expect()` outside tests.
- Use the `log` facade; `src/logging.rs` owns setup. Never log packet or RPC
  payloads.
- Keep the plugin ABI C-compatible, versioned, and append-only. No Rust
  references, trait objects, or allocations may cross the DLL boundary.
- Preserve native layouts using fixture and live-client evidence; serialized
  sizes do not prove in-memory packing. Record authoritative offset evidence in
  [REVIEW.md](REVIEW.md).
- Patch and restore only the RakClient vtable slots owned by the host. Detours
  must use their captured backend state when calling original functions.
- Emulated packets cross incoming listeners exactly once. Nested same-thread
  dispatch must remain non-blocking.
- Typed helpers reuse the single host subscription, keep uncertain text as
  bytes, bound length-prefixed allocations, and serialize replacements before
  the atomic replacement ABI call. Never retain callback-local events.
- Before runtime plugin unload, remove every subscription with
  `HostApi::unregister_and_wait` from a worker thread. Never wait in `DllMain`,
  inside a callback, or call `FreeLibrary` while callbacks may still run.

## Tests and documentation

Put unit tests beside their modules and use observable behavior names. Run the
workspace tests for behavior changes. Wire-format and native-boundary changes
also need exact vectors and a Windows x86 integration check; client-offset
changes require fixture and live validation for each supported build.

Keep dependencies minimal and never commit `target/`, secrets, machine paths,
or proprietary clients and headers. Update both [CORE.md](CORE.md) and
[ARCHITECTURE.md](ARCHITECTURE.md) whenever a feature or module changes.
[README.md](README.md) covers usage, [VALIDATION.md](VALIDATION.md) the live test,
[TODO.md](TODO.md) pending work, and [REVIEW.md](REVIEW.md) durable review evidence.
