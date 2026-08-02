# Sample rak-rs plugin

A minimal independent ASI: it waits for `rak_rs.asi` on a worker thread and
observes an incoming RPC through the typed event API.

```powershell
cargo build --manifest-path examples/sample_plugin/Cargo.toml --release
Copy-Item target/i686-pc-windows-msvc/release/rak_rs_sample_plugin.dll `
  "$env:GTA_DIR/rak_rs_sample_plugin.asi"
```

For runtime unload, call `RakRsPlugin_Shutdown` from a worker and wait for it to
finish before releasing the ASI.
