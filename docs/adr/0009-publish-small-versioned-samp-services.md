# Publish small versioned SA-MP services

New plugins use two exact-version immutable service tables instead of the
legacy 145-field `SampClientSdkApiV1`. `SampServiceV1` owns the minimal stable
SA-MP state, player, chat, and chat-command operations required by the first
service-backed example. `SampNetServiceV1` owns Packet/RPC subscriptions,
callback-local exact-bit access, Native encoded strings, sends, and incoming
emulation. The Legacy service and old export remain unchanged as regression
coverage during migration.

Both tables and every value crossing them are declared in `modkit-abi` with
`#[repr(C)]`, fixed-width lengths, copied inputs, caller-owned outputs, and no
Rust allocator-owned or borrowed types. Operation result values use
`ModResult`. Subscription and receipt identities use the Core service types.
All enum-like inputs and callback actions use integer constants so an unknown
foreign value can be rejected or treated as `Continue` without constructing an
invalid Rust enum.

## Frozen V1 scope

`SampServiceV1` contains, in order after `ServiceHeader`:

1. `version(out: *mut u32)`;
2. `game_state(out: *mut i32)`;
3. `server_info(out: *mut SampServerInfoV1)`;
4. `local_player(out: *mut SampLocalPlayerV1)`;
5. `player_info(id: u16, out: *mut SampPlayerInfoV1)`;
6. `submit_chat_add(style, text, prefix, colours, out_receipt)`;
7. `submit_register_chat_command(name, callback, user_data, release,
   out_subscription, out_receipt)`.

`SampNetServiceV1` contains, in order after `ServiceHeader`:

1. Packet and RPC listener registration with direction, callback, opaque
   context, release callback, and output subscription;
2. callback-local event ID, reset, remaining-bit, exact-bit read, and atomic
   exact-bit replacement operations;
3. Native encoded-string encode and callback-local decode operations using
   caller-provided buffers;
4. queued Packet and RPC sends returning Core receipt IDs;
5. queued incoming Packet and RPC emulation returning Core receipt IDs;
6. `incoming_emulation_ready(out: *mut u8)`.

The exact Rust declarations and layout tests in `modkit-abi` are normative.
V1 tables are not append-only. Later operations require V2 or another service.

## Callback ownership

Every successful registration transfers its opaque context to the host. The
host invokes the supplied release callback exactly once and only after all
callbacks for that subscription drain. Non-blocking unregister disables new
callback starts and schedules deferred reclamation. Successful Core
`unregister_and_wait` returns only after the release callback returns. Failed
registration does not transfer ownership. Unknown callback action values fail
open as `Continue`.

Legacy and new listeners share the existing Runtime registry. Registration
order, `Continue`, `Block`, atomic replacement, and non-concurrent delivery for
one subscription therefore remain unchanged.

## Plugin API

`modkit-sdk` exposes validated low-level service views and Core-backed receipt
and subscription lifetime primitives. A new `samp` crate owns the safe SA-MP
facade and typed `samp-protocol` adapters. It does not expose service tables or
the legacy `HostApi`. The host implements both services as adapters over the
existing backend so legacy and new APIs cannot diverge in Native behavior.

## Consequences

- The service ABI stays small and auditable.
- Network lifecycle support must integrate deferred context reclamation before
  safe subscription drop is available.
- The initial facade deliberately covers only the two proving examples.
- Broad UI, pool, and connection migration requires later service versions.
