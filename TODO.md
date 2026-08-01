# TODO

Use this file as the repository’s single source of truth for planned and active work. Update it in the same change that adds, changes, or completes a task; remove stale entries rather than leaving ambiguous work behind.

## Next Up

- [ ] Add bit-length-preserving atomic replacement and RakNet's Huffman string codec, then implement encoded-string helpers such as `onShowDialog`.
- [ ] Add the remaining complex and bit-packed MoonLoader-style RPC and sync-packet schemas.
- [ ] Add fixture tests for each typed event decoder, atomic rewrite wire layout, and appended ABI field; validation is intentionally deferred for now.
- [ ] Add an end-to-end host plus independently loaded plugin-ASI integration fixture.

## In Progress

- [ ] Manually validate attach, interception, rewrite, cancellation, send, emulation, detach, and shutdown on legal R1, R2, R3.1, R4.2, R5.1, and DL installations.

## Completed

- [x] Establish contributor, core-feature, and architecture documentation.
- [x] Implement the Rust library API, bit stream, event registry, SA-MP version mapping, and guarded Windows x86 hook backend.
- [x] Implement a process-wide `rak_rs.asi` host with a versioned plugin ABI and plugin-side host discovery.
- [x] Restrict the host crate to `cdylib` output so plugins cannot link its hook runtime as an `rlib`.
- [x] Add file-based host lifecycle logging through `log` and `simplelog`.
- [x] Match RakNet compressed RPC-length encoding and pass traffic through when no listener is registered.
- [x] Replace the unsafe partial RakClient vtable clone with guarded in-place slot patches.
- [x] Verify the packed x86 RakNet packet layout and field reads against an independently compiled C++ fixture.
- [x] Remove native hooks before clearing `ACTIVE_BACKEND` during teardown.
- [x] Add a Windows x86 fake RakClient vtable fixture for slot-local patching and restoration.
- [x] Add a Windows x86 MinHook lifecycle fixture for inline-hook install, original calls, disable, removal, and recreation.
- [x] Add initial MoonLoader-style typed RPC helpers for common incoming and outgoing events without adding another hook runtime.
- [x] Expand source-confirmed byte-aligned typed helpers for player, world, vehicle, class, menu, and textdraw RPCs.
- [x] Add a standalone sample plugin that waits for `rak_rs.asi`, handles a typed RPC, and exports a synchronized shutdown entry point.
- [x] Add append-only `unregister_and_wait` ABI coordination so an unload manager can quiesce callbacks before freeing plugin code.
- [x] Add a passive validation ASI and documented workflow for observing incoming packet/RPC callbacks during live GTA testing.
- [x] Keep host build tasks at the workspace root so deploy commands cannot leave stale ASI binaries behind.
- [x] Confirm on R1 that the guarded host survives F5 and an independently loaded plugin receives 2,613 incoming RPC callbacks without null events.
- [x] Validate the corrected packed R1 packet layout in game: valid metadata, three packet callbacks, 2,282 RPC callbacks, and no null events or rejection warnings.
- [x] Expand the validation ASI with named incoming/outgoing packet, timestamp-inner, and RPC ID histograms.
- [x] Expose incoming packet and RPC emulation through the append-only plugin ABI.
- [x] Add a controlled in-game loopback for ABI emulation, ordered rewrite, observation, and cancellation without sending test traffic to the server.
- [x] Confirm on R1 that packet and RPC emulation each cross the ABI once, are rewritten, and are cancelled by the validation loopback.

## Task Format

Write tasks as `- [ ] Imperative description` and keep each item small enough to complete in one focused change. For work that affects user-visible behavior, link the related issue or design note and update [CORE.md](CORE.md) and [ARCHITECTURE.md](ARCHITECTURE.md) as required by [AGENTS.md](AGENTS.md).
