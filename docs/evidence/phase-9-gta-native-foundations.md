# Phase 9 GTA native foundation evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Layout oracle

`tests/fixtures/gta_sa_layout.cpp` is an independent C++ oracle compiled against
plugin-sdk commit `5d7c561152ca2b1dfa0321ca9d35399c2dc99f82` (plugin-sdk 1.005).
The checkout is external and is not vendored into this repository. Set
`PLUGIN_SDK_DIR` to that exact checkout. The root build then compiles the oracle
for `i686-pc-windows-msvc` and enables its Rust comparison test.

Verified command:

```text
$env:PLUGIN_SDK_DIR = "C:\Development\plugin-sdk-phase9"
cargo test gta_sa_profile_layout_matches_the_pinned_plugin_sdk_oracle
```

Result: one oracle comparison passed. The C++ compile-time assertions and Rust
comparison established:

| Fact | Value |
| --- | ---: |
| `sizeof(CVector2D)` | `0x08` |
| `sizeof(CVector)` | `0x0C` |
| `sizeof(CMatrix)` | `0x48` |
| `CMatrix::right` | `0x00` |
| `CMatrix::up` (Rust `forward`) | `0x10` |
| `CMatrix::at` (Rust `up`) | `0x20` |
| `CMatrix::pos` | `0x30` |
| `CMatrix::m_pAttachMatrix` | `0x40` |
| `CMatrix::m_bOwnsAttachedMatrix` | `0x44` |
| `sizeof(CPlaceable)` | `0x18` |
| `CPlaceable::m_placement` | `0x04` |
| `CPlaceable::m_matrix` | `0x14` |
| `sizeof(CEntity)` | `0x38` |
| `sizeof(CPed)` | `0x79C` |
| `CPed::m_fHealth` | `0x540` |
| `CPed::m_fArmour` | `0x548` |

These facts are Grade A under the repository taxonomy because an independent
C++ oracle compiled against the pinned reference SDK and matched the Rust
layout. `RawMatrix` remains host-internal. Public `gta_sa::Matrix` is an owned,
pointer-free semantic value and deliberately does not mirror native padding or
attachment pointers.

## Local-player symbol

Static disassembly of the exact executable at `0x56E210` produced:

```text
8B 44 24 04             mov eax,[esp+4]
85 C0                   test eax,eax
7D 07                   jge +7
0F B6 05 74 CD B7 00    movzx eax,byte ptr [0xB7CD74]
69 C0 90 01 00 00       imul eax,eax,0x190
8B 80 98 CD B7 00       mov eax,[eax+0xB7CD98]
C3                      ret
```

This proves a cdecl `FindPlayerPed(i32) -> CPed*` target at `0x56E210`. A
negative argument selects `CWorld::PlayerInFocus`; the function indexes
`CWorld::Players` and returns its first pointer field. The production profile
records this as Grade A. `gta-sa-native` validates the target and returned ped
range before reading or calling `CPools::GetPedRef`.

## Scope of this evidence

This report verifies the first read-only ped slice only: local-ped resolution,
pool handle conversion through the separately proven Phase 6 target, position,
health, and armour. It does not verify a teleport method, a direct position
write, a vtable slot, callback phase safety, or any stable GTA service ABI.
