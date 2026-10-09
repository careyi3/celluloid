//! The command line, minus opening a window.

use std::path::PathBuf;
use std::process::{Command, Output};

const OLD: &str = r#"{
    "name": "old",
    "grid_config": {"width": 2, "height": 1},
    "metadata": {"frame_delay_ms": 80.0},
    "frames": [{"grid": [[0, 1]], "message": "a"}]
}"#;

fn celluloid(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_celluloid"))
        .args(args)
        .output()
        .unwrap()
}

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("celluloid-cli-{}-{name}", std::process::id()))
}

#[test]
fn version() {
    let out = celluloid(&["--version"]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout.trim(),
        format!("celluloid {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn bad_args_print_usage() {
    let out = celluloid(&["--nope"]);
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("usage:"));
}

#[test]
fn missing_file() {
    let out = celluloid(&["does-not-exist.json"]);
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr)
        .unwrap()
        .contains("no such file"));
}

#[test]
fn convert_upgrades_old_files() {
    let (input, output) = (temp("old.json"), temp("new.json"));
    std::fs::write(&input, OLD).unwrap();

    let out = celluloid(&["convert", input.to_str().unwrap(), output.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let json = std::fs::read_to_string(&output).unwrap();
    std::fs::remove_file(&input).unwrap();
    std::fs::remove_file(&output).unwrap();

    let animation = celluloid_core::from_json(&json).unwrap();
    assert_eq!(animation.format, celluloid_core::FORMAT_VERSION);
    assert_eq!(animation.frame_delay_ms, 80.0);
}
