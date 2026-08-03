# In-Game Validation

Use a legal SA-MP installation and a server that permits client plugins. The
validator logs IDs and counters, never payloads. Its default tests are local:
they rewrite, verify, and block private events before they reach SA-MP or the
server.

## Run the standard check

Close `gta_sa.exe`, then deploy from the repository root. `gta_sa.exe` loads
the ASIs and locks them while it is running; the SA-MP launcher (`samp.exe`)
does not load the ASIs and does not need to be closed.

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
| Direct R1 client helpers | `New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-direct-client.enabled') -ItemType File -Force` | Queues one direct local message dialog, chat entry, and death-window entry; verifies a populated local-player snapshot plus cached game-state, current-server, chat-display, cursor, scoreboard, dialog, chat-input, known animation-table, player-directory, non-streamed player-count, and non-streamed player-max-ID results; then monitors position, health, armour, vehicle-state, all three chat display modes, cursor active/inactive, scoreboard open/closed, dialog active/inactive, and chat-input active/inactive for two minutes. Use only on SA-MP 0.3.7 R1 with the fingerprinted GTA SA 1.0 US executable. The log records only outcomes and the local player ID. |
| R1 player directory | `New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-player-directory.enabled') -ItemType File -Force` | With a second player connected, demand-refreshes IDs through the R1 game-thread pump until one remote directory entry is copied. It checks the copied projections and logs only the outcome and remote player ID. |
| R1 vehicle existence | `New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-vehicle-exists.enabled') -ItemType File -Force` | Demand-refreshes bounded vehicle IDs through the R1 game-thread pump until a defined vehicle is copied. It logs only the outcome and vehicle ID. |
| Coordinated shutdown | `New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-shutdown.enabled') -ItemType File -Force` | Stops validator workers and waits for subscriptions. |

For the direct-helper check, wait until the validator logs `observing`. Within
the next two minutes, walk, take enough damage to change both armour and health,
enter or leave a vehicle, and press F7 until the chat window has cycled through
off, no-shadow, and normal display modes. Open and close the scoreboard with
Tab, then activate and dismiss an ordinary local cursor state (for example,
open and close chat input). Leave the direct dialog visible until the validator
starts observing, then dismiss it.
Confirm its `direct-client state validation passed` line and separately inspect
that its outcome records each chat mode, both cursor categories, and both
scoreboard, dialog, and chat-input states; the direct dialog, local chat entry,
death-window entry, and cached UI reads must add no RPC 61 observation and no
outgoing RPC 61 or 62. (The standard validator intentionally emulates one
incoming RPC 61 before this direct check.)
A release requires this scenario to remain stable through normal shutdown.

For the player-directory check, connect a second player before launching the
validator, create the player-directory marker, and let both clients remain
connected for up to two minutes. Confirm `player-directory self-test passed`
with only a player ID in the log; then have that player disconnect and issue a
fresh directory read during a follow-up run to confirm it becomes `None` after
refresh. The first remote read may be `NotReady`; it must never block a plugin
thread or generate packet/RPC traffic. Exit normally and remove the marker.

The direct R1 helper scenario also reports `player_count=Ok` once the cached
including-NPC player-pool count is nonzero. Check that it is sensible for the
server's visible player list; it is not a streamed-GTA-ped count and does not
send traffic.

The same outcome line reports `player_max_id=Ok`. Confirm its non-streamed
player-pool value is at least the assigned local-player ID; it is not the
separate streamed-GTA-ped maximum and does not send traffic.

For the vehicle-existence check, create the vehicle marker before launching
the validator and remain connected for up to two minutes. Confirm
`vehicle-exists self-test passed` with only one vehicle ID in the log. Initial
lookups may be `NotReady` while the game-thread pump fills the cache; the scan
must not send packet/RPC traffic and must survive normal shutdown.

For the separate runtime-unload check, close GTA and run:

```powershell
cargo make deploy-validation-unload
Remove-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-shutdown.enabled') -ErrorAction SilentlyContinue
New-Item (Join-Path $env:GTA_DIR 'rak-samp-validation-unload.enabled') -ItemType File -Force
```

The external manager owns shutdown in this scenario. A pass reports successful
target shutdown and validation-ASI unload while GTA stays stable.
