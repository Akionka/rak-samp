# Development workflow

Run commands from the repository root.

## Validation

- Build the workspace: `cargo build --workspace`. Add `--release` for a
  distribution build.
- Run tests: `cargo test --workspace`.
- Treat Clippy warnings as errors: `cargo clippy --workspace -- -D warnings`.
- Check formatting: `cargo fmt --check`. Apply formatting with `cargo fmt`.
- Put unit tests beside their modules and name them for observable behavior.
- Run workspace tests for behavior changes.
- Add exact vectors and exercise the independent C++ layout fixture for
  wire-format or native-boundary changes.
- Preserve tests for exact-bit replacement, listener ordering, subscription
  shutdown, and native layout coverage when changing those behaviors.
- After an SDK facade visibility change, compile an external consumer through
  `Samp`, keep any negative privacy doctest minimal, audit exports and public
  signatures, and inspect the generated public API for forbidden low-level
  types. A successful documentation build alone does not prove absence.

## Deployment

- Deploy the release host with `cargo make deploy`; it copies the host to
  `$env:GTA_DIR`.
- Deploy the chat-command example with
  `cargo make deploy-chat-command-example`.
- Close GTA before deployment because Windows locks loaded ASI files.

## Documentation and repository contents

- Update `CORE.md` and `ARCHITECTURE.md` when a feature or module changes.
- Keep usage instructions in `README.md` and planned work in `TODO.md`.
- Do not commit proprietary clients or headers.
