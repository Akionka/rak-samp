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

### Static-first resumption order

This is the concrete order for autonomous work while live GTA evidence is
unavailable. A completed static batch stays `[~]` until its matching item in
the live-evidence section is recorded; it must not be promoted to `[x]` from
fixtures, disassembly, or the E2E mock alone.

1. [~] Build a bounded, demand-refreshed R1 remote-player directory from
   verified `CPlayerPool` accessors. The static implementation exposes only
   copied ID, connection, nickname bytes, NPC flag, score, ping, and ARGB
   colour through `HostApi::player_info` and projections. Requests are queued
   to the game-thread pump, return `NotReady` until the first copy, refresh
   cached requested IDs incrementally, and never expose a ped, pool, or player
   pointer. Exact R1 code signatures, unit/mock/E2E coverage, and an opt-in
   second-player validator are present; retain `[~]` until its R1 lifecycle,
   cache-refresh, disconnect, and shutdown evidence is recorded.
2. [~] Cache the two accessor-only R1 `CPlayerPool::GetCount` modes through
   `HostApi::player_count(include_npcs)`. The static implementation has an
   exact target signature and publishes scalar counts only on the game-thread
   pump; it deliberately covers SF.lua's non-streamed count path, not the
   GTA-ped-based streamed count. Keep provisional until the direct validator
   records a nonzero count and normal shutdown.
3. [~] Cache R1 `CPlayerPool::m_nLargestId` through
   `HostApi::player_max_id()`. The static implementation reads the fixture
   checked pool-prefix scalar only on the game-thread pump after the exact R1
   `UpdateLargestId` signature passes, and publishes a bounded ID. It covers
   only SF.lua's non-streamed branch; the streamed GTA-ped form remains out of
   scope. Keep provisional until the direct validator records a maximum ID at
   least as large as the assigned local ID and normal shutdown.
4. [~] Cache R1 `CVehiclePool::DoesExist` through
   `HostApi::is_vehicle_defined(id)`. The static implementation has exact R1
   `CNetGame::GetVehiclePool` and `DoesExist` signatures, an independent
   packed vehicle-pool fixture for the touched boolean-array offset, and a
   32-ID demand queue drained at four copied booleans per pump. It exposes no
   vehicle or GTA handle. Keep provisional until the opt-in vehicle scan finds
   a defined ID without traffic and normal shutdown.
5. [~] Cache the bounded active R1 dialog core through
   `HostApi::active_local_dialog()`: ID, six-way style, fixed 65-byte caption,
   and server-side flag. The static implementation verifies the exact
   `CDialog::Show` core-field stores and independent packed fixture offsets,
   refreshes only on the game-thread pump, and exposes an owned copy or
   `Ok(None)` when inactive. Keep provisional until the opt-in dialog lifecycle
   validator records its matching active core, post-dismissal `None`, no
   traffic, and normal shutdown.
6. [ ] Evaluate dynamic bounded dialog and chat-input snapshots separately.
   Do not begin until the exact R1 pointer ownership, string/list bounds,
   active-state interaction, and update lifecycle are independently proven.
   Candidate outputs are copied dialog text/buttons/list selection and copied
   chat draft text only; close/select/edit/open/command registration remain
   mutations and are excluded.
