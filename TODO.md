# Repositioning TODO

[PLAN.md](PLAN.md) is the decision-complete implementation plan. The approved
[repositioning proposal](docs/repositioning-proposal.md) records the motivation
and product decisions; when wording differs, follow `PLAN.md`.

The target release is the breaking `0.1.0-alpha.4` cutover to
`samp-client-sdk`. Check an SF.lua mapping only when it is available through the
new facade (or the explicit `unsafe raw` tier), covered by tests, and documented.
An existing `HostApi` helper is reusable implementation material, not completion
of the renamed public API.

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

- [ ] Delete `tests/e2e/`, `examples/validation_plugin/`,
  `examples/validation_unloader/`, `REVIEW.md`, and `VALIDATION.md`.
- [ ] Remove deleted workspace members, cargo-make tasks, CI jobs, release
  inputs, validation references, and obsolete ABI self-tests.
- [ ] Keep the C++ RakNet layout fixture, its Rust tests, and `build.rs` wiring.
- [ ] Move the public crate to `sdk/` as package `samp-client-sdk`; rename the
  host package to `samp-client-sdk-host` and deploy `samp_client_sdk.asi`.
- [ ] Rename crate imports, examples, logs, release archives, repository/docs.rs
  metadata, host discovery, ABI types, and the export to
  `SampClientSdk_GetApiV1`.
- [ ] Replace R1 PE/signature/fingerprint feature gates with fixed offsets plus
  ordinary pointer, range, capacity, and enum validation. Keep build detection
  only for selecting the recognized networking offset table.
- [ ] Rewrite `README.md`, `CORE.md`, `ARCHITECTURE.md`, and `AGENTS.md` for the
  two-pillar SDK, R1 bridge compatibility, layout-fixture rule, and removal of
  the live-validation lifecycle.
- [ ] Keep surviving behavior unchanged for SA-MP 0.3.7 R1 on GTA SA 1.0 US,
  apart from the intentional package/symbol compatibility break.

### Phase 2 — game-thread foundation

- [ ] Hook `CGame::Process` at `0x53E4B0`, retain its trampoline, call the
  original exactly once per entry, and restore it during shutdown.
- [ ] Remove cache and UI pumping from the incoming-packet detour.
- [ ] Add the bounded 256-entry owned `GameCommand` queue and drain one accepted
  snapshot after the original game process call on each tick.
- [ ] Migrate dialog, chat, and death-window queues into `GameCommand`.
- [ ] Queue every plugin-thread native mutation and explicit RakClient
  send/emulation; keep callback-local packet/RPC replacement synchronous.
- [ ] Add host-owned command IDs, fixed `repr(C)` result storage, poll,
  timed-wait, release, detach-on-drop, timeout retry, and shutdown completion.
- [ ] Reject waits from the game thread and listener callbacks.
- [ ] Publish one coherent cache generation per tick: refresh lightweight
  global/pool directories eagerly and heavy requested/active records through
  bounded, deduplicated refresh queues.
- [ ] Clear stale generations and pending heavy records across connection,
  version, and shutdown transitions.

### Phase 3 — facade and feature completion

- [ ] Introduce `Samp::connect`, `Samp::connect_to`, subsystem facades, checked
  SA-MP ID newtypes, typed GTA handles, `CommandReceipt<T>`, and typed errors.
- [ ] Make the raw ABI wrapper private or documentation-hidden and expose native
  addresses only through the explicit `unsafe raw` module.
- [ ] Move subscriptions, typed events, exact sends/emulation, string codecs,
  protocol catalogs, and owned `BitStream` behind `samp.net()`.
- [ ] Migrate all existing cached reads to their facade targets without
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
| [ ] | `sampGetBase` | Unsafe raw | `raw::base` |
| [ ] | `sampGetVersion` | Safe owned/read | `Samp::version` |
| [ ] | `isSampLoaded` | Safe owned/read | `Samp::probe` host-loaded status |
| [ ] | `isSampfuncsLuaLoaded` | Safe owned/read | `Samp::probe` recognized-build status |
| [ ] | `isSampAvailable` | Safe owned/read | `Samp::probe` ready status |

