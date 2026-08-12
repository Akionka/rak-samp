# Network smoke plugin

This opt-in ASI exercises the network paths that are safe to validate against
an isolated server without sending server-bound traffic:

- native SA-MP string compressor encode/decode round-trip;
- a 3-bit incoming packet, allocated and queued natively, then blocked by the
  plugin before SA-MP receives it;
- a 3-bit incoming RPC, dispatched to and blocked by the plugin before the
  original RPC handler can run.

It is a validation aid, not a gameplay plugin. In particular, it does not prove
outbound packet/RPC send or delivery into SA-MP's original RPC handler. Do not
use it on a public server.

```powershell
cargo build --manifest-path examples/network_smoke_plugin/Cargo.toml --release
Copy-Item target/i686-pc-windows-msvc/release/samp_client_sdk_network_smoke_plugin.dll `
  "$env:GTA_DIR/samp_client_sdk_network_smoke_plugin.asi"
```

When SAMPFUNCS is present, success or failure appears in its console. A test
loader may instead query these exports:

| Export | Meaning |
| --- | --- |
| `SampClientSdkNetworkSmoke_Status` | Stage bitset; success is `0x7F`. |
| `SampClientSdkNetworkSmoke_Failure` | First `SampClientSdkResult` on failure. |
| `SampClientSdkNetworkSmoke_Shutdown` | Synchronously removes its callbacks before unload. |

`0x80000000` in the status means a stage failed. The component bits are
documented beside the exported status constants in `src/lib.rs`.
