use crate::canvas::{Canvas, PanelFrame, PanelView};
use crate::nodes::{self, EdgeView, Positions, Scene};
use celluloid_core::{Animation, GraphState, Op, PanelKind, PanelState};
use eframe::egui::{pos2, vec2, Pos2, Rect, Ui, Vec2};
use std::collections::{BTreeMap, BTreeSet};

/// Ideal edge length, in world units (nodes are about 0.7 across).
const SPRING: f32 = 2.0;

/// Positions for every node a graph ever has, worked out once so nodes
/// stay put for the whole animation.
pub struct GraphLayout {
    pub pos: Vec<Pos2>,
    pub extent: Rect,
}

pub fn layout(animation: &Animation, p: usize) -> GraphLayout {
    let Some(PanelKind::Graph { nodes, .. }) = animation.panels.get(p).map(|p| &p.kind) else {
        return GraphLayout {
            pos: Vec::new(),
            extent: Rect::NOTHING,
        };
    };
    let n = nodes.len();
    let mut edges: BTreeSet<(usize, usize)> = BTreeSet::new();
    for op in animation.frames.iter().flat_map(|f| &f.ops) {
        let pair = match op {
            Op::AddEdge { panel, a, b } | Op::EdgeLabel { panel, a, b, .. }
                if *panel as usize == p =>
            {
                vec![(*a, *b)]
            }
            Op::EdgeSet { panel, edges, .. } if *panel as usize == p => {
                edges.iter().map(|[a, b]| (*a, *b)).collect()
            }
            _ => continue,
        };
        for (a, b) in pair {
            let (a, b) = (a as usize, b as usize);
            if a != b && a < n && b < n {
                edges.insert((a.min(b), a.max(b)));
            }
        }
    }
    let fixed: Vec<Option<Pos2>> = nodes
        .iter()
        .map(|d| d.pos.map(|[x, y]| pos2(x, y)))
        .collect();
    let pos = force_layout(n, &edges, &fixed);
    let extent = pos.iter().fold(Rect::NOTHING, |r, &p| {
        r.union(Rect::from_center_size(p, Vec2::ZERO))
    });
    let extent = if extent == Rect::NOTHING {
        Rect::from_center_size(Pos2::ZERO, vec2(4.0, 4.0))
    } else {
        Rect::from_center_size(extent.center(), extent.size().max(vec2(4.0, 3.0))).expand(1.0)
    };
    GraphLayout { pos, extent }
}

/// Spring-electric layout. Pinned nodes are scaled into layout units and
/// held still; the rest start on a spiral (so the result is the same
/// every run) and settle around them. Repulsion only looks at nearby
/// nodes through a grid, so it stays fast for a few thousand nodes.
fn force_layout(n: usize, edges: &BTreeSet<(usize, usize)>, fixed: &[Option<Pos2>]) -> Vec<Pos2> {
    let k = SPRING;
    let mut pos = vec![Pos2::ZERO; n];

    let pinned: Vec<usize> = (0..n).filter(|&i| fixed[i].is_some()).collect();
    let mut centre = Pos2::ZERO;
    if !pinned.is_empty() {
        let bounds = pinned.iter().fold(Rect::NOTHING, |r, &i| {
            r.union(Rect::from_center_size(fixed[i].unwrap(), Vec2::ZERO))
        });
        let side = bounds.size().max_elem().max(1e-6);
        let scale = (pinned.len() as f32).sqrt().max(1.0) * k * 1.2 / side;
        for &i in &pinned {
            pos[i] = ((fixed[i].unwrap() - bounds.center()) * scale).to_pos2();
        }
        if pinned.len() == n {
            return pos;
        }
        centre = Pos2::ZERO;
    }
    let mut j = 0;
    for (i, p) in pos.iter_mut().enumerate() {
        if fixed[i].is_none() {
            let a = j as f32 * 2.399_963;
            *p = centre + vec2(a.cos(), a.sin()) * k * 0.7 * ((j + 1) as f32).sqrt();
            j += 1;
        }
    }

    let neighbours: Vec<(usize, usize)> = edges.iter().copied().collect();
    let iterations = if n > 2000 { 150 } else { 300 };
    let cutoff = 3.0 * k;
    let start_temp = k * (n as f32).sqrt() * 0.3;
    let mut disp = vec![Vec2::ZERO; n];
    for it in 0..iterations {
        disp.iter_mut().for_each(|d| *d = Vec2::ZERO);

        let mut grid: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
        for (i, p) in pos.iter().enumerate() {
            grid.entry(((p.x / cutoff).floor() as i32, (p.y / cutoff).floor() as i32))
                .or_default()
                .push(i);
        }
        for (&(gx, gy), here) in &grid {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let Some(there) = grid.get(&(gx + dx, gy + dy)) else {
                        continue;
                    };
                    for &a in here {
                        for &b in there {
                            if a >= b {
                                continue;
                            }
                            let mut d = pos[a] - pos[b];
                            if d.length_sq() < 1e-6 {
                                d = vec2((a as f32).sin(), (b as f32).cos()) * 0.01;
                            }
                            let dist = d.length();
                            if dist < cutoff {
                                let f = d / dist * (k * k / dist);
                                disp[a] += f;
                                disp[b] -= f;
                            }
                        }
                    }
                }
            }
        }
        for &(a, b) in &neighbours {
            let mid = pos[a] + (pos[b] - pos[a]) * 0.5;
            let cell = (
                (mid.x / cutoff).floor() as i32,
                (mid.y / cutoff).floor() as i32,
            );
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let Some(near) = grid.get(&(cell.0 + dx, cell.1 + dy)) else {
                        continue;
                    };
                    for &i in near {
                        if i == a || i == b {
                            continue;
                        }
                        let d = pos[i] - mid;
                        let dist = d.length().max(0.05);
                        if dist < k {
                            let f = d / dist * (k * k / dist) * 0.5;
                            disp[i] += f;
                            disp[a] -= f * 0.5;
                            disp[b] -= f * 0.5;
                        }
                    }
                }
            }
        }
        for &(a, b) in &neighbours {
            let d = pos[a] - pos[b];
            let dist = d.length().max(1e-3);
            let f = d / dist * (dist * dist / k);
            disp[a] -= f;
            disp[b] += f;
        }

        let temp = start_temp * (1.0 - it as f32 / iterations as f32) + 0.01;
        for i in 0..n {
            if fixed[i].is_some() {
                continue;
            }
            let d = disp[i] - pos[i].to_vec2() * 0.05;
            let len = d.length();
            if len > 0.0 {
                pos[i] += d / len * len.min(temp);
            }
        }
    }
    pos
}

