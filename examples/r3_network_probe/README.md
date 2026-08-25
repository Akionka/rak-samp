# R3-1 full live validation probe

This opt-in ASI validates the R3-1 host profile only against the disposable
loopback server in this directory. It never targets a public server.

The connected pass validates:

- host/version/PE identity, network transport, codec hooks, and outgoing sync
  packet observation;
- `CNetGame`, local player, the server-managed remote NPC, pool values, local
  sync snapshots, remote on-foot sync, and GTA handle round trips;
- chat, death window, chat input, cursor, scoreboard, native chat commands,
  animation lookup, and full client-side dialog state/response capture;
- local-player colour/action, all send-rate writes, and every force-sync path;
- text label and textdraw create/read/update/delete lifecycles;
- controlled local driver, passenger, trailer, and unoccupied vehicle states.

The final opt-in phase validates disconnect cache invalidation, reconnect cache
restoration, and packet/RPC delivery after reconnect. Public R3 unsafe/raw
addresses are outside this probe because that API remains intentionally R1-only.

## Build and install

1. Close the isolated R3 GTA client.
2. Build the host and probe from the repository root:

   ```powershell
   cargo build --release -p samp-client-sdk-host
   cargo build --release -p samp-client-sdk-r3-network-probe
   ```

3. Copy these files into the isolated R3 GTA directory and use the `.asi`
   extension:

   ```text
   D:\cargo-build\i686-pc-windows-msvc\release\samp_client_sdk.dll
   D:\cargo-build\i686-pc-windows-msvc\release\samp_client_sdk_r3_network_probe.dll
   ```

4. Compile `server\r3_network_probe.pwn` and
   `server\npcmodes\r3_probe_bot.pwn` with the server package's `pawncc`.
   Put the outputs in `filterscripts` and `npcmodes`, respectively.
5. Run `server\start_probe.cmd`. It starts `samp-server.exe`, waits for server
   initialization, and then starts the headless `samp-npc.exe`. Direct
   `ConnectNPC` is not used because the supplied R2 server blocks when it
   launches the child during filter-script initialization. The fixture listens
   on `127.0.0.1:7777`.

## One complete test run

1. Start one R3 client and connect to `127.0.0.1:7777`. The fixture launcher
   starts and moves `R3ProbeBot` automatically.
2. Spawn the player. The probe drives scoreboard, chat input, dialogs, and
   vehicle states without timed keyboard input.
3. Do not interfere while the probe opens and closes dialogs and moves the
   player through the controlled vehicle states. The green
   `R3_SDK_INCOMING_20260812` message and the local UI/death messages must be
   visible.
4. Wait for the connected-pass status:

   ```text
   status=0x0FFFFFFF
   failure=0
   ```

5. Type `/r3sdkreconnect`. The probe disconnects and
   reconnects to `127.0.0.1:7777`. Spawn the primary player again if the server
   shows the class-selection screen.
6. Wait for the final status and confirm that a second green incoming marker
   appears:

   ```text
   status=0x3FFFFFFF
   failure=0
   game_state=15
   address_hex=3132372E302E302E31
   hostname_hex=53412D4D50
   port=7777
   reconnect_server_ready=true
   reconnect_local_ready=true
   reconnect_game_state=Some(5)
   reconnect_spawned=Some(true)
   reconnect_incoming_ready=true
   ```

The ASI writes `samp-client-sdk-r3-network-probe.status` in GTA's working
directory. The server console must contain `R3_OUTBOUND_OK`,
`R3_INCOMING_SENT`, and `R3_ENTITIES_SENT`. Vehicle transitions use hidden
chat markers and are cleaned up before reconnect.

If `status` includes `0x80000000`, `failure` contains the first SDK result:
`1` is `NotReady`, `2` is `TimedOut`, and other nonzero values indicate a
native operation failure. Keep both clients, the status file, the host log,
and the server log intact for diagnosis.

Stock `samp-npc.exe` produces deterministic remote on-foot sync. It cannot
emit aim, passenger, or standalone trailer packets. Those packet paths are
validated on the local player; native fixtures cover the remote layouts.

## Optional hot-unload check

Ordinary ASI loaders keep plugins loaded until GTA exits. Windows then reclaims
the process address space, so no explicit teardown is required.

Only a loader that supports hot unload must call
`SampClientSdkR3NetworkProbe_Shutdown` from its worker thread before
`FreeLibrary`. The export must return nonzero. Do not call `FreeLibrary` from
`DllMain` or from an SDK callback.
