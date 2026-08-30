# Phase 10 GTA `CWorld` ground-query evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Verified query

Static disassembly of the exact executable at `0x569660` shows a cdecl
`CWorld::FindGroundZForCoord(float x, float y) -> float` entry. It reads the two
32-bit stack arguments, builds a vertical collision query, calls the internal
vertical-line routine at `0x5674E0`, and returns the resulting Z value through
x87 `ST(0)`. If the collision query reports no hit, it returns the fixed float
at `0x858BA4`.

This exact-binary observation is Grade A for the address, argument layout,
return convention, and read-only behavior. The production profile records only
this query. It does not promote `CWorld::Add`, `CWorld::Remove`, line-of-sight,
or arbitrary entity enumeration.

## Published contract

`GtaSaServiceV2` exposes callback-scoped and queued `find_ground_z` operations.
Both reject non-finite coordinates before native access and reject a non-finite
native result. Direct calls require the validated post-`CGame::Process` context;
queued calls retain global Host FIFO ordering and return a copied `f32` through
a typed receipt. No collision pointer or native entity escapes the Host.

## Live smoke still required

Use the Phase 10 checklist in `docs/native-layout-smoke.md`. Compare direct and
queued results at known road, interior, water-adjacent, and no-ground
coordinates. Record the returned fallback behavior and client liveness before
claiming broader `CWorld` coverage.