7. [ ] Evaluate read-only pool snapshots one module at a time: player-derived
    state first, then labels, textdraws, objects, and pickups. Each needs a
   bounded copied model, independent native-layout
   fixture, direct target/field fingerprint, pump refresh budget, and a
    dedicated opt-in validator. Do not group unrelated pool layouts into one
    profile change.
   The first bounded label sub-batch is `HostApi::is_text_label_defined(id)`,
   a 2,048-ID demand-refreshed cache of only `CLabelPool::m_bNotEmpty[id]`.
   Its fixture, `ResetLabelPool` field signature, four-per-pump budget, and
   opt-in scan are ready; retain `[~]` until a legal R1 run finds a defined
   label without traffic and exits normally.
   The second bounded textdraw sub-batch is
   `HostApi::is_textdraw_defined(pool_index)`, a 2,304-slot demand-refreshed
   cache of only `CTextDrawPool::m_bNotEmpty[pool_index]`. It preserves the raw
   2,048-global then 256-local R1 pool order. Its fixture,
   `ResetTextDrawPool` field signature, four-per-pump budget, and opt-in scan
   are ready; retain `[~]` until a legal R1 run finds a defined textdraw without
   traffic and exits normally.
   The sixth bounded textdraw sub-batch is `HostApi::textdraw(pool_index)`, a
   2,304-slot demand-refreshed numeric copy. Its independent data-layout
   fixture, constructor store signatures, four-per-pump budget, E2E mock, and
   opt-in snapshot scan are ready; retain `[~]` until a legal R1 run verifies a
   visible textdraw without traffic and exits normally. Display strings remain
   a separate pending semantic investigation.
   The third bounded object sub-batch is `HostApi::is_object_defined(id)`, a
   1,000-ID demand-refreshed cache of only `CObjectPool::m_bNotEmpty[id]`. Its
   fixture, `ResetObjectPool` field signature, four-per-pump budget, and opt-in
   scan are ready; retain `[~]` until a legal R1 run finds a defined object
   without traffic and exits normally.
   The fourth bounded gangzone sub-batch is `HostApi::gangzone(id)`, a
   1,024-ID demand-refreshed cache of fixed rectangle and draw-colour scalars.
   Its fixture, `ResetGangZonePool` and `CGangZonePool::Create` field signatures,
   four-per-pump budget, and opt-in scan are ready; retain `[~]` until a legal R1
   run finds a gangzone with correct copied fields, no traffic, and normal exit.
   The fifth bounded label sub-batch is `HostApi::text_label(id)`, a 2,048-ID
   demand-refreshed cache of copied text, colour, position, distance, LOS, and
   attachment-ID fields. Its fixture, `CLabelPool::Create` allocation/copy/
   scalar-store signatures, four-per-pump budget, and opt-in scan are ready;
   retain `[~]` until a legal R1 run finds one visible record, no traffic, and
   exits normally.
8. [ ] Reconcile the remaining typed protocol names below against the existing
   event codecs. Add a named safe convenience only when its exact R1 wire
   vector already exists or can be independently tested. Do not emulate a
   client-side native action merely because it sends the same RPC.
9. [ ] When the static-only list is exhausted, prepare release artifacts and
   validation instructions, then wait for the live R1 scenarios below. Do not
   start mutations, force-sync, reconnection, raw pointer, or raw callback
   APIs without a new explicit experimental/unsafe design.

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
  dismiss it; open and close chat input as well. Confirm its initial
  `active_dialog=Ok` result matches the queued dialog core, its outcome records
  both active-dialog-core `Some`/`None` states alongside cached dialog and
  chat-input active/inactive pairs, with no packet/RPC emission or shutdown
  instability.
- [ ] Confirm the direct-client validator reads R1 animation entry zero as
  `AIRPORT:THRW_BARL_THRW` and resolves those byte strings back to zero. It is
  an automatic static-table check, but must still be recorded with the normal
  R1 lifecycle and shutdown evidence.
- [ ] With a second player connected, enable the player-directory marker and
  confirm the validator records only `player-directory self-test passed` and
  that remote player ID. Verify the first read is nonblocking/possibly
  `NotReady`, a later cached result has the observed connected nickname/NPC,
  colour, score, and ping values, a disconnected result becomes `None` after
  refresh, it generates no packet/RPC traffic, and shutdown is stable.
- [ ] Confirm the direct validator records `player_count=Ok` after joining a
  server and that the cached including-NPC count is nonzero and sensible for
  the visible player list. It must not generate packet/RPC traffic or make
  shutdown unstable. Streamed-ped count remains out of scope pending separate
  GTA-ped evidence.
- [ ] Confirm the direct validator records `player_max_id=Ok` after joining a
  server and that the non-streamed maximum ID is at least the assigned local
  ID. It must generate no packet/RPC traffic and leave normal shutdown stable;
  the streamed-GTA-ped branch remains out of scope pending separate evidence.
- [ ] With the vehicle-exists marker enabled, confirm the validator records
  only `vehicle-exists self-test passed` and one defined vehicle ID. It must
  tolerate initial `NotReady` while the bounded queue is pumped, generate no
  packet/RPC traffic, and leave normal shutdown stable.
- [ ] With the text-label-exists marker enabled on a server that displays a
  3D label, confirm the validator records only
  `text-label-exists self-test passed` and one defined label ID. It must
  tolerate initial `NotReady` while the bounded queue is pumped, generate no
  packet/RPC traffic, and leave normal shutdown stable.
