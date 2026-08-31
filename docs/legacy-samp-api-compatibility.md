# Legacy SA-MP safe API compatibility matrix

Status: Phase 11 migration inventory, 2026-08-30.

This matrix covers every public method declared by the legacy safe facades in
`sdk/src/facade/`. It does not treat private `HostApi` methods or ABI function
pointers as public API. Repeated method names on different facade types are
separate entries.

## Status keys

- **Equivalent**: the service-backed `samp`/`modkit-sdk` API provides the same
  operation, possibly as an owned aggregate or through `Samp::from_host`.
- **Composable**: published service primitives can reproduce the operation, but
  the new facade does not provide the legacy convenience method.
- **Partial**: some data or behavior is available, but the observable contract
  is not equivalent.
- **Missing**: no service-backed safe equivalent exists. Migration needs a new
  exact-version service table, a new facade operation, or an explicit
  will-not-migrate decision.
- **Will not migrate**: the operation is intentionally excluded from core
  services and assigned to compatibility-only scope.

`SampServiceV1` and `SampNetServiceV1` are immutable. Missing operations must
not be appended to either V1 table. See
[`0009-publish-small-versioned-samp-services.md`](adr/0009-publish-small-versioned-samp-services.md).

## Inventory totals

| Legacy source | Public methods |
| --- | ---: |
| `sdk/src/facade/mod.rs` | 28 |
| `sdk/src/facade/local_player.rs` | 48 |
| `sdk/src/facade/network.rs` | 60 |
| `sdk/src/facade/pools.rs` | 29 |
| `sdk/src/facade/ui.rs` | 32 |
| `sdk/src/facade/sampfuncs.rs` | 2 |
| **Total** | **199** |

## Root and probes (`facade/mod.rs`, 28)

| Legacy facade | Status | Legacy methods | Service-backed replacement or gap |
| --- | --- | --- | --- |
| `Samp` | Equivalent | `connect`, `connect_to` | `samp::Samp::connect`; named connection is `modkit_sdk::Host::connect_to` followed by `samp::Samp::from_host`. |
| `Samp` | Equivalent | `status` | `modkit_sdk::Host::status`; callers retain the `Host` used by `Samp::from_host`. |
| `Samp` | Equivalent | `version`, `game_state`, `set_game_state` | `samp::Samp::version`, `samp::Samp::game_state`, `samp::Samp::set_game_state`. |
| `Samp` | Equivalent | `net`, `chat`, `chat_input`, `labels`, `dialogs`, `cursor`, `scoreboard`, `local`, `players`, `anim`, `textdraws`, `objects`, `pickups`, `vehicles`, `gangzones` | `samp::Samp` exposes the same capabilities through direct methods and focused service-backed facades. |
| `Samp` | Composable | `server` | `Samp::server_info` returns the owned aggregate directly; a retained server facade adds no state or behavior. |
| `Samp` | Will not migrate | `probe`, `sampfuncs` | Process-module compatibility probes and SAMPFUNCS exports do not belong in the core `samp` crate; Phase 12 may provide them in `sampfuncs-compat`. |
| `Probe` | Will not migrate | `is_samp_loaded`, `is_sampfuncs_lua_loaded`, `is_sampfuncs_loaded`, `is_samp_available` | Host/service readiness replaces SA-MP availability for new plugins. SAMPFUNCS module probes remain compatibility-only Phase 12 scope. |

## Local player, players, and animations (`facade/local_player.rs`, 48)

