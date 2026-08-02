# In-game validation plugin

This ASI records incoming and outgoing packet/RPC ID histograms and runs a
private local rewrite-and-block test. It logs counters and self-test status to
`rak-rs-validation.log`, never payloads.

Server-bound packet/RPC sends and coordinated callback shutdown are separate
opt-in scenarios controlled by marker files in GTA's working directory. Normal
validation remains local and non-mutating.

See [VALIDATION.md](../../VALIDATION.md) for deployment, test steps, pass
criteria, and diagnosis.
