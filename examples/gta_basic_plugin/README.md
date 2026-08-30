# GTA basic plugin

Minimal 32-bit Windows plugin for `GtaSaServiceV2`. It connects on a worker
thread, registers a typed post-`CGame::Process` callback, copies the local-ped
snapshot, validates its typed handle through the GTA ped pool, queries the
ground height, and copies the current timer state without exposing native
pointers.

Build from the workspace root:

```text
cargo build -p gta-basic-plugin --release
```

Before unloading the DLL, call `GtaBasicPlugin_Shutdown` and require a nonzero result.
