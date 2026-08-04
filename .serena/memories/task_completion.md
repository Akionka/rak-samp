# Completion checks
- Run `cargo fmt --check`.
- Run `cargo test --workspace`.
- Run `cargo clippy --workspace -- -D warnings`.
- Build the full workspace with `cargo build --workspace`; use the Windows x86 target/integration check for native-boundary or client-offset changes.
- For wire/native changes, add exact vector/layout coverage; for behavior changes, add focused unit tests.
- Ensure `CORE.md` and `ARCHITECTURE.md` reflect feature/module changes, with `README.md`/`TODO.md`/other project docs updated when their concern changes.
- Never commit `target/`, secrets, machine-specific paths, proprietary clients, or headers.