use crate::array::{self, ArrayInfo};
use crate::canvas::{color32, describe, Changed, PanelFrame, PanelView};
use crate::graph::{self, GraphLayout};
use crate::{grid, hex, theme, tree};
use celluloid_core::{Animation, At, Op, PanelKind, PanelState, State, Timeline};
use eframe::egui::{
    self, vec2, Align, Color32, Key, Layout, Rect, RichText, Sense, Stroke, Ui, UiBuilder,
};
use std::collections::HashSet;

const SPEEDS: [f32; 11] = [0.1, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0];
const NORMAL_SPEED: usize = 3;

/// Plays one animation with transport controls and an inspector. Give it
/// an [`Animation`] and call [`Viewer::ui`] every frame.
pub struct Viewer {
    timeline: Timeline,
    views: Vec<PanelView>,
    /// Per panel, layout that holds for the whole animation.
    layouts: Vec<PanelLayout>,
    playing: bool,
    speed: usize,
    /// Fractional frames owed to playback.
    clock: f64,
    tween: Option<Tween>,
    cache: Option<FrameCache>,
    outline_changes: bool,
    follow_camera: bool,
    /// Whether any grid or hex panel has a recorded camera to follow.
    has_camera: bool,
    /// Show the transport bar and inspector. Off leaves just the panels,
    /// for hosts that drive playback themselves.
    pub show_controls: bool,
}

/// Fixed per-panel layout, worked out once from the whole animation.
enum PanelLayout {
    Grid,
    Hex(Rect),
    Array(ArrayInfo),
    Tree,
    Graph(GraphLayout),
}

impl PanelLayout {
    fn all(animation: &Animation) -> Vec<Self> {
        (0..animation.panels.len())
            .map(|p| match animation.panels[p].kind {
                PanelKind::Grid { .. } => PanelLayout::Grid,
                PanelKind::Hex { .. } => PanelLayout::Hex(hex::extent(animation, p)),
                PanelKind::Array { .. } => PanelLayout::Array(array::info(animation, p)),
                PanelKind::Tree { .. } => PanelLayout::Tree,
                PanelKind::Graph { .. } => PanelLayout::Graph(graph::layout(animation, p)),
            })
            .collect()
    }
}

struct Tween {
    from: State,
    start: f64,
    duration: f64,
}

/// Things derived from the current frame, recomputed when it changes.
struct FrameCache {
    cursor: usize,
    changed: Vec<Changed>,
    changed_vars: HashSet<String>,
    /// Per panel, how many elements are in each state. `None` for the
    /// empty state of hex boards, which have no fixed size.
    counts: Vec<Vec<Option<usize>>>,
}

impl Viewer {
    /// Start on frame 0, paused.
    pub fn new(animation: Animation) -> Self {
        let views = animation
            .panels
            .iter()
            .map(|_| PanelView::default())
            .collect();
        let has_camera = has_camera(&animation);
        Self {
            layouts: PanelLayout::all(&animation),
            timeline: Timeline::new(animation),
            views,
            playing: false,
            speed: NORMAL_SPEED,
            clock: 0.0,
            tween: None,
            cache: None,
            outline_changes: true,
            follow_camera: true,
            has_camera,
            show_controls: true,
        }
    }

    /// Swap in a new version of the animation (e.g. after a re-run),
    /// keeping the current frame, zoom and play state where possible.
    pub fn replace(&mut self, animation: Animation) {
        let cursor = self.timeline.cursor();
        if animation.panels.len() != self.views.len() {
            self.views = animation
                .panels
                .iter()
                .map(|_| PanelView::default())
                .collect();
        }
        self.views.iter_mut().for_each(PanelView::invalidate);
        self.layouts = PanelLayout::all(&animation);
        self.has_camera = has_camera(&animation);
        self.timeline = Timeline::new(animation);
        self.timeline.seek(cursor);
        self.tween = None;
        self.cache = None;
        if self.timeline.cursor() + 1 >= self.timeline.len() {
            self.playing = false;
        }
    }

    /// The animation being played.
    pub fn animation(&self) -> &Animation {
        self.timeline.animation()
    }

    /// The frame on screen, counting from 0.
    pub fn cursor(&self) -> usize {
        self.timeline.cursor()
    }

