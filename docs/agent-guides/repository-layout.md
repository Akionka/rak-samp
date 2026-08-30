# Repository layout

| Area | Location |
| --- | --- |
| Bootstrap, exported ABI, and logging | `src/lib.rs`, `src/host_api/`, `src/logging.rs` |
| Runtime facade, values, and events | `src/runtime/`, `src/event.rs`, `src/bitstream.rs` |
| Networking offsets | `src/client.rs` |
| Native hooks, commands, and transitional direct operations | `src/platform/win32/`, `src/platform/win32/commands/`, `src/platform/win32/native_client/` |
| SA-MP native profiles, player/pool operations, and backend values | `crates/samp-native/` |
| Public SDK ABI, safe facades, and tests | `sdk/src/abi/`, `sdk/src/facade/`, `sdk/src/host_api/`, `sdk/src/tests/` |
| Explicit unsafe native-address API | `sdk/src/raw.rs` |
| Typed RPC and packet helpers | `sdk/src/events/` |
| Platform-independent protocol and bitstreams | `crates/samp-protocol/` |
| Windows x86 implementation primitives | `crates/modkit-win32/` |
| GTA SA profile and game-process runtime | `crates/gta-sa-native/` |
| Transitional SA-MP UI operations | `src/platform/win32/native_client/` |
| GTA SA value types and handles | `crates/gta-sa/` |
| Minimal and chat-command plugins | `examples/` |
| Independent ABI layout oracle | `tests/fixtures/raknet_layout.cpp` |

Only the host links the native hook backend. Independently loaded ASI plugins
depend on the public SDK.
