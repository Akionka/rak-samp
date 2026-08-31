# Keep native SA-MP pointers Host-internal

Phase 11 migrates only `raw::bitstream_data`, whose pointer refers to plugin-owned `BitStream` storage with an exact local lifetime. The other legacy raw accessors will not migrate because publishing volatile SA-MP singleton, pool, vtable, and RPC-node addresses would turn profile- and thread-dependent Host internals into a durable plugin ABI; a future raw Service requires its own evidence-backed lifetime and version contract.
