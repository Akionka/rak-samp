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
