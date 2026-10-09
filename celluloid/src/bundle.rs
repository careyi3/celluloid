//! `celluloid bundle`: one HTML file holding the web viewer and an
//! animation, to open in a browser or share.

use base64::Engine;

#[cfg(web_bundle)]
const GLUE: &str = include_str!("../web/celluloid_web.js");
#[cfg(web_bundle)]
const WASM: &[u8] = include_bytes!("../web/celluloid_web_bg.wasm");

/// Write `output` (default: `input` with an `.html` extension).
#[cfg(web_bundle)]
pub fn bundle(input: &str, output: Option<&str>) -> Result<(), String> {
    let json = std::fs::read_to_string(input).map_err(|e| format!("{input}: {e}"))?;
    let animation = celluloid_core::from_json(&json).map_err(|e| format!("{input}: {e}"))?;
    let compact = serde_json::to_string(&animation).map_err(|e| e.to_string())?;
    let output = match output {
        Some(o) => o.to_string(),
        None => std::path::Path::new(input)
            .with_extension("html")
            .display()
            .to_string(),
    };
    let html = page(&animation.name, &compact, GLUE, WASM);
    std::fs::write(&output, &html).map_err(|e| format!("{output}: {e}"))?;
    println!(
        "{output}: {} frames, {} KB",
        animation.frames.len(),
        html.len() / 1024
    );
    Ok(())
}

/// Without the web viewer built in, explain how to get it.
#[cfg(not(web_bundle))]
pub fn bundle(_input: &str, _output: Option<&str>) -> Result<(), String> {
    Err(
        "this celluloid was built without the web viewer; run `cargo xtask web` \
         in the celluloid repository, then reinstall"
            .into(),
    )
}

/// The page: a full-window canvas, the animation as inline JSON, and the
/// generated glue with the wasm inlined as base64.
#[cfg_attr(not(web_bundle), allow(dead_code))]
fn page(title: &str, json: &str, glue: &str, wasm: &[u8]) -> String {
    let wasm = base64::engine::general_purpose::STANDARD.encode(wasm);
    let mut html = String::with_capacity(wasm.len() + glue.len() + json.len() + 1024);
    html.push_str(concat!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n",
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>",
    ));
    html.push_str(&escape_html(title));
    html.push_str(concat!(
        "</title>\n<style>html,body{margin:0;height:100%;overflow:hidden;background:#16171b}",
        "canvas{display:block;width:100%;height:100%}</style>\n</head>\n<body>\n",
        "<canvas id=\"celluloid\"></canvas>\n",
        "<script id=\"animation\" type=\"application/json\">",
    ));
    html.push_str(&json.replace("</", "<\\/"));
    html.push_str("</script>\n<script type=\"module\">\n");
    html.push_str(glue);
    html.push_str("\nconst celluloidWasm = Uint8Array.from(atob(\"");
    html.push_str(&wasm);
    html.push_str(concat!(
        "\"), (c) => c.charCodeAt(0));\n",
        "try {\n",
        "  await __wbg_init({ module_or_path: celluloidWasm });\n",
        "  const player = await Player.start(document.getElementById(\"celluloid\"));\n",
        "  player.load(document.getElementById(\"animation\").textContent);\n",
        "  document.body.dataset.state = \"running\";\n",
        "} catch (e) {\n",
        "  document.body.dataset.state = \"failed\";\n",
        "  document.body.insertAdjacentHTML(\"afterbegin\", ",
        "\"<pre style='color:#eee;padding:16px;white-space:pre-wrap'></pre>\");\n",
        "  document.querySelector(\"pre\").textContent = \"celluloid could not start: \" + e;\n",
        "}\n",
        "</script>\n</body>\n</html>\n",
    ));
    html
}

#[cfg_attr(not(web_bundle), allow(dead_code))]
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_keeps_json_inside_its_script() {
        let html = page(
            "a <b>",
            r#"{"name":"</script><script>x"}"#,
            "/*glue*/",
            &[0, 1, 2],
        );
        assert!(html.contains("<title>a &lt;b&gt;</title>"));
        assert!(html.contains(r#"{"name":"<\/script><script>x"}"#));
        assert_eq!(html.matches("</script>").count(), 2);
        assert!(html.contains("atob(\"AAEC\")"));
    }
}
