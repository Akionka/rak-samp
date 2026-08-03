# Pending work

## Completion checklist (preserve across context compaction)

The module inventories below are the authoritative SF.lua function scope. Work
them in this order; do not mark an item complete merely because a public header
or another client build supplies an offset.

- [ ] For every remaining `[ ]` function, classify it as one of: pure owned
  Rust helper, exact existing R1 wire codec, copied R1 game-thread read, queued
  R1 UI call, client-state mutation, force-sync, or raw-pointer API. Record
  the chosen safe replacement or the reason it remains excluded from the safe
  ABI beside its module entry.
- [ ] For every copied R1 read or queued R1 UI call, obtain all of: R1 PE/GTA
  fingerprint, exact target-code signature or independently verified field
  layout, bounded owned ABI model, unavailable/teardown result, unit/mock/E2E
  coverage, and an entry in `REVIEW.md`. Never access it from plugin threads,
  callbacks, bootstrap workers, or `DllMain`.
- [ ] For every existing R1 packet/RPC codec that corresponds to an SF.lua
  helper, add a named `HostApi` convenience only when its behaviour is
  accurately labelled protocol-only. Require exact byte/bit vectors and never
  claim native local-state mutation or force-sync.
- [ ] Keep raw client pointers, memory writes, client-state mutation,
  reconnect/disconnect, and force-sync outside the safe ABI unless a separate
  explicit unsafe/experimental design is approved. Do not silently turn them
  into safe helpers.
- [ ] Keep the independent C++ fixture limited to native-memory layouts. Add
  or update it for every new field access; a serialized wire vector is not a
  native-layout proof.
- [ ] Keep the ABI mock host and independent E2E plugin current for every
  appended safe wrapper. Check append offsets/size, copied-buffer validation,
  queue back-pressure, unsupported/not-ready paths, and game-pump draining.
- [ ] After each bounded implementation batch, run `cargo fmt --check`,
  `cargo test --workspace --target i686-pc-windows-msvc`,
  `cargo clippy --workspace --target i686-pc-windows-msvc -- -D warnings`,
  `cargo make test-e2e`,
  `cargo build --workspace --release --target i686-pc-windows-msvc`, and
  `git diff --check`; commit the passing batch separately.

### Pending live R1 evidence

- [ ] Deploy the current validation plugin to the fingerprinted GTA SA 1.0 US
  + SA-MP 0.3.7 R1 installation with the direct-client marker enabled. Confirm
  that the queued dialog, chat entry, and death-window entry are all visible,
  dismiss the dialog, and exit normally. Preserve only outcome/ID logs; never
  record UI, packet, or RPC payloads.
- [ ] During that run, prove the three direct UI calls cause no incoming or
  outgoing RPC 61/62 and no packet/RPC emission. Distinguish the validator's
  intentional incoming RPC 61 emulation from direct-helper activity.
- [ ] Verify `local_player` after server assignment, then while walking,
  taking armour and health damage, and entering/leaving a vehicle. Confirm the
  ID remains stable, all required fields change, and teardown clears cached
  data without a crash.
- [ ] Verify `server_info` against the selected server's displayed address,
  hostname, and port, then exit normally.
- [ ] With the direct-client marker enabled, cycle the R1 chat display mode
  through off, no-shadow, and normal (F7) while the validator is observing.
  Confirm its outcome line records all three cached modes, that no packet/RPC
  traffic was generated, and that normal shutdown remains stable.
- [ ] In the same run, open and close the scoreboard (Tab) and activate then
  dismiss any normal local cursor state. Confirm the validator records both
  cached scoreboard states and both cursor categories without packet/RPC
  emission or an unstable shutdown.
- [ ] Leave the validator's direct dialog open until it begins observing, then
  dismiss it; open and close chat input as well. Confirm both cached dialog
  and chat-input active/inactive state pairs appear in the outcome line with
  no packet/RPC emission or shutdown instability.
- [ ] Confirm the direct-client validator reads R1 animation entry zero as
  `AIRPORT:THRW_BARL_THRW` and resolves those byte strings back to zero. It is
  an automatic static-table check, but must still be recorded with the normal
  R1 lifecycle and shutdown evidence.
