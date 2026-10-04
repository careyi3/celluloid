//! cargo run -p celluloid-core --example game_of_life
//! then: celluloid game_of_life.json

use celluloid_core::Recorder;

const SIZE: usize = 40;

fn main() {
    let mut rec = Recorder::new("Game of Life");
    rec.frame_delay(100.0);
    let g = rec.grid("life", SIZE, SIZE);
    rec.state(g, "alive", "#59a14f");
    rec.state(g, "born", "#edc948");

    let mut alive = vec![vec![false; SIZE]; SIZE];
    let c = SIZE / 2;
    for (x, y) in [(c, c - 1), (c + 1, c - 1), (c - 1, c), (c, c), (c, c + 1)] {
        alive[y][x] = true;
    }

    for generation in 0..200 {
        let population = alive.iter().flatten().filter(|&&a| a).count();
        rec.var("generation", generation);
        rec.var("population", population);
        if generation == 100 {
            rec.bookmark("generation 100");
        }

        let next = next_generation(&alive);
        rec.redraw(g, |x, y| match (alive[y][x], next[y][x]) {
            (true, _) => "alive",
            (false, true) => "born",
            (false, false) => "empty",
        });
        rec.frame(format!("generation {generation}"));
        alive = next;
    }

    rec.save("game_of_life.json").unwrap();
    println!("wrote game_of_life.json");
}

fn next_generation(alive: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut next = vec![vec![false; SIZE]; SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut neighbours = 0;
            for dy in [SIZE - 1, 0, 1] {
                for dx in [SIZE - 1, 0, 1] {
                    if (dx, dy) != (0, 0) && alive[(y + dy) % SIZE][(x + dx) % SIZE] {
                        neighbours += 1;
                    }
                }
            }
            next[y][x] = matches!((alive[y][x], neighbours), (true, 2 | 3) | (false, 3));
        }
    }
    next
}