### Chat and death window (`chat.lua`, `deathwindow.lua`) — 10

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetChatInfoPtr` | Unsafe raw | `raw::chat` |
| [ ] | `sampAddChatMessage` | Queued mutation | `Chat::add` |
| [ ] | `sampGetChatDisplayMode` | Safe owned/read | `Chat::display_mode` |
| [ ] | `sampSetChatDisplayMode` | Queued mutation | `Chat::set_display_mode` |
| [ ] | `sampGetChatString` | Safe owned/read | `Chat::entry` |
| [ ] | `sampSetChatString` | Queued mutation | `Chat::set_entry` |
| [ ] | `sampIsChatVisible` | Safe owned/read | `Chat::is_visible` |
| [ ] | `sampAddChatMessageEx` | Queued mutation | `Chat::add_with_style` |
| [ ] | `sampGetKillInfoPtr` | Unsafe raw | `raw::death_window` |
| [ ] | `sampAddDeathMessage` | Queued mutation | `Chat::death_window().add` |

### Dialog (`dialog.lua`) — 16

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetDialogInfoPtr` | Unsafe raw | `raw::dialog` |
| [ ] | `sampShowDialog` | Queued mutation | `Dialogs::show` |
| [ ] | `sampCloseCurrentDialogWithButton` | Queued mutation | `Dialogs::close_with_button` |
| [ ] | `sampGetCurrentDialogListItem` | Safe owned/read | `Dialogs::active().selected_item` |
| [ ] | `sampSetCurrentDialogListItem` | Queued mutation | `Dialogs::set_selected_item` |
| [ ] | `sampGetCurrentDialogEditboxText` | Safe owned/read | `Dialogs::active().editbox_text` |
| [ ] | `sampSetCurrentDialogEditboxText` | Queued mutation | `Dialogs::set_editbox_text` |
| [ ] | `sampIsDialogActive` | Safe owned/read | `Dialogs::is_active` |
| [ ] | `sampGetCurrentDialogType` | Safe owned/read | `Dialogs::active().style` |
| [ ] | `sampGetCurrentDialogId` | Safe owned/read | `Dialogs::active().id` |
| [ ] | `sampGetDialogCaption` | Safe owned/read | `Dialogs::active().caption` |
| [ ] | `sampGetDialogText` | Safe owned/read | `Dialogs::active().text` |
| [ ] | `sampIsDialogClientside` | Safe owned/read | `Dialogs::active().is_client_side` |
| [ ] | `sampSetDialogClientside` | Queued mutation | `Dialogs::set_client_side` |
| [ ] | `sampGetListboxItemsCount` | Safe owned/read | `Dialogs::active().items().len` |
| [ ] | `sampGetListboxItemText` | Safe owned/read | `Dialogs::active().items().get` |

### Cursor and game (`game.lua`) — 5

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetMiscInfoPtr` | Unsafe raw | `raw::misc` |
| [ ] | `sampToggleCursor` | Queued mutation | `Cursor::toggle` |
| [ ] | `sampIsCursorActive` | Safe owned/read | `Cursor::is_active` |
| [ ] | `sampGetCursorMode` | Safe owned/read | `Cursor::mode` |
| [ ] | `sampSetCursorMode` | Queued mutation | `Cursor::set_mode` |

### Gangzones (`gangzone.lua`) — 1

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetGangzonePoolPtr` | Unsafe raw | `raw::gangzone_pool` |

