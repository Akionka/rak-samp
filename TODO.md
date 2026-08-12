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
| [ ] | `sampGetStreamedOutPlayerPos` | Safe owned/read | `Player::streamed_out_position` |
| [x] | `onSendRpc` | Safe owned/read | `Net::on_rpc(Direction::Outgoing, ...)` |
| [x] | `onSendPacket` | Safe owned/read | `Net::on_packet(Direction::Outgoing, ...)` |
| [x] | `onReceiveRpc` | Safe owned/read | `Net::on_rpc(Direction::Incoming, ...)` |
| [x] | `onReceivePacket` | Safe owned/read | `Net::on_packet(Direction::Incoming, ...)` |
