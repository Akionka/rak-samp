# Phase 10 GTA vehicle creation static evidence

Date: 2026-08-30
Target: GTA San Andreas 1.0 US, Windows x86
Executable SHA-256: `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`

## Reference workflow

The pinned plugin-sdk reference maps these candidate operations:

| Operation | Reference address |
| --- | ---: |
| `CAutomobile` constructor | `0x6B0A90` |
| `CVehicle::operator new` | `0x6E2D50` |
| `CVehicle::operator delete` | `0x6E2D90` |
| `CWorld::Add` | `0x563220` |
| `CWorld::Remove` | `0x563280` |
| `CPlaceable::SetPosn` | `0x420B80` |
| `CPools::GetVehicleRef` | `0x54FFC0` |
| `CPools::GetVehicle` | `0x54FFF0` |

Only the pool getters, pool root, and `0xA18` slot stride are already promoted
to exact production evidence. The remaining addresses are reference candidates;
they do not prove lifecycle behavior.

A native creator would need to validate a nonnegative model ID, verify its
concrete vehicle type and loaded assets, reserve a vehicle-pool slot,
placement-construct the matching subclass, position it, insert it into
`CWorld`, acquire a positive handle, and verify that the handle resolves back to
the same pointer.

## Publication decision

No creation API is published. `CPool::New` reserves raw storage but does not
construct an object; `CPool::Delete` frees a slot but does not run a destructor.
Vehicle model types require different concrete subclasses. The reference
`CAutomobile` constructor has no status result and may establish resource
ownership before returning. `CWorld::Add` is also void, so a caller cannot infer
success from its return value.

Safe rollback would require exact knowledge of partial-construction behavior,
subclass destructor/resource cleanup, world insertion state, and pool ownership.
The required reverse sequence is world removal, exact subclass cleanup, then
pool deletion. None of those transitions is sufficiently verified. Streaming
asset ownership is separately blocked, but creation remains unsafe even if a
model is already loaded.

Promotion requires exact constructor/destructor disassembly, model-type dispatch
and bounds, loaded-asset readiness, pool registration timing, `CWorld` side
effects, handle acquisition timing, failure injection, and live rollback proof.