### Chat input (`input.lua`) — 9

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetInputInfoPtr` | Unsafe raw | `raw::chat_input` |
| [ ] | `sampRegisterChatCommand` | Queued mutation | `ChatInput::register_command` returning a subscription |
| [ ] | `sampUnregisterChatCommand` | Queued mutation | `ChatCommandSubscription::unregister_and_wait` |
| [ ] | `sampSetChatInputText` | Queued mutation | `ChatInput::set_text` |
| [ ] | `sampGetChatInputText` | Safe owned/read | `ChatInput::text` |
| [ ] | `sampSetChatInputEnabled` | Queued mutation | `ChatInput::set_enabled` |
| [ ] | `sampIsChatInputActive` | Safe owned/read | `ChatInput::is_active` |
| [ ] | `sampIsChatCommandDefined` | Safe owned/read | `ChatInput::is_command_defined` |
| [ ] | `sampProcessChatInput` | Queued mutation | `ChatInput::process` |

### 3D labels (`label.lua`) — 7

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetTextlabelPoolPtr` | Unsafe raw | `raw::text_label_pool` |
| [ ] | `sampCreate3dText` | Queued mutation | `Labels::create` |
| [ ] | `sampIs3dTextDefined` | Safe owned/read | `Labels::exists` |
| [ ] | `sampGet3dTextInfoById` | Safe owned/read | `Labels::get` |
| [ ] | `sampSet3dTextString` | Queued mutation | `Label::set_text` |
| [ ] | `sampDestroy3dText` | Queued mutation | `Label::destroy` |
| [ ] | `sampCreate3dTextEx` | Queued mutation | `Labels::create_at` |

### Net game and animation (`netgame.lua`) — 10

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetSampInfoPtr` | Unsafe raw | `raw::net_game` |
| [ ] | `sampGetSampPoolsPtr` | Unsafe raw | `raw::pools` |
| [ ] | `sampGetServerSettingsPtr` | Unsafe raw | `raw::server_settings` |
| [ ] | `sampGetCurrentServerName` | Safe owned/read | `Server::hostname` |
| [ ] | `sampGetCurrentServerAddress` | Safe owned/read | `Server::address` and `Server::port` |
| [ ] | `sampGetGamestate` | Safe owned/read | `Samp::game_state` |
| [ ] | `sampSetGamestate` | Queued mutation | `Samp::set_game_state` |
| [ ] | `sampGetAnimationNameAndFile` | Safe owned/read | `Animations::get` |
| [ ] | `sampFindAnimationIdByNameAndFile` | Safe owned/read | `Animations::find` |
| [ ] | `sampSetSendrate` | Queued mutation | `Net::set_send_rate` |

### Objects (`object.lua`) — 3

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetObjectPoolPtr` | Unsafe raw | `raw::object_pool` |
| [ ] | `sampGetObjectHandleBySampId` | Safe owned/read | `Object::handle` returning `ObjectHandle` |
| [ ] | `sampGetObjectSampIdByHandle` | Safe owned/read | `ObjectHandle::to_id` |

