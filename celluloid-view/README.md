# celluloid-view

The egui viewer behind [Celluloid](https://github.com/careyi3/celluloid).

Use it to play a Celluloid animation inside your own egui app. If you just want to look at a file, install the [`celluloid`](https://crates.io/crates/celluloid) viewer instead.

You can find the [crates.io listing here](https://crates.io/crates/celluloid-view).

![CI](https://github.com/careyi3/celluloid/actions/workflows/test.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/celluloid-view.svg)](https://crates.io/crates/celluloid-view)
[![Crates.io](https://img.shields.io/crates/d/celluloid-view.svg)](https://crates.io/crates/celluloid-view)

## Setup

```bash
$ cargo add celluloid-view celluloid-core
```

For the web, turn off the default `native` feature:

```bash
$ cargo add celluloid-view --no-default-features
```

## Usage

Make a `Viewer` from an animation and call `ui` every frame:

```rust
use celluloid_view::{apply_theme, Viewer};
use eframe::egui::Ui;

struct MyApp {
    viewer: Viewer,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.viewer.ui(ui);
    }
}

let json = std::fs::read_to_string("day12.json")?;
let viewer = Viewer::new(celluloid_core::from_json(&json)?);
```

The full version is in [`examples/embed.rs`](examples/embed.rs):

```bash
$ cargo run -p celluloid-view --example embed -- day12.json
```

Call `apply_theme` once at startup if you want the viewer's dark theme. Use `seek`, `set_playing` and `cursor` to drive playback from your own code, and set `show_controls` to `false` to hide the built in controls.

## License

MIT
