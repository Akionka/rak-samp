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

These are observations from release hosts in isolated test roots against
`127.0.0.1:7777`. Each root used the pinned DLL for its launch; the regular
`C:\Games\GTASA` installation was not changed. The table summarizes the
evidence below; none of these observations completes the broader layout,
reconnect, or hook-restoration checklists.

| Build | Observed result | Status |
| --- | --- | --- |
| R3-1 | The host reported the public R3-1 identity and matching module PE entry; the codec plus blocked exact-bit packet/RPC smoke passed; copied game/server/local-player/player-pool/chat-input/chat-display/dialog-active/scoreboard-open caches passed fixture and interactive loopback checks (`0x00007FFF`, `failure=0`); and a loopback outbound RPC was acknowledged by the server and surfaced through both the typed callback and SA-MP's normal chat handler. | Partial pass: broader layout, reconnect, and hook-restoration proof remain |
| R5-1 | The corrected static `CGame::Process` hook entered; the codec plus blocked exact-bit packet/RPC smoke passed (`0x0000007F`, `failure=0`); and a loopback outbound RPC was acknowledged by the server and surfaced through both the typed callback and SA-MP's normal chat handler. | Partial pass: broader layout, reconnect, and hook-restoration proof remain |
| DL R1 | Host attached; constructor and `HandleRPCPacket` hooks became ready. The first incoming packet was valid (`18` bytes, `144` bits), then the client exited. | Partial pass |

## R3-1 network smoke observation (2026-08-12)

The pinned R3-1 `samp.dll` was installed only in
`C:\Games\GTASA-SDK-R3-LIVE-TEST`. Its SHA-256 was
`9C9B2CC31A4CED6967420B1880C096B5C4E7630E227AA379BE4019C21B6FDDC1`, and its
PE entry-point RVA was independently read as `0x0CC4D0` before launch.

The existing version-neutral network smoke registered self-blocking packet/RPC
listeners, performed the native codec round trip, waited for a real inbound RPC
to capture the receiver, then submitted one exact three-bit packet-emulation
command. The host logged the R3 constructor target `0x03B57170`, the incoming
RPC target `0x03B5A6A0`, game-tick entry, receiver capture, and successful
game-command completion. The plugin recorded `status=0x0000007F`, `failure=0`.

This proves the selected R3 network-only `AddressSet`—constructor and incoming
RPC hooks, packet allocation, bitstream lock/unlock, string codec, and blocked
exact-bit packet/RPC callbacks—on the pinned client. It sends no custom server
traffic. Outbound delivery and original-handler continuation are covered
separately below.

## R3-1 loopback delivery observation (2026-08-12)

The opt-in [R3 network probe](../examples/r3_network_probe) ran against the
disposable `C:\\Games\\SAMP-R3-LOOPBACK-PROBE` server at `127.0.0.1:7777`.
After the host captured a real incoming-RPC receiver, the probe sent exactly
one fixed chat RPC. The server filter logged `R3_OUTBOUND_OK playerid=0`, sent
the fixed green response, and logged `R3_INCOMING_SENT playerid=0`.

The extended plugin recorded `status=0x0000003F`, `failure=0`: the public host
status was `Ready`, the loaded predicate held, the public version was `R3_1`,
and the SDK's opaque `samp.dll` base parsed to PE entry-point RVA `0x0CC4D0`.
It also installed the typed RPC subscription, captured the receiver, completed
the outgoing command receipt, and observed the matching typed incoming callback.
The operator visually confirmed `R3_SDK_INCOMING_20260812` in normal in-game
chat. This proves the selected R3 runtime identity, outbound RPC route, and
that the SDK's typed listener continued to SA-MP's original incoming-RPC handler
for this message. It does not replace the remaining full network, layout,
reconnect, or unload checks.

## R3-1 CNetGame scalar-cache observation (2026-08-12)

The same isolated R3-1 client and disposable loopback filter produced
`status=0x0000007F`, `failure=0` after the profile published the copied
`Samp::game_state()` and `Samp::server().info()` cache on the game thread. The
probe recorded the live R3 values `game_state=6` (`AwaitingJoin`),
`address=127.0.0.1`, `port=7777`, and the native `CNetGame` host field
`SA-MP`. It then completed the outbound receipt and observed the matching
incoming callback; the operator visually confirmed the green
`R3_SDK_INCOMING_20260812` message.

This validates the R3 CNetGame singleton slot plus every field consumed by the
new read-only scalar slice. The native host field is deliberately not claimed
to be the server-config display hostname. Except for the narrow player-pool,
chat-input, chat-display, dialog-active, and scoreboard-open caches below, broader R3 UI, remote-player,
pool-directory, handle, raw, and mutation helpers remain unsupported.

## R3-1 local-player cache observation (2026-08-12)

The same pinned client and disposable loopback server produced
`status=0x000000FF`, `failure=0` after the public `Samp::local().player()`
cache published a valid, spawned local-player snapshot. Its fixture separately
verified the consumed `CPlayerPool` local ID, `CLocalPlayer` packed sync and
state fields, and `CPed` GTA-ped pointer offset. The probe tolerates the normal
valid pre-spawn snapshot, then requires `spawned=true` plus bounded nickname,
finite health/armour/position/velocity values, and an in-range optional vehicle
ID before it sends the loopback chat marker.

The disposable server logged `R3_OUTBOUND_OK` and `R3_INCOMING_SENT`; the
operator visually confirmed the green `R3_SDK_INCOMING_20260812` message.
This enables only the copied local-player snapshot. R3 raw player addresses,
pool/entity directory reads, sync snapshots, broader UI helpers, and
local-player mutations remain unsupported.

