# R5 network delivery probe

This opt-in ASI is a loopback-only live validation aid. It proves two network
obligations which the no-outbound network smoke intentionally does not cover:

- one client-to-server chat RPC reaches a disposable local server filter;
- a matching server-message RPC reaches a non-blocking SDK listener and then
  continues to SA-MP's original handler.

It sends exactly `R5_SDK_OUTBOUND_20260812` once, only after the host captures
a real incoming-RPC receiver. The matching `r5_network_probe.pwn` filter logs
the outbound marker, suppresses normal chat broadcast, and replies once with
`R5_SDK_INCOMING_20260812`.

The ASI writes `samp-client-sdk-r5-network-probe.status` in the GTA working
directory:

```text
status=0x0000001F
failure=0
```

`0x1F` proves host connection, listener registration, inbound readiness,
successful outbound receipt, and the matching incoming reply callback. The
listener returns `Continue`; a human must additionally verify that the reply
marker appears in ordinary SA-MP chat. That visible result proves the original
incoming-RPC handler ran.

Use only with the supplied disposable loopback server configuration. Do not use
this probe on public servers.
