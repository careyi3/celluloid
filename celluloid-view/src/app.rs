use crate::{theme, Viewer};
use celluloid_core::Animation;
use eframe::egui::{self, RichText, Ui};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, SystemTime};

const POLL: Duration = Duration::from_millis(500);

type Loaded = (PathBuf, Option<SystemTime>, Result<Animation, String>);

/// Native app: a folder of animation files on the left, the selected one
/// playing on the right, reloaded whenever it changes on disk.
pub struct App {
    dir: PathBuf,
    files: Vec<PathBuf>,
    selected: Option<PathBuf>,
    viewer: Option<Viewer>,
    /// Modification time of the selected file when it was last read,
    /// whether or not it parsed. A change means it's worth reading again.
    read_mtime: Option<SystemTime>,
    loading: Option<Receiver<Loaded>>,
    error: Option<String>,
    last_poll: f64,
}

impl App {
    /// `path` is a file to open or a folder to browse; defaults to the
    /// working directory.
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        theme::apply(&cc.egui_ctx);
        let path = path.unwrap_or_else(|| PathBuf::from("."));
        let path = path.canonicalize().unwrap_or(path);
        let (dir, selected) = if path.is_file() {
            let dir = path.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
            (dir, Some(path))
        } else {
            (path, None)
        };

        let mut app = Self {
            files: scan(&dir),
            dir,
            selected: None,
            viewer: None,
            read_mtime: None,
            loading: None,
            error: None,
            last_poll: 0.0,
        };
        if let Some(file) = selected.or_else(|| app.files.first().cloned()) {
            app.open(file, &cc.egui_ctx);
        }
        app
    }

    fn open(&mut self, file: PathBuf, ctx: &egui::Context) {
        if self.selected.as_ref() != Some(&file) {
            self.viewer = None;
            self.error = None;
            self.read_mtime = None;
        }
        self.selected = Some(file.clone());
        self.load(file, ctx);
    }

    fn load(&mut self, file: PathBuf, ctx: &egui::Context) {
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mtime = modified(&file);
            let result = std::fs::read_to_string(&file)
                .map_err(|e| e.to_string())
                .and_then(|json| celluloid_core::from_json(&json).map_err(|e| e.to_string()));
            let _ = tx.send((file, mtime, result));
            ctx.request_repaint();
        });
        self.loading = Some(rx);
    }

    fn receive(&mut self, ctx: &egui::Context) {
        let Some(Ok((file, mtime, result))) = self.loading.as_ref().map(|rx| rx.try_recv()) else {
            return;
        };
        self.loading = None;
        if self.selected.as_ref() != Some(&file) {
            return;
        }
        self.read_mtime = mtime;
        match result {
            Ok(animation) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "{} — celluloid",
                    animation.name
                )));
                match &mut self.viewer {
                    Some(viewer) => viewer.replace(animation),
                    None => self.viewer = Some(Viewer::new(animation)),
                }
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
    }

    fn poll(&mut self, ctx: &egui::Context, now: f64) {
        ctx.request_repaint_after(POLL);
        if now - self.last_poll < POLL.as_secs_f64() || self.loading.is_some() {
            return;
        }
        self.last_poll = now;
        self.files = scan(&self.dir);

        let Some(file) = self.selected.clone() else {
            return;
        };
        let mtime = modified(&file);
        if mtime.is_some() && mtime != self.read_mtime {
            self.load(file, ctx);
        }
    }

    fn files_ui(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("ANIMATIONS").small().strong().color(theme::MUTED));
        ui.label(RichText::new(self.dir.display().to_string()).small().color(theme::MUTED));
        ui.separator();
        if self.files.is_empty() {
            ui.label(RichText::new("No .json files here").color(theme::MUTED));
        }
        let mut clicked = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for file in &self.files {
                let name = file.file_stem().unwrap_or_default().to_string_lossy();
                let selected = self.selected.as_ref() == Some(file);
                if ui.selectable_label(selected, name).clicked() && !selected {
                    clicked = Some(file.clone());
                }
            }
        });
        if let Some(file) = clicked {
            self.open(file, ui.ctx());
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = ui.input(|i| i.time);
        self.receive(&ctx);
        self.poll(&ctx, now);

        egui::Panel::left("files")
            .default_size(180.0)
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(12))
            .show(ui, |ui| self.files_ui(ui));

        if let Some(error) = &self.error {
            egui::Panel::top("error")
                .frame(
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(0x5c, 0x1f, 0x24))
                        .inner_margin(8),
                )
                .show(ui, |ui| {
                    ui.label(RichText::new(error).color(egui::Color32::WHITE));
                });
        }

        match &mut self.viewer {
            Some(viewer) => viewer.ui(ui),
            None => {
                egui::CentralPanel::default_margins().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        if self.loading.is_some() {
                            ui.spinner();
                        } else if self.error.is_none() {
                            ui.label(
                                RichText::new("Pick an animation, or run: celluloid file.json")
                                    .color(theme::MUTED),
                            );
                        }
                    });
                });
            }
        }
    }
}

/// `.json` files in `dir`, newest first.
fn scan(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(Option<SystemTime>, PathBuf)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json") && p.is_file())
        .map(|p| (modified(&p), p))
        .collect();
    files.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    files.into_iter().map(|(_, p)| p).collect()
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}
