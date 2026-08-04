# Review Evidence

Keep only unresolved findings and evidence that defines a native boundary here.
Implementation history belongs in Git; planned work belongs in [TODO.md](TODO.md).

## Open findings

- **Direct R1 profile live gate:** the R1 dialog/snapshot profile is
  fail-closed behind SA-MP and GTA PE/code fingerprints, and its narrow packed
  offsets have an independent fixture. A legal GTA SA 1.0 US + SA-MP 0.3.7 R1
  run still must record stable dialog dismissal, walking/damage/vehicle
  snapshot changes, zero direct-dialog RPC 61/62 observations, and normal
  shutdown before release.
- **Cached R1 server metadata live gate:** `HostApi::server_info` has an
  independent packed fixture and an R1 code anchor, but still needs a legal
  R1 run that compares its copied address, hostname, and port with the selected
  server and confirms normal shutdown.
- **Queued R1 local-chat live gate:** `HostApi::show_local_chat_message` is
  fail-closed behind the R1 PE/GTA checks and a `CChat::AddEntry` code
  signature. A legal R1 run still must confirm a visible chat/info/debug entry,
  no generated RPC 61/62 traffic, and stable normal shutdown.
- **Queued R1 death-window live gate:** `HostApi::show_local_death_message`
  is fail-closed behind the R1 PE/GTA checks and both `CDeathWindow` target
  signatures. A legal R1 run must confirm a visible entry, no generated
  packet/RPC traffic, and stable normal shutdown.
- **Cached R1 chat-display-mode live gate:** `HostApi::local_chat_display_mode`
  is fail-closed behind the same R1 profile and the exact `CChat::GetMode`
  leaf-accessor signature. A legal R1 run must cycle off, no-shadow, and
  normal modes, compare the cached enum with the visible chat state, confirm
  no generated packet/RPC traffic, and exit normally.
- **Cached R1 cursor/scoreboard live gate:** `HostApi::local_cursor_mode` and
  `HostApi::is_local_scoreboard_open` are fail-closed behind exact R1 code
  signatures plus independently checked packed offsets. A legal R1 run must
  observe cursor inactive/active and scoreboard closed/open transitions,
  confirm the cache agrees with the visible UI, produces no traffic, and exits
  normally.
- **Cached R1 dialog/input live gate:** `HostApi::is_local_dialog_active`,
  `HostApi::active_local_dialog`, and `HostApi::is_local_chat_input_active`
  are fail-closed behind exact R1 code signatures plus independently checked
  packed offsets. A legal R1 run must observe the queued dialog core while it
  is active, its `None` state after dismissal, active/inactive flag transitions
  for the direct dialog and normal chat input, confirm the cache agrees with
  the UI, produces no traffic, and exits normally.
- **Cached R1 animation-table live gate:** `HostApi::local_animation` and
  `HostApi::local_animation_id` are fail-closed behind the exact R1 table
  fingerprint. A legal R1 lifecycle run must record the validator's known
  entry/round-trip lookup, no generated traffic, and normal shutdown.
- **Cached R1 player-directory live gate:** `HostApi::player_info` keeps only
  host-owned ID, nickname bytes, local/NPC flags, ARGB colour, score, and ping.
  A remote lookup is demand-refreshed on the game-thread pump through exact
  `CPlayerPool` accessors, never via a plugin-thread client call. A legal R1
  run with a second player must show initial nonblocking `NotReady`, a copied
  entry, disconnected refresh to `None`, no generated traffic, and stable
  shutdown before release.
- **Cached R1 player-count live gate:** `HostApi::player_count` calls the two
  `CPlayerPool::GetCount` accessor modes only from the game-thread pump and
  publishes bounded scalar values. A legal R1 run must confirm a sensible
  nonzero count, no generated traffic, and stable shutdown; streamed GTA-ped
  counting is intentionally not claimed.
- **Cached R1 player-max-ID live gate:** `HostApi::player_max_id` reads only
  the bounded non-streamed R1 player-pool maximum from the game-thread pump.
  A legal R1 run must confirm it is at least the assigned local ID, produces
  no generated traffic, and exits normally; streamed GTA-ped maximum-ID
  semantics are intentionally not claimed.
