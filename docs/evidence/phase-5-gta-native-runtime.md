# Phase 5 GTA native runtime evidence

Status: complete.

## Ownership and identity

- `crates/gta-sa-native` owns the GTA SA 1.0 US game-process profile, hook,
  trampoline, game-thread identity, and tick orchestration.
- The profile accepts only image base `0x00400000` and full executable SHA-256
  `A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26`.
- The hash was recorded on 2026-08-30 from the pinned local GTA executable used
  by the existing R3-1 smoke fixture. The host verifies it before installing
  any native hook.
- SA-MP remains a host-internal `GameTickParticipant`; no plugin-facing GTA API
  is introduced.

## Runtime safety

- Tick order is mark thread, snapshot participant work, call the original once,
  then run the post-process participant pump.
- A regression test proves that a command submitted by the original call is
  deferred until the next frame.
- Shutdown disables the hook, waits off the game thread for active detours, and
  removes the hook before clearing the trampoline.
- If disable or removal fails, a strong runtime remains resident and the detour
  stays in original-only pass-through mode. A later attach cannot replace it.

## Automated validation

The following commands pass on the repository's Windows x86 target:

```text
cargo test -p gta-sa-native -p modkit-win32 -p samp-client-sdk-host --lib --locked
cargo clippy -p gta-sa-native -p modkit-win32 -p samp-client-sdk-host --all-targets --locked -- -D warnings
cargo make quality
cargo build --workspace --release --locked
git diff --check
```

## Live validation

On 2026-08-30, release host artifact SHA-256
`CE7B8EB83C61A24308313ECC5B7532AE168F0FBC1F99919CF7806BF7C394A58B` was loaded
by the pinned R3-1 client. The host accepted GTA executable SHA-256
`A559AA772FD136379155EFA71F00C47AAD34BBFEAE6196B0FE1047D0645CBD26` and logged:

- R3-1 profile recognition;
- enabled `CGame::Process` at `0x0053BEE0` from the GTA-owned runtime;
- first entry into `gta_sa_native::tick`'s game-process detour;
- ready RakClient packet and RPC hooks;
- valid incoming packet/RPC capture; and
- a captured and successfully completed game-thread command.

The client exited normally on user request with no new crash or dump artifact.
It connected to a compatible loopback server that was already owned by a
separate R1 matrix run; no full network-probe result is claimed here.
