# Publish local SA-MP UI as one exact-version service

Phase 11 publishes copied chat, chat-input, dialog, cursor, death-window, and scoreboard state and queued actions under exact-version service ID `0x0000_2004`. These operations share the local UI lifecycle and existing game-thread command queue, so one `SampUiServiceV1` keeps related snapshots and mutations together without extending frozen `SampServiceV1`; fixed-capacity values cross the ABI and every mutation returns a Core receipt.
