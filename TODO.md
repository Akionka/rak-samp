# Repositioning TODO

[PLAN.md](PLAN.md) is the decision-complete implementation plan. The approved
[repositioning proposal](docs/repositioning-proposal.md) records the motivation
and product decisions; when wording differs, follow `PLAN.md`.

The target release is the breaking `0.1.0-alpha.4` cutover to
`samp-client-sdk`. Check an SF.lua mapping only when it is available through the
new facade (or the explicit `unsafe raw` tier), covered by tests, and documented.
An existing `HostApi` helper is reusable implementation material, not completion
of the renamed public API.

## Supporting plans

- [x] Complete the focused [deduplication plan](docs/deduplication-plan.md),
  using its detailed checklist to track implementation and verification.
- [x] Complete the [structural split follow-up plan](docs/structural-split-plan.md),
  reducing large Rust roots without changing public paths, ABI/native layouts,
  or runtime behavior.

### Structural split tracker

- [x] Extract root tests plus SDK types, resolution, subscriptions, and host
  helpers while preserving root SDK re-exports.
- [x] Split R1 singleton/native-type, textdraw, and UI operations.
- [x] Split Win32 backend forwarding, command submission/execution, request
  draining, cache refresh, and native bitstream helpers.
- [x] Extract SDK ABI/API ownership and Host API listener lifecycle.
- [x] Split remaining R1 player/pool/handle operations and event fixture support.
- [x] Consolidate Win32 hook primitives, audit release DLL exports, and complete
  the structural split plan.

