# Completion checks
- Run `cargo make quality` for the complete local quality gate: formatting, `cargo check --workspace --all-targets`, workspace tests, Clippy with warnings denied, and documentation without dependencies.
- Run `cargo make build-release` when release artifacts or workspace build behavior can change.
- For native-boundary or client-offset changes, also run the relevant Windows x86 integration check.
- For wire/native changes, add exact vector/layout coverage; for behavior changes, add focused unit tests.
- Ensure `CORE.md` and `ARCHITECTURE.md` reflect feature/module changes, with `README.md`/`TODO.md`/other project docs updated when their concern changes.
- Never commit `target/`, secrets, machine-specific paths, proprietary clients, or headers.