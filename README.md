# samp-client-sdk

`samp-client-sdk` is a Rust SDK and Windows x86 ASI host for SA-MP client
plugins. The public SDK lives in [`sdk/`](sdk/); the host package,
`samp-client-sdk-host`, installs as `samp_client_sdk.asi`.

## Compatibility

- GTA: San Andreas 1.0 US with an ASI loader.
- SA-MP 0.3.7 R1 is the supported client bridge for state and local mutation.
- Other recognized SA-MP builds retain their networking offset tables for raw
  packet and RPC observation.

The R1 bridge uses approved fixed offsets. Every native access still validates
its pointer, readable range, capacity, and enum values before use.

## Install

Copy the release `samp_client_sdk.asi` into the GTA directory. Close GTA before
replacing an ASI. Lifecycle messages are written to `samp-client-sdk.log` in
GTA's working directory.

To build and deploy from source, install the `i686-pc-windows-msvc` Rust target
and Visual Studio C++ build tools, then run:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make deploy
```

## Plugins

Plugins are 32-bit `cdylib`s that depend on the public `samp-client-sdk`
package and import it as `samp_client_sdk`. Connect through `Samp`; the raw
ABI wrapper is documentation-hidden implementation detail.

```rust
use samp_client_sdk::Samp;

let samp = Samp::connect(std::time::Duration::from_secs(10))?;
let subscription = samp.net().on_rpc(|event| {
    // Inspect, block, or atomically replace this callback-local event.
    samp_client_sdk::SampClientSdkHookAction::Continue
})?;
```

Keep every `Subscription` or `SubscriptionSet`. Before unloading a plugin,
call `unregister_and_wait` from a worker thread. Never wait in `DllMain`, a
listener callback, or the game tick.

`samp_client_sdk::raknet::BitStream` is owned and bounded. Typed events,
protocol catalogs, exact sends, and incoming emulation retain exact-bit and
exactly-once dispatch semantics. Direct state reads are copied into host-owned
snapshots; plugins never dereference native client memory through the safe API.

The [sample plugin](examples/sample_plugin) shows a minimal subscription. The
[chat-command example](examples/chat_command_plugin) can be deployed with:

```powershell
cargo make deploy-chat-command-example
```

## Development

Run these checks from the repository root:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
```

The C++ RakNet layout fixture remains part of the workspace checks. See
[CORE.md](CORE.md) for invariants, [ARCHITECTURE.md](ARCHITECTURE.md) for
ownership, and [TODO.md](TODO.md) for the repositioning tracker.

## License

MIT © 2026 Akionka. See [LICENSE](LICENSE).
