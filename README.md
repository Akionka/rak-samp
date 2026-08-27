# samp-client-sdk

`samp-client-sdk` is a Rust SDK and Windows x86 ASI host for SA-MP client
plugins. The public SDK lives in [`sdk/`](sdk/); the host package,
`samp-client-sdk-host`, installs as `samp_client_sdk.asi`.

## Compatibility

- GTA: San Andreas 1.0 US with an ASI loader.
- SA-MP 0.3.7 R1, R3-1, R5-1, and 0.3.DL R1 support the direct client bridge
  for state, native UI and pool operations, local mutations, sync, and network
  traffic.
- R3-1, R5-1, and DL R1 use independently verified layouts and
  version-specific native RVAs; all passed their complete loopback profiles.
- Other recognized SA-MP builds retain their networking offset tables for raw
  packet and RPC observation.

Each direct bridge uses verified fixed offsets. Every native access still validates
its pointer, readable range, capacity, and enum values before use.

## Install

Copy the release `samp_client_sdk.asi` into the GTA directory. Close GTA before
replacing an ASI. Lifecycle messages are written to `samp-client-sdk.log` in
GTA's working directory.

To build and install from source, install the `i686-pc-windows-msvc` Rust target
and Visual Studio C++ build tools, then run:

```powershell
$env:GTA_DIR = 'D:\Games\GTA San Andreas'
cargo make install
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

Register native chat commands from a worker thread and keep the returned
subscription alive for as long as the handler may run:

```rust
let command = samp.chat_input().register_command(b"hello", |args| {
    // `args` is the bounded text after `/hello`.
})?;
```

Call `command.unregister_and_wait()` from a worker thread before unloading the
plugin.

`samp.chat_input().is_command_defined(b"hello")` queries the latest
game-thread-published command table with exact, case-sensitive name matching.

`samp.labels().create(...)` allocates the first free R1 3D text-label slot on
the game thread. Its typed receipt resolves to the resulting `TextLabelId`.
`samp.labels().set_text(id, text)` queues validated replacement text and
returns a normal completion receipt.

`samp.players().player(id).onfoot_sync()` returns an owned, game-thread-cached
R1 on-foot synchronization snapshot for local or defined remote players.
`samp.players().player(id).vehicle_sync()` provides the corresponding owned
in-car snapshot, while `.passenger_sync()` provides the owned passenger
snapshot and `.trailer_sync()` provides the owned trailer snapshot.
`samp.players().player(id).aim_sync()` provides the owned aim snapshot.
`samp.players().player(id).streamed_out_position()` provides the host-owned R1
radar-marker cache from accepted marker-sync packets for a connected streamed-out
player when a marker is active; its integer-quantized coordinates may be stale.

When SAMPFUNCS is already installed, `samp.probe().is_sampfuncs_loaded()` tests
for its ASI module and `samp.sampfuncs().log_console(b"message")` writes through
SAMPFUNCS's own console logger. This optional bridge never loads or initializes
SAMPFUNCS; absence returns `SampClientSdkResult::NotReady`.

`samp_client_sdk::raknet::BitStream` is owned and bounded. Typed events,
protocol catalogs, exact sends, and incoming emulation retain exact-bit and
exactly-once dispatch semantics. Direct state reads are copied into host-owned
snapshots; plugins never dereference native client memory through the safe API.
`samp.net().incoming_emulation_ready()` exposes only a copied readiness scalar
for packet emulation; it becomes true after the host captures a real incoming
RPC receiver.

The [sample plugin](examples/sample_plugin) shows a minimal subscription. The
[network smoke plugin](examples/network_smoke_plugin) is an opt-in isolated
validation aid for the native codec and blocked exact-bit emulation paths. The
[chat-command example](examples/chat_command_plugin) can be installed with:

```powershell
cargo make install-chat-command-example
```

## Development

Run the complete local quality gate and release build from the repository root:

```powershell
cargo make quality
cargo make build-release
```

The quality gate runs formatting, workspace checks and tests, Clippy with
warnings denied, and documentation. Its tasks are also available separately as
`format-check`, `check`, `test`, `clippy`, and `doc`; use `format` to apply
formatting.

The C++ RakNet layout fixture remains part of the workspace checks. See
[CORE.md](CORE.md) for invariants, [ARCHITECTURE.md](ARCHITECTURE.md) for
ownership, and [TODO.md](TODO.md) for the repositioning tracker.

## License

MIT © 2026 Akionka. See [LICENSE](LICENSE).
