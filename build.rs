use std::env;

fn main() {
    const FIXTURE: &str = "tests/fixtures/raknet_layout.cpp";
    println!("cargo:rerun-if-changed={FIXTURE}");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("x86")
    {
        return;
    }

    cc::Build::new()
        .cpp(true)
        .file(FIXTURE)
        .compile("raknet_layout_fixture");
}
