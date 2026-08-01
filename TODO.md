# TODO

Keep only pending work here; completed work belongs in Git history and
[CORE.md](CORE.md). Use small, imperative checkbox items and update this file
with the implementing change.

## In progress

- [ ] Validate attach, interception, rewrite, cancellation, send, emulation,
  detach, and shutdown on legal R1, R2, R3.1, R4.2, R5.1, and DL installations.

## Backlog

- [ ] Add bit-length-preserving atomic replacement and RakNet Huffman strings,
  then implement encoded-string helpers such as `onShowDialog`.
- [ ] Add the remaining complex and bit-packed MoonLoader-style RPC and sync
  packet schemas.
- [ ] Add fixture tests for each typed decoder, wire rewrite, and appended ABI
  field.
- [ ] Add an end-to-end fixture with the host and an independently loaded
  plugin ASI.
