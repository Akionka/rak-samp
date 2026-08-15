# R3-1 full live validation probe

This opt-in ASI validates the R3-1 host profile only against the disposable
loopback server in this directory. It never targets a public server.

The connected pass validates:

- host/version/PE identity, network transport, codec hooks, and outgoing sync
  packet observation;
- `CNetGame`, local and remote players, pool values, all sync snapshots, and
  GTA handle round trips;
- chat, death window, chat input, cursor, scoreboard, native chat commands,
  animation lookup, and full client-side dialog state/response capture;
- local-player colour/action, all send-rate writes, and every force-sync path;
- text label and textdraw create/read/update/delete lifecycles;
- controlled local and remote driver, passenger, trailer, and unoccupied
  vehicle states.

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

4. Compile `server\r3_network_probe.pwn` with the server package's `pawncc`.
   Put `r3_network_probe.amx` in `server\filterscripts`.
5. Start `server\samp-server.exe`. The fixture listens on
   `127.0.0.1:7777`.

## One complete test run

1. Start two R3 clients and connect both to `127.0.0.1:7777`. Only the primary
   client needs the probe ASI.
2. Spawn both players. On the second client, move and aim for several seconds
   while the primary probe collects remote on-foot and aim snapshots.
3. On the primary client, hold and release `Tab` once.
4. Open chat with `T`, type `R3_SDK_TEXT_CACHE_20260812`, and leave the text in
   the edit box without sending it.
5. Do not interfere while the probe opens and closes dialogs and moves both
   players through the controlled vehicle states. The green
   `R3_SDK_INCOMING_20260812` message and the local UI/death messages must be
   visible.
6. Wait for the connected-pass status:

   ```text
   status=0x0FFFFFFF
   failure=0
   ```

7. Type `/r3sdkreconnect` in the primary client. The probe disconnects and
   reconnects to `127.0.0.1:7777`. Spawn the primary player again if the server
   shows the class-selection screen.
8. Wait for the final status and confirm that a second green incoming marker
   appears:

   ```text
   status=0x3FFFFFFF
   failure=0
   game_state=6
   address_hex=3132372E302E302E31
   hostname_hex=53412D4D50
   port=7777
   ```

The ASI writes `samp-client-sdk-r3-network-probe.status` in GTA's working
directory. The server console must contain `R3_OUTBOUND_OK`,
`R3_INCOMING_SENT`, and `R3_ENTITIES_SENT`. Vehicle transitions use hidden
chat markers and are cleaned up before reconnect.

If `status` includes `0x80000000`, `failure` contains the first SDK result:
`1` is `NotReady`, `2` is `TimedOut`, and other nonzero values indicate a
native operation failure. Keep both clients, the status file, the host log,
and the server log intact for diagnosis.

## Unload and hook-restoration check

After `0x3FFFFFFF`, call `SampClientSdkR3NetworkProbe_Shutdown` from the loader's
worker thread before unloading the probe. It must return nonzero. Close the
isolated client and verify in the host log that the owned MinHook targets and
RakClient vtable slots were restored. Do not call `FreeLibrary` from `DllMain`
or from an SDK callback.
