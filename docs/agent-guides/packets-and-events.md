# Packets and typed events

- Invoke listeners in registration order. Preserve continue, block, and
  exact-bit atomic replacement outcomes.
- Send each emulated packet through incoming listeners exactly once.
- Keep nested same-thread dispatch non-blocking.
- Explicit sends bypass outgoing listeners.
- Keep callback-local replacement synchronous and serialize it before the
  atomic ABI replacement call.
- Typed helpers must reuse the single host subscription.
- Keep text with uncertain encoding as bytes.
- Bound length-prefixed allocations.
- Never retain callback-local events.
- Keep SDK protocol codecs and bit streams independent of the native host so
  they remain testable without a live client.
- Route `Local` protocol actions through the same typed `Net` operations; do
  not imply that a server-bound send also performs a local GTA state change.