### Pickups (`pickup.lua`) — 3

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetPickupPoolPtr` | Unsafe raw | `raw::pickup_pool` |
| [ ] | `sampGetPickupHandleBySampId` | Safe owned/read | `Pickup::handle` returning `PickupHandle` |
| [ ] | `sampGetPickupSampIdByHandle` | Safe owned/read | `PickupHandle::to_id` |

### Players (`player.lua`) — 43

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetPlayerPoolPtr` | Unsafe raw | `raw::player_pool` |
| [ ] | `sampIsPlayerConnected` | Safe owned/read | `Player::is_connected` |
| [ ] | `sampGetPlayerNickname` | Safe owned/read | `Player::nickname` |
| [ ] | `sampSpawnPlayer` | Queued mutation | `LocalPlayer::spawn` |
| [ ] | `sampSendChat` | Queued mutation | `Net::send_chat` |
| [ ] | `sampIsPlayerNpc` | Safe owned/read | `Player::is_npc` |
| [ ] | `sampGetPlayerScore` | Safe owned/read | `Player::score` |
| [ ] | `sampGetPlayerPing` | Safe owned/read | `Player::ping` |
| [ ] | `sampRequestClass` | Queued mutation | `LocalPlayer::request_class` |
| [ ] | `sampSendInteriorChange` | Queued mutation | `LocalPlayer::send_interior_change` |
| [ ] | `sampForceUnoccupiedSyncSeatId` | Queued mutation | `LocalPlayer::force_unoccupied_sync` |
| [ ] | `sampGetCharHandleBySampPlayerId` | Safe owned/read | `Player::ped_handle` returning `PedHandle` |
| [ ] | `sampGetPlayerIdByCharHandle` | Safe owned/read | `PedHandle::to_id` |
| [ ] | `sampGetPlayerArmor` | Safe owned/read | `Player::armour` |
| [ ] | `sampGetPlayerHealth` | Safe owned/read | `Player::health` |
| [ ] | `sampIsPlayerPaused` | Safe owned/read | `Player::is_paused` |
| [ ] | `sampSetSpecialAction` | Queued mutation | `LocalPlayer::set_special_action` |
| [ ] | `sampGetPlayerCount` | Safe owned/read | `Players::count` |
| [ ] | `sampGetMaxPlayerId` | Safe owned/read | `Players::max_id` |
| [ ] | `sampGetPlayerSpecialAction` | Safe owned/read | `Player::special_action` |
| [ ] | `sampStorePlayerOnfootData` | Safe owned/read | `Player::onfoot_sync` owned snapshot |
| [ ] | `sampStorePlayerIncarData` | Safe owned/read | `Player::vehicle_sync` owned snapshot |
| [ ] | `sampStorePlayerPassengerData` | Safe owned/read | `Player::passenger_sync` owned snapshot |
| [ ] | `sampStorePlayerTrailerData` | Safe owned/read | `Player::trailer_sync` owned snapshot |
| [ ] | `sampStorePlayerAimData` | Safe owned/read | `Player::aim_sync` owned snapshot |
| [ ] | `sampSendSpawn` | Queued mutation | `LocalPlayer::send_spawn` |
| [ ] | `sampGetPlayerAnimationId` | Safe owned/read | `Player::animation_id` |
| [ ] | `sampSetLocalPlayerName` | Queued mutation | `LocalPlayer::set_nickname` |
| [ ] | `sampGetPlayerStructPtr` | Unsafe raw | `raw::player` |
| [ ] | `sampSendEnterVehicle` | Queued mutation | `LocalPlayer::send_enter_vehicle` |
| [ ] | `sampSendExitVehicle` | Queued mutation | `LocalPlayer::send_exit_vehicle` |
| [ ] | `sampIsLocalPlayerSpawned` | Safe owned/read | `LocalPlayer::is_spawned` |
| [ ] | `sampGetPlayerColor` | Safe owned/read | `Player::colour` |
| [ ] | `sampForceAimSync` | Queued mutation | `LocalPlayer::force_aim_sync` |
| [ ] | `sampForceOnfootSync` | Queued mutation | `LocalPlayer::force_onfoot_sync` |
| [ ] | `sampForceStatsSync` | Queued mutation | `LocalPlayer::force_stats_sync` |
| [ ] | `sampForceTrailerSync` | Queued mutation | `LocalPlayer::force_trailer_sync` |
| [ ] | `sampForceVehicleSync` | Queued mutation | `LocalPlayer::force_vehicle_sync` |
| [ ] | `sampGetLocalPlayerId` | Safe owned/read | `LocalPlayer::id` |
| [ ] | `sampIsPlayerDefined` | Safe owned/read | `Player::is_defined` |
| [ ] | `sampGetLocalPlayerNickname` | Safe owned/read | `LocalPlayer::nickname` |
| [ ] | `sampGetLocalPlayerColor` | Safe owned/read | `LocalPlayer::colour` |
| [ ] | `sampSetPlayerColor` | Queued mutation | `Player::set_colour` |

