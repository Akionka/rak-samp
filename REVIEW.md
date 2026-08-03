# Review Evidence

Keep only unresolved findings and evidence that defines a native boundary here.
Implementation history belongs in Git; planned work belongs in [TODO.md](TODO.md).

## Open findings

- **Direct R1 profile live gate:** the R1 dialog/snapshot profile is
  fail-closed behind SA-MP and GTA PE/code fingerprints, and its narrow packed
  offsets have an independent fixture. A legal GTA SA 1.0 US + SA-MP 0.3.7 R1
  run still must record stable dialog dismissal, walking/damage/vehicle
  snapshot changes, zero direct-dialog RPC 61/62 observations, and normal
  shutdown before release.

## Windows x86 evidence

- **RakNet packet layout:** `RawPacket` and its embedded `PacketPlayerId` use
  packed offsets. The by-value incoming-RPC `RpcPlayerId` is a distinct aligned
  layout. The independent C++ fixture and an SA-MP 0.3.7 R1 live run support
  this split; do not infer native packing from serialized sizes.
- **Incoming queue:** packet emulation uses the RakPeer receiver captured by
  the incoming-RPC detour, not RakClient pointer arithmetic. R1 loopback
  observed emulated packet and RPC events once each through normal dispatch.
- **Encoded strings:** the host calls the detected client's StringCompressor
  reader and writer as x86 `thiscall` functions. An R1 live test encoded,
  decoded, replaced, and blocked a private dialog without instability.
- **Hook and unload checks:** fixture tests cover owned-slot restoration and
  original calls. An R1 validation run completed callback quiescence before an
  external manager released the validation ASI.
- **Direct R1 profile:** `CDialog::Show` is called as x86 `thiscall` with
  `serverside = false`; the dialog, net-game, player-pool, local-player, and
  ped call/field leads are isolated in `src/platform/win32/r1_client.rs`.
  The R1 source leads are pinned to
  [`blasthacknet/samp-api@6d4db99`](https://github.com/blasthacknet/samp-api/tree/6d4db99ab41f19d1a6a7c6cd48f5878bd1e14b62/include/sampapi/0.3.7-R1).
  They are not treated as live authority: the profile additionally checks the
  R1 PE timestamp/entry point, dialog target signature, GTA PE identity/code,
  and readable pointers before use.
- **Direct R1 dialog signature:** the installed supported binaries identify
  GTA SA 1.0 US as image base `0x00400000`, image size `0x01177000`, and entry
  RVA `0x00424570`, and SA-MP R1 as timestamp `0x5542F47A` and entry RVA
  `0x0031DF13`. At `samp.dll + 0x6B9C0`, `CDialog::Show` begins
  `83 EC 10 53 56 57 8B 7C 24 20 33 DB 3B FB 8B F1`; this exact signature is
  the profile gate. The earlier `55 8B EC` check rejected this valid target
  and made direct helpers return `UnsupportedVersion`.
- **Cached R1 game state:** `CNetGame::GetGameState` at `samp.dll + 0x2E20`
  begins `8B 81 BD 03 00 00 C3` (`mov eax, [ecx + 0x3BD]; ret`) in the
  fingerprinted R1 target. The profile verifies that exact seven-byte signature
  before publishing. The host invokes it only from the incoming-packet game
  thread and copies its `i32` return into an atomic cache. The ABI deliberately
  exposes the result as an opaque scalar: no R1 enum naming, native pointer,
  or synchronous client call crosses the boundary.
- **Direct snapshot layout:** the independent C++ fixture asserts R1 packed
  on-foot (68-byte), in-car (63-byte), and local-player-prefix (92-byte)
  boundaries used to obtain sync position/velocity, special action, animation,
  and vehicle fields. It also asserts `CPed::m_pGamePed` at `0x2A4`, used to
  validate the ped before health/armour calls, and `CPlayerPool`'s local-player
  ID at `0x4`. The R1 source's `CPlayerPool::Find` matches pooled peds and is
  not a local-ID lookup; using it left the local snapshot at `0xFFFF` after
  connection. The direct profile copies name, colour, score, ping, and all
  snapshot fields into host-owned storage; no pointer reaches the ABI.
- **Direct snapshot assignment:** early validation runs showed that the
  player-pool ID alone is insufficient: it can be `0xFFFF` (`65535`) or an
  initial zero before the server's real assignment. R1 `INIT_GAME` (RPC 139)
  carries that authoritative ID, so after the original client handler accepts
  it the host privately decodes only its fixed prefix (never logs payload
  bytes) and records the scalar ID. That prefix is four bit flags, `f32`, one
  bit flag, `f32`, three bit flags, raw little-endian `i32` class count, then
  `u16` player ID. It publishes only pool snapshots that match that ID twice
  consecutively. This keeps server-assigned ID zero valid while rejecting the
  transition. A trial decoder omitted the final flag and class count, producing
  a wrong ID in live validation; its exact wire fixture now covers both fields.
  A trial gate on `CNetGame::GetState` was removed: its post-join state changes
  again during ordinary gameplay and would erase a valid assignment, leaving
  the cache permanently `NotReady`. The state accessor is now separately
  exposed as the nonblocking cached scalar above, never as a snapshot gate. The
  R1 player pool's verified pointers and the retained assignment match are the
  snapshot-readiness boundary; unavailable pools or peds clear that cache and
  report `NotReady`.
- **Direct helper live run (2026-08-03):** the fingerprinted GTA SA 1.0 US +
  SA-MP 0.3.7 R1 client enabled the profile, displayed the queued direct dialog,
  and published stable local-player ID `1`. The opt-in state check completed
  with `position_changed=true`, `health_changed=true`, `armour_changed=true`,
  and `vehicle_changed=true`. The validator's one incoming RPC 61 was its
  separate intentional RPC-emulation check; the direct dialog added no RPC 61
  observation and no outgoing RPC 61 or 62 appeared. The client then exited
  normally with the last host/validator results still passed; `gta_sa.exe` was
  absent afterwards, and no GTA/SA-MP Application Error, Windows Error
  Reporting event, or new dump file appeared in the following 15-minute check.

## Typed protocol evidence

R1 RPC and packet codecs are field-by-field serializers; they do not cast
callback memory to Rust structs. Their catalog and fixture coverage were
checked against the public [SAMP.Lua event catalog](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/events.lua),
[synchronization definitions](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/synchronization.lua),
and [SA-MP RPC list](https://github.com/Brunoo16/samp-packet-list/wiki/RPC-List).
This is not live compatibility evidence for non-R1 clients.

When native behavior changes, record the client build, the observed layout or
behavior, the supporting fixture/live result, and any remaining limitation.
