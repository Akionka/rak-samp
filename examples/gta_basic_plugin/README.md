# GTA basic plugin

Minimal 32-bit Windows plugin for `GtaSaServiceV1`. It connects on a worker thread, registers a typed post-`CGame::Process` callback, and reads the local ped snapshot through `gta-sa` without native pointers.

Build from the workspace root:

```text
cargo build -p gta-basic-plugin --release
```

Before unloading the DLL, call `GtaBasicPlugin_Shutdown` and require a nonzero result.
