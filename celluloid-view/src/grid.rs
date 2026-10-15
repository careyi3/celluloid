use crate::canvas::{self, color32, font_for, state_colors, Canvas, PanelFrame, PanelView};
use crate::theme;
use celluloid_core::{At, GridState, Panel, PanelState, Point};
use eframe::egui::{
    pos2, vec2, Align2, Color32, ColorImage, FontId, Pos2, Rect, Shape, Stroke, StrokeKind,
    TextureOptions, Ui, Vec2,
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
    let camera = cur
        .camera
        .filter(|_| f.follow_camera)
        .map(|cam| (pos2(cam.center[0] + 0.5, cam.center[1] + 0.5), cam.zoom));
    let shown = canvas::ease_camera(ui, view, canvas::framed(world, camera));
    let c = Canvas::begin(ui, view, shown, 12.0, false);
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
    let gap = if cell >= 6.0 {
        (cell * 0.08).clamp(1.0, 2.0)
    } else {
        0.0
    };
    let radius = if cell >= 14.0 {
        (cell * 0.12).min(4.0)
    } else {
        0.0
    };

    if visible > MAX_RECT_CELLS {
        let stale = !matches!(&view.texture, Some((c, _)) if *c == f.cursor);
        if stale {
            let pixels = cur
                .cells
                .iter()
                .map(|&s| canvas::state_color(&colors, s))
                .collect();
            let image = ColorImage::new([width as usize, height as usize], pixels);
            match &mut view.texture {
                Some((frame, tex)) => {
                    tex.set(image, TextureOptions::NEAREST);
                    *frame = f.cursor;
                }
                None => {
                    let tex = ui
                        .ctx()
                        .load_texture("grid", image, TextureOptions::NEAREST);
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
                let color =
                    canvas::tweened(&colors, prev.and_then(|p| p.cells.get(i).copied()), s, f.t);
                shapes.push(Shape::rect_filled(
                    cell_rect(x, y).shrink(gap / 2.0),
                    radius,
                    color,
                ));
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

    draw_lines(&c, f.panel, cur, prev, f.t);
    if cell >= 14.0 {
        draw_notes(&c, cur, prev, f.t);
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
        let inside = |p: Point| (p[0] - x as f32).abs() <= 0.5 && (p[1] - y as f32).abs() <= 0.5;
        let notes: Vec<String> = cur
            .notes()
            .filter(|n| inside(n.at))
            .map(|n| format!("note ({}, {}): {}", n.at[0], n.at[1], n.text))
            .collect();
        c.tooltip(&f, At::Cell([x as i32, y as i32]), &notes);
    }
}

/// Where a grid point is on screen. Whole numbers are cell centres.
fn point_on(c: &Canvas, [x, y]: Point) -> Pos2 {
    c.to_screen(pos2(x + 0.5, y + 0.5))
}

/// Draw the grid's lines over a dark casing so they read on any cell
/// colour. While tweening, new lines fade in, erased ones fade out and
/// restyled ones change colour, so a contour redrawn every frame
/// cross-fades rather than flickering.
fn draw_lines(c: &Canvas, panel: &Panel, cur: &GridState, prev: Option<&GridState>, t: f32) {
    let colors: Vec<Color32> = panel.lines.iter().map(|l| color32(l.color)).collect();
    let color = |s: u16| colors.get(s as usize).copied().unwrap_or(Color32::MAGENTA);
    let width = (c.scale * 0.1).clamp(1.5, 5.0);
    let casing = width + (c.scale * 0.04).clamp(1.0, 2.0) * 2.0;
    let caps = width > 2.5;
    let bounds = c.rect.expand(casing);
    let mut under = Vec::new();
    let mut over = Vec::new();
    let mut add = |from: Point, to: Point, color: Color32, alpha: f32| {
        let (a, b) = (point_on(c, from), point_on(c, to));
        if !bounds.intersects(Rect::from_two_pos(a, b)) {
            return;
        }
        let dark = theme::CANVAS.gamma_multiply(0.85 * alpha);
        let color = color.gamma_multiply(alpha);
        under.push(Shape::line_segment([a, b], Stroke::new(casing, dark)));
        over.push(Shape::line_segment([a, b], Stroke::new(width, color)));
        if caps {
            for p in [a, b] {
                under.push(Shape::circle_filled(p, casing / 2.0, dark));
                over.push(Shape::circle_filled(p, width / 2.0, color));
            }
        }
    };

    for line in cur.lines() {
        match prev.map(|p| p.line(line.from, line.to)) {
            Some(None) => add(line.from, line.to, color(line.style), t),
            Some(Some(old)) if old.style != line.style => {
                let mixed = color(old.style).lerp_to_gamma(color(line.style), t);
                add(line.from, line.to, mixed, 1.0);
            }
            _ => add(line.from, line.to, color(line.style), 1.0),
        }
    }
    if let Some(prev) = prev {
        for old in prev.lines() {
            if cur.line(old.from, old.to).is_none() {
                add(old.from, old.to, color(old.style), 1.0 - t);
            }
        }
    }
    c.painter.extend(under);
    c.painter.extend(over);
}

/// Draw notes as small pills of text. New notes fade in and removed ones
/// fade out while tweening.
fn draw_notes(c: &Canvas, cur: &GridState, prev: Option<&GridState>, t: f32) {
    let font = FontId::proportional((c.scale * 0.28).clamp(8.0, 15.0));
    let draw = |at: Point, text: &str, alpha: f32| {
        let pos = point_on(c, at);
        if !c.rect.expand(c.scale).contains(pos) {
            return;
        }
        let text_color = Color32::from_gray(0xf0).gamma_multiply(alpha);
        let galley = c
            .painter
            .layout_no_wrap(text.to_owned(), font.clone(), text_color);
        let pill = Rect::from_center_size(pos, galley.size() + vec2(6.0, 2.0));
        c.painter.rect_filled(
            pill,
            pill.height() / 2.0,
            theme::CANVAS.gamma_multiply(0.85 * alpha),
        );
        c.painter
            .galley(pill.center() - galley.size() / 2.0, galley, text_color);
    };
    for note in cur.notes() {
        let fresh = prev.is_some_and(|p| p.note(note.at).is_none());
        draw(note.at, &note.text, if fresh { t } else { 1.0 });
    }
    if let Some(prev) = prev {
        for old in prev.notes() {
            if cur.note(old.at).is_none() {
                draw(old.at, &old.text, 1.0 - t);
            }
        }
    }
}
