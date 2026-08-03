# rak-samp

`rak-samp` is a process-wide networking host for Rust ASI plugins in SA-MP. Its
host, `rak_samp.asi`, lets separately loaded plugins observe, block, replace,
send, and emulate RakNet packets and RPCs through a stable C-compatible API.

## Compatibility

- Windows GTA: San Andreas with an ASI loader.
- SA-MP 0.3.7 R1 is the supported typed-event target. Direct local dialogs,
  local-player snapshots, and cached game-state reads additionally require the
  fingerprinted GTA San Andreas 1.0 US executable; unsupported fingerprints return
  `UnsupportedVersion`.
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
register safe closures with `HostApi::on_packet` or `HostApi::on_rpc`.
`on_packet_id` and `on_rpc_id` target one protocol ID; `on_typed_packet` and
`on_typed_rpc` decode a named descriptor. `register_handlers!` groups related
registrations. Callbacks can inspect, block, or replace payloads; replacement
and send/emulation calls use an exact bit length. Callback events must not be retained.

R1 plugins can queue a copied direct local dialog with
`HostApi::show_local_dialog(LocalDialog { .. })` and retrieve an owned cached
snapshot with `HostApi::local_player()`. A successful dialog call means the
host accepted one of its 32 queued requests, not that it was displayed. Neither
helper emulates RPC traffic or exposes client pointers. `local_player` returns
`NotReady` until the server's R1 `INIT_GAME` assignment matches the pool's
local-player ID across two game-thread refreshes.

`HostApi::samp_game_state()` returns the latest game-thread-cached R1
`CNetGame` state as an opaque `i32`; it never calls client code on a plugin
thread and returns `NotReady` before its first publication. The numeric state
is intentionally not mapped to a public enum because that enum is not a stable
SA-MP ABI.

For every recognized client build, `HostApi::samp_version()` returns the
verified build identity and `HostApi::is_samp_available()` reports whether the
host's RakClient hooks are ready. Neither query reads client memory.

Keep each `Subscription` or `SubscriptionSet`. Before runtime unload, call its
`unregister_and_wait` method from a worker thread, then unload the ASI. Never
perform that wait in `DllMain` or in a callback. The
[sample plugin](examples/sample_plugin) is a minimal typed-RPC integration.

The [chat-command example](examples/chat_command_plugin) shows sending a real
chat RPC and displaying a direct local dialog. It sends server-bound traffic,
so run it only where that is allowed:

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
