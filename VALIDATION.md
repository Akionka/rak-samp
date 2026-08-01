# In-Game Validation

Use a legal local SA-MP installation and a server that permits client-side
plugins. The validation ASI passively observes normal traffic and never logs
payloads. Its only mutations are two private, locally emulated marker events;
they are rewritten, verified, and blocked before SA-MP or the server sees them.

## Prepare

1. Close GTA. Windows prevents replacing an ASI while the game is using it.
2. Use a clean GTA directory, or temporarily remove unrelated third-party ASIs
   so a failure can be attributed to rak-rs. Keep your ASI loader and
   `samp.dll`. In particular, remove `SAMPFUNCS.asi`, MoonLoader, CLEO, and
   other RakNet/network hooks for this first run; they change hook chaining and
   make a crash inconclusive.
3. From the repository root, deploy the host and validation plugin:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-validation
```

Confirm these files exist:

```text
<GTA_DIR>\rak_rs.asi
<GTA_DIR>\rak_rs_validation.asi
```

Delete or archive an old `rak-rs-validation.log` if you want a clean session.

## Exercise the hooks

1. Start GTA normally and connect to a server.
2. Wait at least 10 seconds after spawning.
3. Walk, drive, send a chat message, and let normal server traffic continue for
   30 seconds.
4. Press and release F5 ten times, waiting about one second between presses.
5. Disconnect cleanly and exit the game normally.

While GTA is running, counters can be watched from another PowerShell window:

```powershell
Get-Content "$env:GTA_DIR\rak-rs-validation.log" -Wait
```

## Pass criteria

- GTA reaches the server, survives all F5 presses, and exits normally.
- `rak-rs.log` contains `host runtime is ready` and six validation subscription
  registrations.
- `rak-rs-validation.log` contains `ready: six packet/RPC validation callbacks
  registered` and `self-test completed: packet=passed RPC=passed`.
- Incoming traffic produces nonzero `incoming_packet_ids` and
  `incoming_rpc_ids`; walking or driving produces nonzero outgoing sync-packet
  IDs such as `207(ID_PLAYER_SYNC)` or `200(ID_VEHICLE_SYNC)`.
- The histograms include one `254(RAK_RS_SELF_TEST)` incoming packet and one
  `255(RAK_RS_SELF_TEST)` incoming RPC.
- `null_events` and `timestamp_decode_errors` remain zero.

## Diagnose failures

- No validation log: confirm the ASI loader loaded `rak_rs_validation.asi` and
  that the x86 release DLL was copied under that name.
- `host discovery timed out`: inspect `rak-rs.log`; the host was absent, failed
  to recognize `samp.dll`, or did not reach `Ready`.
- RPCs increase but packets stay at zero: the incoming packet vtable path is not
  reaching the plugin; preserve both logs and record the exact SA-MP version.
- A self-test reports `failed`, `timed-out`, or `call-failed`: preserve both
  logs. This distinguishes rewrite/cancellation failure from a client that was
  not ready to dequeue the emulated packet.
- `rejected invalid incoming packet metadata`: preserve `length` and `bit_size`;
  the active client layout does not match the backend and traffic was passed
  through without invoking packet listeners.
- Crash or frozen counters: preserve both logs, the SA-MP version, the action
  immediately before failure, and the Windows crash address/module if shown.

Remove `rak_rs_validation.asi` after validation; it is a diagnostic plugin, not
required for normal rak-rs use.
