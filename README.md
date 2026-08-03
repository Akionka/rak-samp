# rak-samp

`rak-samp` is a process-wide networking host for Rust ASI plugins in SA-MP. Its
host, `rak_samp.asi`, lets separately loaded plugins observe, block, replace,
send, and emulate RakNet packets and RPCs through a stable C-compatible API.

## Compatibility

- Windows GTA: San Andreas with an ASI loader.
- SA-MP 0.3.7 R1 is the supported typed-event target.
- R2, R3.1, R4.2, R5.1, and DL are detected, but are experimental and intended
  for raw-event testing only.

Use it only with client modifications and server traffic that are permitted.

## Install

Download `rak_samp.asi` from a release and copy it into the GTA directory. Close
GTA before replacing an ASI. The host waits for `samp.dll` and writes lifecycle
messages to `rak-samp.log` in GTA's working directory.

To build and deploy from source, install the `i686-pc-windows-msvc` Rust target
and Visual Studio C++ build tools, then run:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy
```

## Plugins

Plugins are 32-bit `cdylib`s that depend on `rak_samp_plugin_api`, not the host.
Start a worker thread, wait for the host with `wait_for_default_host`, and
register callbacks through `HostApi`. Callbacks can inspect, block, or replace
packet and RPC payloads; replacement and send/emulation calls use an exact bit
length. Callback events must not be retained.

Keep each `RakSampSubscription`. Before runtime unload, remove every subscription
with `HostApi::unregister_and_wait` from a worker thread, then unload the ASI.
Never perform that wait in `DllMain` or in a callback. See the
[sample plugin](examples/sample_plugin) for a minimal integration.

The [chat-command example](examples/chat_command_plugin) shows sending a real
chat RPC and displaying a local dialog. It sends server-bound traffic, so run
it only where that is allowed:

```powershell
cargo make deploy-chat-command-example
```

## Validation and project notes

Run `cargo make deploy-validation` to install the local validation plugin; see
[VALIDATION.md](VALIDATION.md) for the procedure. Architecture and ABI design
are summarized in [ARCHITECTURE.md](ARCHITECTURE.md); current support gaps are
in [TODO.md](TODO.md).

## License

MIT © 2026 Akionka. See [LICENSE](LICENSE).
