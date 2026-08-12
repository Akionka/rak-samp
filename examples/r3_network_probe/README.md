# R3-1 network delivery probe

This opt-in ASI is a loopback-only live validation aid. It proves two network
obligations which the no-outbound network smoke intentionally does not cover:

- one client-to-server chat RPC reaches a disposable local server filter;
- a matching server-message RPC reaches a non-blocking SDK listener and then
  continues to SA-MP's original handler.

After the host captures a real incoming-RPC receiver, it verifies the R3 cached
`Samp::game_state()` and `Samp::server().info()` values against the disposable
server. Only then does it send exactly `R3_SDK_OUTBOUND_20260812`. The matching
`r3_network_probe.pwn` filter logs
the outbound marker, suppresses normal chat broadcast, and replies once with
`R3_SDK_INCOMING_20260812`.

The ASI writes `samp-client-sdk-r3-network-probe.status` in the GTA working
directory:

```text
status=0x00003FFF
failure=0
game_state=6
address_hex=3132372E302E302E31
hostname_hex=53412D4D50
port=7777
```

`0x3FFF` also proves the public ready/version APIs and the SDK's opaque module
base identify the R3-1 PE entry point, and that the R3 scalar cache reports its
R3 `AwaitingJoin` state (`6`), loopback address/port, and native `CNetGame`
host field (`SA-MP`), then verifies the public player-pool count (with and
without NPCs) and largest ID as `0`, `0`, and `0` for the single-player
loopback session: the local player is not an entry in `CPlayerPool::GetCount`.
It additionally proves that the public local-player
snapshot is spawned, bounded, and contains finite spatial and health values before it
proves listener registration, inbound readiness, successful outbound receipt,
and the matching incoming reply callback. The probe then waits for an operator
to hold `Tab` long enough to observe the public `Samp::scoreboard().is_open()`
flag as true, then release it so the same cache observes false. It next waits
for the operator to open chat with `T`; it verifies that the cached active flag is true, the
built-in `quit` command is present, and a fixed nonexistent name is absent.
The operator then enters, without sending, `R3_SDK_TEXT_CACHE_20260812`; the
probe verifies the exact owned cached text. The listener returns `Continue`; a human must additionally verify that the
reply marker appears in ordinary SA-MP chat. It then sends one more bounded
loopback marker that makes the disposable filter open a message-box dialog and
verifies the public cached `Samp::dialogs().is_active()` flag while that dialog
is left open. That visible result proves the original incoming-RPC handler ran.

The status fields are a bounded copied snapshot for this opt-in probe. The
native `CNetGame` host field is not asserted to equal the server's configured
display hostname.

Use only with the supplied disposable loopback server configuration. Do not use
this probe on public servers.
