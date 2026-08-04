# Commands
- Build all packages: `cargo build --workspace`; distribution: `cargo build --workspace --release`.
- Test: `cargo test --workspace`.
- Lint: `cargo clippy --workspace -- -D warnings`.
- Format check/apply: `cargo fmt --check` / `cargo fmt`.
- Deploy release host: `cargo make deploy`.
- Deploy current validation setup: `cargo make deploy-validation`; include external unload manager: `cargo make deploy-validation-unload`.
- Deploy chat-command example: `cargo make deploy-chat-command-example`.
- Close GTA before deployment because Windows locks loaded ASIs.
- Windows repository search/listing: prefer `rg PATTERN` and `rg --files`; use PowerShell `Get-Content`/ `Get-ChildItem` when needed.