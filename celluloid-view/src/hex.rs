use crate::canvas::{self, font_for, state_colors, Canvas, Changed, PanelFrame, PanelView};
use crate::theme;
use celluloid_core::{Animation, At, Op, Orientation, PanelKind, PanelState};
use eframe::egui::{pos2, vec2, Align2, Color32, Pos2, Rangef, Rect, Shape, Stroke, Ui};

const SQRT3: f32 = 1.732_050_8;

/// Don't draw the empty background lattice past this many hexes.
const MAX_LATTICE: usize = 60_000;

/// Centre of axial cell `[q, r]` for hexagons of circumradius 1.
fn center(o: Orientation, [q, r]: [i32; 2]) -> Pos2 {
    point(o, [q as f32, r as f32])
}

/// World position of fractional axial coordinates.
fn point(o: Orientation, [q, r]: [f32; 2]) -> Pos2 {
    match o {
        Orientation::Pointy => pos2(SQRT3 * (q + r / 2.0), 1.5 * r),
        Orientation::Flat => pos2(1.5 * q, SQRT3 * (r + q / 2.0)),
    }
}

/// The axial cell containing world point `p`.
fn cell_at(o: Orientation, p: Pos2) -> [i32; 2] {
    let (qf, rf) = match o {
        Orientation::Pointy => (SQRT3 / 3.0 * p.x - p.y / 3.0, 2.0 / 3.0 * p.y),
        Orientation::Flat => (2.0 / 3.0 * p.x, -p.x / 3.0 + SQRT3 / 3.0 * p.y),
    };
    let sf = -qf - rf;
    let (mut q, mut r, s) = (qf.round(), rf.round(), sf.round());
    let (dq, dr, ds) = ((q - qf).abs(), (r - rf).abs(), (s - sf).abs());
    if dq > dr && dq > ds {
        q = -r - s;
    } else if dr > ds {
        r = -q - s;
    }
    [q as i32, r as i32]
}

fn corners(o: Orientation, c: Pos2, radius: f32) -> Vec<Pos2> {
    let start = match o {
        Orientation::Pointy => 30.0f32,
        Orientation::Flat => 0.0,
    };
    (0..6)
        .map(|k| {
            let a = (start + 60.0 * k as f32).to_radians();
            c + vec2(a.cos(), a.sin()) * radius
        })
        .collect()
}

/// World-space bounds of every cell panel `p` ever touches, so the board
/// keeps one size for the whole animation instead of jumping around.
pub fn extent(animation: &Animation, p: usize) -> Rect {
    let Some(PanelKind::Hex { orientation }) = animation.panels.get(p).map(|p| &p.kind) else {
        return Rect::NOTHING;
    };
    let mut bounds = Rect::NOTHING;
    let mut add = |at: &At| {
        if let At::Cell(c) = at {
            bounds.extend_with(center(*orientation, *c));
        }
    };
    for op in animation.frames.iter().flat_map(|f| &f.ops) {
        match op {
            Op::Set { panel, cells, .. } if *panel as usize == p => cells.iter().for_each(&mut add),
            Op::Label { panel, at, .. } if *panel as usize == p => add(at),
            Op::Marker {
                panel,
                at: Some(at),
                ..
            } if *panel as usize == p => add(at),
            _ => {}
        }
    }
    if bounds == Rect::NOTHING {
        bounds = Rect::from_center_size(Pos2::ZERO, vec2(0.0, 0.0));
    }
    bounds.expand(1.2)
}

