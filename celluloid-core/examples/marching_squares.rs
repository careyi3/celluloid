//! cargo run -p celluloid-core --example marching_squares
//! then: celluloid marching_squares.json
//!
//! Traces the outline of some metaballs with marching squares. Each cell
//! is a sample of the field. The scan visits every square of four
//! neighbouring samples, notes its case number at the square's centre and
//! draws that square's piece of the contour between the cell centres.
//! Then the blobs drift and the whole contour is redrawn every frame.

use celluloid_core::{Grid, Recorder};

const WIDTH: usize = 20;
const HEIGHT: usize = 12;
const ISO: f32 = 1.0;

/// Where each blob is at time `t`, and its radius.
fn blobs(t: f32) -> [(f32, f32, f32); 3] {
    [
        (5.0 + 2.0 * t.cos(), 4.5 + t.sin(), 2.4),
        (12.0 - 2.5 * t.sin(), 6.5 + 1.5 * t.cos(), 2.8),
        (15.5, 3.0 + 2.0 * (t * 0.7).sin(), 1.6),
    ]
}

fn field(t: f32) -> Vec<Vec<f32>> {
    let blobs = blobs(t);
    (0..HEIGHT)
        .map(|y| {
            (0..WIDTH)
                .map(|x| {
                    blobs
                        .iter()
                        .map(|&(cx, cy, r)| {
                            let d2 = (x as f32 - cx).powi(2) + (y as f32 - cy).powi(2);
                            r * r / d2.max(0.01)
                        })
                        .sum()
                })
                .collect()
        })
        .collect()
}

/// The case number of the square whose top left sample is `(x, y)`: one
/// bit per corner inside the contour, top left 8, top right 4, bottom
/// right 2, bottom left 1.
fn case(f: &[Vec<f32>], x: usize, y: usize) -> u8 {
    let inside = |x: usize, y: usize| (f[y][x] >= ISO) as u8;
    inside(x, y) << 3 | inside(x + 1, y) << 2 | inside(x + 1, y + 1) << 1 | inside(x, y + 1)
}

/// The contour's pieces inside the square whose top left sample is
/// `(x, y)`, with each end placed along its edge by how far the samples
/// either side are from `ISO`.
fn segments(f: &[Vec<f32>], x: usize, y: usize) -> Vec<[(f32, f32); 2]> {
    let (tl, tr, br, bl) = (f[y][x], f[y][x + 1], f[y + 1][x + 1], f[y + 1][x]);
    let along = |a: f32, b: f32| (ISO - a) / (b - a);
    let (fx, fy) = (x as f32, y as f32);
    let top = (fx + along(tl, tr), fy);
    let right = (fx + 1.0, fy + along(tr, br));
    let bottom = (fx + along(bl, br), fy + 1.0);
    let left = (fx, fy + along(tl, bl));
    let centre_inside = (tl + tr + br + bl) / 4.0 >= ISO;
    match case(f, x, y) {
        1 | 14 => vec![[left, bottom]],
        2 | 13 => vec![[bottom, right]],
        3 | 12 => vec![[left, right]],
        4 | 11 => vec![[top, right]],
        6 | 9 => vec![[top, bottom]],
        7 | 8 => vec![[left, top]],
        5 if centre_inside => vec![[left, top], [bottom, right]],
        5 => vec![[left, bottom], [top, right]],
        10 if centre_inside => vec![[top, right], [left, bottom]],
        10 => vec![[left, top], [bottom, right]],
        _ => vec![],
    }
}

/// Colour every sample by which side of the contour it's on.
fn samples(rec: &mut Recorder, g: Grid, f: &[Vec<f32>], labels: bool) {
    rec.redraw(g, |x, y| if f[y][x] >= ISO { "inside" } else { "outside" });
    for (y, row) in f.iter().enumerate() {
        for (x, &v) in row.iter().enumerate() {
            if labels {
                rec.label(g, (x, y), format!("{:.1}", v.min(9.9)));
            } else {
                rec.clear_label(g, (x, y));
            }
        }
    }
}

pub fn record() -> Recorder {
    let mut rec = Recorder::new("Marching squares");
    rec.frame_delay(40.0);
    let g = rec.grid("field", WIDTH, HEIGHT);
    rec.state(g, "outside", "#1f2a3a");
    rec.state(g, "inside", "#3b6ea5");
    rec.line_color(g, "contour", "#edc948");
    rec.line_color(g, "square", "#e15759");

    let f = field(0.0);
    samples(&mut rec, g, &f, true);
    rec.var("iso", ISO);
    rec.frame("samples: inside the contour where the field is at least 1");

    let mut pieces = 0;
    for y in 0..HEIGHT - 1 {
        for x in 0..WIDTH - 1 {
            rec.clear_lines(g, "square");
            rec.path(
                g,
                [(x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1), (x, y)],
                "square",
            );
            let case = case(&f, x, y);
            if case != 0 && case != 15 {
                rec.note(g, (x as f32 + 0.5, y as f32 + 0.5), case);
            }
            for [a, b] in segments(&f, x, y) {
                rec.line(g, a, b, "contour");
                pieces += 1;
            }
            rec.var("case", case);
            rec.var("pieces", pieces);
            rec.frame(format!("square ({x}, {y}): case {case}"));
        }
    }
    rec.clear_lines(g, "square");
    rec.remove_var("case");
    rec.bookmark("traced");
    rec.frame("the whole contour");

    rec.clear_notes(g);
    for step in 1..=120 {
        let t = step as f32 * 0.05;
        let f = field(t);
        samples(&mut rec, g, &f, false);
        rec.clear_lines(g, "contour");
        let mut pieces = 0;
        for y in 0..HEIGHT - 1 {
            for x in 0..WIDTH - 1 {
                for [a, b] in segments(&f, x, y) {
                    rec.line(g, a, b, "contour");
                    pieces += 1;
                }
            }
        }
        rec.var("pieces", pieces);
        if step == 1 {
            rec.bookmark("drifting");
        }
        rec.frame(format!("t = {t:.2}"));
    }
    rec
}

#[allow(dead_code)]
fn main() {
    record().save("marching_squares.json").unwrap();
    println!("wrote marching_squares.json");
}