- **Cached R1 vehicle-existence live gate:** `HostApi::is_vehicle_defined`
  demand-refreshes a bounded `CVehiclePool::DoesExist` boolean only from the
  game-thread pump. A legal R1 run must observe a defined ID through the
  opt-in scan, no generated traffic, and stable shutdown; no pool/vehicle/GTA
  pointer is exposed.
- **Cached R1 3D text-label-existence live gate:**
  `HostApi::is_text_label_defined` demand-refreshes only the bounded
  `CLabelPool::m_bNotEmpty[id]` boolean from the game-thread pump. A legal R1
  run on a server with a visible 3D label must observe a defined ID through the
  opt-in scan, no generated traffic, and stable shutdown; no label text or
  label/pool pointer is exposed.
- **Cached R1 3D text-label snapshot live gate:** `HostApi::text_label`
  demand-refreshes one bounded copied label record from the game-thread pump.
  A legal R1 run on a server with a visible label must observe one record with
  the matching ID and defined flag, no generated traffic, and stable shutdown;
  logs must contain outcomes and IDs only, never label text or fields.
- **Cached R1 textdraw-existence live gate:** `HostApi::is_textdraw_defined`
  demand-refreshes only the bounded `CTextDrawPool::m_bNotEmpty[pool_index]`
  boolean from the game-thread pump. A legal R1 run on a server with a visible
  textdraw must observe a defined raw pool index through the opt-in scan, no
  generated traffic, and stable shutdown; no textdraw data or pool pointer is
  exposed.
- **Cached R1 numeric textdraw live gate:** `HostApi::textdraw` demand-refreshes
  one bounded copied numeric record from the game-thread pump. A legal R1 run
  on a server with a visible textdraw must observe one record with the matching
  raw pool index and defined flag, no generated traffic, and stable shutdown;
  logs must contain outcomes and indexes only, never display text or fields.
- **Cached R1 object-existence live gate:** `HostApi::is_object_defined`
  demand-refreshes only the bounded `CObjectPool::m_bNotEmpty[id]` boolean from
  the game-thread pump. A legal R1 run on a server with a visible streamed
  object must observe a defined ID through the opt-in scan, no generated traffic,
  and stable shutdown; no object, pool, or GTA pointer is exposed.
- **Cached R1 gangzone live gate:** `HostApi::gangzone` demand-refreshes only a
  bounded fixed gangzone rectangle and two draw colours from the game-thread
  pump. A legal R1 run on a server with a visible gangzone must observe a copied
  record through the opt-in scan, no generated traffic, and stable shutdown; no
  gangzone or pool pointer is exposed.

## Windows x86 evidence

- **GTA SA 1.0 US PE entry point (2026-08-04):** direct inspection of the
  live `C:\Games\GTASA\gta_sa.exe` records the expected 14,383,616-byte file,
  machine `0x014C`, image base `0x00400000`, image size `0x01177000`, and raw
  PE `AddressOfEntryPoint` RVA `0x00424570`. Its loaded entry address is
  therefore `0x00824570`. An earlier inspection incorrectly subtracted the
  image base from an RVA and changed the profile to `0x00024570`; that
  regression caused every direct R1 helper to return `UnsupportedVersion`.
  The strict profile now compares the exact raw PE value and retains the
  executable-code check at image-base plus RVA.

- **Dynamic dialog/input audit boundary:** the upstream multiversion R1
  headers and SF.lua source identify candidate dialog list/edit control
  pointers and a list-selection field, but those references are leads only.
  They do not prove DXUT allocation ownership, control lifetime, text bounds,
  or a matching installed-R1 code path. No dynamic dialog or chat-draft bytes
  are therefore copied by the host yet; a separate fixture, target signatures,
  and live active/inactive scenario are required before that surface exists.

- **R1 player-directory accessor targets:** the installed SA-MP 0.3.7 R1
  `samp.dll` audit confirms `CNetGame::GetPlayerPool` at RVA `0x1160`,
  `CPlayerPool::{IsConnected,GetPlayer,IsNPC,GetName,GetScore,GetPing}` at
  `0x10B0`, `0x10F0`, `0xB680`, `0x13CE0`, `0x6A190`, and `0x6A1C0`, and
  `CRemotePlayer::{GetColorAsARGB,DoesExist,GetStatus}` at `0x12A00`,
  `0x1080`, and `0x12BA0`.
  `DoesExist` begins `83 39 00 74 0D 8A 41 09 84 C0 74 06 B8 01 00 00 00 C3`;
  `GetStatus` has its complete 52-byte R1 body pinned before the profile maps
  status zero to the safe paused boolean; all exact leading code bytes are
  pinned in the R1 profile and unit-tested.
  The SAMP API R1 reference is a lead, not authority. These are accessor calls
  only—this batch introduces no native field layout, so no new C++ layout
  fixture is claimed. The profile still requires the existing strict SA-MP and
  GTA SA 1.0 US fingerprints.
