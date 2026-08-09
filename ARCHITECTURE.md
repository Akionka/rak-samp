# Architecture

Each ASI loads independently. Only the host links the native hook backend;
plugins use the public, versioned SDK package.

## Components

| Area | Location | Purpose |
| --- | --- | --- |
| Bootstrap and ABI | `src/lib.rs`, `src/host_api.rs`, `src/logging.rs` | Start outside `DllMain`, publish status, and export `SampClientSdkApiV1`. |
| Runtime | `src/runtime.rs`, `src/event.rs`, `src/bitstream.rs` | Own bounded event data, dispatch listeners, and send exact RakNet payloads. |
| Native backend | `src/platform/win32.rs`, `src/platform/win32/r1_client.rs`, `src/client.rs` | Select networking offsets, own hooks, and apply the fixed R1 bridge offsets with local validation. |
| Public SDK | `sdk/src/lib.rs`, `sdk/src/facade.rs`, `sdk/src/raknet.rs`, `sdk/src/raw.rs` | Resolve the host through `Samp`, provide safe subsystem views, callback helpers, owned BitStreams, protocol catalogs, and explicit unsafe native-address access. |
| Typed events | `sdk/src/events/` | Provide bounded R1 packet/RPC codecs and mock ABI support. |
| Examples | `examples/` | Show a minimal subscription and a chat-command plugin. |
| Layout fixture | `tests/fixtures/raknet_layout.cpp` | Independently verify native packing for boundary layouts. |
