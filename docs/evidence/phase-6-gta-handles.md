# Phase 6 GTA handle evidence

Status: complete.

## Ownership and safety

- `crates/gta-sa` owns positive, non-null GTA object, pickup, vehicle, and ped
  handle values. The safe wrappers reject raw values less than or equal to zero.
- `crates/gta-sa-native` owns the verified GTA SA 1.0 US
  `CPools::GetPedRef` and `CPools::GetVehicleRef` targets.
- SA-MP IDs remain in the SA-MP facade. Forward and reverse mappings return
  copied scalar values; no native pointer crosses the SDK boundary.
- Player and vehicle conversions obtain their targets from the GTA profile that
  passed exact executable hash and image-base selection during attach.

## Regression coverage

- Cross-profile typed IDs are accepted up to the public maximum. Each mapping
  operation rejects IDs outside the active client profile before cache access or
  request queueing.
- Connection invalidation clears both mapping directions. The next forward and
  reverse reads return `NotReady` and queue fresh resolution instead of serving
  stale values.
- Object, pickup, vehicle, and ped handle types reject zero and negative values.
  Pickup handles remain nominally distinct from object handles.
- Mock ABI tests cover the public forward and reverse mapping paths.

## Automated validation

The following commands pass on the repository's Windows x86 target:

```text
cargo test --workspace
cargo make quality
cargo build --workspace --release --locked
git diff --check
```

The release host artifact SHA-256 is
`A1EAC0C8A22D0336283B0653B634F88F7DE4930D0DEC3C67258D7D5C3B0C2E07`.
The quality gate includes formatting, all-target checks and tests, Clippy with
warnings denied, public documentation, release hygiene, and package verification
for `gta-sa`, `samp-protocol`, and `samp-client-sdk`.

## Live validation

On 2026-08-30, the release host and R1 network probe were loaded by the pinned
SA-MP 0.3.7 R1 client against the compatible loopback server. The server created
object `1`, vehicle `1791`, pickup `0`, and gangzone `0` for local player `0`.

The probe published status `0x800600CF`. This includes
`STATUS_ENTITY_HANDLES` (`1 << 17`) and `STATUS_FORCE_SYNC_RECEIPTS` (`1 << 18`).
The entity bit is published only after object, vehicle, pickup, and ped forward
and reverse mappings all match their source SA-MP IDs. The later failure bit is
outside Phase 6: this manual run did not start the expected NPC, so the following
player-pool check observed equal including/excluding-NPC counts and reported
`NativeCallFailed`. No full-probe success is claimed.

The client remained responsive and exited normally on user request. No new game
directory dump and no Windows Application Error for `gta_sa.exe` were recorded.
