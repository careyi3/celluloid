//! Build tasks: `cargo xtask web` builds the web viewer into `celluloid/web/`,
//! where `celluloid bundle` and pages embedding the viewer pick it up.

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["web"] => web(),
        _ => Err("usage: cargo xtask web".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn web() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(root)
        .args([
            "build",
            "-p",
            "celluloid-web",
            "--lib",
            "--target",
            "wasm32-unknown-unknown",
            "--profile",
            "wasm",
        ])
        .status()
        .map_err(|e| format!("running cargo: {e}"))?;
    if !status.success() {
        return Err("building celluloid-web failed (is the wasm32-unknown-unknown target installed? rustup target add wasm32-unknown-unknown)".into());
    }

    let input = root.join("target/wasm32-unknown-unknown/wasm/celluloid_web.wasm");
    let out = root.join("celluloid/web");
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(&input)
        .web(true)
        .map_err(|e| e.to_string())?
        .typescript(false)
        .omit_default_module_path(false)
        .out_name("celluloid_web")
        .generate(&out)
        .map_err(|e| format!("wasm-bindgen: {e}"))?;

    for file in ["celluloid_web.js", "celluloid_web_bg.wasm"] {
        let size = std::fs::metadata(out.join(file)).map(|m| m.len()).unwrap_or(0);
        println!("{:>9} KB  celluloid/web/{file}", size / 1024);
    }
    Ok(())
}
