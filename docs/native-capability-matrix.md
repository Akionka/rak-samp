# Native Capability Matrix

This is the living feature-by-profile record for the supported SA-MP Native
profiles. It is maintained separately from the immutable [Phase 0 baseline](baselines/phase-0-2026-08-26.md).

Last reviewed: 2026-08-26

The matrix distinguishes four independent facts:

- **Native profile** — the exact target binary identity selected by the Host.
- **Recognition** — whether the loaded module matches that identity.
- **Readiness** — whether the Host has completed lifecycle and hook setup for
  the selected profile. Recognition alone is not readiness.
- **Capability** — one public/native feature family and its current evidence.

An aggregate smoke status never upgrades a capability that its probe did not
exercise. A row marked `live-tested for named surface` is not a claim that every
operation in that family is supported.

## Evidence rules

The [Native Profile Unverified Values register](native-profile-unverified-values.md)
is authoritative for unresolved native facts. The [live smoke record](native-layout-smoke.md)
is authoritative for the exact dated probe surfaces that it names. When the two
records differ, this matrix keeps both facts visible and leaves the unresolved
register item open.

| Grade | Meaning | Production use |
| --- | --- | --- |
| A | Direct evidence from an independent layout fixture, exact-binary live run, or exact-binary disassembly/signature evidence. | May support the named capability surface. |
| B | Two genuinely independent corroborating sources, or one trusted external source plus independent corroboration. | May support the named capability surface; add a local fixture when practical. |
| C | Single-source, inherited, legacy, or weakly verified value. | Not sufficient for a new production claim. |
| U | Unknown or unresolved provenance. | Unsupported; fail closed. |

`A + U` means that the named smoke/fixture fact is Grade A while a different
fact in the same family remains unresolved in the register. It does not turn
the whole row into a Grade A claim. `C/U` means that implementation exists but
there is no sufficient independent evidence for a production claim.

## Profile recognition and readiness

Runtime recognition currently checks a non-zero module base and the PE
entry-point RVA. The SHA-256 values below identify validation artifacts; they
are not an additional runtime hash gate.

| Native profile | Exact identity used by recognition | Validation artifact | Recognition | Readiness record |
| --- | --- | --- | --- | --- |
| R1 | SA-MP 0.3.7 R1; PE entry-point RVA `0x31DF13` | `samp.dll`; SHA-256 `7E30F3C9CD99D5E2932410F486E8139AFFA2DAD19BD65AD9C328F6A4071943F7` | Implemented; exact live confirmation remains open | Not established by a completed live profile run. |
| R3-1 | SA-MP 0.3.7 R3-1; PE entry-point RVA `0x0CC4D0` | `samp.dll`; SHA-256 `9C9B2CC31A4CED6967420B1880C096B5C4E7630E227AA379BE4019C21B6FDDC1` | Recognized in the pinned live run | Ready during the 2026-08-25 full profile probe (`status=0x3FFFFFFF`, `failure=0`). |
| R5-1 | SA-MP 0.3.7 R5-1; PE entry-point RVA `0x0CBC90` | `samp.dll`; SHA-256 `B72B5DBE725F81864CA3F78BC7063BDA56CC05FC7188AF822FA7A754432553A2` | Recognized in the pinned live run | Ready during the 2026-08-15 full profile probe (`status=0x3FFFFFFF`, `failure=0`). |
| DL-R1 | SA-MP 0.3.DL-R1; PE entry-point RVA `0x0FDB60` | `samp.dll`; SHA-256 `BCCDB297464BD382625635BE25585DF07A8FA6668BC0015650708E3EB4FFCD4B` | Recognized in the pinned live run | Ready during the 2026-08-15 full profile probe (`status=0x3FFFFFFF`, `failure=0`). |

R2 and R4-2 remain network-address identities only. They do not receive a
Native direct profile and are outside this matrix.

## Capability matrix

The `Recognition` column is repeated in each row on purpose. It prevents a
capability entry from being read without its exact profile prerequisite.

### R1

