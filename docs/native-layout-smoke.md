# Native-layout live smoke checklists

These checklists gate direct native helpers for each SA-MP build. The minimum
C++ layout fixture covers `CNetGame`, chat input, and dialog state; it is a
starting point, not proof that every native helper is safe. A profile remains
disabled until its needed layout families in [TODO.md](../TODO.md) and every
applicable item below have passed on the exact pinned binary.

Do not commit client binaries, SAMPFUNCS binaries, or vendor headers. Record
the date, host commit, GTA build, test server, result, and any observed crash
or log line in the local test notes for the matching binary hash.

## Common procedure

Run the x86 workspace tests first; they compile the independent C++ fixture in
`tests/fixtures/raknet_layout.cpp`. Then place the matching `samp.dll` in a
GTA SA 1.0 US installation, deploy the host, and test on an isolated server.
Keep the current direct-native profile disabled while running a checklist for a
new build. Stop immediately on a failed pointer/range check, access violation,
hook restoration failure, or mismatched binary identity.

Every checklist below is intentionally unchecked: its existence does not mean
the live validation has run.

## Network smoke plugin

The opt-in [network smoke plugin](../examples/network_smoke_plugin) provides a
repeatable safe first pass for the network-only `AddressSet` on an isolated
server. It round-trips the native codec and queues a fixed three-bit packet;
its incoming listener blocks that packet before SA-MP can process it. It also
dispatches and blocks a fixed three-bit RPC. A complete `0x7F` status therefore
proves the codec, packet allocation/queue lock, incoming-packet hook, and
exact-bit callback handling worked together for that run.

When SAMPFUNCS is absent, inspect the worker-written
`samp-client-sdk-network-smoke.status` in the LIVE game root. `status=0x0000007F`
and `failure=0` is the required passing result; the high status bit marks a
failure and the `failure` line records the first SDK result code.

It deliberately does **not** send traffic to the server or enter SA-MP through
the original RPC handler. Those are separate checklist obligations and must not
be inferred from a passing smoke status.

## Initial isolated attach observations (2026-08-12)

These are partial observations from the release host in the isolated
`C:\Games\GTASA-SDK-LIVE-TEST` root against `127.0.0.1:7777`. The test root
used the pinned DLL for each launch; the regular `C:\Games\GTASA` installation
was not changed. They prove only that the selected inline-hook RVAs can be
installed and, where noted, invoked. They do **not** complete any checklist
below: no packet/RPC send, bitstream lock/unlock, codec round-trip, reconnect,
or hook-restoration test ran.

| Build | Observed result | Status |
| --- | --- | --- |
| R3-1 | Host attached; `CGame::Process`, RakClient constructor, and `HandleRPCPacket` hooks became ready. The first incoming packet was valid (`19` bytes, `152` bits). | Partial pass |
| R5-1 | The corrected static `CGame::Process` hook entered; the codec plus blocked exact-bit packet/RPC smoke passed (`0x0000007F`, `failure=0`); and a loopback outbound RPC was acknowledged by the server and surfaced through both the typed callback and SA-MP's normal chat handler. | Partial pass: broader layout, reconnect, and hook-restoration proof remain |
| DL R1 | Host attached; constructor and `HandleRPCPacket` hooks became ready. The first incoming packet was valid (`18` bytes, `144` bits), then the client exited. | Partial pass |

## R5-1 network smoke observation (2026-08-12)

The isolated R5-1 client was launched through the test-root `samp_debug.exe`
against `127.0.0.1:7777`. The original game-process target (`0x53E4B0`) was
mid-function code; the host now hooks the verified static GTA SA 1.0 US
`CGame::Process` entry at `0x53BEE0` with its zero-argument ABI. The corrected
detour entered before RakClient readiness and drained the game-command queue.

The smoke plugin registered its self-blocking packet/RPC listeners and completed
the native string-codec round trip. A real inbound RPC then captured the native
receiver required for packet emulation. The plugin waited on the host's
pointer-free readiness scalar, submitted exactly one packet-emulation command,
and recorded `status=0x0000007F`, `failure=0`. This proves the selected R5
codec, packet allocation/queue lock, incoming-packet hook, and exact-bit blocked
packet/RPC callback paths together on the pinned client.

The smoke deliberately does not send traffic to the server or invoke SA-MP's
original incoming-RPC handler. Those obligations are covered separately below.

## R5-1 loopback delivery observation (2026-08-12)