Use [SF.lua](https://github.com/SF-lua/SF.lua) and
[SAMP-API.lua](https://github.com/imring/SAMP-API.lua) as the primary native
references for offsets, addresses, structures, field layouts, calling
conventions, enums, and client behavior. Keep the independent C++ layout fixture
for C++↔Rust packing checks when a referenced native structure crosses the host
boundary.

## Delivery checklist

### Baseline

- [x] Commit the pre-repositioning working tree unchanged on `feature/helpers`.
- [x] Record the approved roadmap in [PLAN.md](PLAN.md).
- [x] Confirm the baseline passes formatting, 136 workspace tests, and Clippy
  with warnings denied.

### Phase 1 — cleanup and rebrand

- [x] Delete `tests/e2e/`, `examples/validation_plugin/`,
  `examples/validation_unloader/`, `REVIEW.md`, and `VALIDATION.md`.
- [x] Remove deleted workspace members, cargo-make tasks, CI jobs, release
  inputs, validation references, and obsolete ABI self-tests.
- [x] Keep the C++ RakNet layout fixture, its Rust tests, and `build.rs` wiring.
- [x] Move the public crate to `sdk/` as package `samp-client-sdk`; rename the
  host package to `samp-client-sdk-host` and deploy `samp_client_sdk.asi`.
- [x] Rename crate imports, examples, logs, release archives, repository/docs.rs
  metadata, host discovery, ABI types, and the export to
  `SampClientSdk_GetApiV1`.
- [x] Replace R1 PE/signature/fingerprint feature gates with fixed offsets plus
  ordinary pointer, range, capacity, and enum validation. Keep build detection
  only for selecting the recognized networking offset table.
- [x] Rewrite `README.md`, `CORE.md`, `ARCHITECTURE.md`, and `AGENTS.md` for the
  two-pillar SDK, R1 bridge compatibility, layout-fixture rule, and removal of
  the live-validation lifecycle.
- [x] Keep surviving behavior unchanged for SA-MP 0.3.7 R1 on GTA SA 1.0 US,
  apart from the intentional package/symbol compatibility break.

### Phase 2 — game-thread foundation

- [x] Hook `CGame::Process` at `0x53E4B0`, retain its trampoline, call the
  original exactly once per entry, and restore it during shutdown.
- [x] Remove cache and UI pumping from the incoming-packet detour.
- [x] Add the bounded 256-entry owned `GameCommand` queue and drain one accepted
  snapshot after the original game process call on each tick.
- [x] Migrate dialog, chat, and death-window queues into `GameCommand`.
- [x] Queue every plugin-thread native mutation and explicit RakClient
  send/emulation; keep callback-local packet/RPC replacement synchronous.
- [x] Add host-owned command IDs, fixed `repr(C)` result storage, poll,
  timed-wait, release, detach-on-drop, timeout retry, and shutdown completion.
- [x] Reject waits from the game thread and listener callbacks.
- [x] Publish one coherent cache generation per tick: refresh lightweight
  global/pool directories eagerly and heavy requested/active records through
  bounded, deduplicated refresh queues.
- [x] Clear stale generations and pending heavy records across connection,
  version, and shutdown transitions.

### Phase 3 — facade and feature completion

- [x] Introduce `Samp::connect`, `Samp::connect_to`, subsystem facades, checked
  SA-MP ID newtypes, typed GTA handles, `CommandReceipt<T>`, and typed errors.
- [x] Make the raw ABI wrapper private or documentation-hidden and expose native
  addresses only through the explicit `unsafe raw` module.
- [x] Move subscriptions, typed events, exact sends/emulation, string codecs,
  protocol catalogs, and owned `BitStream` behind `samp.net()`.
- [x] Migrate all existing cached reads to their facade targets without
  plugin-thread native calls.
- [ ] Implement queued UI, player, pool/entity, connection, command-registry,
  sync, and dialog-response mutations in bounded vertical slices.
- [ ] Complete every baseline and extension mapping below; leave no provisional,
  excluded, duplicate, or unclassified function.

### Verification and finalization

- [ ] Test game-hook lifecycle and ordering, queue FIFO/frame boundaries,
  capacity, result/error/timeout paths, deadlock rejection, and shutdown.
- [ ] Test coherent cache generations, transition invalidation, eager directory
  refresh, bounded heavy refresh, and absence of plugin-thread native reads.
- [ ] Add mock-ABI facade coverage for every subsystem, ID/handle bounds, owned
  strings, command result types, and unsafe boundaries.
- [ ] Preserve exact packet/RPC vectors, exactly-once incoming emulation,
  listener ordering, unload synchronization, and C++↔Rust layout tests.
- [ ] Run `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo build --workspace --release --locked`.
- [ ] Mark the proposal implemented and audit the renamed release archive,
  examples, symbols, checksums, README, and license.
- [ ] Create a recoverable backup branch, squash `feature/helpers` since
  `master` into one repositioning commit, verify tree identity, and retain the
  backup until review. Do not rewrite `master` or the existing alpha tags.

## SF.lua compatibility map

Source: [`SF.lua` at `d869b8fb2ac9b527209e05376c19f3c96ee318e5`](https://github.com/SF-lua/SF.lua/tree/d869b8fb2ac9b527209e05376c19f3c96ee318e5).
The baseline contains 207 declared functions. The conditional
`isSampfuncsConsoleActive` fallback and `isSampfuncsLoaded` alias are compatibility
stubs rather than SDK targets. The 14 names commented as future work in
`SFlua/init.lua` are classified separately after the baseline.

Tiers:

- **Safe owned/read** — cached reads, owned `BitStream` operations, pure
  catalogs, typed handle conversion, and owned subscriptions.
- **Queued mutation** — bounded native state/UI changes, connection actions,
  sends, emulation, and native command registration executed on the game tick.
- **Unsafe raw** — native pointers, code addresses, and callback-table access.

<!-- sf-lua-baseline:start -->
### Basic (`basic.lua`) — 5

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetBase` | Unsafe raw | `raw::base` |
| [x] | `sampGetVersion` | Safe owned/read | `Samp::version` |
| [x] | `isSampLoaded` | Safe owned/read | `Samp::probe` host-loaded status |
| [x] | `isSampfuncsLuaLoaded` | Safe owned/read | `Samp::probe` recognized-build status |
| [x] | `isSampAvailable` | Safe owned/read | `Samp::probe` ready status |

### Chat and death window (`chat.lua`, `deathwindow.lua`) — 10

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetChatInfoPtr` | Unsafe raw | `raw::chat` |
| [x] | `sampAddChatMessage` | Queued mutation | `Chat::add` |
| [x] | `sampGetChatDisplayMode` | Safe owned/read | `Chat::display_mode` |
| [x] | `sampSetChatDisplayMode` | Queued mutation | `Chat::set_display_mode` |
| [x] | `sampGetChatString` | Safe owned/read | `Chat::entry` |
| [x] | `sampSetChatString` | Queued mutation | `Chat::set_entry` |
| [x] | `sampIsChatVisible` | Safe owned/read | `Chat::is_visible` |
| [x] | `sampAddChatMessageEx` | Queued mutation | `Chat::add_with_style` |
| [x] | `sampGetKillInfoPtr` | Unsafe raw | `raw::death_window` |
| [x] | `sampAddDeathMessage` | Queued mutation | `Chat::death_window().add` |

### Dialog (`dialog.lua`) — 16

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetDialogInfoPtr` | Unsafe raw | `raw::dialog` |
| [x] | `sampShowDialog` | Queued mutation | `Dialogs::show` |
| [x] | `sampCloseCurrentDialogWithButton` | Queued mutation | `Dialogs::close_with_button` |
| [x] | `sampGetCurrentDialogListItem` | Safe owned/read | `Dialogs::selected_item` |
| [x] | `sampSetCurrentDialogListItem` | Queued mutation | `Dialogs::set_selected_item` |
| [x] | `sampGetCurrentDialogEditboxText` | Safe owned/read | `Dialogs::active().editbox_text` |
| [x] | `sampSetCurrentDialogEditboxText` | Queued mutation | `Dialogs::set_editbox_text` |
| [x] | `sampIsDialogActive` | Safe owned/read | `Dialogs::is_active` |
| [x] | `sampGetCurrentDialogType` | Safe owned/read | `Dialogs::active().style` |
| [x] | `sampGetCurrentDialogId` | Safe owned/read | `Dialogs::active().id` |
| [x] | `sampGetDialogCaption` | Safe owned/read | `Dialogs::active().caption` |
| [x] | `sampGetDialogText` | Safe owned/read | `Dialogs::active().text` |
| [x] | `sampIsDialogClientside` | Safe owned/read | `Dialogs::active().is_client_side` |
| [x] | `sampSetDialogClientside` | Queued mutation | `Dialogs::set_client_side` |
| [x] | `sampGetListboxItemsCount` | Safe owned/read | `Dialogs::list_item_count` |
| [x] | `sampGetListboxItemText` | Safe owned/read | `Dialogs::active().items().get` |

### Cursor and game (`game.lua`) — 5

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetMiscInfoPtr` | Unsafe raw | `raw::misc` |
| [x] | `sampToggleCursor` | Queued mutation | `Cursor::toggle` |
| [x] | `sampIsCursorActive` | Safe owned/read | `Cursor::is_active` |
| [x] | `sampGetCursorMode` | Safe owned/read | `Cursor::mode` |
| [x] | `sampSetCursorMode` | Queued mutation | `Cursor::set_mode` |

### Gangzones (`gangzone.lua`) — 1

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetGangzonePoolPtr` | Unsafe raw | `raw::gangzone_pool` |

### Chat input (`input.lua`) — 9

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetInputInfoPtr` | Unsafe raw | `raw::chat_input` |
| [x] | `sampRegisterChatCommand` | Queued mutation | `ChatInput::register_command` returning a subscription |
| [x] | `sampUnregisterChatCommand` | Queued mutation | `ChatCommandSubscription::unregister_and_wait` |
| [x] | `sampSetChatInputText` | Queued mutation | `ChatInput::set_text` |
| [x] | `sampGetChatInputText` | Safe owned/read | `ChatInput::text` |
| [x] | `sampSetChatInputEnabled` | Queued mutation | `ChatInput::set_enabled` |
| [x] | `sampIsChatInputActive` | Safe owned/read | `ChatInput::is_active` |
| [x] | `sampIsChatCommandDefined` | Safe owned/read | `ChatInput::is_command_defined` |
| [x] | `sampProcessChatInput` | Queued mutation | `ChatInput::process` |

### 3D labels (`label.lua`) — 7

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetTextlabelPoolPtr` | Unsafe raw | `raw::text_label_pool` |
| [x] | `sampCreate3dText` | Queued mutation | `Labels::create` |
| [x] | `sampIs3dTextDefined` | Safe owned/read | `Labels::exists` |
| [x] | `sampGet3dTextInfoById` | Safe owned/read | `Labels::get` |
| [x] | `sampSet3dTextString` | Queued mutation | `Labels::set_text` |
| [x] | `sampDestroy3dText` | Queued mutation | `Labels::delete` |
| [x] | `sampCreate3dTextEx` | Queued mutation | `Labels::create_at` |

### Net game and animation (`netgame.lua`) — 10

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetSampInfoPtr` | Unsafe raw | `raw::net_game` |
| [x] | `sampGetSampPoolsPtr` | Unsafe raw | `raw::pools` |
| [x] | `sampGetServerSettingsPtr` | Unsafe raw | `raw::server_settings` |
| [x] | `sampGetCurrentServerName` | Safe owned/read | `Server::hostname` |
| [x] | `sampGetCurrentServerAddress` | Safe owned/read | `Server::address` and `Server::port` |
| [x] | `sampGetGamestate` | Safe owned/read | `Samp::game_state` |
| [x] | `sampSetGamestate` | Queued mutation | `Samp::set_game_state` |
| [x] | `sampGetAnimationNameAndFile` | Safe owned/read | `Animations::get` |
| [x] | `sampFindAnimationIdByNameAndFile` | Safe owned/read | `Animations::find` |
| [x] | `sampSetSendrate` | Queued mutation | `Net::set_send_rate` |

### Objects (`object.lua`) — 3

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetObjectPoolPtr` | Unsafe raw | `raw::object_pool` |
| [x] | `sampGetObjectHandleBySampId` | Safe owned/read | `Object::handle` returning `ObjectHandle` |
| [x] | `sampGetObjectSampIdByHandle` | Safe owned/read | `ObjectHandle::to_id` |

### Pickups (`pickup.lua`) — 3

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetPickupPoolPtr` | Unsafe raw | `raw::pickup_pool` |
| [x] | `sampGetPickupHandleBySampId` | Safe owned/read | `Pickup::handle` returning `PickupHandle` |
| [x] | `sampGetPickupSampIdByHandle` | Safe owned/read | `PickupHandle::to_id` |

### Players (`player.lua`) — 43

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetPlayerPoolPtr` | Unsafe raw | `raw::player_pool` |
| [x] | `sampIsPlayerConnected` | Safe owned/read | `Player::is_connected` |
| [x] | `sampGetPlayerNickname` | Safe owned/read | `Player::nickname` |
| [x] | `sampSpawnPlayer` | Queued mutation | `Local::spawn` |
| [x] | `sampSendChat` | Queued mutation | `Net::send_chat` |
| [x] | `sampIsPlayerNpc` | Safe owned/read | `Player::is_npc` |
| [x] | `sampGetPlayerScore` | Safe owned/read | `Player::score` |
| [x] | `sampGetPlayerPing` | Safe owned/read | `Player::ping` |
| [x] | `sampRequestClass` | Queued mutation | `Local::request_class` |
| [x] | `sampSendInteriorChange` | Queued mutation | `Local::send_interior_change` |
| [x] | `sampForceUnoccupiedSyncSeatId` | Queued mutation | `Local::force_unoccupied_sync` |
| [x] | `sampGetCharHandleBySampPlayerId` | Safe owned/read | `Player::ped_handle` returning `PedHandle` |
| [x] | `sampGetPlayerIdByCharHandle` | Safe owned/read | `PedHandle::to_id` |
| [x] | `sampGetPlayerArmor` | Safe owned/read | `Player::armour` |
| [x] | `sampGetPlayerHealth` | Safe owned/read | `Player::health` |
| [x] | `sampIsPlayerPaused` | Safe owned/read | `Player::is_paused` |
| [x] | `sampSetSpecialAction` | Queued mutation | `Local::set_special_action` |
| [x] | `sampGetPlayerCount` | Safe owned/read | `Players::count` |
| [x] | `sampGetMaxPlayerId` | Safe owned/read | `Players::max_id` |
| [x] | `sampGetPlayerSpecialAction` | Safe owned/read | `Player::special_action` |
| [x] | `sampStorePlayerOnfootData` | Safe owned/read | `Player::onfoot_sync` owned snapshot |
| [x] | `sampStorePlayerIncarData` | Safe owned/read | `Player::vehicle_sync` owned snapshot |
| [x] | `sampStorePlayerPassengerData` | Safe owned/read | `Player::passenger_sync` owned snapshot |
| [x] | `sampStorePlayerTrailerData` | Safe owned/read | `Player::trailer_sync` owned snapshot |
| [x] | `sampStorePlayerAimData` | Safe owned/read | `Player::aim_sync` owned snapshot |
| [x] | `sampSendSpawn` | Queued mutation | `Local::send_spawn` |
| [x] | `sampGetPlayerAnimationId` | Safe owned/read | `Player::animation_id` |
| [x] | `sampSetLocalPlayerName` | Queued mutation | `Local::set_nickname` |
| [x] | `sampGetPlayerStructPtr` | Unsafe raw | `raw::player` |
| [x] | `sampSendEnterVehicle` | Queued mutation | `Local::send_enter_vehicle` |
| [x] | `sampSendExitVehicle` | Queued mutation | `Local::send_exit_vehicle` |
| [x] | `sampIsLocalPlayerSpawned` | Safe owned/read | `LocalPlayer::is_spawned` |
| [x] | `sampGetPlayerColor` | Safe owned/read | `Player::colour` |
| [x] | `sampForceAimSync` | Queued mutation | `Local::force_aim_sync` |
| [x] | `sampForceOnfootSync` | Queued mutation | `Local::force_onfoot_sync` |
| [x] | `sampForceStatsSync` | Queued mutation | `Local::force_stats_sync` |
| [x] | `sampForceTrailerSync` | Queued mutation | `Local::force_trailer_sync` |
| [x] | `sampForceVehicleSync` | Queued mutation | `Local::force_vehicle_sync` |
| [x] | `sampGetLocalPlayerId` | Safe owned/read | `LocalPlayer::id` |
| [x] | `sampIsPlayerDefined` | Safe owned/read | `Player::is_defined` |
| [x] | `sampGetLocalPlayerNickname` | Safe owned/read | `LocalPlayer::nickname` |
| [x] | `sampGetLocalPlayerColor` | Safe owned/read | `LocalPlayer::colour` |
| [x] | `sampSetPlayerColor` | Queued mutation | `Player::set_colour` |

### RakNet and network actions (`raknet.lua`) — 65

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `raknetBitStreamReadBool` | Safe owned/read | `BitStream::read_bool` |
| [x] | `raknetBitStreamReadBuffer` | Safe owned/read | `BitStream::read_bits` / `read_bytes` |
| [x] | `raknetBitStreamReadInt8` | Safe owned/read | `BitStream::read_u8` |
| [x] | `raknetBitStreamReadInt16` | Safe owned/read | `BitStream::read_u16` |
| [x] | `raknetBitStreamReadInt32` | Safe owned/read | `BitStream::read_u32` |
| [x] | `raknetBitStreamReadFloat` | Safe owned/read | `BitStream::read_f32` |
| [x] | `raknetBitStreamReadString` | Safe owned/read | `BitStream::read_string` |
| [x] | `raknetBitStreamResetReadPointer` | Safe owned/read | `BitStream::reset_read` |
| [x] | `raknetBitStreamResetWritePointer` | Safe owned/read | `BitStream::reset_write` |
| [x] | `raknetBitStreamIgnoreBits` | Safe owned/read | `BitStream::ignore_bits` |
| [x] | `raknetBitStreamSetWriteOffset` | Safe owned/read | `BitStream::set_write_offset` |
| [x] | `raknetBitStreamSetReadOffset` | Safe owned/read | `BitStream::set_read_offset` |
| [x] | `raknetBitStreamGetNumberOfBitsUsed` | Safe owned/read | `BitStream::len_bits` |
| [x] | `raknetBitStreamGetNumberOfBytesUsed` | Safe owned/read | `BitStream::len_bytes` |
| [x] | `raknetBitStreamGetNumberOfUnreadBits` | Safe owned/read | `BitStream::remaining_bits` |
| [x] | `raknetBitStreamGetWriteOffset` | Safe owned/read | `BitStream::write_offset` |
| [x] | `raknetBitStreamGetReadOffset` | Safe owned/read | `BitStream::read_offset` |
| [x] | `raknetBitStreamGetDataPtr` | Unsafe raw | `raw::bitstream_data` |
| [x] | `raknetNewBitStream` | Safe owned/read | `BitStream::new` |
| [x] | `raknetDeleteBitStream` | Safe owned/read | `BitStream` ownership and `Drop` |
| [x] | `raknetResetBitStream` | Safe owned/read | `BitStream::clear` |
| [x] | `raknetBitStreamWriteBool` | Safe owned/read | `BitStream::write_bool` |
| [x] | `raknetBitStreamWriteInt8` | Safe owned/read | `BitStream::write_u8` |
| [x] | `raknetBitStreamWriteInt16` | Safe owned/read | `BitStream::write_u16` |
| [x] | `raknetBitStreamWriteInt32` | Safe owned/read | `BitStream::write_u32` |
| [x] | `raknetBitStreamWriteFloat` | Safe owned/read | `BitStream::write_f32` |
| [x] | `raknetBitStreamWriteBuffer` | Safe owned/read | `BitStream::write_bits` / `write_bytes` |
| [x] | `raknetBitStreamWriteString` | Safe owned/read | `BitStream::write_string` |
| [x] | `raknetBitStreamDecodeString` | Safe owned/read | `Net::decode_string` |
| [x] | `raknetBitStreamEncodeString` | Safe owned/read | `Net::encode_string` |
| [x] | `raknetBitStreamWriteBitStream` | Safe owned/read | `BitStream::write_stream` |
| [x] | `raknetSendRpcEx` | Queued mutation | `Net::send_rpc_with_options` |
| [x] | `raknetSendBitStreamEx` | Queued mutation | `Net::send_packet_with_options` |
| [x] | `raknetSendRpc` | Queued mutation | `Net::send_rpc` |
| [x] | `raknetSendBitStream` | Queued mutation | `Net::send_packet` |
| [x] | `raknetGetRpcName` | Safe owned/read | `Net::rpc_name` |
| [x] | `raknetGetPacketName` | Safe owned/read | `Net::packet_name` |
| [x] | `sampGetRakclientInterface` | Unsafe raw | `raw::rakclient` |
| [x] | `sampGetRakpeer` | Unsafe raw | `raw::rakpeer` |
| [x] | `sampSendAimData` | Queued mutation | `Net::send_aim_sync` |
| [x] | `sampSendBulletData` | Queued mutation | `Net::send_bullet_sync` |
| [x] | `sampSendIncarData` | Queued mutation | `Net::send_vehicle_sync` |
| [x] | `sampSendOnfootData` | Queued mutation | `Net::send_player_sync` |
| [x] | `sampSendSpectatorData` | Queued mutation | `Net::send_spectator_sync` |
| [x] | `sampSendTrailerData` | Queued mutation | `Net::send_trailer_sync` |
| [x] | `sampSendPassengerData` | Queued mutation | `Net::send_passenger_sync` |
| [x] | `sampSendUnoccupiedData` | Queued mutation | `Net::send_unoccupied_sync` |
| [x] | `sampSendDamageVehicle` | Queued mutation | `Net::send_vehicle_damage` |
| [x] | `sampSendScmEvent` | Queued mutation | `Net::send_scm_event` |
| [x] | `sampSendGiveDamage` | Queued mutation | `Net::send_give_damage` |
| [x] | `sampSendTakeDamage` | Queued mutation | `Net::send_take_damage` |
| [x] | `sampSendRequestSpawn` | Queued mutation | `Net::send_request_spawn` |
| [x] | `sampSendClickPlayer` | Queued mutation | `Net::send_click_player` |
| [x] | `sampSendClickTextdraw` | Queued mutation | `Net::send_click_textdraw` |
| [x] | `sampSendDeathByPlayer` | Queued mutation | `Net::send_death_by_player` |
| [x] | `sampSendDialogResponse` | Queued mutation | `Net::send_dialog_response` |
| [x] | `sampSendEditAttachedObject` | Queued mutation | `Net::send_edit_attached_object` |
| [x] | `sampSendEditObject` | Queued mutation | `Net::send_edit_object` |
| [x] | `sampSendMenuQuit` | Queued mutation | `Net::send_menu_quit` |
| [x] | `sampSendMenuSelectRow` | Queued mutation | `Net::send_menu_select_row` |
| [x] | `sampSendPickedUpPickup` | Queued mutation | `Net::send_picked_up_pickup` |
| [x] | `sampSendRconCommand` | Queued mutation | `Net::send_rcon_command` |
| [x] | `sampSendVehicleDestroyed` | Queued mutation | `Net::send_vehicle_destroyed` |
| [x] | `sampDisconnectWithReason` | Queued mutation | `Net::disconnect` |
| [x] | `sampConnectToServer` | Queued mutation | `Net::connect` |

### Scoreboard (`scoreboard.lua`) — 2

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampToggleScoreboard` | Queued mutation | `Scoreboard::toggle` |
| [x] | `sampIsScoreboardOpen` | Safe owned/read | `Scoreboard::is_open` |

### Textdraws (`textdraw.lua`) — 24

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetTextdrawPoolPtr` | Unsafe raw | `raw::textdraw_pool` |
| [x] | `sampTextdrawIsExists` | Safe owned/read | `Textdraws::exists` |
| [x] | `sampTextdrawCreate` | Queued mutation | `Textdraws::create` |
| [x] | `sampTextdrawSetBoxColorAndSize` | Queued mutation | `Textdraws::set_box` |
| [x] | `sampTextdrawGetString` | Safe owned/read | `Textdraw::text` |
| [x] | `sampTextdrawDelete` | Queued mutation | `Textdraws::delete` |
| [x] | `sampTextdrawGetLetterSizeAndColor` | Safe owned/read | `Textdraw::letter_style` |
| [x] | `sampTextdrawGetPos` | Safe owned/read | `Textdraw::position` |
| [x] | `sampTextdrawGetShadowColor` | Safe owned/read | `Textdraw::shadow` |
| [x] | `sampTextdrawGetOutlineColor` | Safe owned/read | `Textdraw::outline` |
| [x] | `sampTextdrawGetStyle` | Safe owned/read | `Textdraw::style` |
| [x] | `sampTextdrawGetProportional` | Safe owned/read | `Textdraw::is_proportional` |
| [x] | `sampTextdrawGetAlign` | Safe owned/read | `Textdraw::alignment` |
| [x] | `sampTextdrawGetBoxEnabledColorAndSize` | Safe owned/read | `Textdraw::box_style` |
| [x] | `sampTextdrawGetModelRotationZoomVehColor` | Safe owned/read | `Textdraw::model_style` |
| [x] | `sampTextdrawSetLetterSizeAndColor` | Queued mutation | `Textdraws::set_letter_style` |
| [x] | `sampTextdrawSetPos` | Queued mutation | `Textdraws::set_position` |
| [x] | `sampTextdrawSetString` | Queued mutation | `Textdraws::set_text` |
| [x] | `sampTextdrawSetModelRotationZoomVehColor` | Queued mutation | `Textdraws::set_model_style` |
| [x] | `sampTextdrawSetOutlineColor` | Queued mutation | `Textdraws::set_outline` |
| [x] | `sampTextdrawSetShadow` | Queued mutation | `Textdraws::set_shadow` |
| [x] | `sampTextdrawSetStyle` | Queued mutation | `Textdraws::set_style` |
| [x] | `sampTextdrawSetProportional` | Queued mutation | `Textdraws::set_proportional` |
| [x] | `sampTextdrawSetAlign` | Queued mutation | `Textdraws::set_alignment` |

### Vehicles (`vehicle.lua`) — 4

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampGetVehiclePoolPtr` | Unsafe raw | `raw::vehicle_pool` |
| [x] | `sampGetCarHandleBySampVehicleId` | Safe owned/read | `Vehicle::handle` returning `VehicleHandle` |
| [x] | `sampGetVehicleIdByCarHandle` | Safe owned/read | `VehicleHandle::to_id` |
| [x] | `sampIsVehicleDefined` | Safe owned/read | `Vehicles::exists` |
<!-- sf-lua-baseline:end -->

## SF.lua `init.lua` extension map — 14

These names are comments in the pinned source rather than part of the 207
declared-function baseline. They are still in scope under the rule that nothing
remains permanently excluded.

| Status | Future global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [x] | `sampHasDialogRespond` | Safe owned/read | `Dialogs::last_response` |
| [x] | `sampForcePassengerSyncSeatId` | Queued mutation | `LocalPlayer::force_passenger_sync` |
| [x] | `sampForceWeaponsSync` | Queued mutation | `LocalPlayer::force_weapons_sync` |
| [x] | `sampGetRakclientFuncAddressByIndex` | Unsafe raw | `raw::rakclient_function` |
| [x] | `sampGetRpcCallbackByRpcId` | Unsafe raw | `raw::rpc_callback` |
| [x] | `sampGetRpcNodeByRpcId` | Unsafe raw | `raw::rpc_node` |
| [x] | `raknetEmulRpcReceiveBitStream` | Queued mutation | `Net::emulate_incoming_rpc` |
| [x] | `raknetEmulPacketReceiveBitStream` | Queued mutation | `Net::emulate_incoming_packet` |
| N/A | `sampSetClientCommandDescription` | SAMPFUNCS-only | Deferred by product decision: SA-MP has no command-description API or storage. |
| [x] | `sampGetStreamedOutPlayerPos` | Safe owned/read | `Player::streamed_out_position` |
| [x] | `onSendRpc` | Safe owned/read | `Net::on_rpc(Direction::Outgoing, ...)` |
| [x] | `onSendPacket` | Safe owned/read | `Net::on_packet(Direction::Outgoing, ...)` |
| [x] | `onReceiveRpc` | Safe owned/read | `Net::on_rpc(Direction::Incoming, ...)` |
| [x] | `onReceivePacket` | Safe owned/read | `Net::on_packet(Direction::Incoming, ...)` |

## Optional SAMPFUNCS interop

| Done | Capability | `samp-client-sdk` target |
| --- | --- | --- |
| [x] | Detect loaded SAMPFUNCS and write its console | `Probe::is_sampfuncs_loaded`, `Samp::sampfuncs().log_console` |

## Hook diagnostics

| Done | Capability | Location |
| --- | --- | --- |
| [x] | Log every successfully enabled MinHook target, detour, and trampoline | `InlineHook::enable` |

## Multi-build native support matrix

Target builds: SA-MP 0.3.7 R1, R3-1, R5-1, and DL. `R5` below always means
R5-1. R2 and R4-2 remain recognized only because `AddressSet` retains their
existing network values; they are outside this support commitment.

`[x]` means the exact value is implemented and covered by the existing R1
profile. `[ ] SAPI` means extract the exact value and layout from the pinned
SAMP-API source, cross-check it against a version-pinned client, then wire and
test it. `[ ] SF/disasm` means establish it from SF.lua plus the matching
SAMPFUNCS/client disassembly. No cell may become `[x]` merely because an RVA is
listed in an external project: it also needs the specified profile test and a
live smoke test. Do not commit client DLLs, SAMPFUNCS DLLs, or proprietary
headers; record their filename, SHA-256, and provenance in test notes instead.

- [x] Create the per-build address/layout/task matrix.
- [x] Inventory the currently supplied client and SAMPFUNCS artifacts without
  committing the binaries themselves.
- [x] Extract every R3-1/R5-1 candidate exposed by the pinned SAMP-API source
  into this matrix; leave gaps explicitly assigned to disassembly.
- [x] Capture a legal, version-pinned validation binary and SHA-256 for R1,
  R3-1, R5-1, and DL; record its source beside the corresponding fixture.
- [x] Add one independent native-layout fixture and live smoke checklist per
  build before enabling a direct helper for that build. The initial non-R1
  gates cover `CNetGame`, chat input, and dialog only; the wider layout-family
  matrix remains incomplete until each helper's consumed fields are proven.
- [x] Statically cross-check every R3-1/R5-1/DL `AddressSet` RVA against the
  pinned client DLLs before treating the existing network path as verified.
- [x] Replace ambiguous compressor-pointer RVAs with verified
  `StringCompressor::Instance()` accessors for R1/R3-1/R5-1/DL; retain the
  slot fallback for unverified R2/R4-2 builds.
- [x] Record isolated R3-1/R5-1/DL attach outcomes without promoting any
  incomplete profile; see `docs/native-layout-smoke.md`.
- [x] Retry the R5-1 isolated attach after clearing the stale launcher; record
  the constructor, RPC-hook, and first-packet result without promoting the
  incomplete profile.
- [x] Add an opt-in isolated network smoke plugin for the native codec and
  blocked exact-bit packet/RPC emulation paths; retain separate live proof for
  outbound sends and original-handler delivery.
- [x] Persist the isolated network smoke stage result without relying on
  SAMPFUNCS, so R5/DL validation is observable from the test root.
- [ ] Keep the existing R1 direct profile active until its replacement passes
  the R1 layout and in-game smoke tests unchanged.

| Artifact evidence | R1 | R3-1 | R5-1 | DL |
| --- | --- | --- | --- | --- |
| Local `samp.dll` | [x] installed R1; entry `0x31DF13`; SHA-256 `7E30F3C9CD99D5E2932410F486E8139AFFA2DAD19BD65AD9C328F6A4071943F7` | [x] extracted from supplied `sa-mp-0.3.7-R3-1-install.exe`; SHA-256 `9C9B2CC31A4CED6967420B1880C096B5C4E7630E227AA379BE4019C21B6FDDC1` | [x] extracted from supplied `sa-mp-0.3.7-R5-1-install.exe`; SHA-256 `B72B5DBE725F81864CA3F78BC7063BDA56CC05FC7188AF822FA7A754432553A2` | [x] extracted from supplied `sa-mp-0.3.DL-R1-install.exe`; SHA-256 `BCCDB297464BD382625635BE25585DF07A8FA6668BC0015650708E3EB4FFCD4B` |
| SAMPFUNCS 5.7.1 disassembly input | [x] SHA-256 `3403C4100993F48A36F86F3F93AD3451A6D99CC4394E8F4516011AEB25886D19` | [x] `EAA0A0AF1E074983E4CFCE619AC611D37DFEA467265DBA10ED94131F968BEF0F` | [x] `642FC80F022EF41D9D6C11988DE03E3BF32AFB4A51B5806F1FECEBC4E73C9FB0` | [x] `81F0E5225FCC4A15F43DB1440AADE25276261878C50A93AFD7C8EE4302590EB3` |

### Build identity and profile gates

| Capability | R1 | R3-1 | R5-1 | DL | Evidence / required validation |
| --- | --- | --- | --- | --- | --- |
| `samp.dll` entry point | [x] `0x31DF13` | [ ] `0x0CC4D0` | [ ] `0x0CBC90` | [ ] `0x0FDB60` | `src/client.rs`; compare PE optional-header RVA and pinned SHA-256 |
| Network `AddressSet` selection | [x] | [ ] live verify | [ ] live verify | [ ] SF/disasm verify | `src/client.rs`; constructor, RPC, packet, lock, codec smoke tests |
| Native profile selected after build detection | [x] `NativeProfile::R1` | [ ] `R3ClientProfile` | [ ] `R5ClientProfile` | [ ] `DlClientProfile` | Version-neutral profile trait/enum; unsupported operations must remain gated |
| Game-process hook target | [x] GTA 1.0 US | [ ] GTA shared verify | [ ] GTA shared verify | [ ] GTA shared verify | Existing MinHook lifecycle test plus in-game attach/detach |
| Dialog-close hook target | [x] | [ ] profile RVA + ABI | [ ] profile RVA + ABI | [ ] SF/disasm | Hook, trampoline, and dialog-response smoke test |
| RakClient vtable contract | [x] | [ ] verify slots/ABI | [ ] verify slots/ABI | [ ] SF/disasm | Packet/RPC send/receive and restoration tests |
| Native bitstream ABI | [x] | [ ] verify | [ ] verify | [ ] SF/disasm | C++ fixture plus exact-bit packet/RPC smoke test |

### Network and codec RVAs (`AddressSet`)

These RVAs are already selected by the current backend. Each non-R1 cell still
needs a matching binary/disassembly check before it is treated as supported;
this table prevents the existing raw-network path from being confused with
complete native-helper support.

Static cross-check against the pinned DLLs is complete: the 21 code RVAs
(constructor, incoming RPC, allocation, lock/unlock, encode, decode) disassemble
as the matching function entries on R3-1, R5-1, and DL. The compressor slots
are mapped in each image's zero-filled `.data` tail and are confirmed by their
native `StringCompressor::Instance()` accessors: R3-1 `0x534F0` loads
`0x121914`, R5-1 `0x53C30` loads `0x121A3C`, and DL `0x536F0` loads
`0x15FA54`. This proves the selected image RVAs, not that packet/RPC behavior
or the live object pointer is safe; those checks remain required below.

| Address / use | R1 | R3-1 | R5-1 | DL | Evidence / validation |
| --- | --- | --- | --- | --- | --- |
| Incoming RPC handler | [x] `0x372F0` | [ ] `0x3A6A0` | [ ] `0x3ADE0` | [ ] `0x3A8A0` | `src/client.rs`; MinHook/trampoline RPC smoke |
| RakClient constructor | [x] `0x33DC0` | [ ] `0x37170` | [ ] `0x378B0` | [ ] `0x37370` | `src/client.rs`; constructor/vtable capture smoke |
| Allocate packet | [x] `0x347E0` | [ ] `0x37B90` | [ ] `0x382D0` | [ ] `0x37D90` | `src/client.rs`; incoming packet emulation smoke |
| Bitstream write lock | [x] `0x35B10` | [ ] `0x38EC0` | [ ] `0x39600` | [ ] `0x390C0` | `src/client.rs`; exact-bit send smoke |
| Bitstream write unlock | [x] `0x35B50` | [ ] `0x38F00` | [ ] `0x39640` | [ ] `0x39100` | `src/client.rs`; exact-bit send smoke |
| String encode | [x] `0x506B0` | [ ] `0x53A60` | [ ] `0x541A0` | [ ] `0x53C60` | SF.lua for R1/R3/R5; codec round-trip |
| String decode | [x] `0x507E0` | [ ] `0x53B90` | [ ] `0x542D0` | [ ] `0x53D90` | SF.lua for R1/R3/R5; codec round-trip |
| String-compressor pointer | [x] `0x10D894` | [ ] `0x121914` | [ ] `0x121A3C` | [ ] `0x15FA54` | Static `StringCompressor::Instance()` cross-check; validate the live object/range before codec call |

### Native singleton and method RVAs

Each row maps one current R1 fixed address from
`src/platform/win32/r1_client/addresses.rs`. R3-1 and R5-1 extraction uses the
matching `sampapi/v037r3` or `sampapi/v037r5` class declaration; DL requires a
version-pinned SAMPFUNCS/client disassembly. A value alone is insufficient: add
the matching call signature and object/layout proof to that profile.

| Native address / use | R1 | R3-1 | R5-1 | DL |
| --- | --- | --- | --- | --- |
| `DIALOG_SINGLETON_RVA` | [x] `0x21A0B8` | [ ] `0x26E898` SAPI | [ ] `0x26EB50` SAPI | [ ] SF/disasm |
| `DIALOG_SHOW_RVA` | [x] `0x6B9C0` | [ ] `0x6F8C0` SAPI | [ ] `0x6FFB0` SAPI | [ ] SF/disasm |
| `DIALOG_CLOSE_RVA` | [x] `0x6C040` | [ ] `0x6FF40` SAPI | [ ] `0x70630` SAPI | [ ] SF/disasm |
| `INPUT_SINGLETON_RVA` | [x] `0x21A0E8` | [ ] `0x26E8CC` SAPI | [ ] `0x26EB84` SAPI | [ ] SF/disasm |
| `INPUT_OPEN_RVA` | [x] `0x657E0` | [ ] `0x68D10` SAPI | [ ] `0x69480` SAPI | [ ] SF/disasm |
| `INPUT_CLOSE_RVA` | [x] `0x658E0` | [ ] `0x68E10` SAPI | [ ] `0x69580` SAPI | [ ] SF/disasm |
| `INPUT_GET_COMMAND_HANDLER_RVA` | [x] `0x65A70` | [ ] `0x68FA0` SAPI | [ ] `0x69710` SAPI | [ ] SF/disasm |
| `INPUT_ADD_COMMAND_RVA` | [x] `0x65AD0` | [ ] `0x69000` SAPI | [ ] `0x69770` SAPI | [ ] SF/disasm |
| `INPUT_PROCESS_RVA` | [x] `0x65D30` | [ ] `0x69260` SAPI | [ ] `0x699D0` SAPI | [ ] SF/disasm |
| `DXUT_EDIT_BOX_SET_TEXT_RVA` | [x] `0x80F60` | [ ] SF/disasm | [ ] SF/disasm | [ ] SF/disasm |
| `DXUT_EDIT_BOX_GET_TEXT_RVA` | [x] `0x81030` | [ ] SF/disasm | [ ] SF/disasm | [ ] SF/disasm |
| `CHAT_SINGLETON_RVA` | [x] `0x21A0E4` | [ ] `0x26E8C8` SAPI | [ ] `0x26EB80` SAPI | [ ] SF/disasm |
| `CHAT_ADD_ENTRY_RVA` | [x] `0x64010` | [ ] `0x67460` SAPI | [ ] `0x67BE0` SAPI | [ ] SF/disasm |
| `CHAT_GET_MODE_RVA` | [x] `0x5D7A0` | [ ] `0x60B40` SAPI | [ ] `0x612B0` SAPI | [ ] SF/disasm |
| `SCOREBOARD_SINGLETON_RVA` | [x] `0x21A0B4` | [ ] `0x26E894` SAPI | [ ] `0x26EB4C` SAPI | [ ] SF/disasm |
| `DEATH_WINDOW_SINGLETON_RVA` | [x] `0x21A0EC` | [ ] `0x26E8D0` SAPI | [ ] `0x26EB88` SAPI | [ ] SF/disasm |
| `DEATH_WINDOW_ADD_MESSAGE_RVA` | [x] `0x66A10` | [ ] `0x69F40` SAPI | [ ] `0x6A6B0` SAPI | [ ] SF/disasm |
| `NET_GAME_SINGLETON_RVA` | [x] `0x21A0F8` | [ ] `0x26E8DC` SAPI | [ ] `0x26EB94` SAPI | [ ] SF/disasm |
| `NET_GAME_GET_STATE_RVA` | [x] `0x2E20` | [ ] `0x2E10` SAPI | [ ] `0x2E30` SAPI | [ ] SF/disasm |
| `NET_GAME_GET_PLAYER_POOL_RVA` | [x] `0x1160` | [ ] `0x1160` SAPI | [ ] `0x1170` SAPI | [ ] SF/disasm |
| `NET_GAME_GET_VEHICLE_POOL_RVA` | [x] `0x1170` | [ ] `0x1170` SAPI | [ ] `0x1180` SAPI | [ ] SF/disasm |
| `NET_GAME_SHUTDOWN_FOR_RESTART_RVA` | [x] `0xA060` | [ ] `0xA1E0` SAPI | [ ] `0xA540` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_LOCAL_PLAYER_RVA` | [x] `0x1A30` | [ ] `0x1A30` SAPI | [ ] `0x1A40` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_LOCAL_SCORE_RVA` | [x] `0x6A1F0` | [ ] `0x6E140` SAPI | [ ] `0x6E8B0` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_LOCAL_PING_RVA` | [x] `0x6A200` | [ ] `0x6E150` SAPI | [ ] `0x6E8C0` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_IS_CONNECTED_RVA` | [x] `0x10B0` | [ ] `0x10B0` SAPI | [ ] `0x10B0` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_REMOTE_PLAYER_RVA` | [x] `0x10F0` | [ ] `0x10F0` SAPI | [ ] `0x10F0` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_IS_NPC_RVA` | [x] `0xB680` | [ ] SAPI layout | [ ] SAPI layout | [ ] SF/disasm |
| `PLAYER_POOL_GET_NAME_RVA` | [x] `0x13CE0` | [ ] `0x16F00` SAPI | [ ] `0x175C0` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_SCORE_RVA` | [x] `0x6A190` | [ ] `0x6E0E0` SAPI | [ ] `0x6E850` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_PING_RVA` | [x] `0x6A1C0` | [ ] `0x6E110` SAPI | [ ] `0x6E880` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_GET_COUNT_RVA` | [x] `0x10520` | [ ] `0x13670` SAPI | [ ] `0x139F0` SAPI | [ ] SF/disasm |
| `PLAYER_POOL_SET_LOCAL_PLAYER_NAME_RVA` | [x] `0xB3E0` | [ ] `0xB5C0` SAPI | [ ] `0xB8A0` SAPI | [ ] SF/disasm |
| `VEHICLE_POOL_DOES_EXIST_RVA` | [x] `0x1140` | [ ] `0x1140` SAPI | [ ] `0x1150` SAPI | [ ] SF/disasm |
| `REMOTE_PLAYER_GET_COLOUR_ARGB_RVA` | [x] `0x12A00` | [ ] `0x15C10` SAPI | [ ] `0x16180` SAPI | [ ] SF/disasm |
| `REMOTE_PLAYER_SET_COLOUR_RVA` | [x] `0x129D0` | [ ] `0x15BE0` SAPI | [ ] `0x16150` SAPI | [ ] SF/disasm |
| `REMOTE_PLAYER_DOES_EXIST_RVA` | [x] `0x1080` | [ ] `0x1080` SAPI | [ ] `0x1080` SAPI | [ ] SF/disasm |
| `REMOTE_PLAYER_GET_STATUS_RVA` | [x] `0x12BA0` | [ ] `0x15DB0` SAPI | [ ] `0x16330` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_GET_PED_RVA` | [x] `0x2D60` | [ ] `0x2D50` SAPI | [ ] `0x2D70` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_GET_COLOUR_ARGB_RVA` | [x] `0x3D90` | [ ] `0x3DA0` SAPI | [ ] `0x3F20` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SET_COLOUR_RVA` | [x] `0x3D40` | [ ] `0x3D50` SAPI | [ ] `0x3ED0` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SET_SPECIAL_ACTION_RVA` | [x] `0x30C0` | [ ] `0x30C0` SAPI | [ ] `0x30F0` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SPAWN_RVA` | [x] `0x3AD0` | [ ] `0x3AD0` SAPI | [ ] `0x3C20` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_UNOCCUPIED_DATA_RVA` | [x] `0x4B30` | [ ] `0x4B60` SAPI | [ ] `0x4D30` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_AIM_DATA_RVA` | [x] `0x4FF0` | [ ] `0x5040` SAPI | [ ] `0x5210` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_ONFOOT_DATA_RVA` | [x] `0x4D10` | [ ] `0x4D40` SAPI | [ ] `0x4F00` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_STATS_RVA` | [x] `0x5AF0` | [ ] `0x5B10` SAPI | [ ] `0x5D00` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_TRAILER_DATA_RVA` | [x] `0x51B0` | [ ] `0x51F0` SAPI | [ ] `0x53D0` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_PASSENGER_DATA_RVA` | [x] `0x5380` | [ ] `0x53B0` SAPI | [ ] `0x5590` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_SEND_INCAR_DATA_RVA` | [x] `0x6E30` | [ ] `0x6E40` SAPI | [ ] `0x7080` SAPI | [ ] SF/disasm |
| `LOCAL_PLAYER_UPDATE_WEAPONS_RVA` | [x] `0x6080` | [ ] `0x6090` SAPI | [ ] `0x6290` SAPI | [ ] SF/disasm |
| `ONFOOT_SEND_RATE_RVA` | [x] `0xEC0A8` | [ ] `0xFE0A8` SAPI | [ ] `0xFE0A8` SAPI | [ ] SF/disasm |
| `INCAR_SEND_RATE_RVA` | [x] `0xEC0AC` | [ ] `0xFE0AC` SAPI | [ ] `0xFE0AC` SAPI | [ ] SF/disasm |
| `AIM_SEND_RATE_RVA` | [x] `0xEC0B0` | [ ] `0xFE0B0` SAPI | [ ] `0xFE0B0` SAPI | [ ] SF/disasm |
| `PED_GET_HEALTH_RVA` | [x] `0xA6610` | [ ] `0xAB4C0` SAPI | [ ] `0xABD50` SAPI | [ ] SF/disasm |
| `PED_GET_ARMOUR_RVA` | [x] `0xA6650` | [ ] `0xAB500` SAPI | [ ] `0xABD90` SAPI | [ ] SF/disasm |
| `GAME_SINGLETON_RVA` | [x] `0x21A10C` | [ ] `0x26E8F4` SAPI | [ ] `0x26EBAC` SAPI | [ ] SF/disasm |
| `GAME_SET_CURSOR_MODE_RVA` | [x] `0x9BD30` | [ ] `0x9FFE0` SAPI | [ ] `0xA06F0` SAPI | [ ] SF/disasm |
| `GAME_PROCESS_INPUT_ENABLING_RVA` | [x] `0x9BC10` | [ ] `0x9FEC0` SAPI | [ ] `0xA05D0` SAPI | [ ] SF/disasm |
| `ANIMATION_TABLE_RVA` | [x] `0xF15B0` | [ ] `0x1039D0` SF.lua | [ ] `0x1039E8` SF.lua | [ ] SF/disasm |
| `CPOOLS_GET_PED_REF` (GTA) | [x] `0x54FF60` | [ ] GTA shared verify | [ ] GTA shared verify | [ ] GTA shared verify |
| `CPOOLS_GET_VEHICLE_REF` (GTA) | [x] `0x54FFC0` | [ ] GTA shared verify | [ ] GTA shared verify | [ ] GTA shared verify |
| `LABEL_POOL_CREATE_RVA` | [x] `0x11C0` | [ ] `0x11C0` SAPI | [ ] `0x11D0` SAPI | [ ] SF/disasm |
| `LABEL_POOL_DELETE_RVA` | [x] `0x12D0` | [ ] `0x12D0` SAPI | [ ] `0x12E0` SAPI | [ ] SF/disasm |
| `TEXTDRAW_POOL_CREATE_RVA` | [x] `0x1AE20` | [ ] `0x1E1C0` SAPI | [ ] `0x1E910` SAPI | [ ] SF/disasm |
| `TEXTDRAW_POOL_DELETE_RVA` | [x] `0x1AD00` | [ ] `0x1E0A0` SAPI | [ ] `0x1E7F0` SAPI | [ ] SF/disasm |

### Native layout and raw-address matrix

The following are not function RVAs, but they are equally required before a
profile may expose a safe read/write or raw opaque address. Every individual
field consumed by the listed current R1 module must be copied into the
build-specific fixture before its profile can be enabled.

| Layout family / current R1 source | R1 | R3-1 | R5-1 | DL | Completion rule |
| --- | --- | --- | --- | --- | --- |
| Singleton pointer storage and object sizes (`singletons.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Validate singleton slot and full readable object range |
| `CNetGame`, server metadata, game state, pool roots (`memory.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Verify every consumed field offset and signedness |
| `CInput`, command table, chat editbox (`ui.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Verify command count/name/proc capacity and native calls |
| `CDialog` and DXUT list/edit controls (`ui.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Verify full dialog/DXUT layouts and callback ABI |
| Chat/death-window history entries (`chat_entries.rs`, `ui.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Verify bounded text fields, colours, and display mode |
| Player/vehicle/object/pickup/gangzone/label pools (`pools.rs`, `players.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Verify pool sizes, not-empty arrays, and pointer indirection |
| Local/remote player records and all sync structures (`players.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Fixture must cover on-foot, in-car, passenger, trailer, aim, stats |
| Textdraw and text-label structures (`textdraws.rs`, `text_labels.rs`) | [x] | [ ] SAPI fixture | [ ] SAPI fixture | [ ] SF/disasm fixture | Verify creation/deletion ABI and every mutable field |
| GTA ped/vehicle handles and `CPools` assumptions (`handles.rs`) | [x] | [ ] GTA shared fixture | [ ] GTA shared fixture | [ ] GTA shared fixture | Reconfirm GTA 1.0 US target and SAMP pointer chain |
| RakPeer size, RPC node table, native bitstream (`raw.rs`, `native_bitstream.rs`) | [x] | [ ] binary fixture | [ ] binary fixture | [ ] SF/disasm fixture | Validate all unsafe opaque-address computations |

### Profile implementation and enablement order

The profile must grow in vertical, testable slices. A build remains
`UnsupportedVersion` for a given API until the row and every dependency above
are complete; never turn on a whole build after only its network addresses are
known.

| Task | R1 | R3-1 | R5-1 | DL | Required proof |
| --- | --- | --- | --- | --- | --- |
| Replace R1-only field with a version-neutral native-profile dispatch boundary | [x] preserve behavior | [x] remains gated | [x] remains gated | [x] remains gated | `NativeProfile::select` test plus unchanged R1 queue/cache tests; no non-R1 direct helper enabled |
| Network observe/send, codec, packet/RPC emulation | [x] | [ ] | [ ] | [ ] | Hook, vtable, and exact-bit smoke tests |
| Lifecycle/version/status and raw module base | [x] | [ ] | [ ] | [ ] | Attachment/version identity test |
| Cached game/server/local-player scalars | [x] | [ ] | [ ] | [ ] | Profile fixture and game-thread publication test |
| UI, dialog, chat input, native command registry | [x] | [ ] | [ ] | [ ] | Layout fixture plus in-game interaction test |
| Player/pool/entity snapshots and handles | [x] | [ ] | [ ] | [ ] | Layout fixture, transition invalidation, in-game smoke |
| Local-player commands and force sync | [x] | [ ] | [ ] | [ ] | Queue/receipt test and in-game packet verification |
| Textdraw/text-label/gangzone commands | [x] | [ ] | [ ] | [ ] | Layout fixture, queue/receipt test, in-game smoke |
| Unsafe raw singleton/function/RakPeer helpers | [x] | [ ] | [ ] | [ ] | Per-build opaque-address fixture; no exposed references |
| Documentation compatibility claim | [x] R1-only direct bridge | [ ] | [ ] | [ ] | Update only after all rows used by the claim are complete |
