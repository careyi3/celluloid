//! Renders the viewer offscreen so it can be looked at without a window:
//! cargo test -p celluloid-view --test screenshot -- --ignored
//! writes target/screenshots/viewer.png

use celluloid_core::{Animation, Recorder};
use celluloid_view::{apply_theme, Viewer};
use eframe::egui;
use egui_kittest::Harness;
use std::collections::VecDeque;

#[path = "../../celluloid-core/examples/avl.rs"]
mod avl;
#[path = "../../celluloid-core/examples/dijkstra.rs"]
mod dijkstra;
#[path = "../../celluloid-core/examples/filesystem.rs"]
mod filesystem;

const MAZE: &[&str] = &[
    "#####################",
    "#S....#.......#.....#",
    "#.###.#.#####.#.###.#",
    "#.#...#.#...#...#...#",
    "#.#.###.#.#.#####.###",
    "#.#.....#.#.......#.#",
    "#.#######.#######.#.#",
    "#.......#.#.....#...#",
    "#######.#.#.###.###.#",
    "#.......#...#.#.....#",
    "#.#########.#.#####.#",
    "#...........#......E#",
    "#####################",
];

fn bfs(with_queue: bool) -> Animation {
    let grid: Vec<Vec<u8>> = MAZE.iter().map(|r| r.bytes().collect()).collect();
    let (w, h) = (grid[0].len(), grid.len());
    let find = |c| {
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .find(|&(x, y)| grid[y][x] == c)
            .unwrap()
    };
    let (start, end) = (find(b'S'), find(b'E'));

    let mut rec = Recorder::new("Maze BFS");
    let g = rec.grid("maze", w, h);
    rec.state(g, "wall", "#0d0e11");
    rec.state(g, "frontier", "#f28e2b");
    rec.state(g, "seen", "#3b5b84");
    rec.state(g, "path", "#edc948");
    rec.redraw(g, |x, y| if grid[y][x] == b'#' { "wall" } else { "empty" });
    rec.label(g, start, "S");
    rec.label(g, end, "E");
    rec.frame("maze");

    let mut dist = vec![vec![usize::MAX; w]; h];
    let mut from = vec![vec![None; w]; h];
    let mut queue = VecDeque::from([start]);
    let q = with_queue.then(|| {
        let q = rec.array("queue", [0]);
        rec.array_style(q, celluloid_core::ArrayStyle::Boxes);
        q
    });
    dist[start.1][start.0] = 0;
    while let Some((x, y)) = queue.pop_front() {
        if let Some(q) = q {
            rec.remove(q, 0);
            rec.marker(q, "next", 0);
        }
        rec.set(g, (x, y), "seen");
        rec.marker(g, "me", (x, y));
        if (x, y) != start && (x, y) != end {
            rec.label(g, (x, y), dist[y][x]);
        }
        if (x, y) == end {
            rec.bookmark("reached E");
            rec.frame("reached the end");
            break;
        }
        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if grid[ny][nx] != b'#' && dist[ny][nx] == usize::MAX {
                dist[ny][nx] = dist[y][x] + 1;
                from[ny][nx] = Some((x, y));
                rec.set(g, (nx, ny), "frontier");
                queue.push_back((nx, ny));
                if let Some(q) = q {
                    rec.push(q, dist[ny][nx]);
                }
            }
        }
        rec.var("distance", dist[y][x]);
        rec.var("queue", queue.len());
        rec.frame(format!("visit ({x}, {y})"));
    }

    let mut at = Some(end);
    while let Some(p) = at {
        rec.set(g, p, "path");
        at = from[p.1][p.0];
    }
    rec.hide_marker(g, "me");
    rec.bookmark("path");
    rec.frame(format!("shortest path: {} steps", dist[end.1][end.0]));
    rec.finish()
}

fn render(name: &str, animation: Animation, frame: usize, hover: egui::Pos2) {
    render_with(name, animation, frame, hover, false);
}

/// With `step`, press → and capture halfway through the tween.
fn render_with(name: &str, animation: Animation, frame: usize, hover: egui::Pos2, step: bool) {
    let mut viewer = Viewer::new(animation);
    viewer.seek(frame);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_pixels_per_point(2.0)
        .with_step_dt(0.075)
        .build_ui_state(
            |ui, viewer: &mut Viewer| {
                apply_theme(ui.ctx());
                viewer.ui(ui);
            },
            viewer,
        );
    harness.run_steps(4);
    harness.hover_at(hover);
    harness.run_steps(4);
    if step {
        harness.key_press(egui::Key::ArrowRight);
        harness.step();
    }

    let image = harness.render().expect("render");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/screenshots");
    std::fs::create_dir_all(&dir).unwrap();
    image.save(dir.join(format!("{name}.png"))).unwrap();
}

