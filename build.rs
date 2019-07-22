use std::env;

fn main() {
    // copied from remoteprocess/build.rs because I couldn't find a way to share this
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_ref() {
        "windows" => println!("cargo:rustc-cfg=unwind"),
        "macos" => println!("cargo:rustc-cfg=unwind"),
        "linux" => {
            // We only support native unwinding on x86_64 linux
            match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_ref() {
                "x86_64" | "aarch64" => println!("cargo:rustc-cfg=unwind"),
                _ => {} 
            }
        },
        _ => {}
    }
}
