# Agent guidance

This Rust 2024 Cargo workspace builds the Windows x86 `samp_client_sdk.asi`
host and provides the public SDK used by independently loaded SA-MP plugins.

- Package and build tool: Cargo (Rust 1.98); `.cargo/config.toml` defaults to
  `i686-pc-windows-msvc`.
- Workspace build: `cargo build --workspace`.
- Workspace type/lint check: `cargo clippy --workspace -- -D warnings`.

Read only the guidance relevant to the task:

- [Repository layout](docs/agent-guides/repository-layout.md) — deciding where
  code belongs.
- [Development workflow](docs/agent-guides/development-workflow.md) — testing,
  formatting, deployment, and documentation updates.
- [Rust and logging](docs/agent-guides/rust-and-logging.md) — editing Rust or
  adding diagnostics.
- [ABI and runtime safety](docs/agent-guides/abi-and-runtime-safety.md) — DLL
  boundaries, native hooks, R1 bridging, or plugin unload.
- [Game-thread commands](docs/agent-guides/game-thread-commands.md) — queued
  native reads or mutations, command receipts, or cache refresh.
- [Packets and typed events](docs/agent-guides/packets-and-events.md) — packet
  dispatch, RPC helpers, subscriptions, or event replacement.