fn quicksort() -> Animation {
    let mut values: Vec<i64> = (0..24).map(|i| (i * 37 + 11) % 41 - 6).collect();
    let mut rec = Recorder::new("Quicksort");
    rec.frame_delay(120.0);
    let a = rec.array("values", values.iter().copied());
    rec.state(a, "pivot", "#e15759");
    rec.state(a, "sorted", "#59a14f");
    rec.frame("unsorted");

    fn sort(rec: &mut Recorder, a: celluloid_core::Array, v: &mut [i64], off: usize) {
        if v.len() <= 1 {
            if v.len() == 1 {
                rec.set(a, off, "sorted");
            }
            return;
        }
        let hi = v.len() - 1;
        rec.marker(a, "lo", off);
        rec.marker(a, "hi", off + hi);
        rec.set(a, off + hi, "pivot");
        let mut i = 0;
        for j in 0..hi {
            rec.marker(a, "i", off + i);
            rec.marker(a, "j", off + j);
            rec.frame(format!("compare {} with pivot {}", v[j], v[hi]));
            if v[j] < v[hi] {
                v.swap(i, j);
                rec.swap(a, off + i, off + j);
                i += 1;
            }
        }
        v.swap(i, hi);
        rec.swap(a, off + i, off + hi);
        rec.set(a, off + i, "sorted");
        rec.frame("place pivot");
        let (left, right) = v.split_at_mut(i);
        sort(rec, a, left, off);
        sort(rec, a, &mut right[1..], off + i + 1);
    }
    sort(&mut rec, a, &mut values, 0);
    rec.bookmark("sorted");
    rec.frame("done");
    rec.finish()
}

fn hex_life() -> Animation {
    use celluloid_core::{hex, Orientation};
    use std::collections::{HashMap, HashSet};

    let mut rec = Recorder::new("Hex life");
    rec.frame_delay(150.0);
    let h = rec.hex("board", Orientation::Pointy);
    rec.state(h, "alive", "#76b7b2");
    rec.state(h, "born", "#edc948");
    let mut alive: HashSet<(i64, i64)> =
        [(0, 0), (1, 0), (0, 1), (-1, 1), (-1, 0), (2, -1), (1, -2), (-2, 2), (0, -1)]
            .into_iter()
            .collect();
    for gen in 0..30 {
        let mut counts: HashMap<(i64, i64), usize> = HashMap::new();
        for &c in &alive {
            for n in hex::neighbours(c) {
                *counts.entry(n).or_default() += 1;
            }
        }
        let next: HashSet<_> = counts
            .iter()
            .filter(|(c, &n)| n == 2 || (alive.contains(c) && n == 3))
            .map(|(c, _)| *c)
            .collect();
        rec.fill(h, "empty");
        rec.set_all(h, alive.iter().copied(), "alive");
        rec.set_all(h, next.difference(&alive).copied(), "born");
        rec.var("generation", gen);
        rec.var("alive", alive.len());
        rec.label(h, (0, 0), "0");
        rec.frame(format!("generation {gen}"));
        alive = next;
    }
    rec.finish()
}

#[test]
#[ignore = "needs a GPU; run explicitly to refresh the screenshots"]
fn screenshots() {
    let animation = bfs(false);
    let frame = animation.frames.len() * 2 / 3;
    render("viewer", animation, frame, egui::pos2(420.0, 330.0));

    let animation = bfs(true);
    let frame = animation.frames.len() / 2;
    render("mixed", animation, frame, egui::pos2(600.0, 640.0));

    let animation = quicksort();
    render("sort", animation, 40, egui::pos2(500.0, 300.0));

    render("hex", hex_life(), 12, egui::pos2(640.0, 380.0));

    let animation = quicksort();
    let before_swap = (1..animation.frames.len())
        .find(|&f| {
            animation.frames[f].ops.iter().any(|op| {
                matches!(op, celluloid_core::Op::Swap { a, b, .. } if a.abs_diff(*b) >= 3)
            })
        })
        .unwrap()
        - 1;
    render_with("swap-bars", animation.clone(), before_swap, egui::pos2(0.0, 0.0), true);
    let mut boxes = animation;
    if let celluloid_core::PanelKind::Array { style, .. } = &mut boxes.panels[0].kind {
        *style = celluloid_core::ArrayStyle::Boxes;
    }
    render_with("swap-boxes", boxes, before_swap, egui::pos2(0.0, 0.0), true);

    let tree = avl::record().finish();
    let rotation = tree.bookmarks().nth(2).unwrap().0;
    render_with("avl-rotating", tree.clone(), rotation - 1, egui::pos2(0.0, 0.0), true);
    let last = tree.frames.len() - 1;
    render("avl", tree, last, egui::pos2(0.0, 0.0));

    let graph = dijkstra::record().finish();
    render("dijkstra-mid", graph.clone(), graph.frames.len() / 2, egui::pos2(0.0, 0.0));
    let last = graph.frames.len() - 1;
    render("dijkstra", graph, last, egui::pos2(0.0, 0.0));

    let fs = filesystem::record().finish();
    let last = fs.frames.len() - 1;
    render("filesystem", fs, last, egui::pos2(0.0, 0.0));
}
