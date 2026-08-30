# Phase 10 GTA camera snapshot evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Verified snapshot facts

The pinned plugin-sdk fixture independently verifies this `CCamera` layout:

| Fact | Value |
| --- | ---: |
| `TheCamera` object | `0xB6F028` |
| `sizeof(CCamera)` | `0xD78` |
| `m_vecGameCamPos` | `+0x908` |
| `m_mCameraMatrix` | `+0x974` |

The exact executable entry at `0x50AE50` is
`lea eax, [ecx + 0x908]; ret`, which confirms the thiscall
`CCamera::GetGameCamPosition` result and field offset. Exact disassembly of
`CCamera::Process` at `0x52B730` shows active-camera processing and derived
state copies. The Host therefore reads only in a validated post-`CGame::Process`
scope.

The native `CMatrix` layout remains fixture-backed: right, forward, up, and
position vectors at offsets `0x00`, `0x10`, `0x20`, and `0x30`. Attached-matrix
pointers, flags, RenderWare camera pointers, target entities, padding, and mode
state are excluded from the public value.

## Published contract

`GtaSaServiceV2` exposes direct and queued owned `GtaCameraSnapshotV1` reads.
The safe `gta-sa` facade exposes `CameraSnapshot` with:

- `game_position: Vector3`;
- `transform: Matrix` containing copied basis and position vectors.

Every source range is guarded with `ReadableRegion`; every published float must
be finite. Direct reads require the callback-scoped post-process context. Queued
reads execute in the same Host FIFO and return through a typed receipt. No
native pointer escapes the Host.

## Controls remain unsupported

Exact static entries include `Restore` (`0x50B930`),
`RestoreWithJumpCut` (`0x50BAB0`), `TakeControl` (`0x50C7C0`),
`TakeControlNoEntity` (`0x50C8B0`), and `TakeControlAttachToEntity`
(`0x50C910`). Their bodies mutate target pointers, ownership/mode/switch fields,
fixed-mode vectors, transition state, and internal camera systems. Capturing the
published pose is not sufficient to restore those effects deterministically.
No control is exposed until mode transitions, target lifetimes, restoration,
and live liveness are verified.

## Automated live observation

On 2026-08-30, the worktree based on commit `6ef0c55` was built and deployed to
the disposable `C:\Games\GTASA-SDK-R3-LIVE-TEST` root. The client used the
pinned GTA executable and SA-MP 0.3.7 R3-1. Artifact SHA-256 values were:

- Host: `612F63734E24A38AB255E287D316D28DC774C289A825B6E297A2015CF6405C13`;
- camera probe: `41DFA26FF9E8F9AFA5C35588F056B1599E66B0AD79B7F422AC179244BE4007EE`;
- GTA executable: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`;
- `samp.dll`: `9C9B2CC31A4CED6967420B1880C096B5C4E7630E227AA379BE4019C21B6FDDC1`.

The loopback R3 server and NPC were started before `samp_debug.exe`. The Host
identified R3-1, installed the `CGame::Process` hook, entered the detour, and
completed the first queued game-command snapshot successfully. The probe paired
a queued timer frame with the direct tick callback and reported:

```text
STATUS=PASS frame=36 camera=game:4488A000,C4FE8000,42B40000;right:BF0F4FF9,BF542055,B1800000;forward:BF536B83,3F0ED5D0,BDA6FD2D;up:BD8A5EC0,3D3AF72D,3F7F25C9;position:4488A000,C4FE8000,42B40000
```

Every copied component was finite. Direct and queued `CameraSnapshot` values
were bitwise identical for frame 36. The client remained running until the
controlled test shutdown; the loopback client, NPC, and server then stopped.

## Live smoke still required

The automated observation proves same-frame direct/queued coherence. It does
not visually prove orientation semantics. Use the Phase 10 checklist in
`docs/native-layout-smoke.md` to compare both copied positions and all basis
vectors with the visible active camera through ordinary camera transitions.
