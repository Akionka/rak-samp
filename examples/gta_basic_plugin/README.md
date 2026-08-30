# GTA basic plugin

Minimal 32-bit Windows probe for `GtaSaServiceV2`. It connects on a worker
thread, registers a typed post-`CGame::Process` callback, exercises the ped
pool, ground-height, timer, and active-camera APIs, then compares direct and
queued camera snapshots from the same timer frame. Native pointers never leave
the Host.

Build from the workspace root:

```text
cargo build -p gta-basic-plugin --release
```

Deploy only to an isolated GTA/SA-MP test root. Copy the resulting
`gta_basic_plugin.dll` beside the Host as `gta-basic-plugin.asi`, start the
matching loopback server, and launch the client normally. The plugin removes
any stale `gta-basic-plugin.status` file at startup and writes one bounded
machine-readable result in the GTA working directory:

```text
STATUS=PASS frame=<u32> camera=game:<bits>;right:<bits>;forward:<bits>;up:<bits>;position:<bits>
```

`PASS` proves that guarded direct and queued snapshots were finite and bitwise
identical for one accepted post-process frame. It does not replace the manual
visible-pose comparison in `docs/native-layout-smoke.md`.

Before unloading the DLL, call `GtaBasicPlugin_Shutdown` and require a nonzero result.