| Legacy facade | Status | Legacy methods | Service-backed replacement or gap |
| --- | --- | --- | --- |
| `Local` | Equivalent | `player` | `samp::Samp::local_player` returns the owned aggregate. |
| `Local` | Equivalent | `request_class`, `send_interior_change`, `send_spawn`, `send_enter_vehicle`, `send_exit_vehicle` | Same typed Protocol convenience methods on `samp::Net`. |
| `Local` | Equivalent | `spawn`, `set_special_action`, `set_nickname`, `force_unoccupied_sync`, `force_aim_sync`, `force_onfoot_sync`, `force_stats_sync`, `force_trailer_sync`, `force_vehicle_sync`, `force_passenger_sync`, `force_weapons_sync` | `samp::Samp::local` queues the same native actions through `SampPlayerServiceV1` and returns Core receipts. |
| `Players` | Equivalent | `get` | `samp::Samp::player` returns `Option<PlayerInfo>`. |
| `Players` | Equivalent | `remote_state`, `is_defined`, `is_paused`, `count`, `max_id` | `samp::Samp::players` uses copied pool state through `SampPlayerServiceV1`. |
| `Players` | Composable | `player` | `samp::Players::player` provides state, sync, mapping, and colour operations; identity/profile fields compose with the owned `PlayerInfo` returned by `Samp::player`. |
| `Players` | Equivalent | `id_by_ped_handle` | `samp::Players::id_by_ped_handle` uses the live copied mapping in `SampPoolServiceV1`. |
| `Player` | Equivalent | `id`, `nickname`, `is_npc`, `score`, `ping`, `colour` | Fields on the owned `samp::PlayerInfo` value. |
| `Player` | Composable | `is_connected`, `is_defined` | `Samp::player(id)?.is_some()` or `Players::is_defined(id)`. |
| `Player` | Equivalent | `armour`, `health`, `is_paused`, `special_action`, `animation_id`, `streamed_out_position`, `onfoot_sync`, `vehicle_sync`, `passenger_sync`, `trailer_sync`, `aim_sync`, `set_colour` | `samp::Players::player` reads owned state/synchronization snapshots and queues colour mutation through `SampPlayerServiceV1`. |
| `Player` | Equivalent | `ped_handle` | `samp::Player::ped_handle` uses the live copied mapping in `SampPoolServiceV1`. |
| `Anim` | Equivalent | `get`, `find` | `samp::Samp::animations` returns owned animation strings and optional IDs through `SampPlayerServiceV1`. |

## Network and server (`facade/network.rs`, 60)

| Legacy facade | Status | Legacy methods | Service-backed replacement or gap |
| --- | --- | --- | --- |
| `Net` | Equivalent | `incoming_emulation_ready` | `samp::Net::incoming_emulation_ready`; the new method preserves host errors instead of collapsing them to `false`. |
| `Net` | Equivalent | `send_packet`, `send_packet_with_options`, `send_rpc`, `send_rpc_with_options` | Same methods on `samp::Net`; the non-options methods use `SendOptions::default()` and all preserve exact bit length. |
| `Net` | Equivalent | `send_packet_stream`, `send_packet_stream_with_options`, `send_rpc_stream`, `send_rpc_stream_with_options` | Same `BitStream` convenience methods on `samp::Net`. |
| `Net` | Equivalent | `emulate_incoming_packet`, `emulate_incoming_rpc` | Same methods on `samp::Net`, with exact bit length and Core-backed receipts. |
| `Net` | Equivalent | `on_packet`, `on_rpc`, `on_incoming_typed_packet`, `on_outgoing_typed_packet`, `on_incoming_typed_rpc`, `on_outgoing_typed_rpc` | Same service-backed subscriptions; callback event data remains callback-scoped. |
| `Net` | Equivalent | `on_packet_id`, `on_rpc_id` | Same methods on `samp::Net`; non-matching event IDs continue without invoking the handler. |
| `Net` | Equivalent | `rpc_name`, `packet_name` | Same catalog helpers on `samp::Net`, backed by `samp-protocol`. |
| `Net` | Equivalent | `encode_string` | `samp::Net::encode_string` returns owned canonical `EncodedBits`; `encode_string_into` retains caller-provided-buffer access. |
| `Net` | Equivalent | `decode_string` | `samp::Net::decode_string` decodes an arbitrary plugin-owned `BitStream` through `SampCodecServiceV1`, advances its read cursor only on success, and returns owned bytes. |
| `Net` | Equivalent | `send_chat`, `send_request_spawn`, `send_request_class`, `send_interior_change`, `send_spawn`, `send_enter_vehicle`, `send_exit_vehicle`, `send_dialog_response`, `send_click_player`, `send_click_textdraw`, `send_death_by_player`, `send_menu_quit`, `send_menu_select_row`, `send_picked_up_pickup`, `send_vehicle_destroyed`, `send_vehicle_damage`, `send_scm_event`, `send_give_damage`, `send_take_damage`, `send_edit_attached_object`, `send_edit_object`, `send_rcon_command`, `send_aim_sync`, `send_bullet_sync`, `send_vehicle_sync`, `send_player_sync`, `send_spectator_sync`, `send_trailer_sync`, `send_passenger_sync`, `send_unoccupied_sync` | Same typed convenience methods on `samp::Net`; RPCs use `HIGH_PRIORITY + RELIABLE`, while Packets use `HIGH_PRIORITY + UNRELIABLE_SEQUENCED`, both on channel 0 without timestamps, before one generic service submission. |
| `Net` | Equivalent | `set_send_rate`, `connect`, `disconnect` | Same queued methods on `samp::Net`, backed by `SampControlServiceV1`. |
| `Server` | Equivalent | `info`, `hostname`, `address`, `port` | `samp::Samp::server_info` returns one owned `ServerInfo` aggregate with the three fields. |

