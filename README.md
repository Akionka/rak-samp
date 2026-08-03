# rak-samp

`rak-samp` is a process-wide networking host for Rust ASI plugins in SA-MP. Its
host, `rak_samp.asi`, lets separately loaded plugins observe, block, replace,
send, and emulate RakNet packets and RPCs through a stable C-compatible API.

## Compatibility

- Windows GTA: San Andreas with an ASI loader.
- SA-MP 0.3.7 R1 is the supported typed-event target. Direct local dialogs,
  local-player snapshots, and cached CNetGame reads additionally require the
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
`HostApi::show_local_dialog(LocalDialog { .. })`, a local chat entry with
`HostApi::show_local_chat_message(LocalChatMessage { .. })`, a local
death-window entry with `HostApi::show_local_death_message(LocalDeathMessage { .. })`,
and retrieve an owned cached snapshot with `HostApi::local_player()`. A successful UI call
means the host accepted one of its 32 queued requests, not that it was
displayed. Neither UI helper emulates RPC traffic or exposes client pointers.
`local_player` returns
`NotReady` until the server's R1 `INIT_GAME` assignment matches the pool's
local-player ID across two game-thread refreshes.
`HostApi` also exposes individual cached local-player ID, nickname, colour,
spawned, health, armour, special-action, and animation-ID queries for the
corresponding SF.lua local-player helpers, plus local score and ping.

`HostApi::player_info(id)` provides a bounded directory entry for the local
player or a demand-refreshed remote R1 player: copied nickname bytes, NPC flag,
ARGB colour, score, and ping. A first remote lookup may return `NotReady` while
the existing game-thread pump copies it; `Ok(None)` is a cached disconnected
result. `is_player_connected`, `is_player_defined`, `is_player_paused`, `player_nickname`,
`is_player_npc`, `player_colour`, `player_score`, and `player_ping` are
projections of that same cache. `is_player_defined` uses R1's exact remote
world-state accessor, so it is stricter than a connection check.
`is_player_paused` matches SF.lua's remote `PLAYER_STATE_NONE` status test and
always returns false for the local player. No player, ped, pool, or GTA handle
crosses the ABI. This R1-only helper remains
provisional until its opt-in second-player live validation is recorded.

`HostApi::remote_player_state(id)` separately returns copied remote health,
armour, special action, and animation ID. Its scalar projections are
`player_health`, `player_armour`, `player_special_action`, and
`player_animation_id`. It uses its own bounded 32-ID request queue and copies
at most four records per game-thread pump, so it never blocks plugin threads.

`HostApi::player_count(include_npcs)` returns the latest game-thread-cached R1
player-pool scalar, including or excluding NPCs. It covers SF.lua's
non-streamed count path only; counting streamed GTA peds requires separate
native evidence and is not exposed. Like every direct R1 read, it is
provisional until live validation is recorded.

`HostApi::player_max_id()` returns the latest game-thread-cached R1
non-streamed player-pool maximum ID. It intentionally does not inspect GTA
peds for SF.lua's streamed alternative, and remains provisional until its
direct live validation is recorded.

`HostApi::is_vehicle_defined(id)` demand-refreshes a bounded, cached R1
vehicle-pool existence flag. A first lookup may return `NotReady` until the
game-thread pump completes it; it never exposes a vehicle, pool, or GTA handle
and remains provisional pending its opt-in live scan.

`HostApi::is_text_label_defined(id)` similarly demand-refreshes only the
bounded R1 3D text-label existence flag. It never reads label text or returns a
label/pool pointer, and remains provisional pending its opt-in live scan.

`HostApi::text_label(id)` demand-refreshes an owned R1 3D-label copy: byte
text, ARGB colour, position, draw distance, LOS flag, and optional attachment
IDs. Its 4,095-byte text copy is made only by the game-thread pump after the
verified R1 allocation path and never exposes a label/pool pointer. It remains
provisional pending its dedicated opt-in live scan.

`HostApi::is_textdraw_defined(pool_index)` similarly copies only the bounded
R1 textdraw existence flag. Its raw 2,304-slot index covers 2,048 global then
256 local slots without guessing an ID space; it never reads textdraw content
or returns a textdraw/pool pointer, and remains provisional pending its opt-in
live scan.

`HostApi::textdraw(pool_index)` demand-refreshes an owned numeric R1 textdraw
copy: letter, position, alignment, box, and model fields. It preserves the raw
2,048-global then 256-local slot order and deliberately excludes display text
until that native buffer's lifecycle is independently proven. It never exposes
a textdraw/pool pointer and remains provisional pending its opt-in live scan.

`HostApi::is_object_defined(id)` similarly demand-refreshes only the bounded
R1 object existence flag. It never returns an object, pool, or GTA handle, and
remains provisional pending its opt-in live scan.

`HostApi::gangzone(id)` returns an owned cached R1 gangzone rectangle and two
Direct3D ARGB draw colours. It never returns a gangzone/pool pointer and remains
provisional pending its opt-in live scan.

`HostApi::samp_game_state()` returns the latest game-thread-cached R1
`CNetGame` state as an opaque `i32`; it never calls client code on a plugin
thread and returns `NotReady` before its first publication. The numeric state
is intentionally not mapped to a public enum because that enum is not a stable
SA-MP ABI.

`HostApi::server_info()` returns copied address bytes, hostname bytes, and port
from the same fingerprinted R1 game-thread pump. It returns `NotReady` before
the host has a valid address and port, and never exposes a client pointer.

`HostApi::local_chat_display_mode()` returns the latest game-thread-cached R1
chat display mode as `Off`, `NoShadow`, or `Normal`; the derived
`HostApi::is_local_chat_visible()` treats every mode except `Off` as visible.
Both return `NotReady` before their first valid publication and never call the
client from a plugin thread.

`HostApi::local_cursor_mode()` exposes the cached R1 cursor enum and
`HostApi::is_local_cursor_active()` derives whether it is non-`None`.
`HostApi::is_local_scoreboard_open()` similarly returns a cached local
scoreboard flag. These are read-only R1 observations; they do not toggle UI,
send traffic, or expose a client pointer.

`HostApi::is_local_dialog_active()` and
`HostApi::is_local_chat_input_active()` expose the same cached, read-only R1
visibility state for the local dialog and chat input. Closing dialogs or
opening/processing chat input remains outside the safe ABI.

`HostApi::active_local_dialog()` returns an owned active-dialog core snapshot:
the ID, typed style, bounded caption bytes, and server-side flag. It returns
`Ok(None)` when no dialog is active and deliberately omits dynamic text,
buttons, edit-box, and list data until their pointer ownership and bounds have
their own R1 evidence. Like the other direct reads, this R1-only helper remains
provisional pending its opt-in live lifecycle validation.

`HostApi::local_animation()` and `HostApi::local_animation_id()` expose owned
lookups over R1's fingerprinted fixed animation table. The table is copied on
the game-thread pump; plugin calls never receive the client table pointer.

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

`HostApi::send_request_class`, `send_interior_change`, `send_spawn`,
`send_enter_vehicle`, and `send_exit_vehicle` provide the matching exact R1
protocol messages for SF.lua's local-player actions. Like every typed send,
they are server-bound traffic only: they do not call the native methods or
mutate local GTA or SA-MP state.

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
