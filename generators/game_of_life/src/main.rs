use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Serialize, Deserialize)]
struct AnimationData {
    name: String,
    algorithm: String,
    created_at: String,
    grid_config: GridConfig,
    metadata: Metadata,
    frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize)]
struct GridConfig {
    width: usize,
    height: usize,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    total_frames: usize,
    has_path: bool,
    frame_delay_ms: f64,
}

#[derive(Serialize, Deserialize)]
struct Frame {
    step: usize,
    grid: Vec<Vec<u8>>,
    message: String,
    highlighted: Vec<(usize, usize)>,
}

fn random_grid(width: usize, height: usize, density: f64) -> Vec<Vec<bool>> {
    let mut rng = rand::thread_rng();
    (0..height)
        .map(|_| (0..width).map(|_| rng.gen::<f64>() < density).collect())
        .collect()
}

fn glider_grid(width: usize, height: usize) -> Vec<Vec<bool>> {
    let mut grid = vec![vec![false; width]; height];

    let start_x = width / 4;
    let start_y = height / 4;

    grid[start_y][start_x + 1] = true;
    grid[start_y + 1][start_x + 2] = true;
    grid[start_y + 2][start_x] = true;
    grid[start_y + 2][start_x + 1] = true;
    grid[start_y + 2][start_x + 2] = true;

    grid
}

fn count_neighbors(grid: &Vec<Vec<bool>>, x: usize, y: usize) -> u8 {
    let height = grid.len();
    let width = grid[0].len();
    let mut count = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = (x as i32 + dx + width as i32) % width as i32;
            let ny = (y as i32 + dy + height as i32) % height as i32;

            if grid[ny as usize][nx as usize] {
                count += 1;
            }
        }
    }

    count
}

fn step_simulation(grid: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let height = grid.len();
    let width = grid[0].len();
    let mut new_grid = vec![vec![false; width]; height];

    for y in 0..height {
        for x in 0..width {
            let neighbors = count_neighbors(grid, x, y);
            let alive = grid[y][x];

            new_grid[y][x] = match (alive, neighbors) {
                (true, 2) | (true, 3) => true,
                (false, 3) => true,
                _ => false,
            };
        }
    }

    new_grid
}

fn grid_to_u8(grid: &Vec<Vec<bool>>) -> Vec<Vec<u8>> {
    grid.iter()
        .map(|row| row.iter().map(|&cell| if cell { 5 } else { 0 }).collect())
        .collect()
}

fn run_game_of_life(initial_grid: Vec<Vec<bool>>, max_generations: usize) -> Vec<Frame> {
    let mut frames = Vec::new();
    let mut grid = initial_grid;

    frames.push(Frame {
        step: 0,
        grid: grid_to_u8(&grid),
        message: "Initial state".to_string(),
        highlighted: vec![],
    });

    for generation in 1..=max_generations {
        grid = step_simulation(&grid);

        frames.push(Frame {
            step: generation,
            grid: grid_to_u8(&grid),
            message: format!("Generation {}", generation),
            highlighted: vec![],
        });
    }

    frames
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let width = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(60);

    let height = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(60);

    let pattern = args.get(3).map(|s| s.as_str()).unwrap_or("random");

    let generations = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(200);

    let output_name = args
        .get(5)
        .cloned()
        .unwrap_or_else(|| format!("game_of_life_{}x{}_{}", width, height, pattern));

    println!("Generating Conway's Game of Life animation:");
    println!("  Grid: {}x{}", width, height);
    println!("  Pattern: {}", pattern);
    println!("  Generations: {}", generations);

    let initial_grid = match pattern {
        "glider" => glider_grid(width, height),
        "random" => random_grid(width, height, 0.3),
        "sparse" => random_grid(width, height, 0.15),
        "dense" => random_grid(width, height, 0.5),
        _ => {
            println!("Unknown pattern '{}', using random", pattern);
            random_grid(width, height, 0.3)
        }
    };

    println!("Running simulation...");
    let frames = run_game_of_life(initial_grid, generations);

    let animation = AnimationData {
        name: output_name.clone(),
        algorithm: "conways_game_of_life".to_string(),
        created_at: Utc::now().to_rfc3339(),
        grid_config: GridConfig { width, height },
        metadata: Metadata {
            total_frames: frames.len(),
            has_path: false,
            frame_delay_ms: 100.0,
        },
        frames,
    };

    let workspace_root = std::env::current_dir().expect("Failed to get current directory");
    let output_path = workspace_root
        .join("animations")
        .join(format!("{}.json", output_name));

    fs::create_dir_all(output_path.parent().unwrap())
        .expect("Failed to create animations directory");

    println!("Writing animation to: {}", output_path.display());
    let json = serde_json::to_string(&animation).expect("Failed to serialize");
    fs::write(&output_path, json).expect("Failed to write file");

    println!("✓ Animation generated successfully!");
    println!("  {} generations", animation.metadata.total_frames - 1);
    println!("  Frame delay: {}ms", animation.metadata.frame_delay_ms);
}
