# Rust GTA SA / SA-MP Modding Infrastructure

This context defines the resident mod host, its native GTA SA and SA-MP
integrations, and the safe Rust API exposed to plugins.

## Language

### Runtime boundaries

**Host**:
The process-lifetime ASI module that owns core hooks, native process state, and
published services.
_Avoid_: SDK, plugin host process

**Plugin**:
A trusted in-process Rust extension that consumes Host services. Initially, a
Plugin is an independently loaded ASI module.
_Avoid_: service, backend, mod host

**Service**:
An exact-version immutable C ABI function table published by the Host.
_Avoid_: extensible table, facade

**Service readiness**:
The runtime ability of a discovered Service operation to reach its native
backend. Discovery of a Service does not imply Service readiness.
_Avoid_: service existence, supported version

**Safe facade**:
The plugin-side Rust API that wraps Services with owned values, checked IDs,
and lifecycle guards.
_Avoid_: service table, native API

**Native backend**:
A Host-only, target-specific implementation that reads or mutates GTA SA or
SA-MP state.
_Avoid_: facade, plugin API

**Native profile**:
The identity of one exact target binary plus its evidence-graded native
contracts. A product version label alone does not identify a Native profile.
_Avoid_: game version, supported version

**GTA pool reference**:
A positive signed 32-bit `GTAREF` token for a `CPool`-backed entity. For the
supported ped, vehicle, and object pools, it encodes a pool slot and a 7-bit
reuse generation. It is neither a native pointer nor a SA-MP pool ID.
_Avoid_: pointer, entity ID, unsigned handle

**GTA pickup reference**:
A positive signed GTA pickup token stored by the SA-MP pickup pool. It is a
distinct handle category, not an `ObjectHandle`, and must use pickup-specific
validation instead of generic `CPool<CObject>` generation rules.
_Avoid_: object handle, pickup ID, object pointer

**SA-MP pool ID**:
An unsigned index into one SA-MP pool, validated against the active Native
profile's bound. It does not identify a GTA object without a separate mapping.
_Avoid_: GTA reference, handle

**Current pool mapping**:
The SF/MoonLoader-compatible association observed in the active SA-MP pool at
the instant a game-thread operation executes. It is neither a persistent
identity relation nor a promise that the mapped ID or handle remains live after
the call.
_Avoid_: cached mapping, stable association, snapshot mapping

**Recognized profile**:
A Native profile whose exact binary identity matches the loaded target. Profile
recognition does not imply that every Capability is available.
_Avoid_: supported build, ready profile

**Capability**:
One feature-level operation or read model available for a Recognized profile
with sufficient native evidence and runtime readiness.
_Avoid_: profile support, whole-version support

**Evidence register**:
The authoritative feature-level record of native facts, provenance, and
evidence grades. A smoke report changes only the facts that it explicitly
verifies.
_Avoid_: smoke log, profile source file

**Deferred reclamation**:
Non-blocking Subscription cleanup that invokes a plugin-provided release
callback only after the Host has drained all in-flight callbacks.
_Avoid_: unregister, immediate drop

**Host work sequence**:
The global FIFO order assigned to off-thread native reads and mutations before
they enter one Game-thread command snapshot.
_Avoid_: cache refresh order, service queue

**Game thread**:
The operating-system thread observed executing `CGame::Process`. A callback is
not assumed to run on the Game thread unless the Host validates it at runtime.
_Avoid_: main thread, callback thread

**Game context**:
A callback-scoped safe capability proving that the callback currently executes
on the Game thread. It does not by itself prove a specific engine phase.
_Avoid_: game pointer, tick snapshot, thread-safe callback

**Native execution constraint**:
The verified thread and engine-phase requirement for one native operation, such
as any Game-thread phase, post-game-process only, render phase only, or queued
only.
_Avoid_: callback safe

**Owned snapshot**:
An owned compound read result that can outlive a Game context or cross a thread
boundary. It is a value, not a synchronization mechanism or necessarily a
cached value.
_Avoid_: live view, cache, native reference

**Persistent cache**:
An optional measured optimization that publishes owned native state for later
reads. It is not the semantic foundation of safe native access.
_Avoid_: snapshot, synchronization

### Protocol boundaries

**Protocol bitstream**:
An owned, platform-independent exact-bit SA-MP/RakNet wire value used by
Protocol codecs. It has no Native allocation or callback-lifetime semantics.
_Avoid_: Host stream, callback event, RawBitStream

**Host transport stream**:
Host-owned mutable exact-bit state that bridges Native RakNet processing and
synchronous callback dispatch. It is not the public Protocol bitstream.
_Avoid_: Protocol bitstream, owned packet value

**Protocol codec**:
A transport-neutral transformation between owned protocol values and the
project's bit-reader/bit-writer contracts.
_Avoid_: callback handler, HostApi adapter

**Wire descriptor**:
The Protocol-owned identity, codec, and trailing-bit policy for one packet or
RPC wire message. It contains no callback registration or action behavior.
_Avoid_: callback descriptor, event handler

**Callback descriptor**:
The SDK-owned adapter that connects a Wire descriptor to callback lifetime,
handler dispatch, replacement, and `Continue`/`Block` policy.
_Avoid_: Wire descriptor, Protocol codec

**Encoded-string codec**:
An injected contract for SA-MP RakNet compressed strings whose Native
implementation remains outside the platform-independent protocol layer.
_Avoid_: UTF-8 codec, Native compressor

**Encoded bits**:
A cursor-free owned wire value containing left-aligned bytes and their exact
meaningful bit length. Its byte storage is minimal and unused low bits are zero.
_Avoid_: BitStream, byte buffer, callback payload

**Message name catalog**:
A non-exhaustive Packet- or RPC-specific mapping from a raw wire ID to a known
diagnostic name. An unknown ID remains a valid raw value.
_Avoid_: message enum, Wire descriptor registry

### Native values

**ARGB colour**:
The public and cache colour value with alpha in the most significant byte.
_Avoid_: raw colour, native colour

**Native RGBA colour**:
The colour representation accepted by the SA-MP native text-label create
method. Native text-label records store ARGB.
_Avoid_: ARGB, public colour
