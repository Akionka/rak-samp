# Commands
- Complete local quality gate: `cargo make quality`; it runs `format-check`, `check`, `test`, `clippy`, and `doc`.
- Individual checks: `cargo make format-check`, `cargo make check`, `cargo make test`, `cargo make clippy`, and `cargo make doc`.
- Apply formatting: `cargo make format`.
- Build all workspace packages: `cargo make build-debug`; distribution: `cargo make build-release`.
- Install the release host: `cargo make install`; debug host: `cargo make install-debug`; chat-command example: `cargo make install-chat-command-example`.
- Install tasks require `GTA_DIR`. Close GTA before installation because Windows locks loaded ASI and PDB files.
- `Makefile.toml` disables Cargo Make core tasks and workspace fan-out; its explicit tasks run once from the workspace root.
- Windows repository search/listing: prefer `rg PATTERN` and `rg --files`; use PowerShell `Get-Content`/`Get-ChildItem` when needed.