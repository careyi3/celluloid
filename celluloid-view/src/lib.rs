//! egui viewer for Celluloid animations.
//!
//! [`Viewer`] plays one animation and can be embedded in any egui app,
//! native or web. With the `native` feature (on by default), [`App`] wraps
//! it with a folder browser and live reload for the desktop.

mod array;
mod canvas;
mod graph;
mod grid;
mod hex;
mod nodes;
mod theme;
mod tree;
mod viewer;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod app;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use app::App;
pub use theme::apply as apply_theme;
pub use viewer::Viewer;

/// Open the desktop viewer on a file or folder (default: the working directory).
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn run(path: Option<std::path::PathBuf>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("celluloid")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([560.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        "celluloid",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, path)))),
    )
}
