# Phase 10 GTA vehicle destruction static evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Reference candidates

The pinned plugin-sdk reference maps `CWorld::Remove` at `0x563280`,
`CVehicle::operator delete` at `0x6E2D90`, and
`DestroyVehicleAndDriverAndPassengers` at `0x6D2250`. It also exposes occupant,
reference, trailer, RenderWare, effect, audio, and model state that a complete
vehicle lifecycle must clean.

The exact vehicle pool lookup and reference getters at `0x54FFF0` and
`0x54FFC0` remain Grade A. They prove stale/empty handle rejection, not object
destruction.

## Pool lifetime limitation

The reference `CPool::Delete` only marks a slot empty and updates the first-free
cursor. It does not:

- remove the entity from `CWorld`;
- invoke a destructor;
- detach driver, passengers, tractor, or trailer links;
- clear engine references;
- release RenderWare, model, effect, fire, or audio resources;
- increment the slot generation at deletion time.

The generation advances on later slot reuse. Therefore pool deletion alone is
not a valid destruction workflow, and a positive lookup result is not ownership
proof.

## Publication decision

No destruction API is published. plugin-sdk omits the hidden vehicle vtable
slot-zero destructor, so its complete-versus-scalar deleting ABI and required
flags are unknown. No exact-binary-verified game routine was found that proves
the full occupant/reference cleanup, world removal, concrete subclass resource
destruction, and pool release sequence.

Public handles contain no provenance. The Host cannot distinguish a Host-owned
vehicle from a game- or SA-MP-owned vehicle. Creation is not published, so even
a nominal Host-owned deletion domain does not exist. Arbitrary handles and
repeated/stale handles must never reach a destructor. Existing lookup validation
can reject stale handles, but it cannot authorize deletion.

Promotion requires a Host-only ownership registry populated by a verified
creator, exact concrete destructor ABI, exact cleanup ordering, world and pool
ownership proof, passenger/reference/trailer cleanup, double-destroy rejection,
old-handle invalidation, generation-reuse tests, and live liveness evidence.
