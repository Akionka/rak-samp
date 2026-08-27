# Toolchain
- Rust 2024 Cargo workspace; minimum/pinned project toolchain Rust 1.98.
- `.cargo/config.toml` defaults builds to `i686-pc-windows-msvc`; primary artifact is the 32-bit Windows MSVC `cdylib` installed as `samp_client_sdk.asi`.
- Workspace: host root, public `samp-client-sdk` in `sdk/`, `samp-protocol` in `crates/samp-protocol/`, and plugin/probe examples.
- Runtime crates: `log`, `simplelog`; Windows x86-only native dependencies: `minhook`, `windows-sys`.
- `cc` compiles the independent C++ native-layout fixture through `build.rs`.
- Cargo Make owns local quality, workspace builds, and GTA installation; install tasks require `GTA_DIR`.