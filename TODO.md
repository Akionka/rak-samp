# Pending work

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

- [ ] `sampGetChatInfoPtr`, `sampGetChatDisplayMode`, `sampSetChatDisplayMode`,
  `sampGetChatString`, `sampSetChatString`, `sampIsChatVisible`
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
  `sampIsDialogActive`, `sampGetCurrentDialogType`, `sampGetCurrentDialogId`,
  `sampGetDialogCaption`, `sampGetDialogText`, `sampIsDialogClientside`,
  `sampSetDialogClientside`, `sampGetListboxItemsCount`, `sampGetListboxItemText`
- [ ] `sampGetMiscInfoPtr`, `sampToggleCursor`, `sampIsCursorActive`,
  `sampGetCursorMode`, `sampSetCursorMode`
- [ ] `sampGetInputInfoPtr`, `sampRegisterChatCommand`,
  `sampUnregisterChatCommand`, `sampSetChatInputText`, `sampGetChatInputText`,
  `sampSetChatInputEnabled`, `sampIsChatInputActive`,
  `sampIsChatCommandDefined`, `sampProcessChatInput`

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
  `sampGetAnimationNameAndFile`, `sampFindAnimationIdByNameAndFile`,
  `sampSetSendrate`
- [~] `sampGetCurrentServerName`, `sampGetCurrentServerAddress` —
  `HostApi::server_info` provides an owned cached R1 address, hostname, and
  port. Keep this status until its dedicated live R1 scenario records a match
  with the selected server and stable normal shutdown.
- [ ] `sampToggleScoreboard`, `sampIsScoreboardOpen`

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
