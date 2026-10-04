//! The Celluloid viewer for the web.
//!
//! Build it with `cargo xtask web`, which writes `celluloid_web.js` and
//! `celluloid_web_bg.wasm` to `celluloid/web/`. Serve those two files and
//! start a [`Player`] on a canvas:
//!
//! ```js
//! import init, { Player } from "./celluloid_web.js";
//! await init();
//! const player = await Player.start(document.getElementById("anim"));
//! player.load(await (await fetch("day12.json")).text());
//! ```
//!
//! Dropping an animation file onto the canvas also loads it.

#![cfg(target_arch = "wasm32")]

use celluloid_view::Viewer;
use eframe::egui::{self, RichText};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

/// Contents of dropped files, read in the background: file name and text.
type Inbox = Rc<RefCell<Vec<(String, Result<Vec<u8>, String>)>>>;

struct WebApp {
    viewer: Option<Viewer>,
    ctx: egui::Context,
    show_controls: bool,
    error: Option<String>,
    inbox: Inbox,
}

impl WebApp {
    fn load(&mut self, json: &str) -> Result<(), String> {
        let animation = celluloid_core::from_json(json).map_err(|e| e.to_string())?;
        let mut viewer = Viewer::new(animation);
        viewer.show_controls = self.show_controls;
        self.viewer = Some(viewer);
        self.error = None;
        Ok(())
    }
}

impl eframe::App for WebApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for file in ui.input(|i| i.raw.dropped_files.clone()) {
            let inbox = self.inbox.clone();
            let ctx = self.ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let name = file.path().display().to_string();
                let bytes = file.bytes_async().await;
                inbox.borrow_mut().push((name, bytes));
                ctx.request_repaint();
            });
        }
        let arrived: Vec<_> = self.inbox.borrow_mut().drain(..).collect();
        for (name, bytes) in arrived {
            let result = bytes.and_then(|b| self.load(&String::from_utf8_lossy(&b)));
            if let Err(e) = result {
                self.error = Some(format!("{name}: {e}"));
            }
        }

        if let Some(error) = &self.error {
            egui::Panel::top("error")
                .frame(egui::Frame::new().fill(egui::Color32::from_rgb(0x5c, 0x1f, 0x24)).inner_margin(8))
                .show(ui, |ui| {
                    ui.label(RichText::new(error).color(egui::Color32::WHITE));
                });
        }
        match &mut self.viewer {
            Some(viewer) => viewer.ui(ui),
            None => {
                egui::CentralPanel::default_margins().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("Drop a celluloid .json file here").weak());
                    });
                });
            }
        }
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(&mut *self)
    }
}

/// A viewer running on a canvas.
#[wasm_bindgen]
pub struct Player {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl Player {
    /// Start a viewer on `canvas`. It sizes itself to the canvas's CSS size.
    pub async fn start(canvas: HtmlCanvasElement) -> Result<Player, JsValue> {
        let runner = eframe::WebRunner::new();
        runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    celluloid_view::apply_theme(&cc.egui_ctx);
                    Ok(Box::new(WebApp {
                        viewer: None,
                        ctx: cc.egui_ctx.clone(),
                        show_controls: true,
                        error: None,
                        inbox: Inbox::default(),
                    }))
                }),
            )
            .await?;
        Ok(Player { runner })
    }

    /// Show an animation, given the contents of its JSON file.
    pub fn load(&self, json: &str) -> Result<(), JsValue> {
        self.with(|app| app.load(json))?.map_err(|e| JsValue::from_str(&e))
    }

    pub fn play(&self) -> Result<(), JsValue> {
        self.with(|app| app.viewer.as_mut().map(|v| v.set_playing(true)))
            .map(drop)
    }

    pub fn pause(&self) -> Result<(), JsValue> {
        self.with(|app| app.viewer.as_mut().map(|v| v.set_playing(false)))
            .map(drop)
    }

    /// Jump to a frame, counting from 0.
    pub fn seek(&self, frame: usize) -> Result<(), JsValue> {
        self.with(|app| app.viewer.as_mut().map(|v| v.seek(frame)))
            .map(drop)
    }

    /// The frame on screen, counting from 0.
    pub fn frame(&self) -> Result<usize, JsValue> {
        self.with(|app| app.viewer.as_ref().map_or(0, Viewer::cursor))
    }

    /// Number of frames in the loaded animation.
    pub fn frames(&self) -> Result<usize, JsValue> {
        self.with(|app| app.viewer.as_ref().map_or(0, Viewer::frames))
    }

    /// Show or hide the transport bar and inspector, leaving just the
    /// panels for pages that drive playback themselves.
    pub fn set_controls(&self, show: bool) -> Result<(), JsValue> {
        self.with(|app| {
            app.show_controls = show;
            if let Some(v) = &mut app.viewer {
                v.show_controls = show;
            }
        })
    }

    /// Stop the viewer and release the canvas.
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    fn with<R>(&self, f: impl FnOnce(&mut WebApp) -> R) -> Result<R, JsValue> {
        let mut app = self
            .runner
            .app_mut::<WebApp>()
            .ok_or_else(|| JsValue::from_str("the viewer has crashed"))?;
        let result = f(&mut app);
        app.ctx.request_repaint();
        Ok(result)
    }
}