- [ ] For every newly added direct native surface, add an opt-in validator
  action before asking for its live run. Record the exact client identity,
  observed outcome, shutdown result, and any RPC/packet absence evidence in
  `REVIEW.md`; move only that helper from `[~]` to `[x]`.
- [ ] Run the same full lifecycle separately on legal R2, R3.1, R4.2, R5.1,
  and DL installations before enabling any direct helper for those builds.

### Release audit

- [ ] Reconcile every one of the 207 pinned SF.lua globals below with a safe
  implementation, a documented protocol-only approximation, or an explicit
  safe-ABI exclusion. Do not leave an unclassified function hidden in a
  comma-separated list.
- [ ] Re-read `README.md`, `CORE.md`, `ARCHITECTURE.md`, `VALIDATION.md`, and
  `REVIEW.md` against the final API. Remove provisional language only when its
  matching live evidence is recorded.
- [ ] Perform the final Windows x86 build/E2E suite and the required live R1
  scenarios on the exact release artifacts before declaring the backlog done.

- [x] Record the GTA SA 1.0 US + SA-MP 0.3.7 R1 direct-client live gate:
  dialog display, populated snapshot, walking/damage/armour/vehicle field
  changes, no direct-dialog RPC 61/62 traffic, and stable normal shutdown.
  Snapshot publication starts after R1 `INIT_GAME` assigns the local-player ID
  and reports `NotReady` beforehand.
- [ ] Validate the complete lifecycle on legal R2, R3.1, R4.2, R5.1, and DL
  installations. Keep direct client helpers unsupported until each profile has
  its own fingerprints, fixture, and live evidence.

## SF.lua-compatible R1 backlog

This is the implementation target, taken from [SF.lua]
at commit `d869b8fb2ac9b527209e05376c19f3c96ee318e5`: its `SFlua/*.lua`
modules define 207 distinct global functions. The list is organized by source
module so it stays auditable against that reference, rather than claiming a
larger SAMPFUNCS opcode catalog. A compatible result does not imply that raw
pointers, memory writes, force-sync, or Lua callback ownership will cross this
safe Rust ABI; those require an explicit unsafe/experimental design.

[SF.lua]: https://github.com/SF-lua/SF.lua/tree/d869b8fb2ac9b527209e05376c19f3c96ee318e5

- [~] `sampGetGamestate`: first implementation. Expose it as the nonblocking,
  cached scalar `HostApi::samp_game_state`, refreshed by the verified R1 game
  thread only.
- [ ] Implement each item below only with an R1 fingerprint, narrow fixture,
  safe ABI data model, automated test, and live-client evidence.

### Basic (`basic.lua`)

- [x] `sampGetVersion` — safe `HostApi::samp_version` reports the host's
  verified SA-MP build identity without a client-memory read.
- [x] `isSampAvailable` — safe `HostApi::is_samp_available` reports that the
  host attached and its RakClient hooks are ready, without dereferencing
  `CNetGame` on the plugin thread.
- [ ] `sampGetBase` — the raw module base remains outside the safe ABI.
- [x] `isSampLoaded` — `HostApi::is_samp_loaded` reports a recognized host
  attachment before RakClient hook readiness; use `is_samp_available` when
  ready hooks are required.

### Chat and death window (`chat.lua`, `deathwindow.lua`)

- [ ] `sampGetChatInfoPtr`, `sampSetChatDisplayMode`, `sampGetChatString`,
  `sampSetChatString`
- [~] `sampGetChatDisplayMode`, `sampIsChatVisible` —
  `HostApi::local_chat_display_mode` and its derived
  `HostApi::is_local_chat_visible` return a game-thread-cached R1 enum only.
  Keep provisional until the dedicated three-mode and shutdown live check.
- [~] `sampAddChatMessage`, `sampAddChatMessageEx` —
  `HostApi::show_local_chat_message` copies one bounded R1 chat/info/debug
  entry for the game-thread pump without sending any packet or RPC. Keep this
  provisional until the dedicated live R1 UI and shutdown scenario runs.
- [ ] `sampGetKillInfoPtr`
- [~] `sampAddDeathMessage` — `HostApi::show_local_death_message` copies one
  bounded R1 death-window entry for the game-thread pump without packet/RPC
  emulation. Keep this provisional until its dedicated live UI/shutdown check.

