# Repository layout

| Area | Location |
| --- | --- |
| Bootstrap, exported ABI, and logging | `src/lib.rs`, `src/host_api/`, `src/logging.rs` |
| Runtime facade, values, and events | `src/runtime/`, `src/event.rs`, `src/bitstream.rs` |
| Networking offsets | `src/client.rs` |
| Host commands, caches, hook callbacks, and native ABI adaptation | `src/platform/win32/`, `src/platform/win32/commands/` |
| SA-MP native profiles, direct operations, and RakClient detours | `crates/samp-native/` |
| Public SDK ABI, safe facades, and tests | `sdk/src/abi/`, `sdk/src/facade/`, `sdk/src/host_api/`, `sdk/src/tests/` |
| Explicit unsafe native-address API | `sdk/src/raw.rs` |
| Typed RPC and packet helpers | `sdk/src/events/` |
| Platform-independent protocol and bitstreams | `crates/samp-protocol/` |
| Windows x86 implementation primitives | `crates/modkit-win32/` |
| GTA SA profile, verified native layouts/calls, local-ped reads, and game-process runtime | `crates/gta-sa-native/` |
| GTA SA pointer-free math, snapshots, and typed handles | `crates/gta-sa/` |
| Minimal and chat-command plugins | `examples/` |
| Independent ABI layout oracles | `tests/fixtures/raknet_layout.cpp`, `tests/fixtures/gta_sa_layout.cpp` |

Only the host links the native hook backend. Independently loaded ASI plugins
depend on the public SDK.
