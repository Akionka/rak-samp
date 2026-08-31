# Publish player state and native synchronization together

Phase 11 publishes remote-player state, owned synchronization snapshots, animation lookup, local-player native actions, and player colour mutations under exact-version service ID `0x0000_2005`. These operations share player-pool identity and profile-specific game-thread refresh/command semantics, so `SampPlayerServiceV1` keeps them together without extending frozen `SampServiceV1`; fixed copied values cross the ABI and mutations return Core receipts.
