# Sample rak-rs plugin

This independent ASI waits for `rak_rs.asi` on a worker thread, registers one
incoming RPC callback, and observes `on_server_message` through the typed event
API.

Build and copy it from the repository root:

```powershell
cargo build --manifest-path examples/sample_plugin/Cargo.toml --release
Copy-Item target/i686-pc-windows-msvc/release/rak_rs_sample_plugin.dll `
  "$env:GTA_DIR/rak_rs_sample_plugin.asi"
```

For runtime unload, call `RakRsPlugin_Shutdown` from a worker thread and free the
ASI only after it returns nonzero. Process termination needs no explicit
shutdown.
