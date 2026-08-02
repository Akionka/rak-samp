# In-Game Validation

Use a legal SA-MP installation and a server that permits client plugins. The
validation ASI logs IDs and counters, never payloads. Its only mutations are
private local marker events that are rewritten, verified, and blocked before
SA-MP or the server receives them. Server-bound sends and coordinated shutdown
are disabled unless their marker files exist.

## Prepare

1. Close GTA.
2. For the first run, use a clean GTA directory with only the ASI loader,
   `samp.dll`, rak-rs, and the validation plugin. Other RakNet hooks make crash
   attribution unreliable.
3. Deploy from the repository root:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-validation
```

Confirm `$env:GTA_DIR` contains `rak_rs.asi` and `rak_rs_validation.asi`.
Archive an old `rak-rs-validation.log` if a clean session is needed.

For the explicit-send scenario, opt in before launching GTA:

```powershell
New-Item (Join-Path $env:GTA_DIR 'rak-rs-validation-send.enabled') -ItemType File -Force
```

This repeats one real eight-byte `ID_STATS_UPDATE` packet captured from the
client and sends one empty `RPC_UPDATE_SCORES_AND_PINGS` request. Use this only
on a server where the extra traffic is permitted.

For the coordinated-shutdown scenario, create this marker as well:

```powershell
New-Item (Join-Path $env:GTA_DIR 'rak-rs-validation-shutdown.enabled') -ItemType File -Force
```

That check stops the plugin workers and synchronizes all callbacks. It does not
call `FreeLibrary`; true ASI unload still requires an external unload manager.

## Runtime unload scenario

Run this as a separate session with GTA closed. Deploy the external manager and
enable its marker:

```powershell
cargo make deploy-validation-unload
Remove-Item (Join-Path $env:GTA_DIR 'rak-rs-validation-shutdown.enabled') -ErrorAction SilentlyContinue
New-Item (Join-Path $env:GTA_DIR 'rak-rs-validation-unload.enabled') -ItemType File -Force
```

The coordinated-shutdown marker must be absent so the external manager owns
the shutdown. The manager waits until all enabled validator self-tests finish,
calls `RakRsPlugin_Shutdown`, and releases the validation ASI's loader reference.

## Run

1. Connect to a server and wait at least 10 seconds after spawning. When the
   send marker is present, remain connected until the client emits a stats
   update and the send self-test completes.
2. Walk, drive, send a chat message, and allow normal traffic for 30 seconds.
3. Press F5 ten times, about one second apart.
4. Disconnect and exit normally.

Watch the log if needed:

```powershell
Get-Content "$env:GTA_DIR\rak-rs-validation.log" -Wait
```

## Pass criteria

- GTA connects, survives the F5 presses, and exits normally.
- `rak-rs.log` reports `host runtime is ready` and six registrations.
- `rak-rs-validation.log` reports six registered callbacks and
  `self-test completed: packet=passed RPC=passed`.
- Incoming packet/RPC histograms are nonzero; walking or driving also produces
  outgoing sync IDs such as `207(ID_PLAYER_SYNC)` or `200(ID_VEHICLE_SYNC)`.
- Histograms contain one `254(RAK_RS_SELF_TEST)` packet and one
  `255(RAK_RS_SELF_TEST)` RPC.
- `null_events` and `timestamp_decode_errors` remain zero.
- With the send marker, the log reports
  `send self-test completed: packet=passed RPC=passed`.
- With the shutdown marker, the log reports
  `shutdown completed; all callbacks quiesced` and
  `shutdown self-test returned 1`; `rak-rs.log` records six synchronized
  unregistrations.
- In the runtime-unload scenario, `rak-rs-validation-unloader.log` reports
  `target shutdown succeeded` and `validation ASI unloaded successfully` while
  GTA remains stable.

## Failures

- No validation log: verify the ASI loader loaded the x86 release DLL under the
  expected name.
- Host discovery timeout: inspect `rak-rs.log` for a missing, unsupported, or
  failed `samp.dll` attachment.
- RPCs increase but packets do not: preserve both logs and the exact SA-MP
  version; the receive vtable path is not reaching the plugin.
- Self-test failure or timeout: preserve both logs; the result distinguishes an
  emulation readiness problem from rewrite or cancellation failure.
- Invalid incoming metadata: preserve `length` and `bit_size`; the backend has
  failed open because the client layout does not match.
- Crash or frozen counters: preserve both logs, client version, last action, and
  any Windows crash address and module.

Remove `rak_rs_validation.asi`, `rak_rs_validation_unloader.asi`, and all three
optional marker files afterward; none is required for normal use.
