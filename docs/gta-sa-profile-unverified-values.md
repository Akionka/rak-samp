# GTA SA profile unverified values

This register is authoritative for GTA SA native facts that do not yet meet
Grade A or B. Do not add these values to `GtaProfileSpec`, publish a Capability,
or expose a production service operation until the required evidence exists.

Verified facts already in `GtaProfileSpec` are documented by their attached
`NativeEvidence` and `docs/evidence/phase-9-gta-native-foundations.md`.

| Operation or value | Reference observation | Grade | Required evidence | Status |
| --- | --- | --- | --- | --- |
| Direct `CPlaceable::SetPosn` use | Exact bodies at `0x420B80`/`0x4241C0` | A static | Behavior proof covering world-sector membership, collision state, movement state, and restoration | Intentionally not exposed; use verified `CPed::Teleport` |
| Direct `CPlaceable`/`CMatrix` position write | Layout oracle proves offsets only | C | Native behavior proof covering RenderWare synchronization, collision state, and restoration | Unsupported |
| `CWorld` operations beyond `FindGroundZForCoord` | Only the ground-height query is in the production profile | U | Exact executable disassembly for each additional singleton/method plus behavior and phase evidence | Unsupported |
| `CStreaming` request/load/release | No production profile symbol or ABI exists | U | Exact symbols, calling conventions, model lifecycle, and game-thread behavior proof | Unsupported |
| `CTimer` converted durations, pause semantics, and startup readiness | Raw frame/game counters and clipped/non-clipped time steps are in the production profile | U | Live unit comparison, pause/menu behavior, first-frame readiness, and wrap behavior | Raw owned snapshot only |
| Camera snapshot/control | No GTA camera layout, singleton, or method exists in the profile | U | Fixture-backed copied fields plus exact method ABI and phase behavior | Unsupported |
| GTA vehicle creation | SA-MP vehicle RPC codecs do not prove a GTA native workflow | U | Verified model loading, constructor, pool registration, world insertion, failure cleanup, and live smoke | Unsupported |
| GTA entity destruction | No destructor/removal workflow is verified | U | Exact ownership sequence, stale-handle revalidation, world/pool cleanup, and live smoke | Unsupported |

## Resolved teleport target

The `0x420B80` file entry is a Hoodlum jump thunk to `0x01566E30`, not an
out-of-image target. Exact disassembly, vtable resolution, cross-language ABI
capture, and the dated live sector/restoration smoke are recorded in
[`phase-9-gta-teleport-static.md`](evidence/phase-9-gta-teleport-static.md).
The production API uses `CPed::Teleport` at vtable slot 14 rather than either
direct `CPlaceable` setter.

## Promotion rule

Promote one row only after its exact fact has reproducible evidence. A layout
oracle does not prove a method address or execution phase. A method disassembly
does not prove that invoking it after `CGame::Process` is safe. Record each
promotion in a dated evidence report and add the fact with `NativeEvidence` to
the data-only GTA profile.
