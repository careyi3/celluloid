use crate::canvas::{Canvas, PanelFrame, PanelView};
use crate::nodes::{self, EdgeView, Positions, Scene};
use celluloid_core::{PanelState, TreeState};
use eframe::egui::{pos2, vec2, Rect, Ui};

const ROW: f32 = 1.3;
/// Space between separate trees in a forest, in node widths.
const FOREST_GAP: f32 = 0.6;

/// Where each node goes. Binary trees (no node with more than two slots)
/// are placed by in-order position, so a lone right child sits to the
/// right and a rotation reads as nodes swinging round. Other trees pack
/// leaves left to right with each parent centred over its children.
pub fn layout(t: &TreeState) -> Positions {
    let mut pos: Positions = vec![None; t.nodes.len()];
    let binary = t.nodes.iter().all(|n| n.children.len() <= 2);
    let mut next_x = 0.0f32;

    fn inorder(t: &TreeState, n: u32, depth: usize, x: &mut f32, pos: &mut Positions) {
        if pos[n as usize].is_some() {
            return;
        }
        let kids = &t.nodes[n as usize].children;
        if let Some(Some(l)) = kids.first() {
            inorder(t, *l, depth + 1, x, pos);
        }
        pos[n as usize] = Some(pos2(*x, depth as f32 * ROW));
        *x += 1.0;
        if let Some(Some(r)) = kids.get(1) {
            inorder(t, *r, depth + 1, x, pos);
        }
    }

    fn packed(t: &TreeState, n: u32, depth: usize, x: &mut f32, pos: &mut Positions) {
        if pos[n as usize].is_some() {
            return;
        }
        pos[n as usize] = Some(pos2(*x, depth as f32 * ROW));
        let kids: Vec<u32> = t.nodes[n as usize].children.iter().flatten().copied().collect();
        if kids.is_empty() {
            *x += 1.0;
            return;
        }
        for &k in &kids {
            packed(t, k, depth + 1, x, pos);
        }
        let first = pos[kids[0] as usize].unwrap().x;
        let last = pos[*kids.last().unwrap() as usize].unwrap().x;
        pos[n as usize] = Some(pos2((first + last) / 2.0, depth as f32 * ROW));
    }

    for root in t.roots() {
        if binary {
            inorder(t, root, 0, &mut next_x, &mut pos);
        } else {
            packed(t, root, 0, &mut next_x, &mut pos);
        }
        next_x += FOREST_GAP;
    }
    pos
}

fn edges(t: &TreeState) -> Vec<EdgeView<'static>> {
    t.nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.present)
        .flat_map(|(p, n)| {
            n.children.iter().flatten().map(move |&c| EdgeView {
                a: p as u32,
                b: c,
                state: 0,
                label: None,
            })
        })
        .collect()
}

pub fn show(ui: &mut Ui, view: &mut PanelView, f: PanelFrame) {
    let PanelState::Tree(cur) = f.cur else {
        return;
    };
    let prev = match f.prev {
        Some(PanelState::Tree(p)) => Some(p),
        _ => None,
    };
    let Some(defs) = f.panel.kind.nodes() else {
        return;
    };

    let cur_pos = layout(cur);
    let prev_pos = prev.map(layout);

    let mut target = nodes::bounds(&cur_pos);
    if let Some(p) = &prev_pos {
        target = target.union(nodes::bounds(p));
    }
    if !target.is_finite() || target == Rect::NOTHING {
        target = Rect::from_center_size(pos2(0.0, 0.0), vec2(1.0, 1.0));
    }
    let target = Rect::from_center_size(target.center(), target.size().max(vec2(6.0, 4.0)))
        .expand(0.8);
    let cam = match view.cam {
        Some(cam) => {
            let dt = ui.input(|i| i.stable_dt).min(0.1);
            let k = 1.0 - (-dt * 8.0).exp();
            let next = Rect::from_min_max(cam.min.lerp(target.min, k), cam.max.lerp(target.max, k));
            if (next.min - target.min).length() + (next.max - target.max).length() > 0.01 {
                ui.ctx().request_repaint();
                next
            } else {
                target
            }
        }
        None => target,
    };
    view.cam = Some(cam);

    let c = Canvas::begin(ui, view, cam, 16.0, false);
    let states = |t: &TreeState| t.nodes.iter().map(|n| n.state).collect::<Vec<_>>();
    let scene = Scene {
        nodes: defs,
        cur: &cur_pos,
        prev: prev_pos.as_ref(),
        edges: edges(cur),
        prev_edges: prev.map(edges).unwrap_or_default(),
        directed: true,
        states: states(cur),
        prev_states: prev.map(states),
        labels: cur.nodes.iter().map(|n| n.label.as_deref()).collect(),
        markers: cur.markers.clone(),
        prev_markers: prev.map(|p| p.markers.clone()),
    };
    let scene = Scene {
        directed: false,
        ..scene
    };
    nodes::paint(c, &f, &scene);
}

#[cfg(test)]
mod tests {
    use super::*;
    use celluloid_core::{Recorder, Timeline};

    fn tree(build: impl FnOnce(&mut Recorder, celluloid_core::Tree)) -> TreeState {
        let mut rec = Recorder::new("t");
        let t = rec.tree("t");
        build(&mut rec, t);
        let mut tl = Timeline::new(rec.finish());
        match &tl.seek(usize::MAX).panels[0] {
            PanelState::Tree(t) => t.clone(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn binary_inorder() {
        let t = tree(|rec, t| {
            rec.left(t, 4, 2);
            rec.right(t, 2, 3);
            rec.right(t, 4, 5);
        });
        let pos = layout(&t);
        let x = |n: usize| pos[n].unwrap().x;
        assert!(x(1) < x(2) && x(2) < x(0) && x(0) < x(3));
        assert_eq!(pos[2].unwrap().y, 2.0 * ROW);
    }

    #[test]
    fn nary_centres_parents() {
        let t = tree(|rec, t| {
            rec.add_child(t, "r", "a");
            rec.add_child(t, "r", "b");
            rec.add_child(t, "r", "c");
        });
        let pos = layout(&t);
        assert_eq!(pos[0].unwrap().x, pos[2].unwrap().x, "r over the middle child");
    }
}