- **R1 remote-player snapshot layout and code paths:** the independent x86
  fixture now reconstructs the packed remote-player prefix through status at
  `0x1D1`: special action `0xBB`, on-foot cache `0xC8`, reported armour/health
  `0x1B8`/`0x1BC`, and animation word `0x1C0`. The R1 profile additionally
  requires the complete 104-byte `CRemotePlayer::Update(OnfootData) + 0x2F`
  signature for the health/armour/action writes and the 43-byte
  `CRemotePlayer::Process + 0x1A6` signature for the animation publication.
  `HostApi::remote_player_state` copies only those scalars through a bounded
  game-thread cache. Live transition evidence remains required before release.
- **R1 player-count accessor target:** the installed SA-MP 0.3.7 R1 `samp.dll`
  audit confirms `CPlayerPool::GetCount(BOOL)` at RVA `0x10520`, with leading
  bytes `8B 54 24 04 56 33 C0 85 D2 57 74 71 33 D2 8B FF`. The profile verifies
  this exact signature before enabling any direct R1 helper. It is a native
  accessor call, not a new native-memory layout, so no C++ layout fixture is
  claimed.
- **R1 player-max-ID field and update target:** the independent C++ fixture
  places the R1 player-pool `m_nLargestId` prefix field at offset `0x00` before
  the already checked local-ID field at `0x04`. The installed SA-MP 0.3.7 R1
  `samp.dll` audit identifies `CPlayerPool::UpdateLargestId` at RVA `0x102B0`
  with leading bytes `56 57 33 F6 B8 02 00 00 00 8D 91 E2 0F 00 00 90`; the
  profile requires that exact target signature before its game-thread pump
  copies the signed field. Values outside the R1 player-ID range are rejected,
  and neither the pool pointer nor a GTA ped reaches the ABI.
- **R1 vehicle-pool existence targets and layout:** the installed SA-MP 0.3.7
  R1 `samp.dll` audit confirms `CNetGame::GetVehiclePool` at RVA `0x1170` with
  bytes `8B 81 CD 03 00 00 8B 40 1C C3`, and
  `CVehiclePool::DoesExist(ID)` at `0x1140` with its complete 29-byte bounds
  check and `m_bNotEmpty[id]` load signature. The independent packed C++
  fixture derives the 40-byte R1 `VehicleInfo` and 100-entry waiting list,
  placing `m_bNotEmpty` at offset `0x3074`, matching the accessor target.
- **R1 3D text-label-pool existence layout:** the pinned R1 C++ lead defines
  packed `CNetGame::m_pPools` at `0x3CD`, with the `CLabelPool*` at pools
  offset `0x0C`. The independent x86 fixture derives the packed 29-byte
  `TextLabel` and therefore `CLabelPool::m_bNotEmpty` at `0xE800` after 2,048
  entries. The installed fingerprinted R1 DLL's `CNetGame::ResetLabelPool` at
  RVA `0x8F00 + 0x15` begins
  `51 56 8B F1 8B 86 CD 03 00 00 57 8B 78 0C 85 FF 74 10`, directly loading
  those two pool-pointer fields. The profile verifies this exact anchor, probes
  the bounded range, and copies only canonical `0/1` flags for the separate
  existence helper on the game-thread pump.
- **R1 3D text-label snapshot layout and ownership:** the same independent
  packed fixture places `TextLabel` at 29 bytes: text pointer `0x00`, ARGB
  colour `0x04`, position `0x08`, draw distance `0x14`, LOS byte `0x18`, and
  player/vehicle attachment IDs at `0x19`/`0x1B`. The installed fingerprinted
  R1 DLL's `CLabelPool::Create` at RVA `0x11C0` has exact signatures at
  `+0x6B` for source-length-plus-terminator allocation, `+0x82` for storing
  the new pointer and copying its terminated bytes, and `+0xCD` for the fixed
  scalar stores. The create RPC already bounds decoded label text to 4,095
  bytes. The profile requires all anchors, checks the existence flag, probes
  each range, copies at most 4,095 non-NUL bytes on the game-thread pump, and
  treats any malformed or unavailable field as `NotReady`; no dynamic pointer
  crosses the ABI.
