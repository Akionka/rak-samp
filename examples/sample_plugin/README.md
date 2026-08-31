# Sample service-backed SA-MP plugin

A minimal independent ASI: it resolves exact-version services from
`samp_client_sdk.asi` on a worker thread and observes an incoming RPC through
the typed `samp` event API.

```powershell
cargo build --manifest-path examples/sample_plugin/Cargo.toml --release
Copy-Item target/i686-pc-windows-msvc/release/samp_client_sdk_sample_plugin.dll `
  "$env:GTA_DIR/samp_client_sdk_sample_plugin.asi"
```

For runtime unload, call `SampClientSdkPlugin_Shutdown` from a worker and wait for it to
finish before releasing the ASI.
