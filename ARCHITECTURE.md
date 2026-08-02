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
| Runtime | [`src/runtime.rs`](src/runtime.rs), [`src/event.rs`](src/event.rs), [`src/bitstream.rs`](src/bitstream.rs) | Safe traffic API, ordered dispatch, bounded exact-bit payloads |
| Native backend | [`src/platform/win32.rs`](src/platform/win32.rs), [`src/client.rs`](src/client.rs) | Version mapping, detours, vtable patches, RakNet conversion and native string codec calls |
| Plugin API | [`plugin_api/src/lib.rs`](plugin_api/src/lib.rs) | Append-only C ABI, host discovery, safe wrappers |
| Typed RPCs | [`plugin_api/src/events.rs`](plugin_api/src/events.rs) | Wire codecs and named event descriptors, including `onShowDialog` |
| Consumers | [`examples/sample_plugin`](examples/sample_plugin), [`examples/chat_command_plugin`](examples/chat_command_plugin), [`examples/validation_plugin`](examples/validation_plugin), [`examples/validation_unloader`](examples/validation_unloader) | Minimal integration, command/send/emulation example, live diagnostics, and external unload validation |

## Lifecycle

1. Host `DllMain` starts the bootstrap worker and returns.
2. The worker initializes logging, waits for `samp.dll`, identifies its entry
   point, and installs the constructor detour.
3. RakClient construction installs the incoming-RPC detour and three in-place
   vtable patches. The bootstrap monitor logs success or changes the host state
   to `Failed` if deferred installation fails.
4. Plugin workers find the already-loaded `rak_rs.asi`, validate ABI version and
   table size, and register callbacks.
5. For runtime unload, a plugin worker calls `unregister_and_wait` for every
   subscription before an external manager frees the module.

The validation unload manager models step 5 from a separately loaded ASI: it
waits for the validator's completion export, invokes its shutdown export, and
calls `FreeLibrary` only after shutdown confirms callback quiescence. An R1
live run confirms the validation ASI disappears while the process-wide host
continues running.

Validation paths are resolved from the validation ASI module handle because
other in-process mods may change GTA's working directory. Native codec tests
retry only `NotReady` until SA-MP initializes its StringCompressor; other codec
errors fail immediately.

Plugins must not wait or perform synchronized teardown in `DllMain`.
The host runtime lives in a process-lifetime `OnceLock<Arc<Runtime>>`; ABI entry
points clone the `Arc` before invoking runtime methods, so re-entrant callbacks
do not retain a host-state lock.

## Event flow

Native detours convert traffic to bounded `BitStream` values only when a
matching listener exists. [`src/host_api.rs`](src/host_api.rs) exposes each ID
and mutable payload as an opaque callback-lifetime event. The registry invokes
listeners in order; each may continue, replace, or block the event.

Typed `Rpc<T>` descriptors filter an ID, decode the payload, run plugin logic,
and encode replacements before the host atomically swaps the complete exact-bit
stream. Byte-aligned descriptors encode entirely in the plugin. A descriptor
that needs a client codec, such as `SHOW_DIALOG`, asks the host to encode only
that field and combines the returned left-aligned bits with ordinary fields.
They add no hooks or long-lived callback runtime.

```text
incoming RPC 61 -> Event -> SHOW_DIALOG decoder
  -> event_read_encoded_string ABI -> Runtime -> Win32 thiscall reader
plugin replacement -> SHOW_DIALOG encoder -> HostApi::encode_string
  -> Runtime -> Win32 thiscall writer -> EncodedPayload(bytes, bit_len)
  -> event_replace_bits ABI -> BitStream -> native RPC continuation
```

The reader operates on a host-owned copy of the callback stream and advances
the real event cursor only after native decoding succeeds. The writer uses a
host-owned temporary native `BitStream`; it returns copied bytes and an exact
bit length. SA-MP function addresses and its StringCompressor pointer come from
the detected build's `AddressSet` and never cross into plugins.

Incoming packet emulation queues a native packet through the RakPeer receiver
captured by the incoming-RPC detour, then uses the normal receive path. Incoming
RPC emulation dispatches immediately and calls the captured native receiver
only if listeners continue. Native pointers never cross the plugin ABI.

The chat-command example demonstrates a nested cross-direction flow. Its one
outgoing subscription decodes command RPC 50; `/rakrs` calls the captured
original RPC method to send chat RPC 101, then feeds an encoded dialog through
incoming RPC 61. Explicit send bypasses outgoing listeners, while the nested
incoming dispatch uses the registry's same-thread re-entry path. A later
outgoing dialog response RPC 62 is blocked when it carries the example's
reserved dialog ID.

## Native boundary

The backend owns all pointer and vtable operations. It patches only slots 6, 8,
and 25, chains their previous functions, and restores a slot only if its value
is still the host detour. Each detour carries an `Arc` of its backend through
original calls so teardown cannot invalidate active calls.

MinHook detours are created disabled. The caller stores the original
trampoline with release ordering before enabling the detour, preventing an
early callback from observing a null original. Native bit counts use checked
`i32` conversions before entering RakNet.

SA-MP's StringCompressor reader and writer are invoked through x86 `thiscall`
function pointers. Rust does not reproduce its Huffman tree. Temporary native
streams use caller-owned storage sized before the call; pointer, capacity, and
resulting bit-length checks reject unexpected native mutations.

Incoming packet structures use packed offsets; the incoming-RPC player value is
separately aligned. Metadata checks fail open before pointer dereference. RPC
envelopes use RakNet compressed lengths. The C++ layout oracle, fake RakClient
vtable, and inline-hook fixture test these boundaries without proprietary client
files; [VALIDATION.md](VALIDATION.md) covers the live integration check.

## Distribution

The configured target is `i686-pc-windows-msvc`. `cargo make deploy` renames the
host DLL to `rak_rs.asi`; `cargo make deploy-validation` also installs the
validation ASI. Marker files opt its server-bound send and coordinated-shutdown
checks into a live session. `cargo make deploy-validation-unload` additionally
installs the external unload manager. See [README.md](README.md) for usage.

CI checks formatting, the locked workspace tests, strict Clippy, and a release
build on `windows-latest`. Tags matching `v*` publish the raw host ASI and PDB,
SHA-256 checksums, and a ZIP containing the host documentation plus the tested
chat-command example. Hyphenated version tags are marked as prereleases.
