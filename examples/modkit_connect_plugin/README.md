# Modkit connect plugin

This x86 ASI example resolves `GtaModHost_GetApiV1` from the process-lifetime
`samp_client_sdk.asi` host. It queries Core V1 and the migration-only Legacy
SA-MP service without falling back to `SampClientSdk_GetApiV1`.

Initialization runs on a worker thread. Before runtime unload, an unload
manager must call `ModkitConnectPlugin_Shutdown` from a worker thread and wait
for a nonzero result before it calls `FreeLibrary`. Do not call the shutdown
export or `FreeLibrary` from `DllMain` or a host callback. Process termination
does not require runtime unload synchronization.
