# In-game validation plugin

This ASI records incoming and outgoing packet/RPC ID histograms and runs a
private local rewrite-and-block test. It logs counters and self-test status to
`rak-rs-validation.log`, never payloads.

See [VALIDATION.md](../../VALIDATION.md) for deployment, test steps, pass
criteria, and diagnosis.
