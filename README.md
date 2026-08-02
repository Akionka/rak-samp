# rak-rs

`rak-rs` is a Rust replacement for the client-side networking features of
SAMPFUNCS. One process-wide host, `rak_rs.asi`, lets independent Rust ASI
plugins observe, block, rewrite, send, or locally emulate SA-MP RakNet packets
and RPCs through a stable ABI.

## Requirements

- Windows GTA: San Andreas with an ASI loader
- SA-MP 0.3.7 R1 (validated)
- SA-MP 0.3.7 R2, R3.1, R4.2, R5.1, and DL are recognized but experimental
- Rust target `i686-pc-windows-msvc`
- Visual Studio C++ build tools for the native layout fixture

Use the project only where client-side modifications are allowed.

## Install

For a tagged release, download `rak_rs.asi` or
`rak-rs-windows-x86.zip` from its GitHub release, verify it against
`SHA256SUMS.txt`, and copy `rak_rs.asi` into the GTA directory. The ZIP also
contains the host PDB and the optional chat-command example.

To build from source, close GTA and deploy from the repository root:

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
callbacks through `HostApi::raw()`. The `events` module provides the complete
SA-MP 0.3.7 R1 typed RPC and packet catalog, while exposing uncertain SA-MP
text as `Vec<u8>`. It includes `events::rpc::incoming::on_show_dialog`,
`events::rpc::incoming::on_create_object`, and compressed packet helpers such as
`events::packet::incoming::on_player_sync`. The skin helper receives
`PlayerSkin { player_id, skin_id }` from incoming RPC 153; compressed dialog
and material text are decoded and encoded by the installed SA-MP client's
native StringCompressor.

Typed replacements use `RpcAction::Replace`. For locally generated typed RPCs,
call a descriptor such as `events::rpc::incoming::SHOW_DIALOG.encode(api, value)` and pass
the returned `as_bytes()` and `len_bits()` to `emulate_incoming_rpc`. Keep the
exact bit length: compressed strings are not necessarily byte-aligned.

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

For a send-and-emulation example, deploy
[`examples/chat_command_plugin`](examples/chat_command_plugin):

```powershell
cargo make deploy-chat-command-example
```

Entering `/rakrs` blocks that command, sends one real chat message as outgoing
RPC 101, and displays a local fake dialog through incoming RPC 61. Its dialog
response is blocked locally. Because the chat message reaches the server, use
this example only where such traffic is permitted.

## Validation

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy-validation
```

The validation ASI records ID histograms and runs a blocked local rewrite test;
it never logs payloads. Optional marker files enable server-bound send and
coordinated-shutdown checks. Follow [VALIDATION.md](VALIDATION.md) for the
procedure and pass criteria.

Use `cargo make deploy-validation-unload` for the separate runtime-unload
scenario. Its manager waits for the validator's self-tests, requests
synchronized shutdown, calls `FreeLibrary`, and verifies that the validation
ASI is no longer loaded. This tooling is not needed by ordinary plugins.

## Limits

The typed catalog is validated against the R1 wire reference. Broader
game-state APIs and live validation on every supported client build remain
pending. Remote compressed sync preserves its protocol bit layout rather than
relying on Rust bitfields. Only SA-MP 0.3.7 R1 is the typed-layout authority;
use the other recognized builds only for raw-event testing. Runtime unload is
safe only through the synchronized shutdown contract above.

## License

MIT © 2026 Akionka. See [LICENSE](LICENSE).