### RakNet and network actions (`raknet.lua`) — 65

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `raknetBitStreamReadBool` | Safe owned/read | `BitStream::read_bool` |
| [ ] | `raknetBitStreamReadBuffer` | Safe owned/read | `BitStream::read_bits` / `read_bytes` |
| [ ] | `raknetBitStreamReadInt8` | Safe owned/read | `BitStream::read_u8` |
| [ ] | `raknetBitStreamReadInt16` | Safe owned/read | `BitStream::read_u16` |
| [ ] | `raknetBitStreamReadInt32` | Safe owned/read | `BitStream::read_u32` |
| [ ] | `raknetBitStreamReadFloat` | Safe owned/read | `BitStream::read_f32` |
| [ ] | `raknetBitStreamReadString` | Safe owned/read | `BitStream::read_string` |
| [ ] | `raknetBitStreamResetReadPointer` | Safe owned/read | `BitStream::reset_read` |
| [ ] | `raknetBitStreamResetWritePointer` | Safe owned/read | `BitStream::reset_write` |
| [ ] | `raknetBitStreamIgnoreBits` | Safe owned/read | `BitStream::ignore_bits` |
| [ ] | `raknetBitStreamSetWriteOffset` | Safe owned/read | `BitStream::set_write_offset` |
| [ ] | `raknetBitStreamSetReadOffset` | Safe owned/read | `BitStream::set_read_offset` |
| [ ] | `raknetBitStreamGetNumberOfBitsUsed` | Safe owned/read | `BitStream::len_bits` |
| [ ] | `raknetBitStreamGetNumberOfBytesUsed` | Safe owned/read | `BitStream::len_bytes` |
| [ ] | `raknetBitStreamGetNumberOfUnreadBits` | Safe owned/read | `BitStream::remaining_bits` |
| [ ] | `raknetBitStreamGetWriteOffset` | Safe owned/read | `BitStream::write_offset` |
| [ ] | `raknetBitStreamGetReadOffset` | Safe owned/read | `BitStream::read_offset` |
| [ ] | `raknetBitStreamGetDataPtr` | Unsafe raw | `raw::bitstream_data` |
| [ ] | `raknetNewBitStream` | Safe owned/read | `BitStream::new` |
| [ ] | `raknetDeleteBitStream` | Safe owned/read | `BitStream` ownership and `Drop` |
| [ ] | `raknetResetBitStream` | Safe owned/read | `BitStream::clear` |
| [ ] | `raknetBitStreamWriteBool` | Safe owned/read | `BitStream::write_bool` |
| [ ] | `raknetBitStreamWriteInt8` | Safe owned/read | `BitStream::write_u8` |
| [ ] | `raknetBitStreamWriteInt16` | Safe owned/read | `BitStream::write_u16` |
| [ ] | `raknetBitStreamWriteInt32` | Safe owned/read | `BitStream::write_u32` |
| [ ] | `raknetBitStreamWriteFloat` | Safe owned/read | `BitStream::write_f32` |
| [ ] | `raknetBitStreamWriteBuffer` | Safe owned/read | `BitStream::write_bits` / `write_bytes` |
| [ ] | `raknetBitStreamWriteString` | Safe owned/read | `BitStream::write_string` |
| [ ] | `raknetBitStreamDecodeString` | Safe owned/read | `Net::decode_string` |
| [ ] | `raknetBitStreamEncodeString` | Safe owned/read | `Net::encode_string` |
| [ ] | `raknetBitStreamWriteBitStream` | Safe owned/read | `BitStream::write_stream` |
| [ ] | `raknetSendRpcEx` | Queued mutation | `Net::send_rpc_with_options` |
| [ ] | `raknetSendBitStreamEx` | Queued mutation | `Net::send_packet_with_options` |
| [ ] | `raknetSendRpc` | Queued mutation | `Net::send_rpc` |
| [ ] | `raknetSendBitStream` | Queued mutation | `Net::send_packet` |
| [ ] | `raknetGetRpcName` | Safe owned/read | `Net::rpc_name` |
| [ ] | `raknetGetPacketName` | Safe owned/read | `Net::packet_name` |
| [ ] | `sampGetRakclientInterface` | Unsafe raw | `raw::rakclient` |
| [ ] | `sampGetRakpeer` | Unsafe raw | `raw::rakpeer` |
| [ ] | `sampSendAimData` | Queued mutation | `Net::send_aim_sync` |
| [ ] | `sampSendBulletData` | Queued mutation | `Net::send_bullet_sync` |
| [ ] | `sampSendIncarData` | Queued mutation | `Net::send_vehicle_sync` |
| [ ] | `sampSendOnfootData` | Queued mutation | `Net::send_player_sync` |
| [ ] | `sampSendSpectatorData` | Queued mutation | `Net::send_spectator_sync` |
| [ ] | `sampSendTrailerData` | Queued mutation | `Net::send_trailer_sync` |
| [ ] | `sampSendPassengerData` | Queued mutation | `Net::send_passenger_sync` |
| [ ] | `sampSendUnoccupiedData` | Queued mutation | `Net::send_unoccupied_sync` |
| [ ] | `sampSendDamageVehicle` | Queued mutation | `Net::send_vehicle_damage` |
| [ ] | `sampSendScmEvent` | Queued mutation | `Net::send_scm_event` |
| [ ] | `sampSendGiveDamage` | Queued mutation | `Net::send_give_damage` |
| [ ] | `sampSendTakeDamage` | Queued mutation | `Net::send_take_damage` |
| [ ] | `sampSendRequestSpawn` | Queued mutation | `Net::send_request_spawn` |
| [ ] | `sampSendClickPlayer` | Queued mutation | `Net::send_click_player` |
| [ ] | `sampSendClickTextdraw` | Queued mutation | `Net::send_click_textdraw` |
| [ ] | `sampSendDeathByPlayer` | Queued mutation | `Net::send_death_by_player` |
| [ ] | `sampSendDialogResponse` | Queued mutation | `Net::send_dialog_response` |
| [ ] | `sampSendEditAttachedObject` | Queued mutation | `Net::send_edit_attached_object` |
| [ ] | `sampSendEditObject` | Queued mutation | `Net::send_edit_object` |
| [ ] | `sampSendMenuQuit` | Queued mutation | `Net::send_menu_quit` |
| [ ] | `sampSendMenuSelectRow` | Queued mutation | `Net::send_menu_select_row` |
| [ ] | `sampSendPickedUpPickup` | Queued mutation | `Net::send_picked_up_pickup` |
| [ ] | `sampSendRconCommand` | Queued mutation | `Net::send_rcon_command` |
| [ ] | `sampSendVehicleDestroyed` | Queued mutation | `Net::send_vehicle_destroyed` |
| [ ] | `sampDisconnectWithReason` | Queued mutation | `Net::disconnect` |
| [ ] | `sampConnectToServer` | Queued mutation | `Net::connect` |

