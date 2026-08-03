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
`HostApi` also exposes individual cached local-player ID, nickname, colour,
spawned, health, armour, special-action, and animation-ID queries for the
corresponding SF.lua local-player helpers, plus local score and ping.

`HostApi::samp_game_state()` returns the latest game-thread-cached R1
`CNetGame` state as an opaque `i32`; it never calls client code on a plugin
thread and returns `NotReady` before its first publication. The numeric state
is intentionally not mapped to a public enum because that enum is not a stable
SA-MP ABI.

For every recognized client build, `HostApi::samp_version()` returns the
verified build identity and `HostApi::is_samp_available()` reports whether the
host's RakClient hooks are ready. `HostApi::is_samp_loaded()` instead reports
that the host has attached to and recognized `samp.dll`, including the brief
interval before hooks are ready. None of these queries reads client memory.

`rak_samp_plugin_api::raknet::{rpc_name, packet_name}` supplies SF.lua's
static RPC and packet labels without requiring host discovery or a client call.

`rak_samp_plugin_api::raknet::BitStream` is an owned, bounded plugin-side
replacement for SF.lua's native bitstream pointer. It supports checked bit,
buffer, numeric, string, cursor, and stream operations; pass it directly to
`HostApi::send_packet_stream` or `HostApi::send_rpc_stream`. Raw data-pointer
access is intentionally unavailable.

`HostApi::encode_string` and `HostApi::decode_string` use the detected
client's native RakNet StringCompressor without exposing its pointer. Decoding
returns owned bytes (at most 4,095), advances only the supplied owned
`BitStream` cursor on success, and leaves that cursor unchanged on failure.

`HostApi::send_chat` is the typed, bounded RPC 101 equivalent of
`sampSendChat`; slash-prefixed text uses the matching command RPC 50.
`HostApi::send_request_spawn` is the exact empty request-spawn RPC 129. Both
send real server-bound traffic, so only use them where that action is
permitted.

The same typed-send layer provides `send_dialog_response`, `send_click_player`,
`send_click_textdraw`, `send_death_by_player`, `send_menu_quit`,
`send_menu_select_row`, `send_picked_up_pickup`, and
`send_vehicle_destroyed`, `send_vehicle_damage`, `send_give_damage`,
`send_take_damage`, `send_edit_attached_object`, `send_edit_object`, and
`send_rcon_command`, and `send_scm_event`. These are exact protocol actions; they do not invoke
native UI or local-player mutation methods. Attached-object edits require the
complete typed payload, including both colour fields that SF.lua leaves
unspecified.

`send_aim_sync`, `send_bullet_sync`, `send_vehicle_sync`, `send_player_sync`,
`send_spectator_sync`, `send_trailer_sync`, `send_passenger_sync`, and
`send_unoccupied_sync` send complete, fixed-layout local synchronization
packets. They do not force or otherwise alter the client's local sync state.

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