    /// Number of frames.
    pub fn frames(&self) -> usize {
        self.timeline.len()
    }

    /// Jump to a frame, counting from 0.
    pub fn seek(&mut self, frame: usize) {
        self.go_to(frame, 0.0, false);
    }

    /// Whether playback is running.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Play or pause. Playing from the last frame starts again from 0.
    pub fn set_playing(&mut self, playing: bool) {
        if playing && self.at_end() {
            self.seek(0);
        }
        self.playing = playing && !self.timeline.is_empty();
        self.clock = 0.0;
    }

    fn at_end(&self) -> bool {
        self.timeline.cursor() + 1 >= self.timeline.len()
    }

    fn last(&self) -> usize {
        self.timeline.len().saturating_sub(1)
    }

    fn frame_secs(&self) -> f64 {
        self.animation().frame_delay_ms.max(1.0) / 1000.0 / SPEEDS[self.speed] as f64
    }

    fn go_to(&mut self, frame: usize, now: f64, animate: bool) {
        let frame = frame.min(self.last());
        if frame == self.timeline.cursor() {
            return;
        }
        let duration = if self.playing {
            (self.frame_secs() * 0.9).min(0.35)
        } else {
            0.15
        };
        let step = frame == self.timeline.cursor() + 1;
        self.tween = (animate && step && duration >= 0.03).then(|| Tween {
            from: self.timeline.state().clone(),
            start: now,
            duration,
        });
        self.timeline.seek(frame);
    }

    fn bookmark_near(&self, forward: bool) -> Option<usize> {
        let cursor = self.timeline.cursor();
        let mut marks = self.animation().bookmarks().map(|(i, _)| i);
        if forward {
            marks.find(|&i| i > cursor)
        } else {
            marks.filter(|&i| i < cursor).last()
        }
    }

    fn cache(&mut self) -> &FrameCache {
        let cursor = self.timeline.cursor();
        if self.cache.as_ref().map(|c| c.cursor) != Some(cursor) {
            self.cache = Some(self.build_cache());
        }
        self.cache.as_ref().unwrap()
    }

    fn build_cache(&self) -> FrameCache {
        let animation = self.animation();
        let mut changed: Vec<Changed> = animation
            .panels
            .iter()
            .map(|_| Changed::Some(HashSet::new()))
            .collect();
        let mut changed_vars = HashSet::new();
        if let Some(frame) = animation.frames.get(self.timeline.cursor()) {
            for op in &frame.ops {
                if let Op::Var { name, .. } = op {
                    changed_vars.insert(name.clone());
                }
                if let Some(c) = op.panel().and_then(|p| changed.get_mut(p as usize)) {
                    note_change(c, op);
                }
            }
        }

        let counts = animation
            .panels
            .iter()
            .zip(&self.timeline.state().panels)
            .map(|(def, state)| {
                let mut counts = vec![Some(0); def.states.len()];
                let mut count = |s: u16| {
                    if let Some(Some(c)) = counts.get_mut(s as usize) {
                        *c += 1;
                    }
                };
                match state {
                    PanelState::Grid(g) => g.cells.iter().for_each(|&s| count(s)),
                    PanelState::Hex(h) => h.cells.values().for_each(|&s| count(s)),
                    PanelState::Array(a) => a.items.iter().for_each(|it| count(it.state)),
                    PanelState::Tree(celluloid_core::TreeState { nodes, .. })
                    | PanelState::Graph(celluloid_core::GraphState { nodes, .. }) => nodes
                        .iter()
                        .filter(|n| n.present)
                        .for_each(|n| count(n.state)),
                }
                if let (PanelState::Hex(_), Some(empty)) = (state, counts.first_mut()) {
                    *empty = None;
                }
                counts
            })
            .collect();

        FrameCache {
            cursor: self.timeline.cursor(),
            changed,
            changed_vars,
            counts,
        }
    }

