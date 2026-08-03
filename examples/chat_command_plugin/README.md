# Chat-command example

This independent ASI intercepts `/raksamp`, prevents the command from reaching
the server, sends one chat message, and displays a local dialog. It also blocks
the response to that dialog. The chat message is server-bound, so use it only
where permitted.

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-chat-command-example
```

Load it with `rak_samp.asi`. Before runtime unload, call its shutdown export from
a worker thread and wait before `FreeLibrary`.
