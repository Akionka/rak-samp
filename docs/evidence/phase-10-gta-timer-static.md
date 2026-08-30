# Phase 10 GTA `CTimer` snapshot evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Verified globals

Static disassembly of the exact executable confirms the following writable
`CTimer` globals and update path:

| Value | Address | Exact-executable evidence |
| --- | ---: | --- |
| frame counter | `0xB7CB4C` | `CTimer::Update` at `0x561B10` increments it at `0x561C5D` |
| non-clipped time step | `0xB7CB58` | `CTimer::UpdateVariables` at `0x5618D0` stores it at `0x561914` |
| clipped time step | `0xB7CB5C` | `CTimer::UpdateVariables` stores the selected value at `0x5619AF` or `0x5619BF` |
| game time in milliseconds | `0xB7CB84` | `CTimer::UpdateVariables` increments it at `0x56193F` |

The address and update-path facts are Grade A. plugin-sdk declarations were used
only as a reference oracle; the exact executable disassembly is the production
evidence source.

## Published contract

`GtaSaServiceV2` exposes a copied `GtaTimerSnapshotV1` through direct
post-`CGame::Process` and queued reads. The safe `gta-sa` facade exposes the same
owned values as `TimerSnapshot`:

- `frame_counter: u32`;
- `game_time_ms: u32`;
- `time_step: f32`;
- `time_step_non_clipped: f32`.

The Host validates every source address with `ReadableRegion` and rejects
non-finite time-step values. Counter wrap behavior remains the native 32-bit
behavior. Time-step values are intentionally documented as engine-native units:
static evidence does not prove a seconds conversion, pause semantics, or
first-frame readiness. No writable timer pointer escapes the Host.

## Live smoke still required

Use the Phase 10 checklist in `docs/native-layout-smoke.md`. Compare successive
snapshots with a measured frame interval, then record native units, pause/menu
behavior, first-frame readiness, counter progression, and wrap assumptions
before publishing converted duration helpers.
