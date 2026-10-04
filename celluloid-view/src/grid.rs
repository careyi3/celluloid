use crate::canvas::{self, font_for, state_colors, Canvas, PanelFrame, PanelView};
use crate::theme;
use celluloid_core::{At, PanelState};
use eframe::egui::{
    pos2, vec2, Align2, Color32, ColorImage, Rect, Shape, Stroke, StrokeKind, TextureOptions, Ui,
    Vec2,
};

/// Above this many visible cells, draw the grid as one texture instead of
/// a rectangle per cell. Tweening and gaps are skipped at that density anyway.
const MAX_RECT_CELLS: usize = 40_000;

pub fn show(ui: &mut Ui, view: &mut PanelView, f: PanelFrame) {
    let PanelState::Grid(cur) = f.cur else {
        return;
    };
    let prev = match f.prev {
        Some(PanelState::Grid(p)) => Some(p),
        _ => None,
    };
    let (width, height) = (cur.width, cur.height);
    let world = Rect::from_min_size(pos2(0.0, 0.0), vec2(width as f32, height as f32));
    let c = Canvas::begin(ui, view, world, 12.0, false);
    if width == 0 || height == 0 {
        return;
    }

    let cell = c.scale;
    let cell_rect = |x: u32, y: u32| {
        Rect::from_min_size(c.to_screen(pos2(x as f32, y as f32)), Vec2::splat(cell))
    };
    let seen = c.visible();
    let span = |lo: f32, hi: f32, n: u32| {
        let a = lo.floor().max(0.0) as u32;
        let b = (hi.ceil().max(0.0) as u32).min(n);
        a..b.max(a)
    };
    let xs = span(seen.left(), seen.right(), width);
    let ys = span(seen.top(), seen.bottom(), height);
    let visible = xs.len() * ys.len();

    let colors = state_colors(f.panel);
    let gap = if cell >= 6.0 { (cell * 0.08).clamp(1.0, 2.0) } else { 0.0 };
    let radius = if cell >= 14.0 { (cell * 0.12).min(4.0) } else { 0.0 };

    if visible > MAX_RECT_CELLS {
        let stale = !matches!(&view.texture, Some((c, _)) if *c == f.cursor);
        if stale {
            let pixels = cur.cells.iter().map(|&s| canvas::state_color(&colors, s)).collect();
            let image = ColorImage::new([width as usize, height as usize], pixels);
            match &mut view.texture {
                Some((frame, tex)) => {
                    tex.set(image, TextureOptions::NEAREST);
                    *frame = f.cursor;
                }
                None => {
                    let tex = ui.ctx().load_texture("grid", image, TextureOptions::NEAREST);
                    view.texture = Some((f.cursor, tex));
                }
            }
        }
        if let Some((_, tex)) = &view.texture {
            let full = Rect::from_min_max(c.to_screen(world.min), c.to_screen(world.max));
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            c.painter.image(tex.id(), full, uv, Color32::WHITE);
        }
    } else {
        let mut shapes = Vec::with_capacity(visible);
        for y in ys.clone() {
            for x in xs.clone() {
                let i = (y * width + x) as usize;
                let s = cur.cells[i];
                let color = canvas::tweened(&colors, prev.and_then(|p| p.cells.get(i).copied()), s, f.t);
                shapes.push(Shape::rect_filled(cell_rect(x, y).shrink(gap / 2.0), radius, color));
            }
        }
        c.painter.extend(shapes);

        if f.outline_changes && cell >= 4.0 {
            if let canvas::Changed::Some(cells) = f.changed {
                let stroke = Stroke::new(1.5, theme::CHANGED.gamma_multiply(1.0 - 0.5 * f.t));
                for at in cells {
                    if let At::Cell([x, y]) = *at {
                        let (x, y) = (x as u32, y as u32);
                        if xs.contains(&x) && ys.contains(&y) {
                            let r = cell_rect(x, y).shrink(gap / 2.0);
                            c.painter.rect_stroke(r, radius, stroke, StrokeKind::Inside);
                        }
                    }
                }
            }
        }
    }

    if cell >= 12.0 {
        let font = font_for(cell, 22.0);
        for (&[x, y], text) in &cur.labels {
            if xs.contains(&x) && ys.contains(&y) {
                let bg = canvas::state_color(&colors, cur.cells[(y * width + x) as usize]);
                c.painter.text(
                    cell_rect(x, y).center(),
                    Align2::CENTER_CENTER,
                    text,
                    font.clone(),
                    theme::text_on(bg),
                );
            }
        }
    }

    for (m, def) in f.panel.markers.iter().enumerate() {
        let Some(at) = cur.markers.get(m).copied().flatten() else {
            continue;
        };
        let to = cell_rect(at[0], at[1]).center();
        let from = match prev.map(|p| p.markers.get(m).copied().flatten()) {
            Some(Some(p)) => Some(Some(cell_rect(p[0], p[1]).center())),
            Some(None) => Some(None),
            None => None,
        };
        theme::marker(&c.painter, def, from, to, (cell * 0.32).max(3.0), f.t);
    }

    let hovered = c.response.hover_pos().and_then(|p| {
        let g = c.to_world(p);
        (g.x >= 0.0 && g.y >= 0.0 && (g.x as u32) < width && (g.y as u32) < height)
            .then_some([g.x as u32, g.y as u32])
    });
    if let Some([x, y]) = hovered {
        c.painter.rect_stroke(
            cell_rect(x, y),
            2.0,
            Stroke::new(1.5, Color32::WHITE),
            StrokeKind::Outside,
        );
        c.tooltip(&f, At::Cell([x as i32, y as i32]), &[]);
    }
}
