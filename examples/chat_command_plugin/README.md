# Chat-command example

This service-backed independent ASI intercepts `/sampclientsdk`, prevents the
command from reaching the server, sends one chat message, and queues a direct
local dialog. It does not emulate incoming RPC 61 or intercept a dialog
response. The dialog uses `SampUiServiceV1`; the chat message is server-bound,
so use it only where permitted.

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-chat-command-example
```

Load it with `samp_client_sdk.asi`. Before runtime unload, call its shutdown export from
a worker thread and wait before `FreeLibrary`.
