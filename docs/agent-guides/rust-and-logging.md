# Rust and logging

- Return explicit errors instead of using `unwrap()` or `expect()` outside
  tests.
- Use the `log` facade; logging setup belongs in `src/logging.rs`.
- Never log packet or RPC payloads.
