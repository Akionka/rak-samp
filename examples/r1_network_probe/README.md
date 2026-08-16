# R1 one-pass live validation probe

This opt-in ASI validates the SA-MP 0.3.7 R1 direct profile only against the
disposable loopback server in `server`. Never copy it into, launch it from, or
modify the main GTA installation.

The probe shares the common R1/R3/R5/DL validator engine. Its R1 configuration
checks the PE entry RVA `0x31DF13`, the R1 game states `15` (awaiting join) and
`14` (connected), and writes one bounded status file:
`samp-client-sdk-r1-network-probe.status`.

The connected pass covers host identity, SAMPFUNCS coexistence when present,
codec round-trip, blocked incoming packet/RPC payloads of exactly three bits,
server/local/player caches, remote NPC state, all sync snapshots, UI,
entities/handles, local mutations, force-sync paths, vehicle phases, and the
explicit R1 opaque raw addresses. The reconnect pass checks cache and raw
connection-state invalidation, restored spawned state, and post-reconnect
incoming delivery.

## Prepare a disposable environment

1. Create separate game and server roots. Do not use the main GTA directory.
2. Copy the pinned R1 `samp.dll` to the disposable game root. Verify it before
   launching:

   ```powershell
   (Get-FileHash (Join-Path $r1GameRoot 'samp.dll') -Algorithm SHA256).Hash
   ```

   It must equal:

   ```text
   7E30F3C9CD99D5E2932410F486E8139AFFA2DAD19BD65AD9C328F6A4071943F7
   ```

   Stop if the hash differs.
3. Build the host and the R1 probe from the repository root:

   ```powershell
   cargo build --release -p samp-client-sdk-host
   cargo build --release -p samp-client-sdk-r1-network-probe
   ```

4. Copy only these files to the disposable game root and rename them to `.asi`:

   ```text
   target\i686-pc-windows-msvc\release\samp_client_sdk.dll
   target\i686-pc-windows-msvc\release\samp_client_sdk_r1_network_probe.dll
   ```

5. Compile `server\r1_network_probe.pwn` and
   `server\npcmodes\r1_probe_bot.pwn` with the disposable server package's
   `pawncc`. Put the generated `.amx` files in `filterscripts` and `npcmodes`.
6. Copy `server.cfg` and `start_probe.cmd`, plus `samp-server.exe` and
   `samp-npc.exe`, into the disposable server root. Run `start_probe.cmd`.
   It starts `R1ProbeBot` on `127.0.0.1:7777`; do not use `ConnectNPC`.

## Run once

1. Connect one disposable R1 client to `127.0.0.1:7777` and spawn it.
2. Do not interfere while the probe drives UI, dialogs, entities, and vehicle
   phases. A green `R1_SDK_INCOMING_20260816` message must appear.
3. Wait for the connected pass:

   ```text
   status=0x0FFFFFFF
   failure=0
   codec_round_trip=true
   incoming_packet_bits=3
   incoming_rpc_bits=3
   ```

4. Type `/r1sdkreconnect`, spawn again if class selection appears, and wait for:

   ```text
   status=0x3FFFFFFF
   failure=0
   reconnect_server_ready=true
   reconnect_local_ready=true
   reconnect_game_state=Some(14)
   reconnect_spawned=Some(true)
   reconnect_incoming_ready=true
   codec_round_trip=true
   incoming_packet_bits=3
   incoming_rpc_bits=3
   ```

Keep the status file, server log, and host log on any failure. `failure=1` is
`NotReady`; `failure=2` is `TimedOut`. Do not claim a pass until every field
above is present. A loader without hot unload is acceptable: lifecycle tests
cover owned hook/vtable cleanup, and no hot-unload pass is claimed.
