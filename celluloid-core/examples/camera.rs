//! cargo run -p celluloid-core --example camera
//! then: celluloid camera.json
//!
//! Two walkers spiral outwards, one on a grid and one on a hex board, with
//! each panel's camera zooming in to follow its walker and pulling back
//! out at the end.

use celluloid_core::{hex, Orientation, Recorder};

const SIZE: usize = 25;
const RINGS: i64 = 10;
const ZOOM: f32 = 4.0;

fn main() {
    let mut rec = Recorder::new("Camera");
    rec.frame_delay(40.0);
    let g = rec.grid("square spiral", SIZE, SIZE);
    rec.state(g, "visited", "#4e79a7");
    let h = rec.hex("hex spiral", Orientation::Pointy);
    rec.state(h, "visited", "#f28e2b");

    let square = square_spiral();
    let hexes = hex_spiral();
    for step in 0..square.len().max(hexes.len()) {
        let zoom = (1.0 + step as f32 * 0.15).min(ZOOM);
        if let Some(&cell) = square.get(step) {
            rec.set(g, cell, "visited");
            rec.marker(g, "walker", cell);
            rec.camera(g, cell, zoom);
        } else {
            rec.reset_camera(g);
        }
        if let Some(&cell) = hexes.get(step) {
            rec.set(h, cell, "visited");
            rec.marker(h, "walker", cell);
            rec.camera(h, cell, zoom);
        } else {
            rec.reset_camera(h);
        }
        if step == hexes.len() {
            rec.bookmark("hex done");
        }
        rec.var("step", step);
        rec.frame(format!("step {step}"));
    }
    rec.reset_camera(g);
    rec.reset_camera(h);
    rec.bookmark("zoomed out");
    rec.frame("done");

    rec.save("camera.json").unwrap();
    println!("wrote camera.json");
}

/// Every cell of the grid, spiralling out from the middle.
fn square_spiral() -> Vec<(usize, usize)> {
    let (mut x, mut y) = ((SIZE / 2) as i64, (SIZE / 2) as i64);
    let mut cells = vec![(x as usize, y as usize)];
    let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut len = 1;
    'outer: loop {
        for (i, (dx, dy)) in dirs.iter().enumerate() {
            for _ in 0..len {
                x += dx;
                y += dy;
                if x < 0 || y < 0 || x >= SIZE as i64 || y >= SIZE as i64 {
                    break 'outer;
                }
                cells.push((x as usize, y as usize));
            }
            if i % 2 == 1 {
                len += 1;
            }
        }
    }
    cells
}

/// Every hex within `RINGS` of the origin, ring by ring.
fn hex_spiral() -> Vec<(i64, i64)> {
    let mut cells = vec![(0, 0)];
    for k in 1..=RINGS {
        let mut cell = (-k, k);
        for side in 0..6 {
            for _ in 0..k {
                cells.push(cell);
                cell = hex::neighbours(cell)[side];
            }
        }
    }
    cells
}