pub fn show(ui: &mut Ui, view: &mut PanelView, f: PanelFrame, extent: Rect) {
    let PanelState::Hex(cur) = f.cur else {
        return;
    };
    let prev = match f.prev {
        Some(PanelState::Hex(p)) => Some(p),
        _ => None,
    };
    let PanelKind::Hex { orientation: o } = f.panel.kind else {
        return;
    };
    let camera = cur
        .camera
        .filter(|_| f.follow_camera)
        .map(|cam| (point(o, cam.center), cam.zoom));
    let shown = canvas::ease_camera(ui, view, canvas::framed(extent, camera));
    let c = Canvas::begin(ui, view, shown, 12.0, false);
    let colors = state_colors(f.panel);
    let radius = c.scale;
    let gap = if radius >= 6.0 {
        (radius * 0.08).clamp(1.0, 2.0)
    } else {
        0.0
    };
    let drawn = radius - gap / 2.0;
    let seen = c.visible().intersect(extent);

    let mut cells: Vec<[i32; 2]> = Vec::new();
    if radius >= 3.0 && seen.is_positive() {
        let scaled = |r: Rangef, k: f32| Rangef::new(r.min * k, r.max * k);
        let (rows, cols) = match o {
            Orientation::Pointy => (
                scaled(seen.y_range(), 1.0 / 1.5),
                scaled(seen.x_range(), 1.0 / SQRT3),
            ),
            Orientation::Flat => (
                scaled(seen.x_range(), 1.0 / 1.5),
                scaled(seen.y_range(), 1.0 / SQRT3),
            ),
        };
        let estimate = (rows.span() + 3.0) * (cols.span() + 3.0);
        if estimate < MAX_LATTICE as f32 {
            for a in rows.min.floor() as i32 - 1..=rows.max.ceil() as i32 + 1 {
                let shift = a as f32 / 2.0;
                for b in
                    (cols.min - shift).floor() as i32 - 1..=(cols.max - shift).ceil() as i32 + 1
                {
                    let cell = match o {
                        Orientation::Pointy => [b, a],
                        Orientation::Flat => [a, b],
                    };
                    if extent.contains(center(o, cell)) {
                        cells.push(cell);
                    }
                }
            }
        }
    }
    if cells.is_empty() {
        cells.extend(cur.cells.keys());
        if let Some(p) = prev {
            cells.extend(p.cells.keys().filter(|k| !cur.cells.contains_key(*k)));
        }
    }

    let visible = c.visible().expand(1.0);
    let mut shapes = Vec::with_capacity(cells.len());
    for &cell in &cells {
        let w = center(o, cell);
        if !visible.contains(w) {
            continue;
        }
        let color = canvas::tweened(&colors, prev.map(|p| p.get(cell)), cur.get(cell), f.t);
        let pts = corners(o, c.to_screen(w), drawn);
        shapes.push(Shape::convex_polygon(pts, color, Stroke::NONE));
    }
    c.painter.extend(shapes);

    if f.outline_changes && radius >= 4.0 {
        if let Changed::Some(changed) = f.changed {
            let stroke = Stroke::new(1.5, theme::CHANGED.gamma_multiply(1.0 - 0.5 * f.t));
            for at in changed {
                if let At::Cell(cell) = *at {
                    let w = center(o, cell);
                    if visible.contains(w) {
                        let pts = corners(o, c.to_screen(w), drawn - 0.75);
                        c.painter.add(Shape::closed_line(pts, stroke));
                    }
                }
            }
        }
    }

    if radius >= 10.0 {
        let font = font_for(radius * 1.6, 22.0);
        for (&cell, text) in &cur.labels {
            let w = center(o, cell);
            if visible.contains(w) {
                let bg = canvas::state_color(&colors, cur.get(cell));
                c.painter.text(
                    c.to_screen(w),
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
        let to = c.to_screen(center(o, at));
        let from = prev.map(|p| {
            p.markers
                .get(m)
                .copied()
                .flatten()
                .map(|p| c.to_screen(center(o, p)))
        });
        theme::marker(&c.painter, def, from, to, (radius * 0.55).max(3.0), f.t);
    }

    if let Some(p) = c.response.hover_pos() {
        let cell = cell_at(o, c.to_world(p));
        let pts = corners(o, c.to_screen(center(o, cell)), radius);
        c.painter
            .add(Shape::closed_line(pts, Stroke::new(1.5, Color32::WHITE)));
        c.tooltip(&f, At::Cell(cell), &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_to_cell_round_trips() {
        for o in [Orientation::Pointy, Orientation::Flat] {
            for q in -4..=4 {
                for r in -4..=4 {
                    let c = center(o, [q, r]);
                    assert_eq!(cell_at(o, c), [q, r]);
                    assert_eq!(cell_at(o, c + vec2(0.3, -0.2)), [q, r], "{o:?} {q},{r}");
                }
            }
        }
    }
}
