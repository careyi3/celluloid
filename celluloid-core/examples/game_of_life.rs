use celluloid_core::{AnimationData, CellState, Frame};

fn main() {
    let mut animation = AnimationData::new("Game of Life", 20, 20).with_frame_delay(100.0);

    let mut grid = vec![vec![CellState::Empty as u8; 20]; 20];

    grid[10][10] = CellState::Start as u8;
    grid[10][11] = CellState::Start as u8;
    grid[11][10] = CellState::Start as u8;
    grid[11][11] = CellState::Start as u8;

    for step in 0..50 {
        let frame = Frame::new(step, grid.clone(), format!("Step {}", step));
        animation.add_frame(frame);

        grid = next_generation(&grid);
    }

    let json = serde_json::to_string_pretty(&animation).unwrap();
    println!("{}", json);
}

fn next_generation(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let height = grid.len();
    let width = grid[0].len();
    let mut new_grid = grid.to_vec();

    for y in 0..height {
        for x in 0..width {
            let neighbors = count_neighbors(grid, x, y);
            let alive = grid[y][x] == CellState::Start as u8;

            new_grid[y][x] = if alive && (neighbors == 2 || neighbors == 3) {
                CellState::Start as u8
            } else if !alive && neighbors == 3 {
                CellState::Start as u8
            } else {
                CellState::Empty as u8
            };
        }
    }

    new_grid
}

fn count_neighbors(grid: &[Vec<u8>], x: usize, y: usize) -> usize {
    let height = grid.len();
    let width = grid[0].len();
    let mut count = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let ny = (y as i32 + dy).rem_euclid(height as i32) as usize;
            let nx = (x as i32 + dx).rem_euclid(width as i32) as usize;

            if grid[ny][nx] == CellState::Start as u8 {
                count += 1;
            }
        }
    }

    count
}
