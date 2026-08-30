use std::{env, path::PathBuf};

fn main() {
    const FIXTURE: &str = "tests/fixtures/raknet_layout.cpp";
    println!("cargo:rerun-if-changed={FIXTURE}");
    println!("cargo:rerun-if-env-changed=PLUGIN_SDK_DIR");
    println!("cargo:rustc-check-cfg=cfg(gta_sa_layout_oracle)");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("x86")
    {
        return;
    }

    cc::Build::new()
        .cpp(true)
        .file(FIXTURE)
        .compile("raknet_layout_fixture");

    if let Ok(plugin_sdk_dir) = env::var("PLUGIN_SDK_DIR") {
        const GTA_FIXTURE: &str = "tests/fixtures/gta_sa_layout.cpp";
        let root = PathBuf::from(plugin_sdk_dir);
        println!("cargo:rerun-if-changed={GTA_FIXTURE}");
        cc::Build::new()
            .cpp(true)
            .flag_if_supported("/std:c++latest")
            .file(GTA_FIXTURE)
            .include(root.join("shared"))
            .include(root.join("shared/game"))
            .include(root.join("plugin_sa"))
            .include(root.join("plugin_sa/game_sa"))
            .include(root.join("plugin_sa/game_sa/enums"))
            .include(root.join("plugin_sa/game_sa/rw"))
            .compile("gta_sa_layout_fixture");
        println!("cargo:rustc-cfg=gta_sa_layout_oracle");
    }
}
