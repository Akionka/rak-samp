# Core Features

`samp-client-sdk` has two pillars:

- `sdk/` is the public Rust package imported as `samp_client_sdk` by ASI
  plugins; its public paths follow cohesive modules rather than compatibility
  aliases.
- `samp-client-sdk-host` owns the Windows x86 bridge and produces
  `samp_client_sdk.asi`; its runtime keeps failure types and send policy
  separate from lifecycle control, while the R1 bridge isolates approved
  native addresses, object layouts, and guarded memory access from operation
  sequencing. The Windows backend also separates bounded producer-side command
  and cache-refresh requests from game-thread execution, with scalar and owned
  snapshot/catalog, player, text-label, and forward/reverse handle-cache reads
  gated on completed game-thread publication.

## Process model

```text
GTA ASI loader
 ├─ samp_client_sdk.asi (samp-client-sdk-host)
 ├─ feature-a.asi ─┐
 └─ feature-b.asi ─┴─> samp-client-sdk ─> SampClientSdk_GetApiV1 ─> host
```

## Lifecycle

1. `DllMain` starts a bootstrap worker and returns.
2. The worker waits for `samp.dll`, recognizes the client build, and installs
   the owned RakClient constructor hook.
3. RakClient construction installs the owned networking hooks and publishes
   ready state.
4. Plugin workers resolve `SampClientSdk_GetApiV1`, validate the table, and
   register owned subscriptions.
5. Plugin workers unregister and wait for callbacks before their DLL unloads.
