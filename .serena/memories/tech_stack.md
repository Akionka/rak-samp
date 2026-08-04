# Toolchain
- Rust 2024 workspace, minimum Rust 1.87.
- Primary artifact: 32-bit Windows MSVC `cdylib` ASI host; docs target `i686-pc-windows-msvc`.
- Key runtime crates: `log`, `simplelog`; Windows x86-only native dependencies: `minhook`, `windows-sys`.
- `cc` build dependency compiles the C++ layout fixture.
- Cargo workspace with host, `plugin_api`, example plugins, and current E2E/validation packages.
- `cargo-make` drives GTA deployment; GTA directory supplied through `GTA_DIR`.