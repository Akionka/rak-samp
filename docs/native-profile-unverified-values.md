# Native Profile Unverified Values

This register records native values whose current Rust declarations lack
independent fixture, binary, or live-client evidence. A migration must use an
explicit unavailable strategy and return `DirectClientError::NotReady` for an
affected operation until the required evidence is added. It must not use a
guessed value or fallback.

| Profile | Operation or value | Current source | Required evidence | Status |
| --- | --- | --- | --- | --- |
| R1 | Native functions and singleton RVAs, except the textdraw setter | `src/platform/win32/r1_client.rs` and `r1_client/` | Shipped R1 binary or an executable pin test | Unverified |
| R1 | Native ABI signatures and calling conventions | R1 native function aliases | ABI fixture or live call validation | Unverified |
| R1 | Remote-player fixture fields without individual Rust assertions | `src/platform/win32/r1_client/` | Field-level fixture assertions | Unverified |
| R3, R5, DL | Non-layout native and singleton RVAs | `src/platform/win32/r3_client.rs` | Shipped binary or executable pin tests for each profile | Unverified |
| R3, R5, DL | Native ABI signatures and calling conventions | `src/platform/win32/r3_client.rs` function aliases | ABI fixture or live call validation | Unverified |
| R3, R5, DL | GTA ped and vehicle handle-conversion targets | `src/platform/win32/r3_client.rs` | Executable GTA fixture or binary evidence | Unverified |
| R3, R5, DL | `NET_GAME_SERVER_SETTINGS_OFFSET` use | `src/platform/win32/r3_client.rs` | Operation integration test or removal after proving it is unnecessary | Fixture-backed but unused |