    /// Draw the viewer into the rest of `ui`. Call this every frame.
    pub fn ui(&mut self, ui: &mut Ui) {
        let now = ui.input(|i| i.time);
        self.handle_keys(ui, now);
        self.advance(ui, now);

        if self.timeline.is_empty() {
            ui.centered_and_justified(|ui| ui.label("This animation has no frames."));
            return;
        }

        if self.show_controls {
            egui::Panel::bottom("transport")
                .frame(
                    egui::Frame::side_top_panel(ui.style())
                        .inner_margin(egui::Margin::symmetric(12, 8)),
                )
                .show(ui, |ui| self.transport_ui(ui, now));
            egui::Panel::right("inspector")
                .default_size(260.0)
                .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(12))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| self.inspector_ui(ui));
                });
        }
        egui::CentralPanel::default_margins()
            .frame(egui::Frame::new().fill(theme::BACKGROUND).inner_margin(12))
            .show(ui, |ui| self.panels_ui(ui, now));
    }

    fn advance(&mut self, ui: &Ui, now: f64) {
        if let Some(t) = &self.tween {
            if now - t.start >= t.duration {
                self.tween = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
        if !self.playing {
            return;
        }
        let dt = ui.input(|i| i.stable_dt).min(0.1) as f64;
        self.clock += dt / self.frame_secs();
        let steps = self.clock.floor();
        if steps >= 1.0 {
            self.clock -= steps;
            let target = self.timeline.cursor() + steps as usize;
            if target >= self.last() {
                self.go_to(self.last(), now, steps == 1.0);
                self.playing = false;
            } else {
                self.go_to(target, now, steps == 1.0);
            }
        }
        ui.ctx().request_repaint();
    }

    fn handle_keys(&mut self, ui: &Ui, now: f64) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        let (keys, shift) = ui.input(|i| {
            let keys: Vec<Key> = [
                Key::Space,
                Key::ArrowLeft,
                Key::ArrowRight,
                Key::Home,
                Key::End,
                Key::OpenBracket,
                Key::CloseBracket,
                Key::Minus,
                Key::Equals,
                Key::Plus,
                Key::Num0,
                Key::O,
                Key::C,
            ]
            .into_iter()
            .filter(|&k| i.key_pressed(k))
            .collect();
            (keys, i.modifiers.shift)
        });
        let jump = if shift { 10 } else { 1 };
        for key in keys {
            let cursor = self.timeline.cursor();
            match key {
                Key::Space => self.set_playing(!self.playing),
                Key::ArrowRight => self.go_to(cursor + jump, now, true),
                Key::ArrowLeft => self.go_to(cursor.saturating_sub(jump), now, false),
                Key::Home => self.seek(0),
                Key::End => self.seek(self.last()),
                Key::OpenBracket => {
                    if let Some(f) = self.bookmark_near(false) {
                        self.seek(f);
                    }
                }
                Key::CloseBracket => {
                    if let Some(f) = self.bookmark_near(true) {
                        self.seek(f);
                    }
                }
                Key::Minus => self.speed = self.speed.saturating_sub(1),
                Key::Equals | Key::Plus => self.speed = (self.speed + 1).min(SPEEDS.len() - 1),
                Key::Num0 => self.views.iter_mut().for_each(PanelView::reset),
                Key::O => self.outline_changes = !self.outline_changes,
                Key::C => self.follow_camera = !self.follow_camera,
                _ => {}
            }
        }
    }

    fn panels_ui(&mut self, ui: &mut Ui, now: f64) {
        self.cache();
        let t = self
            .tween
            .as_ref()
            .map(|tw| ((now - tw.start) / tw.duration).clamp(0.0, 1.0) as f32)
            .map(|t| t * t * (3.0 - 2.0 * t))
            .unwrap_or(1.0);
        let animation = self.timeline.animation();
        let state = self.timeline.state();
        let cache = self.cache.as_ref().unwrap();
        let cursor = self.timeline.cursor();
        let outline_changes = self.outline_changes;
        let follow_camera = self.follow_camera;
        let tween = self.tween.as_ref();

        let full = ui.available_rect_before_wrap();
        let rects = arrange(full, &self.layouts);
        let titled = animation.panels.len() > 1;
        for (i, ((panel, view), layout)) in animation
            .panels
            .iter()
            .zip(self.views.iter_mut())
            .zip(&self.layouts)
            .enumerate()
        {
            let mut ui = ui.new_child(
                UiBuilder::new()
                    .id_salt(("panel", i))
                    .max_rect(rects[i])
                    .layout(Layout::top_down(Align::Min)),
            );
            if titled {
                ui.label(RichText::new(&panel.name).small().color(theme::MUTED));
            }
            let f = PanelFrame {
                panel,
                cur: &state.panels[i],
                prev: tween.map(|tw| &tw.from.panels[i]),
                t,
                cursor,
                changed: &cache.changed[i],
                outline_changes,
                follow_camera,
            };
            match layout {
                PanelLayout::Grid => grid::show(&mut ui, view, f),
                PanelLayout::Hex(extent) => hex::show(&mut ui, view, f, *extent),
                PanelLayout::Array(info) => array::show(&mut ui, view, f, *info),
                PanelLayout::Tree => tree::show(&mut ui, view, f),
                PanelLayout::Graph(layout) => graph::show(&mut ui, view, f, layout),
            }
        }
        ui.allocate_rect(full, Sense::hover());
    }

    fn transport_ui(&mut self, ui: &mut Ui, now: f64) {
        self.scrubber(ui, now);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let cursor = self.timeline.cursor();
            let big = |s: &str| RichText::new(s).size(16.0);

            if ui
                .button(big("⏮"))
                .on_hover_text("First frame (Home)")
                .clicked()
            {
                self.seek(0);
            }
            if ui
                .button(big("⏴").size(12.0))
                .on_hover_text("Step back (←)")
                .clicked()
            {
                self.seek(cursor.saturating_sub(1));
            }
            let play = if self.playing { "⏸" } else { "⏵" };
            let play = egui::Button::new(big(play).color(Color32::WHITE))
                .fill(theme::ACCENT.gamma_multiply(0.8))
                .min_size(vec2(44.0, 0.0));
            if ui.add(play).on_hover_text("Play / pause (Space)").clicked() {
                self.set_playing(!self.playing);
            }
            if ui
                .button(big("⏵").size(12.0))
                .on_hover_text("Step forward (→)")
                .clicked()
            {
                self.go_to(cursor + 1, now, true);
            }
            if ui
                .button(big("⏭"))
                .on_hover_text("Last frame (End)")
                .clicked()
            {
                self.seek(self.last());
            }

            ui.separator();
            ui.label(
                RichText::new(format!("{} / {}", cursor + 1, self.timeline.len())).monospace(),
            );
            ui.separator();

            if ui.small_button("−").on_hover_text("Slower (-)").clicked() {
                self.speed = self.speed.saturating_sub(1);
            }
            ui.label(RichText::new(format!("{}×", SPEEDS[self.speed])).monospace());
            if ui.small_button("+").on_hover_text("Faster (=)").clicked() {
                self.speed = (self.speed + 1).min(SPEEDS.len() - 1);
            }

            let message = &self.animation().frames[cursor].message;
            if !message.is_empty() {
                ui.separator();
                ui.add(egui::Label::new(RichText::new(message).strong()).truncate());
            }
        });
    }

    fn scrubber(&mut self, ui: &mut Ui, now: f64) {
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        let last = self.last().max(1) as f32;
        let track = Rect::from_center_size(rect.center(), vec2(rect.width() - 16.0, 4.0));
        let x_of = |f: usize| track.left() + track.width() * (f as f32 / last);

        painter.rect_filled(track, 2.0, Color32::from_gray(0x33));
        let mut done = track;
        done.set_right(x_of(self.timeline.cursor()));
        painter.rect_filled(done, 2.0, theme::ACCENT);

        let marks: Vec<(usize, String)> = self
            .animation()
            .bookmarks()
            .map(|(i, b)| (i, b.to_owned()))
            .collect();
        for (i, _) in &marks {
            let x = x_of(*i);
            painter.line_segment(
                [
                    egui::pos2(x, rect.top() + 3.0),
                    egui::pos2(x, rect.bottom() - 3.0),
                ],
                Stroke::new(2.0, theme::BOOKMARK),
            );
        }
        painter.circle(
            egui::pos2(x_of(self.timeline.cursor()), rect.center().y),
            7.0,
            Color32::WHITE,
            Stroke::new(2.0, theme::ACCENT),
        );

        let frame_at =
            |x: f32| (((x - track.left()) / track.width()).clamp(0.0, 1.0) * last).round() as usize;
        if response.clicked() || response.dragged() {
            if let Some(p) = response.interact_pointer_pos() {
                self.go_to(frame_at(p.x), now, false);
            }
        }
        if let Some(p) = response.hover_pos() {
            let f = frame_at(p.x);
            let near = marks
                .iter()
                .find(|(i, _)| (x_of(*i) - p.x).abs() < 5.0)
                .map(|(i, b)| (*i, b.clone()));
            response.on_hover_ui_at_pointer(|ui| match near {
                Some((i, name)) => {
                    ui.label(format!("frame {}: {name}", i + 1));
                }
                None => {
                    ui.label(format!("frame {}", f + 1));
                }
            });
        }
    }

    fn inspector_ui(&mut self, ui: &mut Ui) {
        self.cache();
        let mut seek_to = None;
        {
            let animation = self.timeline.animation();
            let state = self.timeline.state();
            let cache = self.cache.as_ref().unwrap();
            let cursor = self.timeline.cursor();

            ui.heading(&animation.name);
            let frame = &animation.frames[cursor];
            if !frame.message.is_empty() {
                ui.label(&frame.message);
            }
            if let Some(b) = &frame.bookmark {
                ui.label(RichText::new(format!("★ {b}")).color(theme::BOOKMARK));
            }

            if !state.vars.is_empty() {
                section(ui, "Variables");
                egui::Grid::new("vars")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        for (name, value) in &state.vars {
                            ui.label(RichText::new(name).color(theme::MUTED));
                            let mut text = RichText::new(value).monospace();
                            if cache.changed_vars.contains(name) {
                                text = text.color(theme::ACCENT);
                            }
                            ui.add(egui::Label::new(text).wrap());
                            ui.end_row();
                        }
                    });
            }

            for (i, panel) in animation.panels.iter().enumerate() {
                section(ui, &panel.name);
                let ps = &state.panels[i];
                for (s, def) in panel.states.iter().enumerate() {
                    ui.horizontal(|ui| {
                        theme::swatch(ui, color32(def.color));
                        ui.label(&def.name);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(
                                    cache.counts[i][s].map_or("–".into(), |c| c.to_string()),
                                )
                                .monospace()
                                .color(theme::MUTED),
                            );
                        });
                    });
                }
                for (m, def) in panel.markers.iter().enumerate() {
                    ui.horizontal(|ui| {
                        theme::dot(ui, color32(def.color));
                        ui.label(&def.name);
                        let at = match ps.marker_at(m) {
                            Some(at) => describe(panel, at),
                            None => "hidden".into(),
                        };
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(at).monospace().color(theme::MUTED));
                        });
                    });
                }
            }

            let marks: Vec<_> = animation.bookmarks().collect();
            if !marks.is_empty() {
                section(ui, "Bookmarks");
                for (f, name) in marks {
                    let text = format!("{:>5}  {name}", f + 1);
                    let selected = f == cursor;
                    if ui
                        .selectable_label(selected, RichText::new(text).monospace())
                        .clicked()
                    {
                        seek_to = Some(f);
                    }
                }
            }
        }
        if let Some(f) = seek_to {
            self.seek(f);
        }

        section(ui, "View");
        ui.checkbox(&mut self.outline_changes, "Outline what changed (O)");
        if self.has_camera {
            ui.checkbox(&mut self.follow_camera, "Follow recorded camera (C)");
        }
        ui.collapsing("Shortcuts", |ui| {
            egui::Grid::new("keys").num_columns(2).show(ui, |ui| {
                for (k, what) in [
                    ("Space", "play / pause"),
                    ("← →", "step (shift: 10)"),
                    ("Home End", "first / last"),
                    ("[ ]", "previous / next bookmark"),
                    ("- =", "slower / faster"),
                    ("0", "reset zoom"),
                    ("C", "follow recorded camera"),
                    ("pinch, ctrl+scroll", "zoom"),
                    ("drag, scroll", "pan"),
                    ("double-click", "reset zoom"),
                ] {
                    ui.label(RichText::new(k).monospace());
                    ui.label(RichText::new(what).color(theme::MUTED));
                    ui.end_row();
                }
            });
        });
    }
}

