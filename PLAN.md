
  # Reposition rak-samp as samp-client-sdk

  > Protocol/SDK boundary status: complete for issues #22 and #39. See the
  > [completion record](docs/evidence/protocol-sdk-boundary-completion.md).
  > This plan continues to track the broader product repositioning.

  ## Summary

  Deliver the approved proposal across all three phases as one breaking 0.1.0-alpha.4 cutover.

  - Public SDK package: samp-client-sdk in sdk/, imported as samp_client_sdk.
  - Host package: samp-client-sdk-host; deployed artifact: samp_client_sdk.asi; log: samp-client-sdk.log.
  - State and mutation facade targets SA-MP 0.3.7 R1 on GTA SA 1.0 US. Existing networking support remains for every
    currently recognized SA-MP build.

  - Remove live-validation/fingerprint policy while retaining bounded ABI checks, unit tests, exact wire vectors, and
    the C++ native-layout fixture.

  - Current baseline is green: formatting, 136 workspace tests, and Clippy with warnings denied.

  ## Implementation

  1. Preserve the current baseline
      - Commit the current working tree as-is, including the proposal, subject to existing ignore rules.
      - Implement subsequent phases as separate reviewable commits on feature/helpers.
      - Treat the currently modified validation code as intentionally superseded by Phase 1.

  2. Phase 1 — cleanup and rebrand
      - Delete tests/e2e/, both validation examples, REVIEW.md, and VALIDATION.md; remove their workspace members,
        cargo-make tasks, CI steps, release references, and documentation links.

      - Keep tests/fixtures/raknet_layout.cpp, its Rust layout tests, and build.rs C++ wiring.
      - Rename packages, crate imports, examples, exported artifacts, logging, release archives, repository metadata,
        documentation URLs, and host discovery constants. Rename the ABI namespace and export to SampClientSdkApiV1
        and SampClientSdk_GetApiV1.

      - Retain minimal SA-MP build detection only to select the correct networking offset table. Replace R1 feature
        fingerprints, executable/code-signature checks, and per-helper gates with the approved fixed offsets and
        ordinary null/range/bounds validation.

      - Rewrite README.md, CORE.md, ARCHITECTURE.md, AGENTS.md, and TODO.md. TODO.md must list all 207 pinned SF.lua
        globals with a facade/raw target and one of the three tiers; remove [~], live-evidence language, and permanent
        exclusions.

      - Keep surviving runtime behavior unchanged for the supported R1/GTA configuration apart from the intentional
        package, symbol, and compatibility break.

  3. Phase 2 — game-thread execution foundation
      - Install an inline CGame::Process hook at GTA SA 1.0 US address 0x53BEE0, capturing and restoring its
        trampoline alongside the existing owned hooks.

      - Each detour invocation calls the original exactly once, drains the commands accepted before that tick, then
        refreshes and atomically publishes the state generation. Commands submitted during draining wait for the next
        tick.

      - Remove cache/UI pumping from the incoming-packet detour so that detour handles networking only.
      - Replace the three UI queues with one bounded 256-entry GameCommand queue using fully owned, bounded payloads.
        Move every native mutation and explicit RakClient send/emulation initiated by plugin threads through this
        queue; callback-local packet/RPC replacement remains synchronous inside its detour.

      - Add host-owned command IDs and completion records. The C ABI exposes typed submission slots plus common poll,
        timed-wait, and release operations using fixed repr(C) result storage.

      - Publish one coherent generation per tick: refresh lightweight global state and pool directories every frame;
        refresh heavy text/entity records only for requested or active IDs, retaining the existing bounded/
        deduplicated request behavior and first-read NotReady.

      - Waiting is rejected from the game thread or any listener callback. Timeout preserves the receipt for retry;
        dropping a receipt detaches the waiter without cancelling execution; shutdown completes pending receipts with
        a shutdown error.

  4. Phase 3 — Rust facade and exhaustive mapping
      - Replace public HostApi usage with Samp::connect(timeout) and Samp::connect_to(module, timeout). Keep the raw
        ABI wrapper private or documentation-hidden.

      - Add lightweight facades returned by samp.net(), server(), local(), players(), textdraws(), labels(),
        objects(), pickups(), vehicles(), gangzones(), dialogs(), chat(), cursor(), scoreboard(), and anim(); expose
        game state and client version directly from Samp.

      - Add checked SA-MP ID newtypes and typed GTA handles. Constructors validate native ranges, raw values are
        explicitly obtainable, and handle-to-ID conversions return Option.

      - Move existing subscriptions, codecs, exact sends, emulation, and owned BitStream behind Net, retaining exact-
        bit and exactly-once dispatch guarantees.

      - Safe reads return owned cached data and never invoke native client code from plugin threads. Safe mutations
        return CommandReceipt<T>: try_take polls, wait(timeout) consumes on completion, and timeout errors retain the
        receipt.

      - Implement remaining UI, player, pool, entity, connection, command-registration, sync, and dialog-response
        functionality in bounded vertical slices. Bounded state-changing operations use queued safe methods; callback
        registrations use owned subscriptions with synchronized unload.

      - Add unsafe raw accessors for pointer/address/callback-table operations that cannot have a safe representation.
        Raw pointers remain explicitly unsafe, are valid only while the host/client is loaded, and never become Rust
        references across the ABI.

      - Complete all 207 mappings: copied values become safe reads, bounded native changes become queued mutations,
        IDs/handles use typed wrappers, RakNet operations use Net/BitStream, callbacks use subscriptions, and pointer/
        code-address operations use unsafe raw. No entry may remain pending or unclassified.

  ## Public Interface and ABI Rules

  - Introduce Samp, subsystem facade types, checked ID/handle newtypes, CommandReceipt<T>, and typed command errors.
  - Setters return Result<CommandReceipt<()>, Error>; create and conversion operations return receipts carrying their
    typed ID/handle result.

  - Rebuild the newly named ABI as SampClientSdkApiV1; the new module/export name makes the alpha compatibility break
    explicit, so the table may be reordered cohesively rather than preserving the old layout.

  - ABI payloads remain bounded and C-compatible. No Rust allocations, references, trait objects, or callback-local
    values cross the boundary; only the explicitly unsafe raw tier may expose native addresses.

  - Preserve subscription shutdown requirements: unregister and wait from a worker thread, never from DllMain, a
    callback, or the game tick.

  ## Verification and Finalization


  - Add command tests for FIFO execution, per-tick snapshot draining, queue capacity, native success/failure, polling,
    timeout/retry, detached receipts, deadlock rejection, and shutdown completion.

  - Add cache tests for coherent generations, connection-transition invalidation, lightweight per-tick refresh,
    bounded heavy requests, and absence of plugin-thread native reads.

  - Add mock-ABI facade tests for every subsystem, ID bounds, handle conversion, owned string limits, raw opt-in
    boundaries, and all mutation result shapes. Preserve existing exact packet/RPC vectors and layout tests.

  - Verify the 207-entry checklist has no duplicate, missing, provisional, or unclassified entry.
  - Run cargo fmt --all -- --check, cargo test --workspace --all-targets --locked, cargo clippy --workspace --all-
    targets --locked -- -D warnings, and cargo build --workspace --release --locked.

  - Update the proposal status to implemented and confirm release packaging contains samp_client_sdk.asi, renamed
    examples, README, license, symbols, and checksums without validation/E2E artifacts.

  - Create a recoverable backup branch, squash every feature/helpers commit since master—including the temporary
    baseline and all implementation commits—into one repositioning commit, verify the squashed tree is identical, and
    retain the backup until review. Existing master history and alpha tags are not rewritten.
