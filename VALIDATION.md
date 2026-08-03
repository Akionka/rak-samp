# In-Game Validation

Use a legal SA-MP installation and a server that permits client plugins. The
validator logs IDs and counters, never payloads. Its default tests are local:
they rewrite, verify, and block private events before they reach SA-MP or the
server.

## Run the standard check

Close GTA, then deploy from the repository root:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-validation
```

Start GTA with `rak_samp.asi` and `rak_samp_validation.asi`, connect to a server,
and use ordinary gameplay for about 30 seconds (walk or drive, chat, and press
F5 several times). Exit normally. The validator writes
`rak-samp-validation.log` beside its ASI; the host writes `rak-samp.log`.

A passing standard run has all of the following:

- GTA remains stable and exits normally.
- The host reports that its runtime is ready.
- The validation log reports `packet=passed RPC=passed dialog=passed`.
- Packet and RPC counters increase, while `null_events` and
  `timestamp_decode_errors` remain zero.

For investigation, preserve both logs, the SA-MP version, and the last action.
If the host never becomes ready, check that the correct x86 ASIs and `samp.dll`
were loaded. If RPCs arrive but packets do not, preserve the logs: the receive
hook may not match the client build.

## Optional scenarios

These marker files opt into behavior beyond the local test. Create them before
starting GTA and remove them afterwards.

| Scenario | Command | Effect |
| --- | --- | --- |
| Explicit send | `New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-send.enabled') -ItemType File -Force` | Sends one test packet and RPC; use only on a permitted server. |
| Coordinated shutdown | `New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-shutdown.enabled') -ItemType File -Force` | Stops validator workers and waits for subscriptions. |

For the separate runtime-unload check, close GTA and run:

```powershell
cargo make deploy-validation-unload
Remove-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-shutdown.enabled') -ErrorAction SilentlyContinue
New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-unload.enabled') -ItemType File -Force
```

The external manager owns shutdown in this scenario. A pass reports successful
target shutdown and validation-ASI unload while GTA stays stable.