- [ ] With the text-label marker enabled on a server that displays a 3D label,
  confirm the validator records only `text-label self-test passed` and one
  label ID. Independently compare the copied content and scalar fields to the
  visible label without logging any label text or fields. It must tolerate
  initial `NotReady`, generate no packet/RPC traffic, and leave normal shutdown
  stable.
- [ ] With the textdraw-exists marker enabled on a server that displays a
  textdraw, confirm the validator records only `textdraw-exists self-test
  passed` and one defined raw pool index. It must tolerate initial `NotReady`
  while the bounded queue is pumped, generate no packet/RPC traffic, and leave
  normal shutdown stable.
- [ ] With the textdraw marker enabled on a server that displays a textdraw,
  confirm the validator records only `textdraw self-test passed` and one raw
  pool index. Independently compare the copied numeric record to the visible
  textdraw without logging its content or fields. It must tolerate initial
  `NotReady`, generate no packet/RPC traffic, and leave normal shutdown stable.
- [ ] With the object-exists marker enabled on a server that displays a
  streamed object, confirm the validator records only `object-exists self-test
  passed` and one defined object ID. It must tolerate initial `NotReady` while
  the bounded queue is pumped, generate no packet/RPC traffic, and leave normal
  shutdown stable.
- [ ] With the gangzone marker enabled on a server that displays a gangzone,
  confirm the validator records only `gangzone self-test passed` and one ID.
  Independently confirm the copied rectangle and both colours match the visible
  gangzone. It must tolerate initial `NotReady`, generate no packet/RPC traffic,
  and leave normal shutdown stable.
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

- [ ] `sampGetChatInfoPtr` — raw pointer API; permanently excluded from the
  safe ABI. `sampSetChatDisplayMode` and `sampSetChatString` are client UI
  mutations; excluded pending an explicit experimental policy. `sampGetChatString`
  is a future copied read only after the R1 chat-ring layout, string capacity,
  and lifecycle are independently proven; it belongs to static-first step 6.
- [~] `sampGetChatDisplayMode`, `sampIsChatVisible` —
  `HostApi::local_chat_display_mode` and its derived
  `HostApi::is_local_chat_visible` return a game-thread-cached R1 enum only.
  Keep provisional until the dedicated three-mode and shutdown live check.
- [~] `sampAddChatMessage`, `sampAddChatMessageEx` —
  `HostApi::show_local_chat_message` copies one bounded R1 chat/info/debug
  entry for the game-thread pump without sending any packet or RPC. Keep this
  provisional until the dedicated live R1 UI and shutdown scenario runs.
- [ ] `sampGetKillInfoPtr` — raw death-window pointer; permanently excluded
  from the safe ABI.
- [~] `sampAddDeathMessage` — `HostApi::show_local_death_message` copies one
  bounded R1 death-window entry for the game-thread pump without packet/RPC
  emulation. Keep this provisional until its dedicated live UI/shutdown check.

### Dialog, cursor, and input (`dialog.lua`, `game.lua`, `input.lua`)

- [x] `sampShowDialog` — represented by safe queued
  `HostApi::show_local_dialog`.
- [ ] `sampGetDialogInfoPtr` — raw pointer API; excluded.
  `sampCloseCurrentDialogWithButton`, `sampSetCurrentDialogListItem`,
  `sampSetCurrentDialogEditboxText`, and `sampSetDialogClientside` mutate
  client UI; excluded.
- [~] `sampGetCurrentDialogType`, `sampGetCurrentDialogId`,
  `sampGetDialogCaption`, and `sampIsDialogClientside` —
  `HostApi::active_local_dialog` returns their bounded copied active-dialog
  core only. Keep provisional until the exact active/dismissed lifecycle,
  no-traffic, and shutdown live check is recorded.
- [ ] `sampGetCurrentDialogListItem`, `sampGetCurrentDialogEditboxText`,
  `sampGetDialogText`, `sampGetListboxItemsCount`, and
  `sampGetListboxItemText` require dynamic pointer/string/list ownership and
  bounds evidence before a separate copied snapshot in static-first step 6.
- [~] `sampIsDialogActive` — `HostApi::is_local_dialog_active` is a cached
  R1 game-thread read only. Keep provisional until direct-dialog active and
  dismissal states are observed in the live lifecycle test.