## Pools and world mappings (`facade/pools.rs`, 29)

| Legacy facade | Status | Legacy methods | Service-backed replacement or gap |
| --- | --- | --- | --- |
| `Textdraws` | Equivalent | `exists`, `get`, `create`, `delete`, `set_position`, `set_style`, `set_letter_style`, `set_proportional`, `set_shadow`, `set_outline`, `set_box`, `set_alignment`, `set_text`, `set_model_style` | `samp::Samp::textdraws` uses checked IDs, owned fixed snapshots, and Core-backed queued mutations through `SampTextdrawServiceV1`. |
| `Labels` | Equivalent | `exists`, `get`, `delete`, `set_text`, `create`, `create_at` | `samp::Labels` uses exact-version `SampTextLabelServiceV1`; automatic creation returns a typed `TextLabelCreateReceipt`. |
| `Objects` | Equivalent | `exists`, `handle`, `id_by_handle` | `samp::Samp::objects` uses checked IDs and optional nonzero handles through `SampPoolServiceV1`. |
| `Pickups` | Equivalent | `handle`, `id_by_handle` | `samp::Samp::pickups` uses checked IDs and optional nonzero handles through `SampPoolServiceV1`. |
| `Vehicles` | Equivalent | `exists`, `handle`, `id_by_handle` | `samp::Samp::vehicles` uses checked IDs and optional nonzero handles through `SampPoolServiceV1`. |
| `Gangzones` | Equivalent | `get` | `samp::Samp::gangzones` returns an owned optional snapshot through `SampPoolServiceV1`. |

## UI (`facade/ui.rs`, 32)

