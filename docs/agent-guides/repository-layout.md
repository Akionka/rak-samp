# Repository layout

| Area | Location |
| --- | --- |
| Bootstrap, exported ABI, and logging | `src/lib.rs`, `src/host_api.rs`, `src/logging.rs` |
| Runtime events and bit streams | `src/runtime.rs`, `src/event.rs`, `src/bitstream.rs` |
| Networking offsets | `src/client.rs` |
| Native hooks and profiles | `src/platform/win32.rs`, `src/platform/win32/native_client/` |
| Public SDK and safe facades | `sdk/src/lib.rs`, `sdk/src/facade/mod.rs`, `sdk/src/facade/local_player.rs`, `sdk/src/facade/network.rs`, `sdk/src/facade/pools.rs`, `sdk/src/facade/ui.rs`, `sdk/src/raknet.rs` |
| Explicit unsafe native-address API | `sdk/src/raw.rs` |
| Typed RPC and packet helpers | `sdk/src/events/` |
| Minimal and chat-command plugins | `examples/` |
| Independent ABI layout oracle | `tests/fixtures/raknet_layout.cpp` |

Only the host links the native hook backend. Independently loaded ASI plugins
depend on the public SDK.
