use std::process::Command;
use which::which;

fn main() {
    println!("cargo:rerun-if-changed=../wasm/src");

    let wasm_pack = which("wasm-pack")
        .expect("wasm-pack not found. Please install it with: cargo install wasm-pack");

    let status = Command::new(wasm_pack)
        .args(&["build", "--target", "web", "--out-dir", "../pkg"])
        .current_dir("../wasm")
        .status()
        .expect("Failed to run wasm-pack");

    if !status.success() {
        panic!("wasm-pack build failed");
    }

    println!("cargo:warning=WASM build completed successfully");
}