- [ ] `sampGetMiscInfoPtr`, `sampToggleCursor`, `sampSetCursorMode` — pointer
  access and UI mutations remain outside the safe ABI.
- [~] `sampIsCursorActive`, `sampGetCursorMode` —
  `HostApi::local_cursor_mode` and its derived
  `HostApi::is_local_cursor_active` copy a cached R1 cursor state only. Keep
  provisional until the dedicated cursor transition and shutdown live check.
- [ ] `sampGetInputInfoPtr` — raw pointer API; excluded. `sampRegisterChatCommand`,
  `sampUnregisterChatCommand`, `sampSetChatInputText`, `sampSetChatInputEnabled`,
  and `sampProcessChatInput` mutate client state or retain a foreign callback;
  excluded. `sampGetChatInputText` is the copied bounded read candidate in
  static-first step 6. `sampIsChatCommandDefined` requires an owned command
  registry design and verified native ownership rules; do not infer it from a
  raw map or a plugin callback pointer.
- [~] `sampIsChatInputActive` — `HostApi::is_local_chat_input_active` is a
  cached R1 game-thread read only. Keep provisional until normal chat-input
  open/close and shutdown live evidence is recorded.

### Pools, labels, objects, pickups, vehicles, textdraws (`gangzone.lua`, `label.lua`, `object.lua`, `pickup.lua`, `vehicle.lua`, `textdraw.lua`)

- [ ] `sampGetGangzonePoolPtr` — raw pointer API; excluded.
- [~] Safe replacement for a useful read-only gangzone view:
  `HostApi::gangzone(id)` demand-refreshes an owned R1 rectangle and two draw
  colours. It never exposes a gangzone/pool pointer. Keep provisional until the
  opt-in gangzone scan verifies one visible record, no traffic, and normal
  shutdown.
- [ ] `sampGetTextlabelPoolPtr` — raw pointer API; excluded.
  `sampCreate3dText`, `sampSet3dTextString`, `sampDestroy3dText`, and
  `sampCreate3dTextEx` mutate the native label pool; excluded.
- [~] `sampIs3dTextDefined` — `HostApi::is_text_label_defined(id)` uses a
  bounded demand-refreshed R1 `CLabelPool::m_bNotEmpty` cache. It exposes only
  the copied boolean, never a label/pool pointer or label text. Keep provisional
  until the opt-in label scan records a defined ID, no traffic, and normal
  shutdown.
- [~] `sampGet3dTextInfoById` — `HostApi::text_label(id)` demand-refreshes a
  bounded copied R1 label snapshot (byte text, ARGB colour, position, distance,
  LOS, and optional attachment IDs). The R1 `CLabelPool::Create` allocation,
  copy, and scalar-store signatures plus an independent packed fixture prove
  the narrow profile. Keep provisional until the opt-in visible-label scan,
  no-traffic result, and normal shutdown are recorded.
- [ ] `sampGetObjectPoolPtr`, `sampGetObjectHandleBySampId`, and
  `sampGetObjectSampIdByHandle` expose native/GTA pointers or handles; excluded
  from the safe ABI rather than wrapped as integer addresses.
- [~] Safe read-only prerequisite for `sampGetObjectHandleBySampId`:
  `HostApi::is_object_defined(id)` is a bounded demand-refreshed R1
  `CObjectPool::m_bNotEmpty` cache. It exposes only the copied boolean, never an
  object/pool pointer or GTA handle. Keep provisional until the opt-in object
  scan records a defined ID, no traffic, and normal shutdown.
- [ ] `sampGetPickupPoolPtr`, `sampGetPickupHandleBySampId`, and
  `sampGetPickupSampIdByHandle` expose native/GTA pointers or handles; excluded.
- [ ] `sampGetVehiclePoolPtr`, `sampGetCarHandleBySampVehicleId`, and
  `sampGetVehicleIdByCarHandle` expose native/GTA pointers or handles; excluded.
- [~] `sampIsVehicleDefined` — `HostApi::is_vehicle_defined(id)` uses a
  bounded, demand-refreshed R1 `CVehiclePool::DoesExist` boolean cache. It
  never exposes the pool, vehicle, or GTA handle. Keep provisional until the
  opt-in vehicle scan records a defined ID, no traffic, and normal shutdown.
