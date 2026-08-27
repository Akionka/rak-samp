# Project map
- Windows x86 process-wide `samp_client_sdk.asi` host: bootstrap/exported ABI/logging in `src/lib.rs`, `src/host_api/`, and `src/logging.rs`; runtime/events/commands in `src/runtime*`, `src/event.rs`, and `src/command.rs`.
- Independently loaded plugins depend on the public SDK in `sdk/`; safe entry is `Samp` plus subsystem facades. `sdk/src/host_api/` is internal ABI plumbing; `sdk/src/raw.rs` is the explicit unsafe native-address API.
- Wire types/codecs/catalogs live in `crates/samp-protocol/`; SDK typed packet/RPC subscriptions and replacements live in `sdk/src/events/`.
- Windows hooks, captured state, queued native operations, and per-client profiles live in `src/platform/win32/`; fixed networking entry offsets remain in `src/client.rs`.
- Independent native-layout oracle: `tests/fixtures/raknet_layout.cpp`, built via `build.rs`; serialized sizes are not native packing evidence.
- Examples include minimal/chat-command plugins, network smoke validation, and R1/R3/R5/DL probes.
- Architecture/docs: `CORE.md`, `ARCHITECTURE.md`, usage in `README.md`, backlog in `TODO.md`; focused agent guides live in `docs/agent-guides/`.
- Read `mem:tech_stack` for toolchain/targets, `mem:conventions` for code/ABI invariants, `mem:suggested_commands` for Cargo Make workflows, and `mem:task_completion` for completion checks.