fn has_camera(animation: &Animation) -> bool {
    animation
        .frames
        .iter()
        .flat_map(|f| &f.ops)
        .any(|op| matches!(op, Op::Camera { .. }))
}

fn section(ui: &mut Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(
        RichText::new(title.to_uppercase())
            .small()
            .strong()
            .color(theme::MUTED),
    );
    ui.separator();
}

/// Split `full` between panels: grids and hex boards side by side on top,
/// arrays in strips underneath (or sharing the space if there's nothing else).
fn arrange(full: Rect, layouts: &[PanelLayout]) -> Vec<Rect> {
    const GAP: f32 = 12.0;
    let strips = layouts
        .iter()
        .filter(|l| matches!(l, PanelLayout::Array(_)))
        .count();
    let areas = layouts.len() - strips;
    let strip_h = if areas == 0 {
        (full.height() - GAP * strips.saturating_sub(1) as f32) / strips.max(1) as f32
    } else {
        (full.height() * 0.3 / strips.max(1) as f32).clamp(110.0, 240.0)
    };
    let strips_h = if strips == 0 {
        0.0
    } else {
        strips as f32 * strip_h + GAP * strips as f32
    };
    let area_h = (full.height() - strips_h).max(0.0);
    let area_w = (full.width() - GAP * areas.saturating_sub(1) as f32) / areas.max(1) as f32;

    let (mut a, mut s) = (0, 0);
    layouts
        .iter()
        .map(|l| match l {
            PanelLayout::Array(_) => {
                let top = if areas == 0 {
                    full.top()
                } else {
                    full.top() + area_h + GAP
                } + s as f32 * (strip_h + GAP);
                s += 1;
                Rect::from_min_size(egui::pos2(full.left(), top), vec2(full.width(), strip_h))
            }
            _ => {
                let left = full.left() + a as f32 * (area_w + GAP);
                a += 1;
                Rect::from_min_size(egui::pos2(left, full.top()), vec2(area_w, area_h))
            }
        })
        .collect()
}

