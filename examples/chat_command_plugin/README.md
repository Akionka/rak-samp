# Chat-command example

This independent ASI intercepts `/raksamp`, prevents the command from reaching
the server, sends one chat message, and queues a direct local dialog. It does
not emulate incoming RPC 61 or intercept a dialog response. The direct dialog
requires the fingerprinted GTA SA 1.0 US + SA-MP 0.3.7 R1 profile; the chat
message is server-bound, so use it only where permitted.

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-chat-command-example
```

Load it with `rak_samp.asi`. Before runtime unload, call its shutdown export from
a worker thread and wait before `FreeLibrary`.
