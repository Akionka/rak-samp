# Conventions and invariants
- Rust naming/rustfmt; focused modules; explicit error propagation rather than `unwrap`/`expect` outside tests.
- Use the `log` facade; initialization belongs in `src/logging.rs`; never log packet or RPC payloads.
- Plugin ABI is C-compatible, bounded, versioned, and ALPHA compatibility breaks must be explicit. Never cross the DLL boundary with Rust references, trait objects, or allocations.
- Patch/restore only host-owned RakClient vtable slots. Detours call originals through their captured backend state.
- Emulated incoming events traverse listeners exactly once; nested same-thread dispatch remains non-blocking.
- Typed helpers reuse one host subscription, preserve uncertain text as bytes, bound length-prefixed allocations, serialize replacement payloads before atomic replacement calls, and never retain callback-local events.
- Before runtime unload, unregister and wait from a worker thread only; never wait in `DllMain` or callbacks.
- Behavior changes require unit tests beside modules with observable names. Native layout/wire changes require exact vectors and the C++ fixture; offset changes currently require fixture/live evidence.
- Update both `CORE.md` and `ARCHITECTURE.md` whenever a feature or module changes.