### Dialog, cursor, and input (`dialog.lua`, `game.lua`, `input.lua`)

- [x] `sampShowDialog` — represented by safe queued
  `HostApi::show_local_dialog`.
- [ ] `sampGetDialogInfoPtr`, `sampCloseCurrentDialogWithButton`,
  `sampGetCurrentDialogListItem`, `sampSetCurrentDialogListItem`,
  `sampGetCurrentDialogEditboxText`, `sampSetCurrentDialogEditboxText`,
  `sampGetCurrentDialogType`, `sampGetCurrentDialogId`,
  `sampGetDialogCaption`, `sampGetDialogText`, `sampIsDialogClientside`,
  `sampSetDialogClientside`, `sampGetListboxItemsCount`, `sampGetListboxItemText`
- [~] `sampIsDialogActive` — `HostApi::is_local_dialog_active` is a cached
  R1 game-thread read only. Keep provisional until direct-dialog active and
  dismissal states are observed in the live lifecycle test.
- [ ] `sampGetMiscInfoPtr`, `sampToggleCursor`, `sampSetCursorMode` — pointer
  access and UI mutations remain outside the safe ABI.
- [~] `sampIsCursorActive`, `sampGetCursorMode` —
  `HostApi::local_cursor_mode` and its derived
  `HostApi::is_local_cursor_active` copy a cached R1 cursor state only. Keep
  provisional until the dedicated cursor transition and shutdown live check.
- [ ] `sampGetInputInfoPtr`, `sampRegisterChatCommand`,
  `sampUnregisterChatCommand`, `sampSetChatInputText`, `sampGetChatInputText`,
  `sampSetChatInputEnabled`,
  `sampIsChatCommandDefined`, `sampProcessChatInput`
- [~] `sampIsChatInputActive` — `HostApi::is_local_chat_input_active` is a
  cached R1 game-thread read only. Keep provisional until normal chat-input
  open/close and shutdown live evidence is recorded.

### Pools, labels, objects, pickups, vehicles, textdraws (`gangzone.lua`, `label.lua`, `object.lua`, `pickup.lua`, `vehicle.lua`, `textdraw.lua`)

- [ ] `sampGetGangzonePoolPtr`
- [ ] `sampGetTextlabelPoolPtr`, `sampCreate3dText`, `sampIs3dTextDefined`,
  `sampGet3dTextInfoById`, `sampSet3dTextString`, `sampDestroy3dText`,
  `sampCreate3dTextEx`
- [ ] `sampGetObjectPoolPtr`, `sampGetObjectHandleBySampId`,
  `sampGetObjectSampIdByHandle`
- [ ] `sampGetPickupPoolPtr`, `sampGetPickupHandleBySampId`,
  `sampGetPickupSampIdByHandle`
- [ ] `sampGetVehiclePoolPtr`, `sampGetCarHandleBySampVehicleId`,
  `sampGetVehicleIdByCarHandle`, `sampIsVehicleDefined`
- [ ] `sampGetTextdrawPoolPtr`, `sampTextdrawIsExists`, `sampTextdrawCreate`,
  `sampTextdrawSetBoxColorAndSize`, `sampTextdrawGetString`,
  `sampTextdrawDelete`, `sampTextdrawGetLetterSizeAndColor`,
  `sampTextdrawGetPos`, `sampTextdrawGetShadowColor`,
  `sampTextdrawGetOutlineColor`, `sampTextdrawGetStyle`,
  `sampTextdrawGetProportional`, `sampTextdrawGetAlign`,
  `sampTextdrawGetBoxEnabledColorAndSize`,
  `sampTextdrawGetModelRotationZoomVehColor`,
  `sampTextdrawSetLetterSizeAndColor`, `sampTextdrawSetPos`,
  `sampTextdrawSetString`, `sampTextdrawSetModelRotationZoomVehColor`,
  `sampTextdrawSetOutlineColor`, `sampTextdrawSetShadow`,
  `sampTextdrawSetStyle`, `sampTextdrawSetProportional`, `sampTextdrawSetAlign`

### Net game and scoreboard (`netgame.lua`, `scoreboard.lua`)

- [ ] `sampGetSampInfoPtr`, `sampGetSampPoolsPtr`,
  `sampGetServerSettingsPtr`, `sampSetGamestate`,
  `sampSetSendrate`
