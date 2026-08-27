# ABI and runtime safety

- The native bridge supports verified SA-MP 0.3.7 R1, R3-1, R5-1, and DL-R1
  profiles on GTA SA 1.0 US. Select the exact recognized build before using
  its approved fixed offsets.
- Start bootstrap and native initialization on a worker outside `DllMain`;
  `DllMain` must return without doing that work.
- Keep the plugin ABI C-compatible and versioned. During ALPHA, the contract
  may intentionally break without incrementing the ABI version. From BETA
  onward, increment the ABI version for every breaking change.
- ABI values use copied data, fixed storage, explicit capacities, and function
  pointers. Do not pass Rust references, trait objects, allocator-owned values,
  or borrowed native objects across the DLL boundary.
- Safe APIs return owned snapshots. Keep the underlying ABI wrapper internal
  to the SDK.
- Resolve public SDK access through `Samp` and subsystem facades. Do not expose
  `HostApi` directly, through re-exports or aliases, or transitively through
  public constructors, methods, associated types, bounds, or return values.
- The explicit unsafe `raw` API may expose native addresses only as opaque
  values. Never construct Rust references to client memory, and do not promise
  validity beyond the lifetime of the matching loaded client.
- Preserve native layouts with the independent C++ fixture; serialized sizes
  alone do not establish in-memory packing.
- At each profile-specific bridge operation boundary, validate every pointer,
  range, capacity, and enum before using approved fixed offsets. Do not reuse
  R1 state constants, pointer chains, or layouts for classic profiles.
- Patch and restore only the RakClient vtable slots owned by the host.
- Detours must call originals through their captured backend state.
- Before unloading a plugin at runtime, remove every subscription with
  `HostApi::unregister_and_wait` from a worker thread.
- Never wait in `DllMain`, a callback, or the game tick.
- Never call `FreeLibrary` while callbacks can run.
