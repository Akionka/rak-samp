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

- [ ] `sampGetBase`, `sampGetVersion`, `isSampLoaded`,
  `isSampfuncsLuaLoaded`, `isSampAvailable`

### Chat and death window (`chat.lua`, `deathwindow.lua`)

- [ ] `sampGetChatInfoPtr`, `sampAddChatMessage`, `sampGetChatDisplayMode`,
  `sampSetChatDisplayMode`, `sampGetChatString`, `sampSetChatString`,
  `sampIsChatVisible`, `sampAddChatMessageEx`
- [ ] `sampGetKillInfoPtr`, `sampAddDeathMessage`

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
  `sampGetServerSettingsPtr`, `sampGetCurrentServerName`,
  `sampGetCurrentServerAddress`, `sampSetGamestate`,
  `sampGetAnimationNameAndFile`, `sampFindAnimationIdByNameAndFile`,
  `sampSetSendrate`
- [ ] `sampToggleScoreboard`, `sampIsScoreboardOpen`

### Players (`player.lua`)

- [ ] `sampGetPlayerPoolPtr`, `sampIsPlayerConnected`, `sampGetPlayerNickname`,
  `sampSpawnPlayer`, `sampSendChat`, `sampIsPlayerNpc`, `sampGetPlayerScore`,
  `sampGetPlayerPing`, `sampRequestClass`, `sampSendInteriorChange`,
  `sampForceUnoccupiedSyncSeatId`, `sampGetCharHandleBySampPlayerId`,
  `sampGetPlayerIdByCharHandle`, `sampGetPlayerArmor`, `sampGetPlayerHealth`,
  `sampIsPlayerPaused`, `sampSetSpecialAction`, `sampGetPlayerCount`,
  `sampGetMaxPlayerId`, `sampGetPlayerSpecialAction`,
  `sampStorePlayerOnfootData`, `sampStorePlayerIncarData`,
  `sampStorePlayerPassengerData`, `sampStorePlayerTrailerData`,
  `sampStorePlayerAimData`, `sampSendSpawn`, `sampGetPlayerAnimationId`,
  `sampSetLocalPlayerName`, `sampGetPlayerStructPtr`, `sampSendEnterVehicle`,
  `sampSendExitVehicle`, `sampIsLocalPlayerSpawned`, `sampGetPlayerColor`,
  `sampForceAimSync`, `sampForceOnfootSync`, `sampForceStatsSync`,
  `sampForceTrailerSync`, `sampForceVehicleSync`
- [x] `sampGetLocalPlayerId`, `sampGetLocalPlayerNickname`,
  `sampGetLocalPlayerColor`, `sampIsLocalPlayerSpawned`,
  `sampGetPlayerArmor`, `sampGetPlayerHealth`, `sampGetPlayerSpecialAction`,
  `sampGetPlayerAnimationId` — available for the local player through the
  single cached `HostApi::local_player` snapshot; remote-player calls remain
  pending.
- [ ] `sampIsPlayerDefined`, `sampSetPlayerColor`

### RakNet and network actions (`raknet.lua`)

- [ ] `raknetBitStreamReadBool`, `raknetBitStreamReadBuffer`,
  `raknetBitStreamReadInt8`, `raknetBitStreamReadInt16`,
  `raknetBitStreamReadInt32`, `raknetBitStreamReadFloat`,
  `raknetBitStreamReadString`, `raknetBitStreamResetReadPointer`,
  `raknetBitStreamResetWritePointer`, `raknetBitStreamIgnoreBits`,
  `raknetBitStreamSetWriteOffset`, `raknetBitStreamSetReadOffset`,
  `raknetBitStreamGetNumberOfBitsUsed`, `raknetBitStreamGetNumberOfBytesUsed`,
  `raknetBitStreamGetNumberOfUnreadBits`, `raknetBitStreamGetWriteOffset`,
  `raknetBitStreamGetReadOffset`, `raknetBitStreamGetDataPtr`,
  `raknetNewBitStream`, `raknetDeleteBitStream`, `raknetResetBitStream`,
  `raknetBitStreamWriteBool`, `raknetBitStreamWriteInt8`,
  `raknetBitStreamWriteInt16`, `raknetBitStreamWriteInt32`,
  `raknetBitStreamWriteFloat`, `raknetBitStreamWriteBuffer`,
  `raknetBitStreamWriteString`, `raknetBitStreamDecodeString`,
  `raknetBitStreamEncodeString`, `raknetBitStreamWriteBitStream`,
  `raknetSendRpcEx`, `raknetSendBitStreamEx`, `raknetSendRpc`,
  `raknetSendBitStream`, `raknetGetRpcName`, `raknetGetPacketName`
- [ ] `sampGetRakclientInterface`, `sampGetRakpeer`, `sampSendAimData`,
  `sampSendBulletData`, `sampSendIncarData`, `sampSendOnfootData`,
  `sampSendSpectatorData`, `sampSendTrailerData`, `sampSendPassengerData`,
  `sampSendUnoccupiedData`, `sampSendDamageVehicle`, `sampSendScmEvent`,
  `sampSendGiveDamage`, `sampSendTakeDamage`, `sampSendRequestSpawn`,
  `sampSendClickPlayer`, `sampSendClickTextdraw`, `sampSendDeathByPlayer`,
  `sampSendDialogResponse`, `sampSendEditAttachedObject`, `sampSendEditObject`,
  `sampSendMenuQuit`, `sampSendMenuSelectRow`, `sampSendPickedUpPickup`,
  `sampSendRconCommand`, `sampSendVehicleDestroyed`,
  `sampDisconnectWithReason`, `sampConnectToServer`

### SF.lua’s explicit future items (`init.lua`)

- [ ] `sampHasDialogRespond`, `sampForcePassengerSyncSeatId`,
  `sampForceWeaponsSync`, `sampGetRakclientFuncAddressByIndex`,
  `sampGetRpcCallbackByRpcId`, `sampGetRpcNodeByRpcId`,
  `raknetEmulRpcReceiveBitStream`, `raknetEmulPacketReceiveBitStream`,
  `sampSetClientCommandDescription`, `sampGetStreamedOutPlayerPos`,
  `onSendRpc`, `onSendPacket`, `onReceiveRpc`, `onReceivePacket`
