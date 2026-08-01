# Sample rak-rs plugin

This independent ASI waits for `rak_rs.asi`, registers one incoming RPC
callback, and uses the typed `on_server_message` helper. It observes messages
without changing or blocking them.

Build it from the repository root:

```powershell
cargo build --manifest-path examples/sample_plugin/Cargo.toml --release
Copy-Item target/i686-pc-windows-msvc/release/rak_rs_sample_plugin.dll `
  "$env:GTA_DIR/rak_rs_sample_plugin.asi"
```

GTA's ASI loader may load the sample and `rak_rs.asi` in either order; the
sample waits from a worker thread until the host reports ready.

For runtime unload, an unload manager must call the exported
`RakRsPlugin_Shutdown` on a worker thread and continue only when it returns
nonzero. It waits for both the discovery worker and active rak-rs callbacks.
Normal process termination does not need an explicit unload.
