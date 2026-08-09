# Core Features

`samp-client-sdk` has two pillars:

- `sdk/` is the public Rust package imported as `samp_client_sdk` by ASI
  plugins.
- `samp-client-sdk-host` owns the Windows x86 bridge and produces
  `samp_client_sdk.asi`.

## Runtime and ABI

The host bootstraps outside `DllMain`, waits for `samp.dll`, selects a
recognized networking offset table, and then publishes attached and ready
state. Lifecycle and ABI diagnostics go to `samp-client-sdk.log`; packet and
RPC payloads are never logged.

`SampClientSdkApiV1` is the C-compatible ABI table exported as
`SampClientSdk_GetApiV1`. It carries only copied data, fixed storage, explicit
capacities, and function pointers. Rust references, trait objects, heap
allocations, and native pointers never cross this boundary.

Listeners run in registration order. They may continue, block, or atomically
replace exact-bit packet/RPC payloads. Emulated incoming traffic crosses
incoming listeners exactly once, and nested same-thread dispatch remains
non-blocking. Explicit sends bypass outgoing listeners.

Plugins connect through `Samp`, which exposes `net`, `server`, `local`, player
and pool views, UI views, checked SA-MP IDs, and command receipts. The
underlying ABI wrapper is documentation-hidden and remains an internal bridge.
`Samp::probe()` exposes owned loaded, recognized-build, and ready predicates
without exposing a module address or reading client memory.
The explicit unsafe `raw` module also exposes documented R1 singleton and pool
addresses plus host-captured RakClient, player-pool, and vehicle-pool
interfaces as opaque pointers. Its bounded RakClient-vtable accessor returns
opaque code addresses without constructing Rust references.
The explicit `raw` module exposes opaque native addresses only through unsafe
functions; it never constructs Rust references to client memory.

## R1 bridge

SA-MP 0.3.7 R1 on GTA SA 1.0 US is the state and mutation bridge target. Its
approved fixed offsets are used after recognized-build selection. Each native
operation independently checks object pointers, readable ranges, IDs,
capacities, and enum values; invalid or not-yet-ready state is reported as an
error rather than exposed to plugin threads.

