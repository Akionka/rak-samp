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
