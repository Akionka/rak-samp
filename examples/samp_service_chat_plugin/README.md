# SA-MP service chat plugin

This independently loaded ASI uses the Phase 7 `samp` facade. Enter
`/sampservice` in the SA-MP chat to add a local confirmation message through
`SampServiceV1`.

The plugin does not use the legacy `samp-client-sdk` ABI. An unload manager must
call `SampServiceChatPlugin_Shutdown` from a worker thread before `FreeLibrary`.
