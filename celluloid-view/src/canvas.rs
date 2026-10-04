use crate::theme;
use celluloid_core::{At, Panel, PanelState};
use eframe::egui::{
    self, Color32, CursorIcon, Painter, Pos2, Rect, Response, RichText, Sense, TextureHandle, Ui,
    Vec2,
};
use std::collections::HashSet;

/// Zoom and pan for one panel. Zoom 1 fits the whole panel.
pub struct PanelView {
    zoom: f32,
    pan: Vec2,
    /// Cached image of a dense grid, and the frame it shows.
    pub texture: Option<(usize, TextureHandle)>,
    /// World area shown by panels whose camera eases after their content.
    pub cam: Option<Rect>,
}

impl Default for PanelView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            texture: None,
            cam: None,
        }
    }
}

impl PanelView {
    /// Forget cached drawing, e.g. after the animation was reloaded.
    pub fn invalidate(&mut self) {
        self.texture = None;
    }

    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }
}

/// Elements changed by the current frame's ops.
pub enum Changed {
    All,
    Some(HashSet<At>),
}

impl Changed {
    /// Whether to outline `at`. A fill changes everything, and outlining
    /// everything says nothing, so `All` outlines nothing.
    pub fn contains(&self, at: At) -> bool {
        match self {
            Changed::All => false,
            Changed::Some(set) => set.contains(&at),
        }
    }
}

/// What a panel needs to draw one frame.
pub struct PanelFrame<'a> {
    pub panel: &'a Panel,
    pub cur: &'a PanelState,
    /// The previous frame's state while tweening towards `cur`.
    pub prev: Option<&'a PanelState>,
    /// Tween progress, eased, 0..=1.
    pub t: f32,
    pub cursor: usize,
    pub changed: &'a Changed,
    pub outline_changes: bool,
}

/// A panel's drawing area, mapping world coordinates (cells, hex units,
/// array slots) to the screen.
pub struct Canvas {
    pub response: Response,
    pub painter: Painter,
    pub rect: Rect,
    pub scale: f32,
    pub origin: Pos2,
}

impl Canvas {
    /// Allocate the rest of `ui` and fit `world` into it, `margin` points
    /// from the edges. Pinch or ctrl+scroll zooms around the pointer;
    /// scroll and drag pan; double-click resets. With `horizontal`, only
    /// `world`'s width is fitted and zooming and panning are sideways only.
    pub fn begin(ui: &mut Ui, view: &mut PanelView, world: Rect, margin: f32, horizontal: bool) -> Self {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;
        let painter = painter.with_clip_rect(rect);
        painter.rect_filled(rect, 6.0, theme::CANVAS);

        let room = rect.size() - Vec2::splat(2.0 * margin);
        let size = world.size().max(Vec2::splat(1e-3));
        let fit = if horizontal {
            room.x / size.x
        } else {
            (room / size).min_elem()
        }
        .max(1e-3);

        let mask = if horizontal { Vec2::X } else { Vec2::splat(1.0) };
        if response.hovered() {
            let (scroll, zoom_delta) = ui.input(|i| (i.smooth_scroll_delta(), i.zoom_delta()));
            if let Some(pointer) = response.hover_pos() {
                let new_zoom = (view.zoom * zoom_delta).clamp(0.25, 400.0);
                let factor = new_zoom / view.zoom;
                let anchor = (pointer - rect.center()) * mask;
                view.pan = anchor - (anchor - view.pan) * factor;
                view.zoom = new_zoom;
            }
            view.pan += scroll * mask;
        }
        if response.dragged() {
            view.pan += response.drag_delta() * mask;
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }
        if response.double_clicked() {
            view.reset();
        }

        let scale = fit * view.zoom;
        let origin = rect.center() + view.pan - world.center().to_vec2() * scale;
        Self {
            response,
            painter,
            rect,
            scale,
            origin,
        }
    }

    pub fn to_screen(&self, p: Pos2) -> Pos2 {
        self.origin + p.to_vec2() * self.scale
    }

    pub fn to_world(&self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.scale).to_pos2()
    }

    /// The part of world space that's on screen.
    pub fn visible(&self) -> Rect {
        Rect::from_min_max(self.to_world(self.rect.min), self.to_world(self.rect.max))
    }

    /// Show details of the hovered element; `extra` lines go after the state.
    pub fn tooltip(self, f: &PanelFrame, at: At, extra: &[String]) {
        if self.response.dragged() {
            return;
        }
        self.response.on_hover_ui_at_pointer(|ui| {
            ui.label(RichText::new(describe(f.panel, at)).monospace().strong());
            if let Some(def) = f.cur.state_at(at).and_then(|s| f.panel.states.get(s as usize)) {
                ui.horizontal(|ui| {
                    theme::swatch(ui, color32(def.color));
                    ui.label(&def.name);
                });
            }
            for line in extra {
                ui.label(line);
            }
            if let Some(label) = f.cur.label_at(at) {
                ui.label(format!("label: {label}"));
            }
            for (m, def) in f.panel.markers.iter().enumerate() {
                if f.cur.marker_at(m) == Some(at) {
                    ui.horizontal(|ui| {
                        theme::dot(ui, color32(def.color));
                        ui.label(&def.name);
                    });
                }
            }
        });
    }
}

/// How to name an element to a person: a node's name, otherwise its
/// position or index.
pub fn describe(panel: &Panel, at: At) -> String {
    match (panel.kind.nodes(), at) {
        (Some(nodes), At::Index(n)) => nodes
            .get(n as usize)
            .map_or_else(|| at.to_string(), |d| d.name.clone()),
        _ => at.to_string(),
    }
}

pub fn color32(c: celluloid_core::Color) -> Color32 {
    let [r, g, b] = c.0;
    Color32::from_rgb(r, g, b)
}

pub fn state_colors(panel: &Panel) -> Vec<Color32> {
    panel.states.iter().map(|s| color32(s.color)).collect()
}

/// Colour of state `s`, magenta if the file refers to a state that
/// doesn't exist (validation should have caught it).
pub fn state_color(colors: &[Color32], s: u16) -> Color32 {
    colors.get(s as usize).copied().unwrap_or(Color32::MAGENTA)
}

/// Fade from the previous state's colour while tweening.
pub fn tweened(colors: &[Color32], prev: Option<u16>, cur: u16, t: f32) -> Color32 {
    match prev {
        Some(p) if p != cur => state_color(colors, p).lerp_to_gamma(state_color(colors, cur), t),
        _ => state_color(colors, cur),
    }
}

/// Font size that fits text in an element `size` points across.
pub fn font_for(size: f32, max: f32) -> egui::FontId {
    egui::FontId::proportional((size * 0.45).clamp(8.0, max))
}