/// Record which elements `op` changed, tracking how array inserts, removes
/// and moves shift the positions noted so far.
fn note_change(changed: &mut Changed, op: &Op) {
    let Changed::Some(set) = changed else {
        return;
    };
    let shift = |set: &mut HashSet<At>, f: &dyn Fn(u32) -> Option<u32>| {
        *set = set
            .drain()
            .filter_map(|at| match at {
                At::Index(i) => f(i).map(At::Index),
                cell => Some(cell),
            })
            .collect();
    };
    match op {
        Op::Set { cells, .. } => set.extend(cells.iter().copied()),
        Op::Fill { .. } => *changed = Changed::All,
        Op::Value { at, .. } => {
            set.insert(At::Index(*at));
        }
        Op::Swap { a, b, .. } => {
            set.insert(At::Index(*a));
            set.insert(At::Index(*b));
        }
        Op::Insert { at, .. } => {
            let at = *at;
            shift(set, &|i| Some(if i >= at { i + 1 } else { i }));
            set.insert(At::Index(at));
        }
        Op::Remove { at, .. } => {
            let at = *at;
            shift(set, &|i| match i.cmp(&at) {
                std::cmp::Ordering::Less => Some(i),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(i - 1),
            });
        }
        Op::Move { from, to, .. } => {
            let (from, to) = (*from, *to);
            shift(set, &|i| {
                if i == from {
                    return None;
                }
                let i = if i > from { i - 1 } else { i };
                Some(if i >= to { i + 1 } else { i })
            });
            set.insert(At::Index(to));
        }
        Op::Child { child: Some(c), .. } => {
            set.insert(At::Index(*c));
        }
        Op::Label { .. }
        | Op::Marker { .. }
        | Op::Camera { .. }
        | Op::Var { .. }
        | Op::AddNode { .. }
        | Op::RemoveNode { .. }
        | Op::Child { child: None, .. }
        | Op::AddEdge { .. }
        | Op::RemoveEdge { .. }
        | Op::EdgeSet { .. }
        | Op::EdgeLabel { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_changes_follow_shifts() {
        let mut changed = Changed::Some(HashSet::new());
        let ops = [
            Op::Value {
                panel: 0,
                at: 2,
                value: 1.0,
            },
            Op::Insert {
                panel: 0,
                at: 0,
                value: 1.0,
            },
            Op::Move {
                panel: 0,
                from: 3,
                to: 1,
            },
            Op::Remove { panel: 0, at: 0 },
        ];
        for op in &ops {
            note_change(&mut changed, op);
        }
        let Changed::Some(set) = changed else {
            panic!()
        };
        assert_eq!(set, HashSet::from([At::Index(0)]));
    }
}
