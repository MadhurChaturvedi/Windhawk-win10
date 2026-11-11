fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        panic!(
            "windhawk-core only targets Windows (specifically Windows 10 x86_64). Detected target_os = {:?}",
            target_os
        );
    }

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch != "x86_64" {
        println!(
            "cargo:warning=Target architecture is {:?}, but recommended target is x86_64",
            target_arch
        );
    }
}
