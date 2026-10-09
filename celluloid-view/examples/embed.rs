//! A Viewer inside your own eframe app.
//! cargo run -p celluloid-view --example embed -- day12.json

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

fn main() -> eframe::Result {
    let path = std::env::args().nth(1).expect("usage: embed FILE.json");
    let json = std::fs::read_to_string(path).expect("could not read file");
    let animation = celluloid_core::from_json(&json).expect("not a celluloid file");

    eframe::run_native(
        "embed",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(MyApp {
                viewer: Viewer::new(animation),
            }))
        }),
    )
}