- [~] `sampGetAnimationNameAndFile`, `sampFindAnimationIdByNameAndFile` —
  `HostApi::local_animation` and `HostApi::local_animation_id` read an owned
  cached copy of the fingerprinted fixed R1 table. Keep provisional until the
  automatic known-entry lookup and normal shutdown live check are recorded.
- [~] `sampGetCurrentServerName`, `sampGetCurrentServerAddress` —
  `HostApi::server_info` provides an owned cached R1 address, hostname, and
  port. Keep this status until its dedicated live R1 scenario records a match
  with the selected server and stable normal shutdown.
- [ ] `sampToggleScoreboard` — direct scoreboard mutation remains outside the
  safe ABI pending separate native-call evidence and an explicit policy.
- [~] `sampIsScoreboardOpen` — `HostApi::is_local_scoreboard_open` returns a
  cached R1 game-thread read only. Keep provisional until the open/close and
  shutdown live check.

### Players (`player.lua`)

- [ ] `sampGetPlayerPoolPtr`, `sampIsPlayerConnected`, `sampGetPlayerNickname`,
  `sampSpawnPlayer`, `sampIsPlayerNpc`,
  `sampForceUnoccupiedSyncSeatId`, `sampGetCharHandleBySampPlayerId`,
  `sampGetPlayerIdByCharHandle`, `sampGetPlayerArmor`, `sampGetPlayerHealth`,
  `sampIsPlayerPaused`, `sampSetSpecialAction`, `sampGetPlayerCount`,
  `sampGetMaxPlayerId`, `sampGetPlayerSpecialAction`,
  `sampStorePlayerOnfootData`, `sampStorePlayerIncarData`,
  `sampStorePlayerPassengerData`, `sampStorePlayerTrailerData`,
  `sampStorePlayerAimData`, `sampGetPlayerAnimationId`,
  `sampSetLocalPlayerName`, `sampGetPlayerStructPtr`,
  `sampIsLocalPlayerSpawned`, `sampGetPlayerColor`,
  `sampForceAimSync`, `sampForceOnfootSync`, `sampForceStatsSync`,
  `sampForceTrailerSync`, `sampForceVehicleSync`
- [x] `sampGetLocalPlayerId`, `sampGetLocalPlayerNickname`,
  `sampGetLocalPlayerColor`, `sampIsLocalPlayerSpawned`,
  `sampGetPlayerArmor`, `sampGetPlayerHealth`, `sampGetPlayerSpecialAction`,
  `sampGetPlayerAnimationId` — explicit safe local-player query methods reuse
  the single cached `HostApi::local_player` snapshot; remote-player calls
  remain pending.
- [~] `sampGetPlayerScore`, `sampGetPlayerPing` — `HostApi::local_player_score`
  and `HostApi::local_player_ping` cover the local player; player-ID based
  remote queries remain pending.
- [~] `sampRequestClass`, `sampSendInteriorChange`, `sampSendSpawn`,
  `sampSendEnterVehicle`, `sampSendExitVehicle` — the corresponding
  `HostApi::send_*` methods serialize the exact R1 outbound RPCs, but remain
  protocol-only and do not invoke SF.lua's native local-player state changes.
- [x] `sampSendChat` — `HostApi::send_chat` serializes the typed, bounded
  server-bound RPC 101 payload, or RPC 50 for slash-prefixed commands, through
  the original RakClient send path.
- [ ] `sampIsPlayerDefined`, `sampSetPlayerColor`

### RakNet and network actions (`raknet.lua`)

- [x] `raknetGetRpcName`, `raknetGetPacketName` — pure catalog lookups are
  available as `rak_samp_plugin_api::raknet::{rpc_name, packet_name}`.
