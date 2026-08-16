# Native Profile Unverified Values

This register records native values whose current Rust declarations lack
independent fixture, binary, or live-client evidence. A migration must use an
explicit unavailable strategy and return `DirectClientError::NotReady` for an
affected operation until the required evidence is added. It must not use a
guessed value or fallback.

| Profile | Operation or value | Current source | Required evidence | Status |
| --- | --- | --- | --- | --- |
| R1 | Singleton RVAs: dialog `0x21A0B8`, input `0x21A0E8`, chat `0x21A0E4`, scoreboard `0x21A0B4`, death window `0x21A0EC`, NetGame `0x21A0F8`, game `0x21A10C` | `r1_client/addresses.rs` | Shipped R1 binary or an executable pin test for each RVA | Unverified |
| R1 | Dialog and input method RVAs: `0x6B9C0`, `0x6C040`, `0x657E0`, `0x658E0`, `0x65A70`, `0x65AD0`, `0x65D30`, `0x80F60`, `0x81030` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Chat, death-window, and game method RVAs: `0x64010`, `0x5D7A0`, `0x66A10`, `0x9BD30`, `0x9BC10` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | NetGame method RVAs: `0x2E20`, `0x1160`, `0x1170`, `0xA060` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Player-pool method RVAs: `0x1A30`, `0x6A1F0`, `0x6A200`, `0x10B0`, `0x10F0`, `0xB680`, `0x13CE0`, `0x6A190`, `0x6A1C0`, `0x10520`, `0xB3E0` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Vehicle and remote-player method RVAs: `0x1140`, `0x12A00`, `0x129D0`, `0x1080`, `0x12BA0` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Local-player method RVAs: `0x2D60`, `0x3D90`, `0x3D40`, `0x30C0`, `0x3AD0`, `0x4B30`, `0x4FF0`, `0x4D10`, `0x5AF0`, `0x51B0`, `0x5380`, `0x6E30`, `0x6080` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Ped method RVAs: `0xA6610`, `0xA6650` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Send-rate and animation RVAs: `0xEC0A8`, `0xEC0AC`, `0xEC0B0`, `0xF15B0` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Text-label method RVAs: `0x11C0`, `0x12D0` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Textdraw create/delete RVAs: `0x1AE20`, `0x1AD00` | `r1_client/addresses.rs` | Shipped R1 binary or executable pin tests | Unverified |
| R1 | Textdraw setter RVA: `0xAC870` | `r1_client/addresses.rs`; literal pin test | Shipped R1 binary or executable pin test | Literal pin only |
| R1 | Native ABI signatures and calling conventions for every R1 direct call | `r1_client/native_types.rs` | ABI fixture or live call validation for each signature | Unverified |
| R1 | Remote-player fixture fields without individual Rust assertions | `src/platform/win32/r1_client/` | Field-level fixture assertions | Unverified |
| R3 | Singleton RVAs: NetGame `0x26E8DC`, dialog `0x26E898`, input `0x26E8CC`, chat `0x26E8C8`, scoreboard `0x26E894`, game `0x26E8F4` | `src/platform/win32/r3_client.rs` | Shipped R3 binary or executable pin tests for each RVA | Unverified |
| R3 | Connection, pool, player, ped, send-rate, and animation RVAs recorded in `R3_SPEC` | `src/platform/win32/r3_client.rs` | Shipped R3 binary or executable pin tests for each RVA | Unverified |
| R3 | Dialog, input, chat, game, text-label, and textdraw RVAs recorded in `R3_SPEC` | `src/platform/win32/r3_client.rs` | Shipped R3 binary or executable pin tests for each RVA | Unverified |
| R3 | Native ABI signatures and calling conventions for every direct call | `src/platform/win32/r3_client.rs` function aliases | ABI fixture or live call validation for each signature | Unverified |
| R3 | `NET_GAME_SERVER_SETTINGS_OFFSET` at `0x3D5` | Packed R3 layout inference | Independent fixture member assertion or binary evidence | Fixture-backed only by inference; unused |
| R5 | Every non-layout RVA recorded in `R5_SPEC` | `src/platform/win32/r3_client.rs` build selectors | Shipped R5 binary or executable pin tests | Unverified |
| R5 | Native ABI signatures and calling conventions | `src/platform/win32/r3_client.rs` function aliases | ABI fixture or live call validation for each signature | Unverified |
| R5 | GTA ped and vehicle handle-conversion targets | GTA 1.0 US constants | Executable fixture or binary evidence | Unverified; operation unavailable |
| DL | Every non-layout RVA recorded in `DL_SPEC` | `src/platform/win32/r3_client.rs` DL `build_value` selectors | Shipped DL binary or executable pin tests | Unverified |
| DL | Native ABI signatures and calling conventions | `src/platform/win32/r3_client.rs` function aliases | ABI fixture or live call validation for each signature | Unverified |
| DL | GTA ped and vehicle handle-conversion targets | GTA 1.0 US constants | Executable fixture or binary evidence | Unverified; operation unavailable |
| R3, R5, DL | Non-layout native and singleton RVAs | `src/platform/win32/r3_client.rs` | Shipped binary or executable pin tests for each profile | Unverified |
| R3, R5, DL | Native ABI signatures and calling conventions | `src/platform/win32/r3_client.rs` function aliases | ABI fixture or live call validation | Unverified |
| R3, R5, DL | GTA ped and vehicle handle-conversion targets | `src/platform/win32/r3_client.rs` | Executable GTA fixture or binary evidence | Unverified |
| R3, R5, DL | `NET_GAME_SERVER_SETTINGS_OFFSET` use | `src/platform/win32/r3_client.rs` | Operation integration test or removal after proving it is unnecessary | Fixture-backed but unused |
