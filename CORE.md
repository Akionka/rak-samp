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
  snapshot/catalog, chat-history, gangzone, object, player, text-label, textdraw,
  vehicle, and forward/reverse handle-cache reads gated on completed game-thread
  publication. Native outbound packet and RPC sends execute from the game-thread
  command pump through the captured original RakClient functions. Incoming
  packet emulation transfers a native allocation into the captured receive queue;
  incoming RPC emulation dispatches listeners once before calling the captured
  trampoline directly. RPC envelope helpers preserve optional timestamps and
  exact payload bit lengths across listener replacement. Native outgoing stream
  dispatch publishes listener mutations synchronously and fails open on rewrite
  errors. Raw incoming packet dispatch validates packed metadata before copying
  owned callback data; the incoming detour retains return/deallocate/retry
  ownership through captured native-call wrappers. Incoming network detours
  keep packet ownership and RPC receiver/player publication in `hooks.rs`;
  malformed RPC envelopes fail open to the captured trampoline. Outgoing
  detours call captured originals through non-owning ABI wrappers after
  synchronous listener dispatch. Hook installation and restoration remain
  root-owned.

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
