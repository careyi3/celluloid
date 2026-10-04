//! cargo run -p celluloid-core --example dijkstra
//! then: celluloid dijkstra.json

use celluloid_core::Recorder;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

const EDGES: &[(&str, &str, u32)] = &[
    ("A", "B", 4),
    ("A", "C", 2),
    ("B", "C", 5),
    ("B", "D", 10),
    ("C", "E", 3),
    ("E", "D", 4),
    ("D", "F", 11),
    ("E", "G", 7),
    ("G", "F", 2),
    ("F", "H", 3),
    ("G", "H", 9),
    ("C", "G", 12),
    ("H", "I", 1),
    ("D", "I", 15),
];

pub fn record() -> Recorder {
    let mut rec = Recorder::new("Dijkstra");
    rec.frame_delay(400.0);
    let g = rec.graph("roads", false);
    rec.state(g, "frontier", "#f28e2b");
    rec.state(g, "done", "#3b5b84");
    rec.state(g, "path", "#edc948");
    rec.state(g, "tree", "#59a14f");

    let mut adj: HashMap<&str, Vec<(&str, u32)>> = HashMap::new();
    for &(a, b, w) in EDGES {
        rec.edge_label(g, a, b, w);
        adj.entry(a).or_default().push((b, w));
        adj.entry(b).or_default().push((a, w));
    }
    rec.frame("graph");

    let (start, goal) = ("A", "I");
    let mut dist: HashMap<&str, u32> = HashMap::from([(start, 0)]);
    let mut via: HashMap<&str, &str> = HashMap::new();
    let mut heap = BinaryHeap::from([Reverse((0, start))]);
    rec.label(g, start, "0");

    while let Some(Reverse((d, node))) = heap.pop() {
        if d > dist[node] {
            continue;
        }
        rec.set(g, node, "done");
        rec.marker(g, "at", node);
        rec.var("distance", d);
        rec.frame(format!("visit {node} at {d}"));
        if node == goal {
            rec.bookmark("reached goal");
            break;
        }
        for &(next, w) in &adj[node] {
            let nd = d + w;
            if dist.get(next).is_none_or(|&old| nd < old) {
                if let Some(old) = via.get(next) {
                    rec.set_edge(g, *old, next, "default");
                }
                dist.insert(next, nd);
                via.insert(next, node);
                heap.push(Reverse((nd, next)));
                rec.set(g, next, "frontier");
                rec.set_edge(g, node, next, "tree");
                rec.label(g, next, nd);
            }
        }
        rec.var("queue", heap.len());
        rec.frame(format!("relax edges from {node}"));
    }

    let mut at = goal;
    rec.set(g, at, "path");
    while let Some(&prev) = via.get(at) {
        rec.set(g, prev, "path");
        rec.set_edge(g, prev, at, "path");
        at = prev;
    }
    rec.hide_marker(g, "at");
    rec.bookmark("shortest path");
    rec.frame(format!("shortest path {start} to {goal}: {}", dist[goal]));
    rec
}

#[allow(dead_code)]
fn main() {
    record().save("dijkstra.json").unwrap();
    println!("wrote dijkstra.json");
}
