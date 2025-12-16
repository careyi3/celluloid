use std::process::Command;
use which::which;

fn main() {
    println!("cargo:rerun-if-changed=../wasm/src/lib.rs");
    println!("cargo:rerun-if-changed=../wasm/Cargo.toml");

    if std::env::var("SKIP_WASM_BUILD").is_ok() {
        println!("cargo:warning=Skipping WASM build (SKIP_WASM_BUILD is set)");
        return;
    }

    let wasm_pack = match which("wasm-pack") {
        Ok(path) => path,
        Err(_) => {
            println!("cargo:warning=wasm-pack not found, skipping WASM build. Install with: cargo install wasm-pack");
            return;
        }
    };

    let status = Command::new(wasm_pack)
        .args(&["build", "--target", "web", "--out-dir", "../pkg", "--dev"])
        .current_dir("../wasm")
        .status()
        .expect("Failed to run wasm-pack");

    if !status.success() {
        panic!("wasm-pack build failed");
    }

    println!("cargo:warning=WASM build completed successfully");
}