- [ ] `sampGetTextdrawPoolPtr` — raw pointer API; excluded. `sampTextdrawCreate`,
  `sampTextdrawSetBoxColorAndSize`, `sampTextdrawDelete`,
  `sampTextdrawSetLetterSizeAndColor`, `sampTextdrawSetPos`,
  `sampTextdrawSetString`, `sampTextdrawSetModelRotationZoomVehColor`,
  `sampTextdrawSetOutlineColor`, `sampTextdrawSetShadow`, `sampTextdrawSetStyle`,
  `sampTextdrawSetProportional`, and `sampTextdrawSetAlign` mutate native UI;
  excluded.
- [~] `sampTextdrawIsExists` — `HostApi::is_textdraw_defined(pool_index)` uses
  a bounded demand-refreshed R1 `CTextDrawPool::m_bNotEmpty` cache. It preserves
  the raw 2,048-global then 256-local slot order and exposes only the copied
  boolean. Keep provisional until the opt-in textdraw scan records a defined
  slot, no traffic, and normal shutdown.
- [~] `sampTextdrawGetLetterSizeAndColor`, `sampTextdrawGetPos`,
  `sampTextdrawGetShadowColor`, `sampTextdrawGetOutlineColor`,
  `sampTextdrawGetStyle`, `sampTextdrawGetProportional`, `sampTextdrawGetAlign`,
  `sampTextdrawGetBoxEnabledColorAndSize`, and
  `sampTextdrawGetModelRotationZoomVehColor` — `HostApi::textdraw(pool_index)`
  copies the proven numeric R1 fields through a bounded game-thread cache.
  Keep provisional until its dedicated live snapshot scan confirms a visible
  record without traffic or shutdown instability.
- [ ] `sampTextdrawGetString` needs a distinct fixed bounded-copy design after
  the R1 display-string allocation, replacement, and lifetime semantics are
  independently proven. Do not expose a native string or pool pointer.

### Net game and scoreboard (`netgame.lua`, `scoreboard.lua`)

- [ ] `sampGetSampInfoPtr`, `sampGetSampPoolsPtr`, and
  `sampGetServerSettingsPtr` are raw pointers; excluded. `sampSetGamestate`
  and `sampSetSendrate` directly mutate client state/timing; excluded pending
  an explicit unsafe experimental design.
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

- [ ] `sampGetPlayerPoolPtr`, `sampGetCharHandleBySampPlayerId`,
  `sampGetPlayerIdByCharHandle`, and `sampGetPlayerStructPtr` expose native or
  GTA pointers/handles; excluded. `sampSpawnPlayer`, `sampSetSpecialAction`,
  `sampSetLocalPlayerName`, `sampForceUnoccupiedSyncSeatId`,
  `sampForceAimSync`, `sampForceOnfootSync`, `sampForceStatsSync`,
  `sampForceTrailerSync`, and `sampForceVehicleSync` are native mutations or
  force-sync APIs; excluded. Remote `sampGetPlayerArmor`, `sampGetPlayerHealth`,
  `sampGetPlayerSpecialAction`, and `sampGetPlayerAnimationId` require their
  own remote-player layout or accessor evidence and a copied snapshot in
  static-first step 7. `sampStorePlayerOnfootData`, `sampStorePlayerIncarData`,
  `sampStorePlayerPassengerData`, `sampStorePlayerTrailerData`, and
  `sampStorePlayerAimData` may only become owned typed sync copies after exact
  source-field/layout proof; they must never write to a plugin-supplied pointer.
  Preparation audit (not implementation evidence): the installed R1 DLL's
  `CRemotePlayer::Update(OnfootData, TICK)` at RVA `0x139A0` copies the
  fixture-sized 68-byte on-foot record to `this + 0xC8`, converts its health
  and armour bytes into reported floats at `+0x1BC` and `+0x1B8`, and writes
  its special-action byte to `+0xBB`. Before exposing any snapshot, add an
  independent packed remote-player fixture, exact complete update-path
  signatures for every copied field (including animation lifecycle), bounded
  cache/queue tests, ABI mock/E2E coverage, and an opt-in second-client live
  scenario that exercises damage, special action, and animation transitions.
