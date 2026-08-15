# R3-1 live validation probe

This opt-in ASI validates the R3-1 host profile only against the disposable
loopback server included in this directory. It never targets a public server.

It validates, in one GTA session:

- host/version/PE identity and the R3 network transport;
- cached `CNetGame`, local player, player pool, chat, cursor, scoreboard,
  chat-input, dialog, and remote-player values;
- `CDialog::Close` hook prerequisites, server dialog snapshot, and native
  local-player aim/on-foot/stats/weapons sync command receipts;
- server-created R3 object, vehicle, pickup, gangzone, and local-player ped
  handle round trips (`ID → handle → ID`);
- loopback outgoing chat, incoming server-message callback, and original SA-MP
  chat rendering.

`disconnect` / `connect` are intentionally not issued automatically: they tear
down the current session and require a separate operator-approved smoke test.

## Build and install

1. Close GTA and SA-MP.
2. Build the host and this probe from the repository root:

   ```powershell
   cargo build --release -p samp-client-sdk-host
   cargo build --release -p samp-client-sdk-r3-network-probe
   ```

3. Copy `D:\cargo-build\i686-pc-windows-msvc\release\samp_client_sdk.dll`
   and `D:\cargo-build\i686-pc-windows-msvc\release\samp_client_sdk_r3_network_probe.dll`
   into the isolated R3 GTA directory. Rename them to `.asi` if the loader
   requires that extension. Do not overwrite the normal GTA installation.
4. Compile `server\r3_network_probe.pwn` with `pawncc` from the supplied SA-MP
   server package. Put the resulting `.amx` in `server\filterscripts`.
5. Start `server\samp-server.exe` from this directory. It listens on
   `127.0.0.1:7777`.

## One complete test run

1. Start **two** isolated R3 clients and connect both to `127.0.0.1:7777`.
   Leave the second client connected; it is the remote-player directory test.
2. Spawn with the primary client. The ASI begins automatically.
3. When prompted by the sequence, hold and release `Tab` once.
4. Open chat with `T`, type `R3_SDK_TEXT_CACHE_20260812`, and leave it in the
   edit box without sending it.
5. The probe sends the remaining markers itself. Leave the server dialog open
   until the status file reports success. The primary chat must display the
   green `R3_SDK_INCOMING_20260812` message.

The ASI writes `samp-client-sdk-r3-network-probe.status` in GTA's working
directory. A complete automated run is:

```text
status=0x0007FFFF
failure=0
game_state=6
address_hex=3132372E302E302E31
hostname_hex=53412D4D50
port=7777
```

The server console must also contain `R3_OUTBOUND_OK`, `R3_INCOMING_SENT`, and
`R3_ENTITIES_SENT` for the primary player. Entity IDs are intentionally created
near that player only after the probe requests them, so they are streamed and
the handle test is deterministic.

If `status` includes `0x80000000`, `failure` contains the first SDK result:
`1` is `NotReady`, `2` is `TimedOut`, and other nonzero values are a native
operation failure. Keep both clients and the status file intact for diagnosis.

## Deliberately separate destructive smoke

After the successful status above, an operator may test R3 disconnect/reconnect
in a fresh disposable session. This is not part of the automatic ASI because it
terminates its own evidence source. Record whether the client returns to the
server without a crash, then restart the complete probe from step 1.