## R3-1 player-pool scalar observation (2026-08-12)

The same isolated client and disposable loopback produced `status=0x00001FFF`,
`failure=0` after the game-thread cache published public
`Samp::players().count(true)`, `count(false)`, and `max_id()` values. The
single-player session recorded `0`, `0`, and `Some(0)`: `GetCount` enumerates
remote `CPlayerPool` records and deliberately excludes the local player, while
the largest-ID scalar still identifies the local ID.

The independent fixture pins the packed R3 `CPlayerPool` size `0x2F3E`,
`m_nLargestId` at `0x00`, and the `CPlayerInfo` NPC flag at `0x28`. The pinned
image's `0x13670` entry was checked before the native `__thiscall` invocation.
This enables only copied count and largest-ID scalars; remote-player records,
player directory reads, raw pool addresses, handles, and every mutation remain
unsupported.

## R3-1 chat-input cache observation (2026-08-12)

With the same pinned client and disposable loopback server, the extended probe
recorded `status=0x000001FF`, `failure=0`. After the normal green
`R3_SDK_INCOMING_20260812` message, the operator opened chat with `T` and left
it open long enough for the game-thread cache to publish. The probe then read
`Samp::chat_input().is_active()` as true, found the built-in exact `quit`
command name, and confirmed that the deliberately absent
`r3_sdk_probe_missing_command` name was false. In the same session, the
operator entered but did not send `R3_SDK_TEXT_CACHE_20260812`; the probe
copied that exact bounded text from the R3 DXUT editbox cache.

The independent fixture pins the consumed R3 `CInput` fields: command-name
table offset `0x24C`, 33-byte name capacity, command-count offset `0x14DC`,
enabled-flag offset `0x14E0`, and editbox pointer offset `0x08`. The installed
R3 image's `0x84F40` `GetText` entry was also checked as the expected forwarding
thunk before the live read. This enables only copied active, bounded text, and
command-name reads. Dialog detail, chat rendering, command registration, and
every UI mutation remain R1-only.

## R3-1 dialog-active cache observation (2026-08-12)

The same probe first completed the chat-input active/name/text check, then submitted its bounded
`R3_SDK_DIALOG_REQUEST_20260812` loopback marker. The disposable server logged
`R3_DIALOG_SENT playerid=0` and displayed a normal SA-MP message-box dialog.
`failure=0` after `Samp::dialogs().is_active()` observed true from the
game-thread cache.
With that dialog left open, the probe recorded `status=0x00001FFF`,
`failure=0`, including the player-pool scalar stage above.

The fixture pins the R3 `CDialog` size `0x29D`, active-flag offset `0x28`, and
caption offset `0x40`; the live read consumes only the guarded active flag. It
does not read dialog controls or strings, hook close, capture responses, or
perform mutations. Those helpers remain R1-only.

## R3-1 scoreboard-open cache observation (2026-08-12)

The extended probe then produced `status=0x00003FFF`, `failure=0` on the same
isolated client and disposable loopback server. After the green
`R3_SDK_INCOMING_20260812` message, the operator held `Tab` until the native
scoreboard appeared and then released it. The game-thread cache observed the
public `Samp::scoreboard().is_open()` flag first as true and then as false.

The independent fixture pins packed R3 `CScoreboard` size `0x44` and the
32-bit `m_bIsEnabled` flag at offset `0x00`; the `CScoreboard` source resolves
the singleton pointer slot at image RVA `0x26E894`. The live read consumes only
that guarded flag. Scoreboard writes and every other unproven UI helper remain
R1-only.

## R3-1 chat-display cache observation (2026-08-12)

The same isolated client and disposable loopback server produced
`status=0x00007FFF`, `failure=0` after the probe automatically read the public
`Samp::chat().display_mode()` cache before sending its outbound marker. The
native result was accepted only if it was one of the documented display modes
`0`, `1`, or `2`.

The pinned R3 `CChat` source resolves its singleton pointer slot at image RVA
`0x26E8C8` and declares `GetMode` at `0x60B40` with `int __thiscall(CChat*)`.
The implementation guards the singleton pointer, invokes that accessor only on
the game thread, and publishes its scalar result. Chat display-mode writes,
history reads/writes, rendering, and all other unproven chat helpers remain
R1-only.

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

- [x] Confirm the PE entry-point RVA is `0x0CC4D0` and the recorded fingerprint
  matches the pinned hash.
- [x] Verify the network-only `AddressSet`: constructor hook, inbound RPC,
  packet allocation, bitstream lock/unlock, and string codec round-trip.
- [x] Prove loopback outbound RPC delivery and non-blocking original incoming-RPC
  handler continuation with a disposable server filter and human chat check.
- [x] Verify public host readiness/version/probe state and the opaque module base
  against the pinned R3-1 PE entry point.
- [x] Prove the minimum `CNetGame`, `CInput`, and `CDialog` values against the
  independent fixture. This is only an activation prerequisite; it does not
  enable any direct helper.
- [x] Publish and read the R3 CNetGame game-state/server scalar cache on the
  game thread, with a fixture gate and loopback observation.
- [x] Publish and read the R3 local-player snapshot on the game thread, with
  `CPlayerPool`/`CLocalPlayer`/`CPed` fixture gates and spawned loopback smoke.
- [x] Publish and read the R3 chat-input active flag, bounded text, and exact
  command lookup on the game thread, with a `CInput` fixture gate and interactive loopback smoke.
- [x] Publish and read the R3 dialog active flag on the game thread, with a
  `CDialog` fixture gate and server-dialog loopback smoke.
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
