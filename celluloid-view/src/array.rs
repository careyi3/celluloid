use crate::canvas::{self, color32, font_for, state_colors, Canvas, PanelFrame, PanelView};
use crate::theme;
use celluloid_core::{Animation, ArrayStyle, At, Item, Op, PanelKind, PanelState};
use eframe::egui::{pos2, vec2, Align2, Color32, FontId, Rect, Shape, Stroke, StrokeKind, Ui};
use std::collections::HashMap;

/// Widest an item gets, so short arrays don't turn into slabs.
const MAX_SLOT: f32 = 64.0;
const POINTER_ROW: f32 = 20.0;

/// Scale for an array panel, fixed for the whole animation so bars don't
/// rescale and slots don't resize as items come and go.
#[derive(Clone, Copy)]
pub struct ArrayInfo {
    pub max_len: usize,
    pub lo: f64,
    pub hi: f64,
}

pub fn info(animation: &Animation, p: usize) -> ArrayInfo {
    let Some(PanelKind::Array { values, .. }) = animation.panels.get(p).map(|p| &p.kind) else {
        return ArrayInfo {
            max_len: 0,
            lo: 0.0,
            hi: 1.0,
        };
    };
    let mut len = values.len();
    let mut max_len = len;
    let (mut lo, mut hi) = (0.0f64, 0.0f64);
    let mut see = |v: f64| {
        lo = lo.min(v);
        hi = hi.max(v);
    };
    values.iter().copied().for_each(&mut see);
    for op in animation.frames.iter().flat_map(|f| &f.ops) {
        if op.panel() != Some(p as u16) {
            continue;
        }
        match op {
            Op::Insert { value, .. } => {
                see(*value);
                len += 1;
                max_len = max_len.max(len);
            }
            Op::Remove { .. } => len = len.saturating_sub(1),
            Op::Value { value, .. } => see(*value),
            _ => {}
        }
    }
    if hi <= lo {
        hi = lo + 1.0;
    }
    ArrayInfo { max_len, lo, hi }
}

