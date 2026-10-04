//! cargo run -p celluloid-core --example hex_life
//! then: celluloid hex_life.json

use celluloid_core::{hex, Orientation, Recorder};
use std::collections::{HashMap, HashSet};

fn main() {
    let mut rec = Recorder::new("Hex life");
    rec.frame_delay(150.0);
    let h = rec.hex("board", Orientation::Pointy);
    rec.state(h, "alive", "#76b7b2");
    rec.state(h, "born", "#edc948");

    let seed = ["..##..", ".#.##.", "##..#.", ".#.#..", "..#..."];
    let mut alive: HashSet<(i64, i64)> = HashSet::new();
    for (row, line) in seed.iter().enumerate() {
        for (col, c) in line.chars().enumerate() {
            if c == '#' {
                alive.insert(hex::odd_r(col as i64, row as i64));
            }
        }
    }

    let mut previous: HashSet<(i64, i64)> = HashSet::new();
    for generation in 0..60 {
        let mut neighbours: HashMap<(i64, i64), usize> = HashMap::new();
        for &cell in &alive {
            for n in hex::neighbours(cell) {
                *neighbours.entry(n).or_default() += 1;
            }
        }
        let next: HashSet<_> = neighbours
            .iter()
            .filter(|(c, &n)| n == 2 || (alive.contains(c) && (n == 3 || n == 4)))
            .map(|(c, _)| *c)
            .collect();

        let shown: HashSet<_> = alive.union(&next).copied().collect();
        rec.set_all(h, previous.difference(&shown).copied(), "empty");
        rec.set_all(h, alive.iter().copied(), "alive");
        rec.set_all(h, next.difference(&alive).copied(), "born");
        previous = shown;
        rec.var("generation", generation);
        rec.var("alive", alive.len());
        rec.frame(format!("generation {generation}"));
        alive = next;
    }

    rec.save("hex_life.json").unwrap();
    println!("wrote hex_life.json");
}