- **R1 textdraw-pool existence layout:** the pinned R1 C++ lead defines the
  packed `CTextDrawPool*` at `CNetGame::m_pPools + 0x10`; its 2,304-BOOL
  `m_bNotEmpty` array starts at offset `0`, with 2,048 global slots followed by
  256 local slots. The independent x86 fixture checks both offsets and the
  0x4800-byte prefix. The installed fingerprinted R1 DLL's
  `CNetGame::ResetTextDrawPool` at RVA `0x8C20 + 0x15` begins
  `51 56 8B F1 8B 86 CD 03 00 00 57 8B 78 10 85 FF 74 10`, directly loading
  those pool-pointer fields. The profile verifies this exact anchor, probes
  the bounded range, and copies only canonical `0/1` flags on the game-thread
  pump; textdraw content is intentionally not read.
- **R1 numeric textdraw layout and ownership:** the independent packed fixture
  places the `CTextDraw` data block at `0x963`, behind each textdraw object
  pointer in `CTextDrawPool` at `0x2400 + pool_index * 4`. The installed
  fingerprinted R1 DLL's `CTextDraw` constructor at RVA `0xACF10` has exact
  core scalar-store and model-field signature regions at `+0x19` and `+0xB4`;
  the R1 profile requires both before any numeric copy. The game-thread pump
  checks the pool flag, canonical booleans, readable ranges, and finite floats,
  then publishes only owned scalars. Display-string storage is deliberately
  excluded until its separate allocation and lifecycle semantics are proven.
- **R1 object-pool existence layout:** the pinned R1 C++ lead defines the
  packed `CObjectPool*` at `CNetGame::m_pPools + 0x04`, with its signed largest
  ID at offset `0` and 1,000-BOOL `m_bNotEmpty` array at offset `0x04`. The
  independent x86 fixture checks both offsets. The installed fingerprinted R1
  DLL's `CNetGame::ResetObjectPool` at RVA `0x8CC0 + 0x15` begins
  `51 56 8B F1 8B 86 CD 03 00 00 57 8B 78 04 85 FF 74 10`, directly loading
  those pool-pointer fields. The profile verifies this exact anchor, probes the
  bounded range, and copies only canonical `0/1` flags on the game-thread pump;
  object state and GTA handles are intentionally not read.
- **R1 gangzone snapshot layout:** the pinned R1 C++ lead defines the packed
  `CGangZonePool*` at `CNetGame::m_pPools + 0x08`, with 1,024 object pointers
  followed by `m_bNotEmpty` at `0x1000`; its fixed `GangZone` record is 24 bytes
  of four rectangle floats and two Direct3D colours. The independent x86 fixture
  checks those sizes and offsets. The installed fingerprinted R1 DLL's
  `CNetGame::ResetGangZonePool` at RVA `0x8D60 + 0x15` begins
  `51 56 8B F1 8B 86 CD 03 00 00 57 8B 78 08 85 FF 74 10`. Its
  `CGangZonePool::Create` target at RVA `0x2170 + 0x19` clears the indexed
  pointer and `m_bNotEmpty` entry with
  `C7 04 BE 00 00 00 00 C7 84 BE 00 10 00 00 00 00 00 00`, then at
  `+0x39` writes all six 24-byte record fields. The profile verifies both
  anchors, checks ranges, and copies only finite scalar fields on the game-thread
  pump.
  The profile validates both signatures, checks the addressed boolean range
  before calling, and copies only a bounded BOOL.

- **RakNet packet layout:** `RawPacket` and its embedded `PacketPlayerId` use
  packed offsets. The by-value incoming-RPC `RpcPlayerId` is a distinct aligned
  layout. The independent C++ fixture and an SA-MP 0.3.7 R1 live run support
  this split; do not infer native packing from serialized sizes.
- **Incoming queue:** packet emulation uses the RakPeer receiver captured by
  the incoming-RPC detour, not RakClient pointer arithmetic. R1 loopback
  observed emulated packet and RPC events once each through normal dispatch.
