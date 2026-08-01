# rak-rs

`rak-rs` is a Rust replacement for the client-side networking features of
SAMPFUNCS. It installs one process-wide SA-MP hook host, `rak_rs.asi`, and lets
independently loaded Rust ASI plugins receive, block, rewrite, or send RakNet
packets and RPCs through a stable ABI.

## What you need

- Windows GTA: San Andreas with a working ASI loader.
- A supported SA-MP 0.3.7 client: R1, R2, R3.1, R4.2, R5.1, or DL.
- Rust's `i686-pc-windows-msvc` target.
- Visual Studio C++ build tools, used by the native x86 layout fixture.

Use only with installations and servers where client-side modification is
allowed.

## Install the host

From this repository, set the GTA directory and deploy the release ASI:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy
```

This builds `rak_rs.dll` and copies it to `$env:GTA_DIR\rak_rs.asi`. Start GTA
normally. The host waits for `samp.dll`; lifecycle messages are written to
`rak-rs.log` in GTA's working directory.

## Write a plugin

Each feature is its own 32-bit `cdylib`/ASI. Depend on the ABI crate, never on
the root host crate:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
rak_rs_plugin_api = { path = "../rak_rs/plugin_api" }
```

From a worker thread—not `DllMain`—call `wait_for_default_host`. Register raw
packet/RPC callbacks through `HostApi::raw()`. Retain every
`RakRsSubscription`. Before a runtime unload, call
`HostApi::unregister_and_wait` for each subscription from a shutdown worker and
do not free the ASI until every call succeeds. The call returns
`CallbackInProgress` instead of deadlocking if it is made from inside a rak-rs
callback.

Use `HostApi::emulate_incoming_packet` or `HostApi::emulate_incoming_rpc` to
inject local incoming traffic. Pass only the payload bytes after the packet or
RPC ID and their exact bit length. Incoming listeners run normally and may
rewrite or block the emulated event.

The `events` module makes common RPCs typed. For example, an incoming callback
can use `events::incoming::on_server_message` and return:

- `RpcAction::Continue` to preserve the message;
- `RpcAction::Block` to suppress it; or
- `RpcAction::Replace(message)` to replace the complete byte-aligned payload.

Common incoming and outgoing chat, command, dialog-response, player-state,
checkpoint, vehicle, spawn, death, and map-marker RPCs are included. SA-MP
text is exposed as `Vec<u8>` because it is not guaranteed to be UTF-8.

See [`examples/sample_plugin`](examples/sample_plugin) for a complete ASI that
waits for the host, uses a typed event helper, and exports a synchronized
shutdown function.

## Validate in game

The [`validation_plugin`](examples/validation_plugin) records named
incoming/outgoing packet and RPC ID histograms. It also emulates two private
local events, rewrites them in one callback, verifies them in the next, and
blocks both before SA-MP handles them.
Deploy it with:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-validation
```

It writes counters and last-observed IDs to `rak-rs-validation.log`. Follow
[`VALIDATION.md`](VALIDATION.md) for the complete isolated test procedure and
pass/failure criteria.

## Status and limits

`rak_rs.asi` is the only module that installs MinHook and SA-MP hooks; adding
plugins does not add hooks. Typed support is intentionally incomplete: encoded
string events such as `onShowDialog`, bit-packed sync schemas, broader
game-state APIs, and full client-build validation are still pending. Runtime
unload is supported only when the unload manager follows the explicit
synchronized-shutdown contract described above.

## License

MIT © 2026 Akionka. See [LICENSE](LICENSE).
