# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust 2024 workspace. The root `rak_rs` crate builds the process-wide `rak_rs.asi` host; its public runtime and ABI export live in `src/`. The `plugin_api/` crate is the only dependency intended for independently loaded plugin ASIs; its `src/events.rs` module provides typed RPC helpers above the raw ABI. [`examples/sample_plugin/`](examples/sample_plugin) is the minimal consumer and unload-handshake example; [`examples/validation_plugin/`](examples/validation_plugin) records live packet/RPC histograms and runs a blocked local emulation self-test. The Windows x86 hook backend is isolated in `src/platform/win32.rs`, and [`tests/fixtures/`](tests/fixtures) contains independently compiled native ABI oracles. Add reusable code as focused modules and keep version-specific SA-MP offsets in `src/client.rs`. Keep generated Cargo output in `target/`; it is intentionally ignored by Git.

## Build, Test, and Development Commands

Run commands from the repository root:

- `cargo build --workspace` compiles the host and plugin ABI; add `--release` for distribution builds.
- `cargo test --workspace` runs host and plugin-ABI unit tests on the installed Windows x86 target.
- `cargo clippy --workspace -- -D warnings` runs linting and treats warnings as errors.
- `cargo fmt --check` verifies Rust formatting; use `cargo fmt` to apply it.
- `cargo make deploy` builds the release host and copies it as `rak_rs.asi` to `$env:GTA_DIR`; close GTA first because Windows locks loaded ASIs.
- `cargo make deploy-validation` builds and deploys both the host and in-game validation ASI.

## Coding Style & Naming Conventions

Follow `rustfmt` defaults: four-space indentation and idiomatic Rust layout. Use `snake_case` for functions, variables, files, and modules; `PascalCase` for structs, enums, and traits; and `SCREAMING_SNAKE_CASE` for constants. Prefer small, single-purpose modules and explicit error handling over `unwrap()` or `expect()` in non-test code. Public APIs should have concise `///` documentation when their purpose is not self-evident.

Use the `log` facade for host diagnostics; [`src/logging.rs`](src/logging.rs) owns the `simplelog` setup. Log lifecycle, hook, and ABI-registration decisions at appropriate levels, but never record packet/RPC payloads or other sensitive player data.

Changes to RakNet wire encoding or native hook boundaries require exact protocol vectors and a Windows x86 integration check. Keep Rust native structures synchronized with the independent C++ layout fixture; do not infer in-memory packing from serialized wire sizes or compiler defaults. Treat field-offset evidence from a supported live client as authoritative and record it in [`REVIEW.md`](REVIEW.md). Never replace the full RakClient vtable with a partial copy; patch and restore only the explicit slots the host owns.

Preserve dispatch and teardown invariants: emulated packets must cross incoming listeners once, same-thread nested dispatch must not deadlock, and detours must pass their captured backend state to original-call helpers instead of looking up the global backend again.

Before unloading plugin code at runtime, remove every subscription with `HostApi::unregister_and_wait` from a worker thread and wait for success. Never wait from `DllMain` or an active rak-rs callback, and never call `FreeLibrary` while a callback may still refer to the plugin module.

Typed event helpers belong in `plugin_api/src/events.rs`: preserve the one-host-hook design, document each RPC ID and wire layout, keep text as bytes unless its encoding is guaranteed, and cap length-prefixed reads before allocation. Serialize replacement payloads before calling the append-only ABI's atomic replacement function; do not retain callback-local `RakRsEventV1` values or introduce a second callback/hook runtime.

## Testing Guidelines

Place unit tests beside the code they cover inside a `#[cfg(test)] mod tests` module. Put black-box or multi-module behavior tests in `tests/` once that directory is needed. Name tests by observable behavior, such as `parses_empty_input` or `returns_error_for_invalid_header`. Add or update tests for behavior changes and run `cargo test --workspace` before opening a pull request. Changes to client offsets also require fixture tests and manual validation against every supported SA-MP build. ABI changes require compatibility tests and must remain append-only within a versioned API table.

## Commit & Pull Request Guidelines

The repository has no committed history yet, so no established commit style exists. Use concise, imperative subjects such as `Add request parser` or `Fix empty-input handling`; keep each commit focused. Pull requests should explain the change and its motivation, link the related issue when one exists, list validation performed (for example, `cargo fmt --check` and `cargo test`), and include screenshots or command output when a user-visible behavior changes.

## Configuration & Dependencies

Keep dependency additions minimal and record them in `Cargo.toml`. Do not commit `target/`, local environment files, secrets, or machine-specific paths. Do not commit proprietary SA-MP clients or headers; use legal local installations for compatibility checks. Document any required runtime configuration in the README when it is introduced.

## Project Documentation

- [CORE.md](CORE.md) explains the current feature behavior and its code references.
- [ARCHITECTURE.md](ARCHITECTURE.md) describes how crate entities interact.
- [README.md](README.md) gives users installation and plugin-authoring instructions.
- [VALIDATION.md](VALIDATION.md) defines the isolated in-game validation procedure and pass criteria.
- [TODO.md](TODO.md) tracks planned, active, and completed work.
- [REVIEW.md](REVIEW.md) preserves open review findings and their resolution context.
- **Required:** Update both `CORE.md` and `ARCHITECTURE.md` whenever a feature or module is added, edited, or removed. Keep code-entity references and interoperability details accurate.
