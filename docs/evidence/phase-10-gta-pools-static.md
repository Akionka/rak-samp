# Phase 10 GTA pools and vehicle snapshot evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Exact pool lookup symbols

`objdump -d -Mintel` against the exact executable verified these cdecl targets:

| Operation | Address | Native behavior |
| --- | ---: | --- |
| `CPools::GetPed` | `0x54FF90` | Splits the handle into slot index and generation byte, compares the byte-map entry, and returns the `0x7C4`-stride pool slot or null. |
| `CPools::GetVehicle` | `0x54FFF0` | Splits the handle into slot index and generation byte, compares the byte-map entry, and returns the `0xA18`-stride pool slot or null. |
| `CPools::GetObject` | `0x550050` | Splits the handle into slot index and generation byte, compares the byte-map entry, and returns the `0x19C`-stride pool slot or null. |

The disassembly also reads the exact pool roots at `0xB74490`, `0xB74494`, and
`0xB7449C`. The production API calls the verified getters instead of duplicating
the byte-map algorithm. A stale generation therefore returns null before any
snapshot field is read. These exact-binary observations are Grade A.

The previously verified `CPools::GetPedRef` and `CPools::GetVehicleRef` targets
remain unchanged and retain the Phase 6 evidence.

## Layout oracle

`tests/fixtures/gta_sa_layout.cpp` now checks the pinned plugin-sdk commit
`5d7c561152ca2b1dfa0321ca9d35399c2dc99f82` and establishes:

| Fact | Value |
| --- | ---: |
| `sizeof(CVehicle)` | `0x5A0` |
| `CVehicle::m_fHealth` | `0x4C0` |
| `sizeof(CObject)` | `0x17C` |
| `sizeof(CPool<CPed, CCopPed>)` | `0x14` |
| `CPool::m_pObjects` | `0x00` |
| `CPool::m_byteMap` | `0x04` |
| `CPool::m_nSize` | `0x08` |

Vehicle position reuses the existing Grade A `CPlaceable`/`CEntity` matrix and
embedded-placement facts. The public snapshot copies only position and health.
No native pointer or pool storage crosses the service ABI.
Before invoking a getter, the Host validates the pool root/header, positive
handle, decoded slot index against the live capacity, one byte-map entry, and
the exact slot range. It then requires the getter result to equal the validated
slot address. An arbitrary positive integer therefore cannot make the native
getter index beyond the pool allocation.


Verified command:

```text
$env:PLUGIN_SDK_DIR = "C:\Development\plugin-sdk-phase9"
cargo test -p samp-client-sdk-host --all-targets --locked
```

Result: 176 host tests passed, including the pinned C++ oracle and exact-version
GTA service discovery. The focused GTA/modkit suites passed 60 tests.

## Published contract

`GtaSaServiceV1` remains immutable. `GtaSaServiceV2` retains the V1 operations
and adds post-game-process pool existence reads plus direct and queued vehicle
snapshots. Both exact versions remain discoverable. The safe `gta-sa` facade
accepts only positive typed handles and maps a stale vehicle handle to
`Ok(None)` for snapshots or `false` for existence.

Queued reads use the shared 256-entry Host command sequence. A receipt observes
the pool state when its command executes. A mismatched typed result accessor
does not consume another GTA read result.

## Live smoke still required

Before Phase 10 is complete, run the Phase 10 smoke checklist against this exact
executable: validate one live and one stale ped, vehicle, and object handle;
compare direct and queued existence results; read a streamed vehicle snapshot;
then despawn it and confirm the same handle returns absent without a crash.
