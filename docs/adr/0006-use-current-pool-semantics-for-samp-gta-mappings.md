# Use current-pool semantics for SA-MP-to-GTA mappings

SA-MP-to-GTA forward and reverse conversions inspect the live SA-MP pool when
the game-thread operation executes, matching the normal SF/MoonLoader semantic
model. The safe Rust facade represents an absent mapping as `Ok(None)`; the C
Service ABI returns `MOD_NOT_FOUND` with a zeroed output. Off-thread conversions
observe state when their queued command executes. `PickupHandle` remains a
distinct pickup reference rather than an `ObjectHandle`. Reverse lookup returns
the first active matching slot in ascending SA-MP ID order if native state
contains duplicates. Every later native use independently revalidates its
handle; no conversion creates a persistent identity relation.
