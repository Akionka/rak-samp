# Phase 9 GTA teleport static evidence

Date: 2026-08-30
Target SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`
Image base: `0x00400000`

## Hoodlum position thunk

The file entry at `0x420B80` is `E9 AB 62 14 01`, a relative jump to
`0x01566E30` inside the executable `.HOODLUM` section. The target implements the
x86 thiscall `CPlaceable::SetPosn(float, float, float)`: it reads the matrix
pointer at `this + 0x14`, writes `x/y/z` to matrix offsets `0x30/0x34/0x38`, or
falls back to embedded placement offsets `0x04/0x08/0x0C`, and returns with
`ret 0x0C`.

The independent `CPlaceable::SetPosn(CVector const&)` implementation at
`0x4241C0` uses the same pointer and field offsets. These facts resolve the
previous apparent address contradiction. The direct setters are not used by
the safe teleport API because they do not maintain world-sector membership or
clear ped movement state.

## Verified virtual teleport

Exact executable vtable facts:

| Fact | Address/value |
| --- | ---: |
| `CEntity` vtable | `0x863928` |
| `CEntity` slot 14 | `0x403E80` (base no-op, `ret 0x10`) |
| `CPed` vtable | `0x86C358` |
| `CPlayerPed` vtable | `0x86D168` |
| `CPed`/`CPlayerPed` slot 14 | `0x5E4110` |

Disassembly at `0x5E4110` proves the virtual method:

- uses x86 thiscall with `ECX = this` and `ret 0x10`;
- accepts a 12-byte by-value vector and one four-byte argument slot;
- removes the ped through `CWorld::Remove` at `0x563280`;
- updates the attached matrix or embedded placement position;
- clears a ped flag and one owned reference;
- re-adds the ped through `CWorld::Add` at `0x563220`;
- zeros the movement vectors at `this + 0x44` and `this + 0x50`.

The production backend accepts only the exact `CPed` or `CPlayerPed` vptr,
resolves slot 14 through guarded memory, and requires the resolved target to be
exactly `0x5E4110`. The C++ layout oracle also calls a Rust thiscall capture
function through the plugin-sdk `CVector, bool` signature. This verifies the
Rust by-value vector and byte-sized bool declaration across the MSVC C++ ABI.

These method, vtable, slot, target, and ABI facts are Grade A static evidence.

## Live phase evidence

On 2026-08-30, the release host and a disposable x86 smoke plugin ran against
the pinned GTA executable and the isolated R1 loopback server at
`C:\Games\SAMP-R1-LOOPBACK-PROBE`. Process inspection confirmed that this was
the only active `samp-server.exe`; the stale R3 server was stopped before the
accepted run.

The post-`CGame::Process` callback:

1. copied the local ped snapshot;
2. synchronously teleported the ped by `+150.0` X, crossing world sectors;
3. synchronously copied and verified the destination;
4. restored the original position;
5. submitted queued teleport by `+175.0` X followed by a queued compound
   snapshot in the same global FIFO;
6. waited for both receipts off-thread, verified the queued destination, and
   queued and waited for restoration.

The host log recorded
`PHASE9_SECTOR_OK direct_150 queued_175 restore`. Both native commands completed
successfully, the client remained live until the controlled smoke shutdown,
and the disposable plugin was removed afterward. This promotes
`POST_GAME_PROCESS_ONLY` for the verified `CPed::Teleport` path to Grade A
static plus dated live evidence.
