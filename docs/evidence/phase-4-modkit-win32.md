# Phase 4 `modkit-win32` evidence

Status: complete.

## Ownership

- `crates/modkit-win32` owns generic Windows x86 memory, PE/module, protected
  write, and MinHook primitives.
- SA-MP detours and all GTA/SA-MP addresses remain under
  `src/platform/win32`.
- Public plugin crates do not depend on `modkit-win32`.

## Safety evidence

- Range validation checks every `VirtualQuery` region and rejects guard,
  no-access, incompatible, overflowed, and uncommitted ranges.
- `VirtualAlloc`/`VirtualProtect` tests cover read-only, no-access, guarded, and
  mixed-protection cross-page ranges.
- Protected writes validate one committed region, support unaligned copied
  values, and report failure to restore the original protection.
- PE reads use checked address arithmetic and guarded copied reads.
- RakClient vtable discovery validates the object pointer, the complete slot
  range, and each copied entry before patching.
- `InlineHook::create` is unsafe because target/detour signature compatibility
  cannot be validated at runtime.

## Automated validation

The following commands pass on the repository's Windows x86 target:

```text
cargo test -p modkit-win32 -p samp-client-sdk-host --lib --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo make quality
cargo build --workspace --release --locked
git diff --check
```

## Live validation

On 2026-08-30, the release host artifact with SHA-256
`09DBDAB2D2902A15D0F9F85F176EA452DF8C014DBF6783657DF4C3066AA36CD5` was loaded
by SA-MP 0.3.7 R3-1 against the local loopback server. The host log confirmed:

- R3-1 profile recognition;
- enabled `CGame::Process`, `CDialog::Close`, RakClient constructor, and
  `RakClient::HandleRPCPacket` hooks;
- entry into the game-process detour;
- valid incoming packet metadata and incoming-RPC receiver capture;
- successful execution of the first queued game command; and
- normal user-requested process exit with no new crash artifact.

The legacy full network probe reached status `0x800600CF` before reporting
`NativeCallFailed` in its player-pool fixture because the restarted loopback
server did not start the expected NPC. This later fixture failure is outside
the Phase 4 memory and hook acceptance scope; no full network-probe pass is
claimed for this run.