fn edges(g: &GraphState) -> Vec<EdgeView<'_>> {
    g.edges
        .iter()
        .map(|(&[a, b], e)| EdgeView {
            a,
            b,
            state: e.state,
            label: e.label.as_deref(),
        })
        .collect()
}

fn present(g: &GraphState, layout: &GraphLayout) -> Positions {
    g.nodes
        .iter()
        .enumerate()
        .map(|(i, n)| n.present.then(|| layout.pos.get(i).copied()).flatten())
        .collect()
}

pub fn show(ui: &mut Ui, view: &mut PanelView, f: PanelFrame, layout: &GraphLayout) {
    let PanelState::Graph(cur) = f.cur else {
        return;
    };
    let prev = match f.prev {
        Some(PanelState::Graph(p)) => Some(p),
        _ => None,
    };
    let Some(defs) = f.panel.kind.nodes() else {
        return;
    };
    let cur_pos = present(cur, layout);
    let prev_pos = prev.map(|p| present(p, layout));
    let c = Canvas::begin(ui, view, layout.extent, 16.0, false);
    let states = |g: &GraphState| g.nodes.iter().map(|n| n.state).collect::<Vec<_>>();
    let scene = Scene {
        nodes: defs,
        cur: &cur_pos,
        prev: prev_pos.as_ref(),
        edges: edges(cur),
        prev_edges: prev.map(edges).unwrap_or_default(),
        directed: cur.directed,
        states: states(cur),
        prev_states: prev.map(states),
        labels: cur.nodes.iter().map(|n| n.label.as_deref()).collect(),
        markers: cur.markers.clone(),
        prev_markers: prev.map(|p| p.markers.clone()),
    };
    nodes::paint(c, &f, &scene);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_stable_and_spread() {
        let edges: BTreeSet<(usize, usize)> = (0..9).map(|i| (i, i + 1)).collect();
        let a = force_layout(10, &edges, &[None; 10]);
        let b = force_layout(10, &edges, &[None; 10]);
        assert_eq!(a, b, "deterministic");
        for i in 0..10 {
            for j in i + 1..10 {
                assert!(a[i].distance(a[j]) > 0.5, "{i} and {j} overlap");
            }
        }
        for (x, y) in edges {
            assert!(a[x].distance(a[y]) < 3.0 * SPRING, "edge {x}-{y} stretched");
        }
    }

    #[test]
    fn pinned_nodes_keep_their_shape() {
        let fixed = [Some(pos2(0.0, 0.0)), Some(pos2(10.0, 0.0)), None];
        let edges = BTreeSet::from([(0, 2), (1, 2)]);
        let p = force_layout(3, &edges, &fixed);
        assert_eq!(p[0].y, p[1].y);
        assert!(
            p[0].x < p[2].x && p[2].x < p[1].x,
            "free node settles between"
        );
    }
}
