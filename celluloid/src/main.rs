mod bundle;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
celluloid: view Celluloid animations

usage:
  celluloid [PATH]              open a .json file, or browse a folder (default: .)
  celluloid convert IN [OUT]    rewrite an old full-grid file in the current format
                                (default OUT: overwrite IN)
  celluloid bundle IN [OUT]     write one self-contained HTML page playing IN
                                (default OUT: IN with .html)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["-h" | "--help"] => {
            println!("{USAGE}");
            Ok(())
        }
        ["-V" | "--version"] => {
            println!("celluloid {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        ["convert", input] => convert(input, input),
        ["convert", input, output] => convert(input, output),
        ["bundle", input] => bundle::bundle(input, None),
        ["bundle", input, output] => bundle::bundle(input, Some(output)),
        [] => view(None),
        [path] if !path.starts_with('-') => view(Some(PathBuf::from(path))),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn view(path: Option<PathBuf>) -> Result<(), String> {
    if let Some(p) = &path {
        if !p.exists() {
            return Err(format!("{}: no such file or folder", p.display()));
        }
    }
    celluloid_view::run(path).map_err(|e| e.to_string())
}

fn convert(input: &str, output: &str) -> Result<(), String> {
    let json = std::fs::read_to_string(input).map_err(|e| format!("{input}: {e}"))?;
    let animation = celluloid_core::from_json(&json).map_err(|e| format!("{input}: {e}"))?;
    let out = serde_json::to_string(&animation).map_err(|e| e.to_string())?;
    std::fs::write(output, out).map_err(|e| format!("{output}: {e}"))?;
    println!(
        "{output}: {} frames ({} -> {} bytes)",
        animation.frames.len(),
        json.len(),
        std::fs::metadata(output).map(|m| m.len()).unwrap_or(0)
    );
    Ok(())
}
