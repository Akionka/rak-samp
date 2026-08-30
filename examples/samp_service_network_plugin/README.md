# SA-MP service network plugin

This independently loaded ASI uses the Phase 7 `SampNetServiceV1` path. It
registers a typed outgoing `SendChat` RPC callback and counts valid messages.

Call `SampServiceNetworkPlugin_ObservedChatCount` to read the counter. The
plugin does not use the legacy `samp-client-sdk` ABI. An unload manager must call
`SampServiceNetworkPlugin_Shutdown` from a worker thread before `FreeLibrary`.
