//! Drawing shared by tree and graph panels: given where each node sits in
//! the previous and current frame, draw edges, nodes, labels and markers,
//! tweening between the two.

use crate::canvas::{self, state_colors, Canvas, PanelFrame};
use crate::theme;
use celluloid_core::{At, NodeDef};
use eframe::egui::{vec2, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, Vec2};
use std::collections::HashMap;

/// Node radius in world units. Layouts space nodes about 1 unit apart.
pub const RADIUS: f32 = 0.34;

const EDGE: Color32 = Color32::from_rgb(0x4a, 0x4f, 0x58);

/// Node positions in world space, indexed by node; `None` where the node
/// doesn't exist.
pub type Positions = Vec<Option<Pos2>>;

pub struct EdgeView<'a> {
    pub a: u32,
    pub b: u32,
    pub state: u16,
    pub label: Option<&'a str>,
}

pub struct Scene<'a> {
    pub nodes: &'a [NodeDef],
    pub cur: &'a Positions,
    /// Positions at the previous frame, while tweening.
    pub prev: Option<&'a Positions>,
    pub edges: Vec<EdgeView<'a>>,
    pub prev_edges: Vec<EdgeView<'a>>,
    pub directed: bool,
    /// State of each node now, and before while tweening.
    pub states: Vec<u16>,
    pub prev_states: Option<Vec<u16>>,
    pub labels: Vec<Option<&'a str>>,
    pub markers: Vec<Option<u32>>,
    pub prev_markers: Option<Vec<Option<u32>>>,
}

/// Bounding box of the nodes that exist, in world space.
pub fn bounds(positions: &Positions) -> Rect {
    positions
        .iter()
        .flatten()
        .fold(Rect::NOTHING, |r, &p| r.union(Rect::from_center_size(p, Vec2::ZERO)))
}