### Scoreboard (`scoreboard.lua`) — 2

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampToggleScoreboard` | Queued mutation | `Scoreboard::toggle` |
| [ ] | `sampIsScoreboardOpen` | Safe owned/read | `Scoreboard::is_open` |

### Textdraws (`textdraw.lua`) — 24

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetTextdrawPoolPtr` | Unsafe raw | `raw::textdraw_pool` |
| [ ] | `sampTextdrawIsExists` | Safe owned/read | `Textdraws::exists` |
| [ ] | `sampTextdrawCreate` | Queued mutation | `Textdraws::create` |
| [ ] | `sampTextdrawSetBoxColorAndSize` | Queued mutation | `Textdraw::set_box` |
| [ ] | `sampTextdrawGetString` | Safe owned/read | `Textdraw::text` |
| [ ] | `sampTextdrawDelete` | Queued mutation | `Textdraw::delete` |
| [ ] | `sampTextdrawGetLetterSizeAndColor` | Safe owned/read | `Textdraw::letter_style` |
| [ ] | `sampTextdrawGetPos` | Safe owned/read | `Textdraw::position` |
| [ ] | `sampTextdrawGetShadowColor` | Safe owned/read | `Textdraw::shadow` |
| [ ] | `sampTextdrawGetOutlineColor` | Safe owned/read | `Textdraw::outline` |
| [ ] | `sampTextdrawGetStyle` | Safe owned/read | `Textdraw::style` |
| [ ] | `sampTextdrawGetProportional` | Safe owned/read | `Textdraw::is_proportional` |
| [ ] | `sampTextdrawGetAlign` | Safe owned/read | `Textdraw::alignment` |
| [ ] | `sampTextdrawGetBoxEnabledColorAndSize` | Safe owned/read | `Textdraw::box_style` |
| [ ] | `sampTextdrawGetModelRotationZoomVehColor` | Safe owned/read | `Textdraw::model_style` |
| [ ] | `sampTextdrawSetLetterSizeAndColor` | Queued mutation | `Textdraw::set_letter_style` |
| [ ] | `sampTextdrawSetPos` | Queued mutation | `Textdraw::set_position` |
| [ ] | `sampTextdrawSetString` | Queued mutation | `Textdraw::set_text` |
| [ ] | `sampTextdrawSetModelRotationZoomVehColor` | Queued mutation | `Textdraw::set_model_style` |
| [ ] | `sampTextdrawSetOutlineColor` | Queued mutation | `Textdraw::set_outline` |
| [ ] | `sampTextdrawSetShadow` | Queued mutation | `Textdraw::set_shadow` |
| [ ] | `sampTextdrawSetStyle` | Queued mutation | `Textdraw::set_style` |
| [ ] | `sampTextdrawSetProportional` | Queued mutation | `Textdraw::set_proportional` |
| [ ] | `sampTextdrawSetAlign` | Queued mutation | `Textdraw::set_alignment` |

