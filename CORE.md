# Core Features

`rak-samp` builds one Windows x86 host, `rak_samp.asi`. Independent plugins link
only `rak_samp_plugin_api`; the host owns SA-MP detection and native hooks.

## Runtime

Bootstrap runs outside `DllMain`, waits for `samp.dll`, and exposes a ready or
failed host state. Lifecycle and ABI diagnostics go to `rak-samp.log`; packet and
RPC payloads are never logged.

Plugins subscribe to incoming or outgoing packets and RPCs. A matching listener
can continue, block, or atomically replace an exact-bit payload. Listeners run
in registration order; nested dispatch on the same thread remains non-blocking.
Events are callback-local. Explicit sends bypass outgoing listeners, while
incoming emulation follows the normal incoming dispatch path exactly once.

## ABI and plugin safety

`RakSampApiV1` is append-only and C-compatible: Rust references, trait objects,
allocations, and native pointers do not cross the DLL boundary. Payload sizes
and bit counts are checked before they reach RakNet.

A plugin must keep its subscriptions and, before runtime unload, call
`HostApi::unregister_and_wait` for each one from a worker thread. Waiting in
`DllMain` or a callback is invalid because callbacks may still be active.

## Typed events

`events::rpc` and `events::packet` add named R1 codecs over the raw API. They
validate full payload consumption, preserve protocol bit layout, bound dynamic
data, and leave uncertain text as bytes. Encoded SA-MP strings use the client's
StringCompressor rather than a Rust reimplementation.

SA-MP 0.3.7 R1 is the typed-layout authority. Other recognized clients may use
raw callbacks but are not typed-layout compatible until live validation is
recorded.

## Native boundary

The Windows backend owns client addresses, detours, vtable changes, and native
string-codec calls. It restores only hooks it owns and keeps captured backend
state valid for in-flight original calls. Native layouts are covered by the C++
fixture and live evidence in [REVIEW.md](REVIEW.md).

See [ARCHITECTURE.md](ARCHITECTURE.md) for component ownership and
[VALIDATION.md](VALIDATION.md) for the live check.