pub fn paint(c: Canvas, f: &PanelFrame, s: &Scene) {
    let t = f.t;
    let r = RADIUS * c.scale;
    let colors = state_colors(f.panel);

    let at = |n: u32| -> Option<(Pos2, f32)> {
        let cur = s.cur.get(n as usize).copied().flatten();
        let prev = s.prev.map(|p| p.get(n as usize).copied().flatten());
        match (prev, cur) {
            (None, Some(q)) => Some((c.to_screen(q), 1.0)),
            (Some(Some(p)), Some(q)) => Some((c.to_screen(p.lerp(q, t)), 1.0)),
            (Some(None), Some(q)) => Some((c.to_screen(q), t)),
            (Some(Some(p)), None) => Some((c.to_screen(p), 1.0 - t)),
            _ => None,
        }
    };

    let key = |a: u32, b: u32| if s.directed || a <= b { (a, b) } else { (b, a) };
    let before: HashMap<(u32, u32), &EdgeView> =
        s.prev_edges.iter().map(|e| (key(e.a, e.b), e)).collect();
    let now: HashMap<(u32, u32), &EdgeView> = s.edges.iter().map(|e| (key(e.a, e.b), e)).collect();
    let edge_color = |state: u16| {
        if state == 0 {
            EDGE
        } else {
            canvas::state_color(&colors, state)
        }
    };
    let mut labels = Vec::new();
    let mut draw_edge = |e: &EdgeView, alpha: f32, prev_state: Option<u16>| {
        let (Some((pa, aa)), Some((pb, ab))) = (at(e.a), at(e.b)) else {
            return;
        };
        let alpha = alpha * aa.min(ab);
        let d = pb - pa;
        if d.length() < 2.0 * r + 1.0 {
            return;
        }
        let dir = d.normalized();
        let (from, to) = (pa + dir * r, pb - dir * r);
        let color = match prev_state {
            Some(p) if p != e.state => edge_color(p).lerp_to_gamma(edge_color(e.state), t),
            _ => edge_color(e.state),
        }
        .gamma_multiply(alpha);
        let width = if e.state == 0 { 1.5 } else { 2.5 };
        c.painter.line_segment([from, to], Stroke::new(width, color));
        if s.directed {
            let size = (r * 0.5).clamp(5.0, 12.0);
            let back = to - dir * size;
            let side = vec2(-dir.y, dir.x) * size * 0.5;
            c.painter.add(Shape::convex_polygon(
                vec![to, back + side, back - side],
                color,
                Stroke::NONE,
            ));
        }
        if let Some(label) = e.label {
            labels.push(((from + (to - from) * 0.5), label.to_owned(), alpha));
        }
    };
    for e in &s.edges {
        let old = before.get(&key(e.a, e.b));
        let alpha = if s.prev.is_some() && old.is_none() { t } else { 1.0 };
        draw_edge(e, alpha, old.map(|o| o.state));
    }
    if s.prev.is_some() {
        for e in &s.prev_edges {
            if !now.contains_key(&key(e.a, e.b)) {
                draw_edge(e, 1.0 - t, None);
            }
        }
    }
    if r >= 6.0 {
        let font = FontId::proportional((r * 0.7).clamp(9.0, 13.0));
        for (p, text, alpha) in labels {
            let galley = c.painter.layout_no_wrap(text, font.clone(), Color32::from_gray(0xd0));
            let rect = Rect::from_center_size(p, galley.size() + vec2(6.0, 2.0));
            c.painter.rect_filled(rect, 3.0, theme::CANVAS.gamma_multiply(alpha));
            c.painter.galley(rect.min + vec2(3.0, 1.0), galley, Color32::from_gray(0xd0));
        }
    }

    let name_font = FontId::proportional((r * 0.8).clamp(8.0, 18.0));
    let names_fit = r >= 9.0
        && s.nodes.iter().all(|d| {
            c.painter
                .layout_no_wrap(d.name.clone(), name_font.clone(), Color32::WHITE)
                .size()
                .x
                <= 2.0 * r - 4.0
        });
    let label_font = FontId::proportional((r * 0.6).clamp(9.0, 13.0));
    let n_nodes = s.cur.len().max(s.prev.map_or(0, |p| p.len()));
    for n in 0..n_nodes as u32 {
        let Some((p, alpha)) = at(n) else {
            continue;
        };
        let cur = s.states.get(n as usize).copied().unwrap_or(0);
        let prev = s.prev_states.as_ref().and_then(|ps| ps.get(n as usize).copied());
        let fill = canvas::tweened(&colors, prev, cur, t).gamma_multiply(alpha);
        c.painter.circle(p, r, fill, Stroke::new(1.5, theme::CANVAS));
        if f.outline_changes && f.changed.contains(At::Index(n)) && s.cur[n as usize].is_some() {
            let stroke = Stroke::new(1.5, theme::CHANGED.gamma_multiply(1.0 - 0.5 * t));
            c.painter.circle_stroke(p, r + 3.0, stroke);
        }
        if names_fit {
            if let Some(def) = s.nodes.get(n as usize) {
                let color = theme::text_on(fill).gamma_multiply(alpha);
                c.painter.text(p, Align2::CENTER_CENTER, &def.name, name_font.clone(), color);
            }
        }
        if r >= 6.0 {
            if let Some(Some(label)) = s.labels.get(n as usize) {
                c.painter.text(
                    p + vec2(0.0, r + 3.0),
                    Align2::CENTER_TOP,
                    label,
                    label_font.clone(),
                    Color32::from_gray(0xd8).gamma_multiply(alpha),
                );
            }
        }
    }

    let mut taken: HashMap<u32, usize> = HashMap::new();
    for (m, def) in f.panel.markers.iter().enumerate() {
        let Some(node) = s.markers.get(m).copied().flatten() else {
            continue;
        };
        let k = taken.entry(node).or_default();
        let angle = (-45.0f32 + 50.0 * *k as f32).to_radians();
        *k += 1;
        let offset = vec2(angle.cos(), angle.sin()) * r * 1.05;
        let screen = |n: u32, pos: &Positions| pos.get(n as usize).copied().flatten().map(|w| c.to_screen(w) + offset);
        let Some(to) = screen(node, s.cur) else {
            continue;
        };
        let from = match (&s.prev_markers, s.prev) {
            (Some(pm), Some(pp)) => Some(pm.get(m).copied().flatten().and_then(|n| screen(n, pp))),
            _ => None,
        };
        theme::marker(&c.painter, def, from, to, (r * 0.45).max(4.0), t);
    }

    if let Some(pointer) = c.response.hover_pos() {
        let hit = (0..s.cur.len() as u32)
            .filter(|&n| s.cur[n as usize].is_some())
            .filter_map(|n| at(n).map(|(p, _)| (n, p.distance(pointer))))
            .filter(|&(_, d)| d <= r.max(6.0))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((n, _)) = hit {
            let (p, _) = at(n).unwrap();
            c.painter.circle_stroke(p, r + 1.5, Stroke::new(1.5, Color32::WHITE));
            c.tooltip(f, At::Index(n), &[]);
        }
    }
}