Safe reads copy host-owned snapshots. Local UI requests now enter one bounded
256-entry, fully owned `GameCommand` queue with FIFO tick-snapshot semantics.
The ABI exposes typed dialog/chat/death submissions plus fixed `repr(C)`
receipt/result storage, poll, timed wait, and release. Receipt drops detach
waiters without cancelling their copied commands; timeouts remain retryable and
shutdown completes every retained receipt. The owned GTA SA 1.0 US
`CGame::Process` detour at `0x53E4B0` snapshots accepted commands before the
native process call, invokes that original exactly once, then drains the
snapshot and refreshes caches. Incoming-packet detours handle networking only.
Waits are rejected on the game thread and inside listener callbacks.
Explicit packet/RPC sends and emulation likewise copy into that queue, so no
plugin thread invokes RakClient directly. The public `Net` facade returns a
`CommandReceipt<()>` for those operations, so plugins can poll or wait for the
later native completion. R1 local-player spawning, established special actions,
local- and connected-remote player colour updates, bounded local nickname
updates, unoccupied vehicle synchronization, R1 reconnect/disconnect control,
and textdraw-pool deletion, bounded display-string, box, position, letter-style,
proportional, shadow, outline, or three-way alignment updates use that same
queue and receipt path; their inputs are validated before the native call or
scalar write.
Documented R1 3D text-label deletion likewise uses a checked label ID, a
validated native pool, and the same queued receipt path.
Cached R1 textdraw records also copy their fixed, NUL-terminated display
strings before exposing owned bytes through the SDK.
Chat-history entry replacements use fixed source-backed text and prefix limits
and execute only through the game-thread receipt queue.
Chat-history reads copy the same fixed fields through a bounded game-thread
cache before returning owned text, prefix, and colours.
3D text-label creation validates finite geometry and bounded text, then calls
the documented R1 label-pool create method through the receipt queue.
Finite textdraw model rotation, zoom, and vehicle-colour updates use the
same receipt-bearing game-thread queue after fixture-backed range validation.
The active dialog's list-item count is copied as a non-negative scalar from
the validated DXUT listbox; no native item container crosses the ABI.
Active dialog list selections are copied from the game-thread snapshot and can
be updated through the same receipt-bearing queue after listbox validation.
The active dialog snapshot also carries bounded copies of the dialog body
text, the DXUT editbox text, and the validated listbox item strings; the
editbox text can be replaced through a bounded receipt-bearing mutation.
`Cursor::toggle` uses the native R1 cursor transition and re-enables input when
hiding the cursor, rather than writing the cursor-mode field directly. The
R1 chat display mode is likewise changed only by a queued, validated
`CChat::m_nMode` write. The
active R1 dialog can be closed through its two validated native response
buttons on that same queue. The
chat-input facade copies bounded NUL-free text from the game-thread cache and
uses the native R1 DXUT edit-box and `CInput` transitions for text updates,
enablement, and command processing. The
unsafe raw tier also exposes the validated R1 RakPeer base derived from the
captured RakClient interface; it remains an opaque address with no Rust
reference or lifetime guarantee beyond the live client. The
documentation-hidden legacy ABI calls continue
to report submission acceptance only.
The `Net` facade applies the same receipt semantics to typed chat, RPC, and
sync-packet helpers, after locally encoding their bounded protocol values.
The R1 cursor-mode setter, scoreboard toggle, and dialog client-side setter are
likewise queued and validate their input plus writable fixture-backed game
fields before changing them; the dialog bridge explicitly maps the inverse
native `server_side` flag.
Game-state writes accept only the established R1 `CNetGame` states and use the
fixture-backed `CNetGame` scalar from that same queue.
`raw::server_settings` is an explicitly unsafe, opaque R1 pointer; its packed
`CNetGame` offset is independently asserted by the C++ fixture.
The pickup-pool raw pointer follows the same opaque-lifetime rule and its pool
slot is likewise fixture-checked.
`raw::player` is captured from the validated R1 local-player lookup on the
game thread and is cleared on connection transitions and shutdown.
Pool handle reads (objects, pickups, vehicles, and player peds) are cached on
the game thread through bounded, deduplicated request queues: ID→handle reads
drain per-ID requests, and handle→ID reads drain bounded scans of the matching
pool. Vehicle and ped handles convert validated GTA SA pointers through the
fixed `CPools::GetVehicleRef`/`GetPedRef` targets, and the facade exposes the
results as typed non-null GTA handle newtypes.
`raw::bitstream_data` exposes only a lifetime-bound pointer to the SDK-owned
bounded bit-stream storage; it is not a native RakNet allocation.
Local facade protocol actions reuse the exact receipt-bearing `Net` path and
make no claim to perform a corresponding local GTA state transition.
Each R1 game tick brackets cache refresh with a monotonic generation and
invalidates all connection-bound entity directories and pending heavy refreshes
when SA-MP crosses a connection boundary.

## Native boundary and tests

The Windows backend owns native addresses, vtable patches, inline detours, and
RakNet calls. It restores only the vtable slots and detours it owns, and calls
original hook targets through captured backend state.

`tests/fixtures/raknet_layout.cpp`, compiled by `build.rs` for Windows x86,
remains the independent C++↔Rust layout oracle. It includes the full native
`DXUTComboBoxItem` signature (256-byte text, `void*` data, windef `RECT`
active rectangle, and visibility flag) using the real `windows.h` `RECT` and
default alignment, matching the pinned SF.lua declaration. Unit tests preserve
exact packet/RPC vectors, exact-bit replacement, listener ordering,
subscription shutdown, and layout coverage.

Before a runtime plugin unload, remove every subscription with
`unregister_and_wait` from a worker thread. Do not wait in `DllMain`, a
callback, or the game tick, and do not free a plugin while callbacks can run.
