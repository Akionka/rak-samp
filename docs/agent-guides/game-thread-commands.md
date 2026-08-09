# Game-thread commands

- Copy local UI requests, explicit sends, and incoming emulation into the one
  shared, bounded 256-entry `GameCommand` queue. Plugin threads must not call
  RakClient or other native client operations directly.
- Preserve FIFO tick-snapshot semantics: snapshot accepted commands before
  the native game-process call, invoke the original exactly once, then drain
  that snapshot and refresh caches.
- Keep incoming-packet detours limited to networking work.
- Validate all inputs and destination pointers, ranges, capacities, enums, and
  fixture-backed fields before the native call or write.
- Return owned `CommandReceipt<T>` values for receipt-bearing operations.
  Preserve poll, timed-wait, and release behavior.
- Reject waits on the game thread and inside listener callbacks.
- Dropping a receipt detaches its waiter without cancelling its copied command.
  A timed-out receipt remains retryable, and shutdown completes every retained
  receipt.
- Bracket each game-tick cache refresh with a monotonic generation. When the
  connection state changes, invalidate connection-bound entity caches,
  pending heavy refreshes, and captured connection-bound raw addresses.
