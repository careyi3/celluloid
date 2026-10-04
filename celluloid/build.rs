//! Embeds the web viewer for `celluloid bundle` when `cargo xtask web` has
//! built it into `web/`; without it the binary still builds, and `bundle`
//! explains how to get it.

use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(web_bundle)");
    println!("cargo::rerun-if-changed=web");
    let web = Path::new("web");
    if web.join("celluloid_web.js").exists() && web.join("celluloid_web_bg.wasm").exists() {
        println!("cargo::rerun-if-changed=web/celluloid_web.js");
        println!("cargo::rerun-if-changed=web/celluloid_web_bg.wasm");
        println!("cargo::rustc-cfg=web_bundle");
    }
}