| Native profile | Capability | Recognition | Readiness | Capability state | Evidence source | Grade | Limitation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R1 | Profile identity and PE entry point | R1 identity is implemented; live entry-point check is open | Not established by a completed live run | Recognized by current code; validation pending | `src/platform/win32/native_client/profiles/r1.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | The checklist still requires exact entry-point, attach, and fingerprint confirmation. |
| R1 | Host lifecycle and RakClient hook readiness | Recognized R1 required | Host may report `Ready` only after lifecycle setup | Implemented; R1-specific live readiness unverified | `sdk/src/host_api/status.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | No completed R1 attach, hook, or restoration record exists. |
| R1 | Network codec and exact-bit packet/RPC paths | Recognized R1 required | Ready state required | Implemented; live native path unverified | `src/client.rs`; `src/platform/win32/native_bitstream.rs`; packet/RPC tests; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | Tests cover code behavior; they do not replace an R1 client run. |
| R1 | Outbound delivery and incoming original-handler continuation | Recognized R1 required | Ready state and receiver required | Implemented; unverified | [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | U | No R1 loopback delivery or original-handler observation is recorded. |
| R1 | Disconnect, reconnect, and hook/vtable restoration | Recognized R1 required | Ready state required | Implemented; unverified | `src/platform/win32/hooks.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | U | Reconnect, unload, stale-pointer, and restoration checks remain open. |
| R1 | Game state and server metadata caches | Recognized R1 required | Game-thread cache must be published | Implemented; evidence pending | `src/platform/win32/reads.rs`; `src/platform/win32/native_client/connection.rs`; evidence register | C/U | R1 non-layout values remain unverified. |
| R1 | Local-player snapshot and local-player mutations | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/native_client/players.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | No R1 live helper exercise is recorded. |
| R1 | Player-pool scalars and remote-player records | Recognized R1 required | Game-thread cache must be published | Implemented; evidence pending | `src/platform/win32/native_client/players.rs`; `refresh.rs`; evidence register | C/U | Pool and direct-call facts remain unverified. |
| R1 | Local/remote sync snapshots and force-send operations | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `sdk/src/facade/local_player.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | No R1 sync or force-send live result is recorded. |
| R1 | Entity pools and existence reads | Recognized R1 required | Game-thread cache must be published | Implemented; evidence pending | `src/platform/win32/native_client/pools.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | Object, pickup, vehicle, label, textdraw, and gangzone helpers need exact R1 evidence. |
| R1 | GTA handle conversion | Recognized R1 required | Ready state and valid pool state required | Implemented; unverified | `src/platform/win32/native_client/handles.rs`; evidence register | C/U | No completed R1 live handle round trip is recorded. |
| R1 | Chat display, chat history, and death window | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/chat_entries.rs`; `src/platform/win32/native_client/ui.rs`; evidence register | C/U | No R1 live UI/helper exercise is recorded. |
| R1 | Chat input and command registry | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/commands.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | Input, command, and DXUT values remain unverified. |
| R1 | Dialog core, snapshots, controls, response hook, and mutations | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/native_client/ui.rs`; [R1 smoke checklist](native-layout-smoke.md#sa-mp-037-r1) | C/U | No R1 dialog interaction or response-hook result is recorded. |
| R1 | Cursor and scoreboard | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/native_client/ui.rs`; evidence register | C/U | No R1 live flag or mutation result is recorded. |
| R1 | 3D text labels | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/native_client/text_labels.rs`; evidence register | C/U | Label RVAs and calls remain unverified. |
| R1 | Textdraws | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/native_client/textdraws.rs`; evidence register | C/U | Textdraw RVAs and ABI remain unverified. |
| R1 | Gangzones | Recognized R1 required | Game-thread cache must be published | Implemented; evidence pending | `src/platform/win32/gangzones.rs`; evidence register | C/U | No R1 live gangzone result is recorded. |
| R1 | Animation catalog and send rates | Recognized R1 required | Game-thread cache or command pump required | Implemented; evidence pending | `src/platform/win32/native_client/players.rs`; evidence register | C/U | Animation and send-rate native facts remain unverified. |
| R1 | Unsafe raw native addresses | Recognized R1 required | No readiness claim makes an unsafe address safe | NotReady / no production claim | `sdk/src/raw.rs`; evidence register | U | Raw addresses are explicitly outside a verified capability claim. |
| R1 | Optional SAMPFUNCS console interop | Recognized R1 required | Depends on an external SAMPFUNCS export | Conditional; not a Native profile capability | `sdk/src/facade/sampfuncs.rs`; handoff evidence rules | C/U | External availability and ABI are not verified here. |

### R3-1

| Native profile | Capability | Recognition | Readiness | Capability state | Evidence source | Grade | Limitation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R3-1 | Profile identity and PE entry point | R3-1 identity matched | Ready in the pinned live run | Recognized | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25) | A | Runtime uses the entry point; the hash identifies the tested artifact. |
| R3-1 | Host lifecycle and RakClient hook readiness | R3-1 identity matched | Ready observed in the full probe | Ready for the named run | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25); `sdk/src/host_api/status.rs` | A | This records the run, not perpetual readiness for every binary with the same label. |
| R3-1 | Network codec and exact-bit packet/RPC paths | R3-1 identity matched | Ready and receiver captured | Live-tested for named network surface | [R3 network smoke](native-layout-smoke.md#r3-1-network-smoke-observation-2026-08-12); [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25) | A | Network smoke does not by itself prove every native helper. |
| R3-1 | Outbound delivery and incoming original-handler continuation | R3-1 identity matched | Ready in loopback run | Live-tested for one outbound RPC and matching continuation | [R3 loopback delivery observation](native-layout-smoke.md#r3-1-loopback-delivery-observation-2026-08-12) | A | The result is one fixed loopback message, not all RPCs. |
| R3-1 | Disconnect, reconnect, and hook/vtable restoration | R3-1 identity matched | Reconnect passed in full probe | Partial: reconnect and post-reconnect paths tested | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25); [R3 checklist](native-layout-smoke.md#sa-mp-037-r3-1) | A + U | Broader helper coverage and unload/restoration checklist items remain open. |
| R3-1 | Game state and server metadata caches | R3-1 identity matched | Cache published on Game thread | Live-tested for game/server scalar cache | [R3 CNetGame scalar-cache observation](native-layout-smoke.md#r3-1-cnetgame-scalar-cache-observation-2026-08-12); [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25) | A + U | The register still lists other R3 non-layout values as unverified. |
| R3-1 | Local-player snapshot and local-player mutations | R3-1 identity matched | Cache/commands ready in full probe | Live-tested for named player surface | [R3 local-player cache observation](native-layout-smoke.md#r3-1-local-player-cache-observation-2026-08-12); [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25) | A + U | Do not infer every local-player operation from the aggregate status. |
| R3-1 | Player-pool scalars and remote-player records | R3-1 identity matched | Pool cache ready in full probe | Live-tested for named pool/entity surface | [R3 player-pool scalar observation](native-layout-smoke.md#r3-1-player-pool-scalar-observation-2026-08-12); [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25) | A + U | The register still requires per-value reconciliation for broader records. |
| R3-1 | Local/remote sync snapshots and force-send operations | R3-1 identity matched | Sync paths ready in full probe | Live-tested for named sync surface | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25) | A + U | Only the probe's sync records and sends are covered. |
| R3-1 | Entity pools and existence reads | R3-1 identity matched | Entity paths ready in full probe | Live-tested for named entity surface | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25); `src/platform/win32/profile_layout_tests.rs` | A + U | Fixture coverage and the smoke surface do not close every native value in the register. |
| R3-1 | GTA handle conversion | R3-1 identity matched | Handle path exercised in full probe | Live-tested for named entity-handle round trips; register unresolved | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25); evidence register | A + U | Keep the register item open until every consumed ABI/value is reconciled. |
| R3-1 | Chat display, chat history, and death window | R3-1 identity matched | Named display cache ready | Partial: chat display tested; history/death helpers unresolved | [R3 chat-display observation](native-layout-smoke.md#r3-1-chat-display-cache-observation-2026-08-12); evidence register | A + U | Do not extend chat-display evidence to chat history or death-window calls. |
| R3-1 | Chat input and command registry | R3-1 identity matched | Input cache ready in interactive probe | Partial: active/text/lookup tested; registration unresolved | [R3 chat-input observation](native-layout-smoke.md#r3-1-chat-input-cache-observation-2026-08-12); evidence register | A + U | The named probe did not prove every command mutation. |
| R3-1 | Dialog core, snapshots, controls, response hook, and mutations | R3-1 identity matched | Active flag ready in probe | Partial: active flag tested; details and mutations unresolved | [R3 dialog-active observation](native-layout-smoke.md#r3-1-dialog-active-cache-observation-2026-08-12); evidence register | A + U | Do not infer controls, response hooks, or writes from `is_active()`. |
| R3-1 | Cursor and scoreboard | R3-1 identity matched | Flag caches ready in probe | Live-tested for cursor/scoreboard flags | [R3 complete enabled-surface observation](native-layout-smoke.md#r3-1-complete-enabled-surface-observation-2026-08-12) | A + U | Writes and other UI helpers remain unresolved. |
| R3-1 | 3D text labels | R3-1 identity matched | Path exercised in full probe | Live-tested for named label surface | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25); evidence register | A + U | Per-RVA/ABI register entries remain unresolved. |
| R3-1 | Textdraws | R3-1 identity matched | Path exercised in full probe | Live-tested for named textdraw surface | [R3 full profile observation](native-layout-smoke.md#r3-1-full-profile-pass-2026-08-25); evidence register | A + U | Do not infer untested textdraw operations. |
| R3-1 | Gangzones | R3-1 identity matched | Fixture path available; live capability not established | Fixture-backed only; operation unresolved | `src/platform/win32/profile_layout_tests.rs`; evidence register | A + U | No R3 live gangzone operation is named in the smoke record. |
| R3-1 | Animation catalog and send rates | R3-1 identity matched | Not established for this family | Implemented; unverified | `src/platform/win32/native_client/players.rs`; evidence register | C/U | The R3 full probe summary does not claim the animation catalog/send-rate family. |
| R3-1 | Unsafe raw native addresses | R3-1 identity matched | No readiness claim makes an unsafe address safe | NotReady / no production claim | `sdk/src/raw.rs`; evidence register | U | Raw-address use remains outside the verified public surface. |
| R3-1 | Optional SAMPFUNCS console interop | R3-1 identity matched | Depends on an external SAMPFUNCS export | Conditional; not a Native profile capability | `sdk/src/facade/sampfuncs.rs`; handoff evidence rules | C/U | External availability and ABI are not verified here. |

### R5-1

| Native profile | Capability | Recognition | Readiness | Capability state | Evidence source | Grade | Limitation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R5-1 | Profile identity and PE entry point | R5-1 identity matched | Ready in the pinned live run | Recognized | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15) | A | Runtime uses the entry point; the hash identifies the tested artifact. |
| R5-1 | Host lifecycle and RakClient hook readiness | R5-1 identity matched | Ready observed in the full probe | Ready for the named run | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); `sdk/src/host_api/status.rs` | A | This records the run, not perpetual readiness for every binary with the same label. |
| R5-1 | Network codec and exact-bit packet/RPC paths | R5-1 identity matched | Ready and receiver captured | Live-tested for named network surface | [R5 network smoke](native-layout-smoke.md#r5-1-network-smoke-observation-2026-08-12); [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15) | A | Network smoke does not by itself prove every native helper. |
| R5-1 | Outbound delivery and incoming original-handler continuation | R5-1 identity matched | Ready in loopback run | Live-tested for one outbound RPC and matching continuation | [R5 loopback delivery observation](native-layout-smoke.md#r5-1-loopback-delivery-observation-2026-08-12); [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15) | A | The loopback message is a named probe, not all RPCs. |
| R5-1 | Disconnect, reconnect, and hook/vtable restoration | R5-1 identity matched | Reconnect and owned-slot teardown covered | Live-tested for named reconnect/lifecycle surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); [R5 checklist](native-layout-smoke.md#sa-mp-037-r5-1) | A | The ASI loader exits with GTA; unload evidence comes from lifecycle tests. |
| R5-1 | Game state and server metadata caches | R5-1 identity matched | Caches ready in full probe | Live-tested for named cache surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); `src/platform/win32/profile_layout_tests.rs` | A + U | Other R5 native values remain listed in the evidence register. |
| R5-1 | Local-player snapshot and local-player mutations | R5-1 identity matched | Local path ready in full probe | Live-tested for named player surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Do not infer every local-player operation from the aggregate status. |
| R5-1 | Player-pool scalars and remote-player records | R5-1 identity matched | Pool path ready in full probe | Live-tested for named pool/entity surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); `src/platform/win32/profile_layout_tests.rs` | A + U | The register still requires per-value reconciliation. |
| R5-1 | Local/remote sync snapshots and force-send operations | R5-1 identity matched | Sync paths ready in full probe | Live-tested for named sync surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15) | A + U | Only the probe's sync records and sends are covered. |
| R5-1 | Entity pools and existence reads | R5-1 identity matched | Entity paths ready in full probe | Live-tested for named entity surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); `src/platform/win32/profile_layout_tests.rs` | A + U | Fixture and smoke evidence do not close every register item. |
| R5-1 | GTA handle conversion | R5-1 identity matched | Handle round trip observed in full probe | Conflicting records: live-tested surface, register still says unavailable | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); [evidence register](native-profile-unverified-values.md) | A + U | Do not advertise this capability as fully verified until the conflict is reconciled. |
| R5-1 | Chat display, chat history, and death window | R5-1 identity matched | Named UI path ready in full probe | Partial: named UI surface tested; history/death remainder unresolved | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Aggregate UI coverage does not prove every chat/death helper. |
| R5-1 | Chat input and command registry | R5-1 identity matched | Input and command paths ready in full probe | Live-tested for named input/command surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Per-value ABI and untested mutations remain unresolved. |
| R5-1 | Dialog core, snapshots, controls, response hook, and mutations | R5-1 identity matched | Dialog path ready in full probe | Live-tested for named dialog surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Only operations exercised by the probe are covered. |
| R5-1 | Cursor and scoreboard | R5-1 identity matched | UI paths ready in full probe | Live-tested for named UI surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Other UI writes remain subject to individual evidence. |
| R5-1 | 3D text labels | R5-1 identity matched | Label path ready in full probe | Live-tested for named label lifecycle | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | The register still lists non-layout RVAs and ABIs as unresolved. |
| R5-1 | Textdraws | R5-1 identity matched | Textdraw path ready in full probe | Live-tested for named textdraw lifecycle | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | The corrected setter (`0xB2F60`) is evidence for that run only; untested calls remain open. |
| R5-1 | Gangzones | R5-1 identity matched | Fixture path available; live capability not established | Fixture-backed only; operation unresolved | `src/platform/win32/profile_layout_tests.rs`; evidence register | A + U | No R5 live gangzone operation is named in the smoke record. |
| R5-1 | Animation catalog and send rates | R5-1 identity matched | Animation path ready in full probe | Live-tested for named animation/send-rate surface | [R5 full direct-profile observation](native-layout-smoke.md#r5-1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Per-value reconciliation remains open. |
| R5-1 | Unsafe raw native addresses | R5-1 identity matched | No readiness claim makes an unsafe address safe | NotReady / no production claim | `sdk/src/raw.rs`; evidence register | U | Raw-address use remains outside the verified public surface. |
| R5-1 | Optional SAMPFUNCS console interop | R5-1 identity matched | Depends on an external SAMPFUNCS export | Conditional; not a Native profile capability | `sdk/src/facade/sampfuncs.rs`; handoff evidence rules | C/U | External availability and ABI are not verified here. |

### DL-R1

| Native profile | Capability | Recognition | Readiness | Capability state | Evidence source | Grade | Limitation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| DL-R1 | Profile identity and PE entry point | DL-R1 identity matched | Ready in the pinned live run | Recognized | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15) | A | Runtime uses the entry point; the hash identifies the tested artifact. |
| DL-R1 | Host lifecycle and RakClient hook readiness | DL-R1 identity matched | Ready observed in the full probe | Ready for the named run | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); `sdk/src/host_api/status.rs` | A | This records the run, not perpetual readiness for every binary with the same label. |
| DL-R1 | Network codec and exact-bit packet/RPC paths | DL-R1 identity matched | Ready and receiver captured | Live-tested for named network surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); [DL checklist](native-layout-smoke.md#sa-mp-03dl-r1) | A | Network smoke does not by itself prove every native helper. |
| DL-R1 | Outbound delivery and incoming original-handler continuation | DL-R1 identity matched | Ready in full probe | Live-tested for named outbound/incoming surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15) | A | The result covers the recorded probe, not all RPCs. |
| DL-R1 | Disconnect, reconnect, and hook/vtable restoration | DL-R1 identity matched | Reconnect and owned-slot teardown covered | Live-tested for named reconnect/lifecycle surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); [DL checklist](native-layout-smoke.md#sa-mp-03dl-r1) | A | The ASI loader exits with GTA; unload evidence comes from lifecycle tests. |
| DL-R1 | Game state and server metadata caches | DL-R1 identity matched | Caches ready in full probe | Live-tested for named cache surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); `src/platform/win32/profile_layout_tests.rs` | A + U | Other DL native values remain listed in the evidence register. |
| DL-R1 | Local-player snapshot and local-player mutations | DL-R1 identity matched | Local path ready in full probe | Live-tested for named player surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Do not infer every local-player operation from the aggregate status. |
| DL-R1 | Player-pool scalars and remote-player records | DL-R1 identity matched | Pool path ready in full probe | Live-tested for named pool/entity surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); `src/platform/win32/profile_layout_tests.rs` | A + U | The register still requires per-value reconciliation. |
| DL-R1 | Local/remote sync snapshots and force-send operations | DL-R1 identity matched | Sync paths ready in full probe | Live-tested for named sync surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15) | A + U | Only the probe's sync records and sends are covered. |
| DL-R1 | Entity pools and existence reads | DL-R1 identity matched | Entity paths ready in full probe | Live-tested for named entity surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); `src/platform/win32/profile_layout_tests.rs` | A + U | DL uses a distinct 2100-entry object-pool limit; other register items remain open. |
| DL-R1 | GTA handle conversion | DL-R1 identity matched | Handle round trip observed in full probe | Conflicting records: live-tested surface, register still says unavailable | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); [evidence register](native-profile-unverified-values.md) | A + U | Do not advertise this capability as fully verified until the conflict is reconciled. |
| DL-R1 | Chat display, chat history, and death window | DL-R1 identity matched | Named UI path ready in full probe | Partial: named UI surface tested; history/death remainder unresolved | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Aggregate UI coverage does not prove every chat/death helper. |
| DL-R1 | Chat input and command registry | DL-R1 identity matched | Input and command paths ready in full probe | Live-tested for named input/command surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Per-value ABI and untested mutations remain unresolved. |
| DL-R1 | Dialog core, snapshots, controls, response hook, and mutations | DL-R1 identity matched | Dialog path ready in full probe | Live-tested for named dialog surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Only operations exercised by the probe are covered. |
| DL-R1 | Cursor and scoreboard | DL-R1 identity matched | UI paths ready in full probe | Live-tested for named UI surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Other UI writes remain subject to individual evidence. |
| DL-R1 | 3D text labels | DL-R1 identity matched | Label path ready in full probe | Live-tested for named label lifecycle | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | The register still lists non-layout RVAs and ABIs as unresolved. |
| DL-R1 | Textdraws | DL-R1 identity matched | Textdraw path ready in full probe | Live-tested for named textdraw lifecycle | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | The corrected DL values are evidence for the named run; untested calls remain open. |
| DL-R1 | Gangzones | DL-R1 identity matched | Fixture path available; live capability not established | Fixture-backed only; operation unresolved | `src/platform/win32/profile_layout_tests.rs`; evidence register | A + U | No DL live gangzone operation is named in the smoke record. |
| DL-R1 | Animation catalog and send rates | DL-R1 identity matched | Animation path ready in full probe | Live-tested for named animation/send-rate surface | [DL full direct-profile observation](native-layout-smoke.md#dl-r1-complete-direct-profile-observation-2026-08-15); evidence register | A + U | Per-value reconciliation remains open. |
| DL-R1 | Unsafe raw native addresses | DL-R1 identity matched | No readiness claim makes an unsafe address safe | NotReady / no production claim | `sdk/src/raw.rs`; evidence register | U | The DL probe explicitly excludes public unsafe/raw addresses. |
| DL-R1 | Optional SAMPFUNCS console interop | DL-R1 identity matched | Depends on an external SAMPFUNCS export | Conditional; not a Native profile capability | `sdk/src/facade/sampfuncs.rs`; handoff evidence rules | C/U | External availability and ABI are not verified here. |

## Maintenance rules

1. Add a row or narrow an existing row when a capability is split into a new
   independently exercised operation.
2. Record the exact profile, binary identity, date, evidence source, and grade.
3. Keep `Unsupported`, `NotReady`, and `Unverified` entries explicit. Do not
   replace them with a blank cell or a whole-profile status.
4. A smoke result upgrades only the capability and exact operation named by the
   smoke. It does not upgrade inherited profile values.
5. Reconcile the [evidence register](native-profile-unverified-values.md)
   before changing an `A + U` or conflicting row to an unqualified supported
   state.

This document records capability evidence only. It does not change Native
profiles, runtime behavior, support gates, or the immutable Phase 0 baseline.
