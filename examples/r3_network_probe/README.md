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
status=0x0000007F
failure=0
game_state=6
address_hex=3132372E302E302E31
hostname_hex=53412D4D50
port=7777
```

`0x7F` also proves the public ready/version APIs and the SDK's opaque module
base identify the R3-1 PE entry point, and that the R3 scalar cache reports its
R3 `AwaitingJoin` state (`6`), loopback address/port, and native `CNetGame`
host field (`SA-MP`), before it proves listener registration,
inbound readiness, successful outbound receipt, and the matching incoming reply
callback. The listener returns `Continue`; a human must additionally verify that
the reply marker appears in ordinary SA-MP chat. That visible result proves the
original incoming-RPC handler ran.

The status fields are a bounded copied snapshot for this opt-in probe. The
native `CNetGame` host field is not asserted to equal the server's configured
display hostname.

Use only with the supplied disposable loopback server configuration. Do not use
this probe on public servers.
