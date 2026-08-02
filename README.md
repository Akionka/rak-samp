# rak-rs

`rak-rs` is a Rust replacement for the client-side networking features of
SAMPFUNCS. One process-wide host, `rak_rs.asi`, lets independent Rust ASI
plugins observe, block, rewrite, send, or locally emulate SA-MP RakNet packets
and RPCs through a stable ABI.

## Requirements

- Windows GTA: San Andreas with an ASI loader
- SA-MP 0.3.7 R1, R2, R3.1, R4.2, R5.1, or DL
- Rust target `i686-pc-windows-msvc`
- Visual Studio C++ build tools for the native layout fixture

Use the project only where client-side modifications are allowed.

## Install

Close GTA, then deploy from the repository root:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy
```

This copies the release host to `$env:GTA_DIR\rak_rs.asi`. The host waits for
`samp.dll` and writes lifecycle messages to `rak-rs.log` in GTA's working
directory.

## Plugins

Build each plugin as a 32-bit `cdylib` and depend on the ABI crate, not the host:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
rak_rs_plugin_api = { path = "../rak_rs/plugin_api" }
```

From a worker thread, call `wait_for_default_host`, then register packet or RPC
callbacks through `HostApi::raw()`. The `events` module provides typed helpers
for common byte-aligned RPCs and exposes SA-MP text as `Vec<u8>`.

Use `HostApi::send_packet` and `HostApi::send_rpc` for explicit traffic. Their
payload slices exclude the packet or RPC ID, and `bit_len` identifies the exact
number of meaningful bits. Explicit sends bypass outgoing listeners to avoid
recursive callbacks. Timestamped packet sends are currently rejected.

Retain every `RakRsSubscription`. Before unloading a plugin at runtime, call
`HostApi::unregister_and_wait` for each subscription from a shutdown worker and
wait for success. Do not wait from `DllMain` or a rak-rs callback.

Incoming emulation accepts payload bytes after the packet/RPC ID plus their
exact bit length. Emulated events follow normal incoming rewrite and block
rules. See the [sample plugin](examples/sample_plugin) for a complete consumer.

## Validation

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-validation
```

The validation ASI records ID histograms and runs a blocked local rewrite test;
it never logs payloads. Optional marker files enable server-bound send and
coordinated-shutdown checks. Follow [VALIDATION.md](VALIDATION.md) for the
procedure and pass criteria.

## Limits

Encoded strings (including `onShowDialog`), bit-packed sync schemas, broader
game-state APIs, and live validation on every supported client build remain
pending. Runtime unload is safe only through the synchronized shutdown contract
above.

## License

MIT © 2026 Akionka. See [LICENSE](LICENSE).
