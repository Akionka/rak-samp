# In-game validation plugin

This ASI records incoming and outgoing packet/RPC ID histograms and runs a
private local rewrite-and-block tests. These include an RPC 61 dialog whose
compressed text is encoded by SA-MP, decoded, rewritten, decoded again, and
blocked before it is displayed. The plugin logs counters and self-test status
next to its ASI in `rak-rs-validation.log`, never payloads. Each session
rewrites that file, and every record begins with an RFC 3339 UTC timestamp. It
waits for SA-MP's native StringCompressor to become ready before starting the
dialog test.

For R1 traffic matching the completed typed catalog, it separately counts typed
RPC and packet callbacks and makes one value-preserving replacement per family.
That exercises compressed remote player/vehicle and marker sync without logging
their data.

Server-bound packet/RPC sends and coordinated callback shutdown are separate
opt-in scenarios controlled by marker files in GTA's working directory. Normal
validation remains local and non-mutating.

The plugin also exports `RakRsValidation_SelfTestsComplete` so the independent
[validation unload manager](../validation_unloader) can wait before requesting
synchronized shutdown and releasing this ASI.

See [VALIDATION.md](../../VALIDATION.md) for deployment, test steps, pass
criteria, and diagnosis.
