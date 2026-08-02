# Chat-command example plugin

This independent ASI demonstrates typed outgoing and fake incoming RPCs through
the shared rak-rs host. Enter `/rakrs` in SA-MP chat to:

1. block the command so RPC 50 is not sent to the server;
2. explicitly send `rak-rs example: SEND_CHAT RPC works` as RPC 101; and
3. emulate incoming RPC 61 to display a local information dialog.

The plugin also blocks RPC 62 for its reserved dialog ID (`0x7F00`), preventing
the fake dialog response from reaching the server. The chat message is real
server-bound traffic, so use the example only where it is permitted.

Close GTA and deploy from the repository root:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-chat-command-example
```

At runtime, `rak_rs.asi` must be loaded alongside
`rak_rs_chat_command_example.asi`. For runtime unload, call the exported
`RakRsChatCommand_Shutdown` from a worker thread before `FreeLibrary`.
