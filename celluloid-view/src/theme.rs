use eframe::egui::{self, Color32, CornerRadius, Sense, Stroke, Ui, Vec2};

pub const BACKGROUND: Color32 = Color32::from_rgb(0x16, 0x17, 0x1b);
pub const SURFACE: Color32 = Color32::from_rgb(0x1e, 0x20, 0x25);
pub const CANVAS: Color32 = Color32::from_rgb(0x11, 0x12, 0x15);
pub const ACCENT: Color32 = Color32::from_rgb(0x6c, 0x9e, 0xf8);
pub const BOOKMARK: Color32 = Color32::from_rgb(0xed, 0xc9, 0x48);
pub const CHANGED: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const MUTED: Color32 = Color32::from_rgb(0x8b, 0x90, 0x9a);

/// Use the viewer's dark theme for the whole app.
pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        v.panel_fill = SURFACE;
        v.window_fill = SURFACE;
        v.extreme_bg_color = CANVAS;
        v.faint_bg_color = BACKGROUND;
        v.selection.bg_fill = ACCENT.gamma_multiply(0.45);
        v.selection.stroke = Stroke::new(1.0, ACCENT);
        v.hyperlink_color = ACCENT;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_gray(0x2c));
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
    });
}

/// Black or white, whichever reads better on `bg`.
pub fn text_on(bg: Color32) -> Color32 {
    let [r, g, b, _] = bg.to_array();
    let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luma > 150.0 {
        Color32::from_gray(0x10)
    } else {
        Color32::from_gray(0xf0)
    }
}

pub fn swatch(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(2), color);
}

pub fn dot(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 5.0, color);
}

/// A marker dot gliding from `from` to `to`. `from` is `None` when not
/// tweening and `Some(None)` when the marker is just appearing.
pub fn marker(
    painter: &egui::Painter,
    def: &celluloid_core::MarkerDef,
    from: Option<Option<egui::Pos2>>,
    to: egui::Pos2,
    radius: f32,
    t: f32,
) {
    let (center, radius) = match from {
        Some(Some(p)) => (p.lerp(to, t), radius),
        Some(None) => (to, radius * t),
        None => (to, radius),
    };
    let [r, g, b] = def.color.0;
    let color = Color32::from_rgb(r, g, b);
    painter.circle(center, radius, color, Stroke::new(1.5, CANVAS));
    if radius >= 8.0 {
        if let Some(initial) = def.name.chars().next() {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                initial.to_uppercase(),
                egui::FontId::proportional(radius * 1.1),
                text_on(color),
            );
        }
    }
}
