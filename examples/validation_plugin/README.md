# In-game validation plugin

This ASI validates all process-wide incoming/outgoing packet and RPC paths.
Four observers update fixed atomic ID histograms, and timestamped packets are
also classified by their inner ID. Two earlier callbacks handle only private
16-byte test markers: a background worker locally emulates packet 254 and RPC
255, the first callback rewrites each marker, and the observer verifies and
blocks it. No self-test event reaches SA-MP or the server. A reporter appends
aggregate counters, named nonzero histograms, and self-test status to
`rak-rs-validation.log` every five seconds.

Use [`VALIDATION.md`](../../VALIDATION.md) for the complete build, installation,
test, and diagnosis procedure.