The opt-in [R5 network probe](../examples/r5_network_probe) ran against the
disposable `C:\\Games\\SAMP-R5-LOOPBACK-PROBE` server at `127.0.0.1:7777`.
After the host captured a real incoming-RPC receiver, the probe sent exactly
one fixed chat RPC. The server filter logged
`R5_OUTBOUND_OK playerid=0`, sent the fixed green response, and logged
`R5_INCOMING_SENT playerid=0`.

The plugin recorded `status=0x0000001F`, `failure=0`: host connected, typed
RPC subscription installed, receiver captured, outgoing command receipt
succeeded, and the matching typed incoming callback ran. The operator also
visually confirmed `R5_SDK_INCOMING_20260812` in the normal in-game chat. This
proves the selected R5 outbound RPC route and that the SDK's typed listener
continued to SA-MP's original incoming-RPC handler for this message. It does
not replace the remaining full network, layout, reconnect, or unload checks.

## SA-MP 0.3.7 R1

Pinned artifact: installed `samp.dll`, SHA-256
`7E30F3C9CD99D5E2932410F486E8139AFFA2DAD19BD65AD9C328F6A4071943F7`.

- [ ] Confirm the PE entry-point RVA is `0x31DF13` and the logged fingerprint
  matches the pinned hash.
- [ ] Attach, join an isolated server, and verify constructor/RPC hooks and
  RakClient vtable restoration on unload.
- [ ] Exercise packet/RPC send, receive emulation, string codec, and exact-bit
  read/write paths.
- [ ] Exercise the existing R1 cache, dialog, chat, command, player/pool,
  textdraw, text-label, and local-sync helpers.
- [ ] Disconnect, reconnect, unload the host, and confirm no stale callback,
  vtable patch, or native pointer remains.

## SA-MP 0.3.7 R3-1

Pinned artifact: `sa-mp-0.3.7-R3-1-install.exe` → `samp.dll`, SHA-256
`9C9B2CC31A4CED6967420B1880C096B5C4E7630E227AA379BE4019C21B6FDDC1`.

- [ ] Confirm the PE entry-point RVA is `0x0CC4D0` and the logged fingerprint
  matches the pinned hash.
- [ ] Verify the network-only `AddressSet`: constructor hook, inbound RPC,
  packet allocation, bitstream lock/unlock, and string codec round-trip.
- [ ] Prove the profile's `CNetGame`, `CInput`, and `CDialog` values against
  the fixture before enabling any helper that consumes them.
- [ ] For each newly enabled UI, cache, pool, player, or sync helper, validate
  its complete layout family and run the corresponding in-game interaction.
- [ ] Disconnect, reconnect, unload the host, and confirm all hooks restore.

## SA-MP 0.3.7 R5-1

Pinned artifact: `sa-mp-0.3.7-R5-1-install.exe` → `samp.dll`, SHA-256
`B72B5DBE725F81864CA3F78BC7063BDA56CC05FC7188AF822FA7A754432553A2`.

- [ ] Confirm the PE entry-point RVA is `0x0CBC90` and the logged fingerprint
  matches the pinned hash.
- [ ] Verify the network-only `AddressSet`: constructor hook, inbound RPC,
  packet allocation, bitstream lock/unlock, and string codec round-trip.
- [ ] Prove the profile's `CNetGame`, `CInput`, and `CDialog` values against
  the fixture before enabling any helper that consumes them.
- [ ] For each newly enabled UI, cache, pool, player, or sync helper, validate
  its complete layout family and run the corresponding in-game interaction.
- [ ] Disconnect, reconnect, unload the host, and confirm all hooks restore.

## SA-MP 0.3.DL R1

Pinned artifact: `sa-mp-0.3.DL-R1-install.exe` → `samp.dll`, SHA-256
`BCCDB297464BD382625635BE25585DF07A8FA6668BC0015650708E3EB4FFCD4B`.

- [ ] Confirm the PE entry-point RVA is `0x0FDB60` and the logged fingerprint
  matches the pinned hash.
- [ ] Verify the network-only `AddressSet`: constructor hook, inbound RPC,
  packet allocation, bitstream lock/unlock, and string codec round-trip.
- [ ] Prove the profile's `stSAMP`, `stInputInfo`, and `stDialogInfo` values
  against the fixture before enabling any helper that consumes them.
- [ ] Obtain or disassemble evidence for every additional DL layout family;
  do not reuse a non-DL layout merely because a field name matches.
- [ ] Disconnect, reconnect, unload the host, and confirm all hooks restore.