### Vehicles (`vehicle.lua`) — 4

| Done | SF.lua global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampGetVehiclePoolPtr` | Unsafe raw | `raw::vehicle_pool` |
| [ ] | `sampGetCarHandleBySampVehicleId` | Safe owned/read | `Vehicle::handle` returning `VehicleHandle` |
| [ ] | `sampGetVehicleIdByCarHandle` | Safe owned/read | `VehicleHandle::to_id` |
| [ ] | `sampIsVehicleDefined` | Safe owned/read | `Vehicles::exists` |
<!-- sf-lua-baseline:end -->

## SF.lua `init.lua` extension map — 14

These names are comments in the pinned source rather than part of the 207
declared-function baseline. They are still in scope under the rule that nothing
remains permanently excluded.

| Done | Future global | Tier | `samp-client-sdk` target |
| --- | --- | --- | --- |
| [ ] | `sampHasDialogRespond` | Safe owned/read | `Dialogs::last_response` |
| [ ] | `sampForcePassengerSyncSeatId` | Queued mutation | `LocalPlayer::force_passenger_sync` |
| [ ] | `sampForceWeaponsSync` | Queued mutation | `LocalPlayer::force_weapons_sync` |
| [ ] | `sampGetRakclientFuncAddressByIndex` | Unsafe raw | `raw::rakclient_function` |
| [ ] | `sampGetRpcCallbackByRpcId` | Unsafe raw | `raw::rpc_callback` |
| [ ] | `sampGetRpcNodeByRpcId` | Unsafe raw | `raw::rpc_node` |
| [ ] | `raknetEmulRpcReceiveBitStream` | Queued mutation | `Net::emulate_incoming_rpc` |
| [ ] | `raknetEmulPacketReceiveBitStream` | Queued mutation | `Net::emulate_incoming_packet` |
| [ ] | `sampSetClientCommandDescription` | Queued mutation | `ChatInput::set_command_description` |
| [ ] | `sampGetStreamedOutPlayerPos` | Safe owned/read | `Player::streamed_out_position` |
| [ ] | `onSendRpc` | Safe owned/read | `Net::on_rpc(Direction::Outgoing, ...)` |
| [ ] | `onSendPacket` | Safe owned/read | `Net::on_packet(Direction::Outgoing, ...)` |
| [ ] | `onReceiveRpc` | Safe owned/read | `Net::on_rpc(Direction::Incoming, ...)` |
| [ ] | `onReceivePacket` | Safe owned/read | `Net::on_packet(Direction::Incoming, ...)` |