pub fn show(ui: &mut Ui, view: &mut PanelView, f: PanelFrame, info: ArrayInfo) {
    let PanelState::Array(cur) = f.cur else {
        return;
    };
    let prev = match f.prev {
        Some(PanelState::Array(p)) => Some(p),
        _ => None,
    };
    let PanelKind::Array { style, .. } = f.panel.kind else {
        return;
    };

    let slots = info.max_len.max(1) as f32;
    let width = slots.max((ui.available_width() - 32.0) / MAX_SLOT);
    let world = Rect::from_x_y_ranges(slots / 2.0 - width / 2.0..=slots / 2.0 + width / 2.0, 0.0..=1.0);
    let c = Canvas::begin(ui, view, world, 16.0, true);
    let slot = c.scale;
    let gap = if slot >= 6.0 { (slot * 0.12).clamp(1.0, 6.0) } else { 0.0 };
    let left = |i: f32| c.to_screen(pos2(i, 0.0)).x;

    let roomy = slot >= 14.0;
    let label_band = if roomy { 18.0 } else { 0.0 };
    let value_band = if roomy && style == ArrayStyle::Bars { 16.0 } else { 0.0 };
    let index_band = if roomy { 16.0 } else { 0.0 };
    let pointer_band = f.panel.markers.len().min(3) as f32 * POINTER_ROW;
    let top = c.rect.top() + 10.0 + label_band + value_band;
    let bottom = c.rect.bottom() - 10.0 - index_band - pointer_band;
    let below = if info.lo < 0.0 { value_band } else { 0.0 };
    let height = (bottom - below - top).max(4.0);

    let y_of = |v: f64| bottom - below - ((v - info.lo) / (info.hi - info.lo)) as f32 * height;
    let zero = y_of(0.0);
    let box_size = (slot - gap).min(height).min(72.0);

    let item_rect = |x: f32, v: f64| -> Rect {
        let l = left(x) + gap / 2.0;
        let r = left(x + 1.0) - gap / 2.0;
        match style {
            ArrayStyle::Bars => {
                let mut y = y_of(v);
                if v != 0.0 && (y - zero).abs() < 2.0 {
                    y = zero - 2.0 * v.signum() as f32;
                }
                Rect::from_x_y_ranges(l..=r, y.min(zero)..=y.max(zero))
            }
            ArrayStyle::Boxes => Rect::from_center_size(
                pos2((l + r) / 2.0, (top + bottom) / 2.0),
                vec2(box_size, box_size),
            ),
        }
    };

    let colors = state_colors(f.panel);
    let radius = if slot >= 12.0 { 3.0 } else { 0.0 };
    let before: HashMap<u32, (usize, &Item)> = prev
        .map(|p| p.items.iter().enumerate().map(|(i, it)| (it.id, (i, it))).collect())
        .unwrap_or_default();

    if style == ArrayStyle::Bars && info.lo < 0.0 {
        c.painter.line_segment(
            [pos2(c.rect.left(), zero), pos2(c.rect.right(), zero)],
            Stroke::new(1.0, Color32::from_gray(0x3a)),
        );
    }

    if let Some(p) = prev {
        let now: std::collections::HashSet<u32> = cur.items.iter().map(|it| it.id).collect();
        for (i, it) in p.items.iter().enumerate() {
            if !now.contains(&it.id) {
                let color = canvas::state_color(&colors, it.state).gamma_multiply(1.0 - f.t);
                c.painter.rect_filled(item_rect(i as f32, it.value).shrink(f.t * 4.0), radius, color);
            }
        }
    }

    let mut order: Vec<(usize, bool)> = cur
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| (i, before.get(&it.id).is_some_and(|(j, _)| *j != i)))
        .collect();
    order.sort_by_key(|&(_, moved)| moved);

    let font_value = match style {
        ArrayStyle::Bars => FontId::proportional((slot * 0.38).clamp(9.0, 13.0)),
        ArrayStyle::Boxes => font_for(box_size, 20.0),
    };
    for (i, moved) in order {
        let it = &cur.items[i];
        let old = before.get(&it.id);
        let (x, value, alpha) = match old {
            Some(&(j, o)) => (
                j as f32 + (i as f32 - j as f32) * f.t,
                o.value + (it.value - o.value) * f.t as f64,
                1.0,
            ),
            None if prev.is_some() => (i as f32, it.value * f.t as f64, f.t),
            None => (i as f32, it.value, 1.0),
        };
        let mut r = item_rect(x, value);
        if moved && style == ArrayStyle::Boxes {
            let dir = if old.is_some_and(|&(j, _)| j < i) { -1.0 } else { 1.0 };
            r = r.translate(vec2(0.0, dir * (f.t * std::f32::consts::PI).sin() * box_size * 0.6));
        }
        let color = canvas::tweened(&colors, old.map(|(_, o)| o.state), it.state, f.t)
            .gamma_multiply(alpha);
        c.painter.rect_filled(r, radius, color);

        if f.outline_changes && f.changed.contains(At::Index(i as u32)) && slot >= 4.0 {
            let stroke = Stroke::new(1.5, theme::CHANGED.gamma_multiply(1.0 - 0.5 * f.t));
            c.painter.rect_stroke(r, radius, stroke, StrokeKind::Inside);
        }

        if roomy {
            let text = fmt_value(it.value);
            match style {
                ArrayStyle::Bars => {
                    let (y, anchor) = if it.value < 0.0 {
                        (r.bottom() + 2.0, Align2::CENTER_TOP)
                    } else {
                        (r.top() - 2.0, Align2::CENTER_BOTTOM)
                    };
                    c.painter.text(
                        pos2(r.center().x, y),
                        anchor,
                        text,
                        font_value.clone(),
                        theme::MUTED.gamma_multiply(alpha),
                    );
                }
                ArrayStyle::Boxes => {
                    c.painter.text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        text,
                        font_value.clone(),
                        theme::text_on(color),
                    );
                }
            }
            if let Some(label) = &it.label {
                c.painter.text(
                    pos2(r.center().x, c.rect.top() + 10.0),
                    Align2::CENTER_TOP,
                    label,
                    FontId::proportional(11.0),
                    Color32::from_gray(0xe0).gamma_multiply(alpha),
                );
            }
        }
    }

    if roomy {
        let font = FontId::monospace(10.0);
        for i in 0..cur.items.len() {
            c.painter.text(
                pos2(left(i as f32 + 0.5), bottom + 4.0),
                Align2::CENTER_TOP,
                i,
                font.clone(),
                Color32::from_gray(0x60),
            );
        }
    }

    let mut taken: HashMap<u32, usize> = HashMap::new();
    for (m, def) in f.panel.markers.iter().enumerate() {
        let Some(at) = cur.markers.get(m).copied().flatten() else {
            continue;
        };
        let row = taken.entry(at).or_default();
        let y = bottom + index_band + 4.0 + *row as f32 * POINTER_ROW;
        *row += 1;
        let (x, alpha) = match prev.map(|p| p.markers.get(m).copied().flatten()) {
            Some(Some(from)) => (from as f32 + (at as f32 - from as f32) * f.t, 1.0),
            Some(None) => (at as f32, f.t),
            None => (at as f32, 1.0),
        };
        let cx = left(x + 0.5);
        let color = color32(def.color).gamma_multiply(alpha);
        c.painter.add(Shape::convex_polygon(
            vec![pos2(cx, y), pos2(cx + 6.0, y + 10.0), pos2(cx - 6.0, y + 10.0)],
            color,
            Stroke::NONE,
        ));
        c.painter.text(
            pos2(cx + 8.0, y + 5.0),
            Align2::LEFT_CENTER,
            &def.name,
            FontId::proportional(12.0),
            color,
        );
    }

    if let Some(p) = c.response.hover_pos() {
        let i = c.to_world(p).x.floor();
        if i >= 0.0 && (i as usize) < cur.items.len() {
            let i = i as usize;
            let it = &cur.items[i];
            let r = item_rect(i as f32, it.value);
            c.painter.rect_stroke(r, radius, Stroke::new(1.5, Color32::WHITE), StrokeKind::Outside);
            c.tooltip(&f, At::Index(i as u32), &[format!("value: {}", fmt_value(it.value))]);
        }
    }
}

/// Whole numbers without a decimal point, others to three places.
fn fmt_value(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        assert_eq!(fmt_value(3.0), "3");
        assert_eq!(fmt_value(-12.0), "-12");
        assert_eq!(fmt_value(1.5), "1.5");
        assert_eq!(fmt_value(0.1234), "0.123");
    }
}