- [x] `raknetNewBitStream`, `raknetDeleteBitStream`, `raknetResetBitStream`,
  `raknetBitStreamReadBool`, `raknetBitStreamReadBuffer`,
  `raknetBitStreamReadInt8`, `raknetBitStreamReadInt16`,
  `raknetBitStreamReadInt32`, `raknetBitStreamReadFloat`,
  `raknetBitStreamReadString`, `raknetBitStreamResetReadPointer`,
  `raknetBitStreamResetWritePointer`, `raknetBitStreamIgnoreBits`,
  `raknetBitStreamSetWriteOffset`, `raknetBitStreamSetReadOffset`,
  `raknetBitStreamGetNumberOfBitsUsed`, `raknetBitStreamGetNumberOfBytesUsed`,
  `raknetBitStreamGetNumberOfUnreadBits`, `raknetBitStreamGetWriteOffset`,
  `raknetBitStreamGetReadOffset`,
  `raknetBitStreamWriteBool`, `raknetBitStreamWriteInt8`,
  `raknetBitStreamWriteInt16`, `raknetBitStreamWriteInt32`,
  `raknetBitStreamWriteFloat`, `raknetBitStreamWriteBuffer`,
  `raknetBitStreamWriteString`, `raknetBitStreamWriteBitStream` — safe owned
  equivalents are in `rak_samp_plugin_api::raknet::BitStream`; raw data pointers
  and invalid/uninitialized cursor states are intentionally unavailable.
- [x] `raknetBitStreamEncodeString` — use `HostApi::encode_string` then
  `BitStream::write_encoded_string`.
- [~] `raknetBitStreamDecodeString` — `HostApi::decode_string` decodes through
  copied, owned `BitStream` storage and advances its cursor only on success;
  retain this status until the dedicated direct-ABI R1 live scenario runs.
- [ ] `raknetBitStreamGetDataPtr` — raw client/plugin memory pointers remain
  outside the safe ABI.
- [x] `raknetSendRpcEx`, `raknetSendRpc` — safe stream convenience methods are
  `HostApi::send_rpc_stream` and `HostApi::send_rpc`; timestamped sends remain
  rejected by the host policy.
- [x] `raknetSendBitStreamEx`, `raknetSendBitStream` — represented by
  `HostApi::send_packet_stream` and `HostApi::send_packet` with the packet ID
  explicit rather than embedded in an unchecked native bitstream.
- [ ] `sampGetRakclientInterface`, `sampGetRakpeer`,
  `sampDisconnectWithReason`, `sampConnectToServer`
- [x] `sampSendRequestSpawn` — `HostApi::send_request_spawn` sends the exact
  empty, server-bound RPC 129 without invoking native local-player methods.
- [x] `sampSendDialogResponse`, `sampSendClickPlayer`,
  `sampSendClickTextdraw`, `sampSendDeathByPlayer`, `sampSendMenuQuit`,
  `sampSendMenuSelectRow`, `sampSendPickedUpPickup`,
  `sampSendVehicleDestroyed` — matching `HostApi::send_*` helpers serialize
  the typed outgoing RPCs and send them through the original RakClient path.
- [x] `sampSendDamageVehicle`, `sampSendGiveDamage`, `sampSendTakeDamage`,
  `sampSendEditAttachedObject`, `sampSendEditObject`, `sampSendRconCommand` —
  matching `HostApi::send_*` helpers serialize the exact bounded outgoing RPC
  or packet. Attached-object edits require the complete typed payload,
  including both colours omitted by SF.lua's partial helper signature.
- [x] `sampSendScmEvent` — `HostApi::send_scm_event` maps the complete typed
  R1 RPC 96 payload into its required wire order.
- [x] `sampSendAimData`, `sampSendBulletData`, `sampSendIncarData`,
  `sampSendOnfootData`, `sampSendSpectatorData`, `sampSendTrailerData`,
  `sampSendPassengerData`, `sampSendUnoccupiedData` — matching complete typed
  `HostApi::send_*_sync` helpers serialize the fixed-layout packet and send it
  through the original RakClient path. They do not force or mutate local sync
  state.

### SF.lua’s explicit future items (`init.lua`)

- [ ] `sampHasDialogRespond`, `sampForcePassengerSyncSeatId`,
  `sampForceWeaponsSync`, `sampGetRakclientFuncAddressByIndex`,
  `sampGetRpcCallbackByRpcId`, `sampGetRpcNodeByRpcId`,
  `raknetEmulRpcReceiveBitStream`, `raknetEmulPacketReceiveBitStream`,
  `sampSetClientCommandDescription`, `sampGetStreamedOutPlayerPos`,
  `onSendRpc`, `onSendPacket`, `onReceiveRpc`, `onReceivePacket`