- **Encoded strings:** the host calls the detected client's StringCompressor
  reader and writer as x86 `thiscall` functions. An R1 live test encoded,
  decoded, replaced, and blocked a private dialog without instability. The
  owned `HostApi::decode_string` path reuses that verified reader behind copied
  input/output buffers, a 16 MiB input cap, a 4,096-byte terminating output
  buffer, and a scalar returned read cursor; it introduces no new native
  address, layout, or pointer ABI.
- **Hook and unload checks:** fixture tests cover owned-slot restoration and
  original calls. An R1 validation run completed callback quiescence before an
  external manager released the validation ASI.
- **Direct R1 profile:** `CDialog::Show` is called as x86 `thiscall` with
  `serverside = false`; the dialog, net-game, player-pool, local-player, and
  ped call/field leads are isolated in `src/platform/win32/r1_client.rs`.
  The R1 source leads are pinned to
  [`blasthacknet/samp-api@6d4db99`](https://github.com/blasthacknet/samp-api/tree/6d4db99ab41f19d1a6a7c6cd48f5878bd1e14b62/include/sampapi/0.3.7-R1).
  They are not treated as live authority: the profile additionally checks the
  R1 PE timestamp/entry point, dialog target signature, GTA PE identity/code,
  and readable pointers before use.
- **R1 local nickname accessor correction (2026-08-04):** live inspection
  after a successful profile match found an assigned local ID and a valid
  packed local nickname at player-pool offset `0x0A`. Calling the candidate
  `samp.dll + 0x13CD0` target with the pool as `this` interpreted nickname
  bytes as a pointer and kept the complete local snapshot in `NotReady`. The
  snapshot now reuses the independently fingerprinted
  `CPlayerPool::GetName(pool, id)` target at `+0x13CE0`; its local-ID path
  explicitly addresses the packed string at `+0x0A`. No client address or
  nickname contents are recorded in runtime logs.
- **Direct R1 dialog signature:** the installed supported binaries identify
  GTA SA 1.0 US as image base `0x00400000`, image size `0x01177000`, and entry
  RVA `0x00424570`, and SA-MP R1 as timestamp `0x5542F47A` and entry RVA
  `0x0031DF13`. At `samp.dll + 0x6B9C0`, `CDialog::Show` begins
  `83 EC 10 53 56 57 8B 7C 24 20 33 DB 3B FB 8B F1`; this exact signature is
  the profile gate. The earlier `55 8B EC` check rejected this valid target
  and made direct helpers return `UnsupportedVersion`.
- **Direct R1 chat signature:** static analysis of the installed fingerprinted
  R1 DLL found `CChat::AddEntry` at `samp.dll + 0x64010` beginning
  `55 56 8B E9 57 8D BD 32 01 00 00 8D B5 2E 02 00`. Its debug-entry wrapper at
  `+0x645A0` calls that target with entry type `8`; the target uses x86
  `thiscall` (`ECX` is `CChat`) and five scalar/pointer arguments. The
  `CChat` singleton pointer at `+0x21A0E4` and internal capacities of 143 text
  bytes plus 27 prefix bytes are used only after the same R1 PE, GTA, code,
  and readable-pointer gates. The host copies and terminates its own strings
  on the game-thread call; no client pointer or packet/RPC emulation reaches
  the ABI.
- **Cached R1 chat display mode:** static analysis of the same fingerprinted
  R1 DLL found `CChat::GetMode` at `samp.dll + 0x5D7A0` beginning
  `8B 41 08 C3` (`mov eax, [ecx + 8]; ret`). The profile verifies all four
  bytes, acquires the already-verified `CChat` singleton at `+0x21A0E4`, and
  calls the accessor only from the incoming-packet game-thread pump. It
  accepts only the three R1 values `0` (off), `1` (no-shadow), and `2`
  (normal), then atomically caches the scalar. Plugins receive only the
  converted enum or `NotReady`; no client layout, pointer, synchronous native
  call, packet, or RPC crosses the ABI.
- **Cached R1 cursor and scoreboard state:** the pinned R1 C++ leads place
  `CGame::m_nCursorMode` at packed offset `0x55` and
  `CScoreboard::m_bIsEnabled` at `0x00`; the independent x86 fixture asserts
  both. The installed fingerprinted R1 DLL's `CGame::ProcessInputEnabling` at
  `samp.dll + 0x9BC10` begins
  `56 8B F1 8B 46 55 57 33 FF 3B C7 0F 85 07 01 00`, loading the cursor field.
  `CScoreboard::Close` at `+0x6A320` begins
  `56 8B F1 83 3E 00 74 3C 8B 46 34 85 C0 74 35 C6`, while `Enable` at
  `+0x6AD30` begins
  `56 8B F1 83 3E 00 75 43 8B 46 34 85 C0 74 3C C6`; both compare the
  scoreboard flag at offset zero. The profile checks these exact signatures,
  validates the R1 `CGame` singleton at `+0x21A10C` and `CScoreboard`
  singleton at `+0x21A0B4`, then copies only cursor values `0..=4` and
  scoreboard values `0/1` from the game-thread pump into atomic caches. The
  ABI returns converted scalars only; it cannot toggle UI, expose a pointer,
  or synchronously call the client.
- **Cached R1 dialog core and chat-input state:** the pinned R1 C++ leads place
  `CDialog::m_bIsActive` at packed offset `0x28`, `m_nType` at `0x2C`,
  `m_nId` at `0x30`, fixed `m_szCaption[65]` at `0x40`, and
  `m_bServerside` at `0x81`; `CInput::m_bEnabled` is at `0x14E0`. The
  independent x86 fixture asserts every touched offset. The installed
  fingerprinted R1 DLL stores the dialog core at `CDialog::Show + 0x48` as
  `89 7E 30 89 46 2C 89 8E 81 00 00 00 8D 56 40`; the profile verifies this
  exact field-store anchor before copying only the active dialog's ID, typed
  style, fixed bounded caption, and server-side flag. It never follows dynamic
  text, button, edit-box, or list pointers.
  The installed fingerprinted R1 DLL's `CDialog::Show` at `+0x6B9C0` begins
  `83 EC 10 53 56 57 8B 7C 24 20 33 DB 3B FB 8B F1 7D 17 39 5E 28 0F`,
  explicitly comparing the dialog flag. `CInput::Open` at `+0x657E0` begins
  `83 EC 10 56 8B F1 8B 86 E0 14 00 00 85 C0 0F 85`, and `Close` at
  `+0x658E0` begins `56 8B F1 8B 86 E0 14 00 00 85 C0 74 39 8B 4E 08`;
  both load the input flag. The profile verifies all three signatures, checks
  the dialog singleton at `+0x21A0B8` and input singleton at `+0x21A0E8`, and
  copies only canonical `0/1` values from the game-thread pump into atomic
  caches. The ABI returns flags only; it does not expose a client pointer,
  close a dialog, mutate input, or send packet/RPC traffic.
- **Cached R1 animation table:** static analysis of the installed fingerprinted
  R1 DLL found 1,812 fixed 36-byte `name:file` entries at
  `samp.dll + 0xF15B0`. The exact first padded entry is
  `AIRPORT:THRW_BARL_THRW`; every installed entry was bounded, nonempty,
  colon-separated, and unique during the static audit. The profile verifies
  that first complete 36-byte entry before copying and parsing the whole table
  only on the game-thread pump. It accepts only bounded one-colon entries,
  retains owned name/file bytes, and exposes a fixed-buffer ABI lookup by ID
  or byte pair. Neither client memory nor a table pointer reaches a plugin.
- **Direct R1 death-window signatures:** static analysis of the installed
  fingerprinted R1 DLL found `CDeathWindow::AddMessage` at `samp.dll + 0x66A10`
  as `E9 1B FF FF FF`, a thunk to `CDeathWindow::AddEntry` at `+0x66930`, which
  begins `8B D1 E8 49 F6 FF FF 8A 44 24 14 8B 4C 24 10 88`. Direct call sites
  load the `CDeathWindow` singleton at `+0x21A0EC` into ECX before five
  pointer/scalar arguments. The private helper passes copied NUL-terminated
  24-byte-bounded names and scalar colours/weapon only from the game-thread
  pump. It exposes neither native pointers nor packet/RPC emulation.
- **Cached R1 game state:** `CNetGame::GetGameState` at `samp.dll + 0x2E20`
  begins `8B 81 BD 03 00 00 C3` (`mov eax, [ecx + 0x3BD]; ret`) in the
  fingerprinted R1 target. The profile verifies that exact seven-byte signature
  before publishing. The host invokes it only from the incoming-packet game
  thread and copies its `i32` return into an atomic cache. The ABI deliberately
  exposes the result as an opaque scalar: no R1 enum naming, native pointer,
  or synchronous client call crosses the boundary.
- **Cached R1 server metadata:** the pinned R1 `CNetGame` lead uses packed
  `m_szHostAddress[257]` at `0x20`, `m_szHostname[257]` at `0x121`, and
  `m_nPort` at `0x225`. The independent x86 fixture asserts all three offsets
  and its `m_nGameState` at `0x3BD`, independently matching the fingerprinted
  `GetGameState` code signature above. The host copies at most 256 bytes from
  each NUL-terminated string and accepts only a nonempty address and nonzero
  `u16` port, on the incoming-packet game thread. The profile and readable
  pointer checks remain mandatory; only owned copied bytes and the scalar port
  reach the ABI.
- **Direct snapshot layout:** the independent C++ fixture asserts R1 packed
  on-foot (68-byte), in-car (63-byte), and local-player-prefix (92-byte)
  boundaries used to obtain sync position/velocity, special action, animation,
  and vehicle fields. It also asserts `CPed::m_pGamePed` at `0x2A4`, used to
  validate the ped before health/armour calls, and `CPlayerPool`'s local-player
  ID at `0x4`. The R1 source's `CPlayerPool::Find` matches pooled peds and is
  not a local-ID lookup; using it left the local snapshot at `0xFFFF` after
  connection. The direct profile copies name, colour, score, ping, and all
  snapshot fields into host-owned storage; no pointer reaches the ABI.
- **Direct snapshot assignment:** early validation runs showed that the
  player-pool ID alone is insufficient: it can be `0xFFFF` (`65535`) or an
  initial zero before the server's real assignment. R1 `INIT_GAME` (RPC 139)
  carries that authoritative ID, so after the original client handler accepts
  it the host privately decodes only its fixed prefix (never logs payload
  bytes) and records the scalar ID. That prefix is four bit flags, `f32`, one
  bit flag, `f32`, three bit flags, raw little-endian `i32` class count, then
  `u16` player ID. It publishes only pool snapshots that match that ID twice
  consecutively. This keeps server-assigned ID zero valid while rejecting the
  transition. A trial decoder omitted the final flag and class count, producing
  a wrong ID in live validation; its exact wire fixture now covers both fields.
  A trial gate on `CNetGame::GetState` was removed: its post-join state changes
  again during ordinary gameplay and would erase a valid assignment, leaving
  the cache permanently `NotReady`. The state accessor is now separately
  exposed as the nonblocking cached scalar above, never as a snapshot gate. The
  R1 player pool's verified pointers and the retained assignment match are the
  snapshot-readiness boundary; unavailable pools or peds clear that cache and
  report `NotReady`.
- **Direct helper live run (2026-08-03):** the fingerprinted GTA SA 1.0 US +
  SA-MP 0.3.7 R1 client enabled the profile, displayed the queued direct dialog,
  and published stable local-player ID `1`. The opt-in state check completed
  with `position_changed=true`, `health_changed=true`, `armour_changed=true`,
  and `vehicle_changed=true`. The validator's one incoming RPC 61 was its
  separate intentional RPC-emulation check; the direct dialog added no RPC 61
  observation and no outgoing RPC 61 or 62 appeared. The client then exited
  normally with the last host/validator results still passed; `gta_sa.exe` was
  absent afterwards, and no GTA/SA-MP Application Error, Windows Error
  Reporting event, or new dump file appeared in the following 15-minute check.

## Typed protocol evidence

R1 RPC and packet codecs are field-by-field serializers; they do not cast
callback memory to Rust structs. Their catalog and fixture coverage were
checked against the public [SAMP.Lua event catalog](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/events.lua),
[synchronization definitions](https://github.com/THE-FYP/SAMP.Lua/blob/c0f2de815425b20615f93816f36372d3a03110f2/samp/synchronization.lua),
and [SA-MP RPC list](https://github.com/Brunoo16/samp-packet-list/wiki/RPC-List).
This is not live compatibility evidence for non-R1 clients.

When native behavior changes, record the client build, the observed layout or
behavior, the supporting fixture/live result, and any remaining limitation.
