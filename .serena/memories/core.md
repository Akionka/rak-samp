# Project map
- Windows x86 process-wide ASI host; runtime/ABI implementation in `src/`.
- Separately loaded plugins depend only on `plugin_api/`; typed packet/RPC helpers live in `plugin_api/src/events/`.
- Native Windows hooks are isolated in `src/platform/win32.rs`; SA-MP offsets belong in `src/client.rs`.
- Independent native-layout oracle: `tests/fixtures/raknet_layout.cpp`, built via `build.rs`; do not treat serialized sizes as native packing evidence.
- Examples: minimal plugin, chat-command plugin, and currently validation tooling.
- Architecture/docs: `CORE.md`, `ARCHITECTURE.md`, usage in `README.md`, backlog in `TODO.md`.
- ABI and unload invariants: bounded C-compatible versioned boundary; no Rust references/trait objects/allocations across DLLs; unregister every subscription with `HostApi::unregister_and_wait` from a worker thread before unload; never wait in `DllMain`/callbacks or unload while callbacks may run.
- Read `mem:tech_stack` for toolchain/targets, `mem:conventions` for code and ABI rules, `mem:suggested_commands` for workflows, and `mem:task_completion` for completion checks.