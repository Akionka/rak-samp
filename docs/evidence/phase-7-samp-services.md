# Phase 7 SA-MP Services Evidence

Date: 2026-08-30

Phase 7 moves new plugin code off `SampClientSdkApiV1` without removing or
changing the legacy API.

## Implemented boundary

- `modkit-abi` defines fixed-layout, exact-version `SampServiceV1` and
  `SampNetServiceV1` tables. ADR 0009 records their frozen V1 layout and
  callback-context ownership rules.
- `modkit-sdk` validates both tables and exposes low-level service views.
- The host adapts both services to the existing runtime/backend. Network and
  chat callback state is released exactly once after callback drain.
- The new `samp` crate owns safe snapshots, typed chat styles, Core-backed
  receipts/subscriptions, callback-local network events, and typed
  `samp-protocol` adapters.
- `samp-service-chat-plugin` and `samp-service-network-plugin` use only the new
  facade. Existing legacy examples are unchanged.

## Automated validation

The following commands passed on the Phase 7 branch:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace -j 1
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --release
```

The gate includes fixed ABI layout tests, host adapter/reclamation tests, the
existing Protocol exact-vector suites, the new typed-route ID and chat vector
tests, all legacy SDK tests, and both new example builds. A dependency audit
confirms that neither new example imports `samp-client-sdk`.

After the live-start correction in `b39b66f`, focused tests, Clippy with
warnings denied, and a release build for the host and both examples passed
again.

## Controlled R3 smoke

The release host and both service-backed examples were deployed from the local
workspace target to `C:\Games\GTASA-SDK-R3-LIVE-TEST`. The old full R3 probe was
disabled for isolation. Deployed SHA-256 values:

| Artifact | SHA-256 |
| --- | --- |
| `samp_client_sdk.asi` | `234B74E6C544AB4E3E37B6EAC3F9F6B1C8B4086170C1610FE442ED22A9341EE9` |
| `samp_service_chat_plugin.asi` | `CE74B3DB08E5A76773D726A43FDA0A66C67EC9D208A06F5B99CF6CFBC5279FEE` |
| `samp_service_network_plugin.asi` | `0C01D1F5C8A3E8294B025445EB0EE37304AF8AF7E682E34E40B3C963E414A25B` |

The first launch exposed an example-only startup race: the host needed about 49
seconds to become ready, while both examples stopped waiting after 30 seconds.
The chat example also supplied the leading `/` to the exact native command
registry. Commit `b39b66f` extends startup waiting to 120 seconds and registers
the native name as `sampservice`; user input remains `/sampservice`.

The corrected fixture was launched through its `samp_debug.exe` against the R3
loopback server. The host log recorded readiness, modkit RPC subscription 2,
and successful execution of the chat registration command. Interactive checks
then passed:

- an ordinary outgoing chat message displayed `Phase 7 SampNetServiceV1 typed
  RPC path works`;
- `/sampservice` displayed `Phase 7 SampServiceV1 chat path works`;
- the game remained stable.
