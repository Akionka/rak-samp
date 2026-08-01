# Architecture

## Process model

```text
GTA ASI loader
 ├─ rak_rs.asi (one host and hook runtime)
 ├─ feature-a.asi ─┐
 └─ feature-b.asi ─┴─> rak_rs_plugin_api ─> RakRs_GetApiV1 ─> host
```

Each ASI loads independently. The host is a `cdylib` and the only module linked
to MinHook; plugins link only the ABI client crate.

## Components

| Area | Files | Responsibility |
| --- | --- | --- |
| Bootstrap | [`src/lib.rs`](src/lib.rs), [`src/host_api.rs`](src/host_api.rs), [`src/logging.rs`](src/logging.rs) | Start outside loader lock, publish readiness/API, own logging |
| Runtime | [`src/runtime.rs`](src/runtime.rs), [`src/event.rs`](src/event.rs), [`src/bitstream.rs`](src/bitstream.rs) | Safe traffic API, ordered dispatch, bounded payloads |
| Native backend | [`src/platform/win32.rs`](src/platform/win32.rs), [`src/client.rs`](src/client.rs) | Version mapping, detours, vtable patches, RakNet conversion |
| Plugin API | [`plugin_api/src/lib.rs`](plugin_api/src/lib.rs) | C ABI types, host discovery, safe wrappers |
| Typed RPCs | [`plugin_api/src/events.rs`](plugin_api/src/events.rs) | Byte-aligned codecs and named event descriptors |
| Consumers | [`examples/sample_plugin`](examples/sample_plugin), [`examples/validation_plugin`](examples/validation_plugin) | Minimal integration and live diagnostics |

## Lifecycle

1. Host `DllMain` starts the bootstrap worker and returns.
2. The worker initializes logging, waits for `samp.dll`, identifies its entry
   point, and installs the constructor detour.
3. RakClient construction installs the incoming-RPC detour and three in-place
   vtable patches. The host reports `Ready`.
4. Plugin workers find the already-loaded `rak_rs.asi`, validate ABI version and
   table size, and register callbacks.
5. For runtime unload, a plugin worker calls `unregister_and_wait` for every
   subscription before an external manager frees the module.

Plugins must not wait or perform synchronized teardown in `DllMain`.

## Event flow

Native detours convert traffic to bounded `BitStream` values only when a
matching listener exists. [`src/host_api.rs`](src/host_api.rs) exposes each ID
and mutable payload as an opaque callback-lifetime event. The registry invokes
listeners in order; each may continue, replace, or block the event.

Typed `Rpc<T>` descriptors filter an ID, decode the payload, run plugin logic,
and encode replacements locally before the host atomically swaps the complete
byte-aligned stream. They add no hooks or long-lived callback runtime.

Incoming packet emulation queues a native packet through the RakPeer receiver
captured by the incoming-RPC detour, then uses the normal receive path. Incoming
RPC emulation dispatches immediately and calls the captured native receiver
only if listeners continue. Native pointers never cross the plugin ABI.

## Native boundary

The backend owns all pointer and vtable operations. It patches only slots 6, 8,
and 25, chains their previous functions, and restores a slot only if its value
is still the host detour. Each detour carries an `Arc` of its backend through
original calls so teardown cannot invalidate active calls.

Incoming packet structures use packed offsets; the incoming-RPC player value is
separately aligned. Metadata checks fail open before pointer dereference. RPC
envelopes use RakNet compressed lengths. The C++ layout oracle, fake RakClient
vtable, and inline-hook fixture test these boundaries without proprietary client
files; [VALIDATION.md](VALIDATION.md) covers the live integration check.

## Distribution

The configured target is `i686-pc-windows-msvc`. `cargo make deploy` renames the
host DLL to `rak_rs.asi`; `cargo make deploy-validation` also installs the
validation ASI. See [README.md](README.md) for usage.