- [~] `sampGetPlayerCount` — `HostApi::player_count(include_npcs)` caches the
  two R1 `CPlayerPool::GetCount` scalar modes, covering SF.lua's non-streamed
  `GetCount(true)` path. The streamed-ped form remains pending GTA-ped layout
  evidence. Keep this scalar cache provisional until its direct live check.
- [~] `sampGetMaxPlayerId` — `HostApi::player_max_id()` caches the exact R1
  `CPlayerPool::m_nLargestId` scalar and covers SF.lua's non-streamed branch.
  Its streamed-GTA-ped branch remains pending separate native-layout evidence.
  Keep this cache provisional until its direct live check.
- [~] `sampIsPlayerConnected`, `sampGetPlayerNickname`, `sampIsPlayerNpc`,
  remote forms of `sampGetPlayerScore`, `sampGetPlayerPing`, and
  `sampGetPlayerColor` — `HostApi::player_info` plus its projections use a
  bounded, demand-refreshed R1 accessor-only directory. The local-ID path is
  derived from the existing local snapshot. Keep provisional until a second
  connected player, refresh/disconnect transition, no-traffic result, and
  shutdown are recorded by the opt-in validator.
- [x] `sampGetLocalPlayerId`, `sampGetLocalPlayerNickname`,
  `sampGetLocalPlayerColor`, `sampIsLocalPlayerSpawned`,
  `sampGetPlayerArmor`, `sampGetPlayerHealth`, `sampGetPlayerSpecialAction`,
  `sampGetPlayerAnimationId` — explicit safe local-player query methods reuse
  the single cached `HostApi::local_player` snapshot; remote-player calls
  remain pending.
- [~] `sampGetPlayerScore`, `sampGetPlayerPing` — `HostApi::player_score` and
  `HostApi::player_ping` cover cached local and demand-refreshed remote IDs;
  the existing `local_player_*` projections remain available. Keep provisional
  with the player-directory live gate.
- [~] `sampIsPlayerPaused` — `HostApi::is_player_paused(id)` reuses the
  bounded player directory and fingerprinted R1 `CRemotePlayer::GetStatus`
  accessor; it maps only `PLAYER_STATE_NONE` to true and always returns false
  for the local player. Keep provisional with the second-player live gate.
- [~] `sampRequestClass`, `sampSendInteriorChange`, `sampSendSpawn`,
  `sampSendEnterVehicle`, `sampSendExitVehicle` — the corresponding
  `HostApi::send_*` methods serialize the exact R1 outbound RPCs, but remain
  protocol-only and do not invoke SF.lua's native local-player state changes.
- [x] `sampSendChat` — `HostApi::send_chat` serializes the typed, bounded
  server-bound RPC 101 payload, or RPC 50 for slash-prefixed commands, through
  the original RakClient send path.
- [~] `sampIsPlayerDefined` — `HostApi::is_player_defined(id)` reuses the
  bounded directory cache and the fingerprinted R1 `CRemotePlayer::DoesExist`
  accessor. It distinguishes a connected remote player from a defined
  client-world object without exposing either native object or ped. Keep
  provisional with the second-player directory live gate. `sampSetPlayerColor`
  mutates a remote client entity and is excluded.

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
- [ ] `sampGetRakclientInterface` and `sampGetRakpeer` are raw client pointers;
  excluded. `sampDisconnectWithReason` and `sampConnectToServer` change
  connection state and are excluded from this in-process host policy.
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

- [ ] `sampHasDialogRespond` requires a response-lifecycle model and remains a
  future copied-state candidate only after a dedicated dialog profile. The
  force-sync functions `sampForcePassengerSyncSeatId` and
  `sampForceWeaponsSync` are excluded. `sampGetRakclientFuncAddressByIndex`,
  `sampGetRpcCallbackByRpcId`, and `sampGetRpcNodeByRpcId` expose raw code or
  callback pointers; excluded. `raknetEmulRpcReceiveBitStream` and
  `raknetEmulPacketReceiveBitStream` require an explicit event-emulation design
  and cannot bypass the host's exactly-once listener path. `sampSetClientCommandDescription`
  mutates native command state; excluded. `sampGetStreamedOutPlayerPos` is a
  future copied remote-player query only after a separate R1 layout proof.
  `onSendRpc`, `onSendPacket`, `onReceiveRpc`, and `onReceivePacket` are
  already represented by the owned, scoped host subscription API; no Lua-style
  global callback or raw event pointer will be added.