| Legacy facade | Status | Legacy methods | Service-backed replacement or gap |
| --- | --- | --- | --- |
| `Dialogs` | Equivalent | `active`, `last_response`, `is_active`, `selected_item`, `list_item_count`, `set_selected_item`, `show`, `set_client_side`, `close_with_button`, `set_editbox_text` | `samp::Ui::dialogs` uses owned fixed snapshots and Core-backed queued mutation receipts through `SampUiServiceV1`. |
| `Chat` | Equivalent | `add`, `add_with_style`, `display_mode`, `entry`, `set_display_mode`, `set_entry`, `death_window` | `samp::Chat` exposes the same state and mutations; `add_death` replaces the weightless intermediate death-window facade. |
| `Chat` | Composable | `is_visible` | Compare `Chat::display_mode()` with `ChatDisplayMode::Off`. |
| `ChatInput` | Equivalent | `register_command` | `samp::Chat::register_command`; ownership and draining use the common Core subscription. |
| `ChatInput` | Equivalent | `is_active`, `text`, `is_command_defined`, `set_text`, `set_enabled`, `process` | `samp::Ui::chat_input` uses copied state and Core-backed receipts through `SampUiServiceV1`. |
| `DeathWindow` | Equivalent | `add` | `samp::Chat::add_death` queues the same bounded copied request without a separate facade. |
| `Cursor` | Equivalent | `mode`, `is_active`, `set_mode`, `toggle` | `samp::Ui::cursor` uses typed modes and queued actions through `SampUiServiceV1`. |
| `Scoreboard` | Equivalent | `is_open`, `toggle` | `samp::Ui::scoreboard` uses copied state and queued actions through `SampUiServiceV1`. |

## SAMPFUNCS compatibility (`facade/sampfuncs.rs`, 2)

| Legacy facade | Status | Legacy methods | Service-backed replacement or gap |
| --- | --- | --- | --- |
| `Sampfuncs` | Will not migrate | `is_loaded`, `log_console` | These compatibility operations do not belong in core SA-MP services. Phase 12 decides whether `sampfuncs-compat` publishes the external console export. |

## Command receipts

Legacy `CommandReceipt<()>` behavior maps to the Core-backed
`samp::CommandReceipt`: `try_take`/`poll`, waiting, explicit release,
release-on-drop, timeout retry, wait rejection, and shutdown completion stay
host-owned. The optional raw receipt ID is available for diagnostics only.

`samp::TextLabelCreateReceipt` maps successful automatic creation from
`CommandCompletionV1.value0` to a checked `TextLabelId`. Completion status
remains authoritative; failure never becomes a successful zero sentinel.

## Compatibility-only decisions

Phase 11 explicitly does not migrate process-module probes or SAMPFUNCS console
integration into the core `samp` crate. They do not describe SA-MP service
state, require optional external-module semantics, and remain Phase 12
`sampfuncs-compat` scope. This decision closes the safe-facade matrix without
publishing misleading core aliases.

## Unsafe raw inventory

The 23 public `unsafe` accessors in `sdk/src/raw.rs` are outside the 199-method
safe-facade count:

`base`, `rakclient`, `rakpeer`, `player_pool`, `vehicle_pool`, `player`,
`rakclient_function`, `rpc_node`, `rpc_callback`, `bitstream_data`, `chat`,
`death_window`, `dialog`, `misc`, `chat_input`, `net_game`, `server_settings`,
`pools`, `text_label_pool`, `textdraw_pool`, `pickup_pool`, `object_pool`, and
`gangzone_pool`.

`samp::raw::bitstream_data` is migrated because its pointer refers only to
plugin-owned `BitStream` storage and its lifetime can be stated exactly.

The other 22 accessors will not migrate. They expose volatile SA-MP module,
singleton, pool, vtable, or RPC-node addresses whose profile, thread, and
lifetime contracts cannot be represented by the current Services without
making Host-internal native pointers a de facto public ABI. They remain only in
the frozen legacy SDK during its deprecation window. A future raw Service
requires a separate evidence-backed ABI decision; safe facade coverage does not
imply raw-address coverage.

## Phase 11 implementation order

1. Add facade-only Protocol convenience methods where `SampNetServiceV1`
   already provides the exact operation.
2. Design a new immutable SA-MP service version for connection, local/player,
   UI, and pool operations. Reuse the current host/native backend; do not copy
   undocumented constants into a new profile.
3. Add typed text-label creation completion and parity tests.
4. Decide every raw accessor and every remaining Missing row explicitly.
5. Migrate examples, retain one legacy smoke, then document deprecation of
   `SampClientSdk_GetApiV1` without removing it.
