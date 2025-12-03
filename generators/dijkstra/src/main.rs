use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
enum CellState {
    Empty = 0,
    Obstacle = 1,
    Start = 2,
    End = 3,
    Visited = 4,
    Path = 5,
}

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

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: usize,
    position: (usize, usize),
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn generate_grid(width: usize, height: usize, obstacle_percent: f64) -> Vec<Vec<CellState>> {
    let mut rng = rand::thread_rng();
    let mut grid = vec![vec![CellState::Empty; width]; height];

    for y in 0..height {
        for x in 0..width {
            if rng.gen::<f64>() < obstacle_percent {
                grid[y][x] = CellState::Obstacle;
            }
        }
    }

    grid[0][0] = CellState::Start;
    grid[height - 1][width - 1] = CellState::End;

    grid
}

fn run_dijkstra(
    grid: &[Vec<CellState>],
    start: (usize, usize),
    end: (usize, usize),
) -> (Vec<Frame>, Option<Vec<(usize, usize)>>) {
    let mut frames = Vec::new();
    let height = grid.len();
    let width = grid[0].len();

    let mut dist = vec![vec![usize::MAX; width]; height];
    let mut prev = vec![vec![None; width]; height];
    let mut heap = BinaryHeap::new();

    let mut current_grid = grid.to_vec();
    let mut nodes_explored = 0;

    dist[start.1][start.0] = 0;
    heap.push(State {
        cost: 0,
        position: start,
    });

    while let Some(State { cost, position }) = heap.pop() {
        let (x, y) = position;

        if position == end {
            break;
        }

        if cost > dist[y][x] {
            continue;
        }

        if current_grid[y][x] != CellState::Start && current_grid[y][x] != CellState::End {
            current_grid[y][x] = CellState::Visited;
        }

        nodes_explored += 1;

        frames.push(Frame {
            step: frames.len(),
            grid: grid_to_u8(&current_grid),
            message: format!("Exploring... Nodes visited: {}", nodes_explored),
            highlighted: vec![position],
        });

        let neighbors = [
            (x.wrapping_sub(1), y),
            (x.saturating_add(1), y),
            (x, y.wrapping_sub(1)),
            (x, y.saturating_add(1)),
        ];

        for &(nx, ny) in &neighbors {
            if nx >= width || ny >= height {
                continue;
            }

            if current_grid[ny][nx] == CellState::Obstacle {
                continue;
            }

            let next_cost = cost + 1;

            if next_cost < dist[ny][nx] {
                dist[ny][nx] = next_cost;
                prev[ny][nx] = Some(position);
                heap.push(State {
                    cost: next_cost,
                    position: (nx, ny),
                });
            }
        }
    }

    let mut path = Vec::new();
    let mut current = Some(end);

    while let Some(pos) = current {
        path.push(pos);
        current = prev[pos.1][pos.0];
    }

    path.reverse();

    if path.is_empty() || path[0] != start {
        frames.push(Frame {
            step: frames.len(),
            grid: grid_to_u8(&current_grid),
            message: "No path found!".to_string(),
            highlighted: vec![],
        });
        return (frames, None);
    }

    for (i, &(x, y)) in path.iter().enumerate() {
        if current_grid[y][x] != CellState::Start && current_grid[y][x] != CellState::End {
            current_grid[y][x] = CellState::Path;
        }

        frames.push(Frame {
            step: frames.len(),
            grid: grid_to_u8(&current_grid),
            message: format!("Drawing path: {}/{}", i + 1, path.len()),
            highlighted: vec![(x, y)],
        });
    }

    frames.push(Frame {
        step: frames.len(),
        grid: grid_to_u8(&current_grid),
        message: format!(
            "Complete! Path length: {} | Nodes explored: {}",
            path.len(),
            nodes_explored
        ),
        highlighted: vec![],
    });

    (frames, Some(path))
}

fn grid_to_u8(grid: &[Vec<CellState>]) -> Vec<Vec<u8>> {
    grid.iter()
        .map(|row| row.iter().map(|&cell| cell as u8).collect())
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let width = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(50);
    let height = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(50);
    let obstacle_percent = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.2);
    let output_name = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| format!("dijkstra_{}x{}", width, height));

    println!("Generating Dijkstra animation:");
    println!("  Grid: {}x{}", width, height);
    println!("  Obstacles: {}%", (obstacle_percent * 100.0) as u32);

    let grid = generate_grid(width, height, obstacle_percent);
    let start = (0, 0);
    let end = (width - 1, height - 1);

    println!("Running Dijkstra's algorithm...");
    let (frames, path) = run_dijkstra(&grid, start, end);

    let animation = AnimationData {
        name: output_name.clone(),
        algorithm: "dijkstra".to_string(),
        created_at: Utc::now().to_rfc3339(),
        grid_config: GridConfig { width, height },
        metadata: Metadata {
            total_frames: frames.len(),
            has_path: path.is_some(),
            frame_delay_ms: 50.0,
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
    println!("  {} frames", animation.metadata.total_frames);
    println!("  Path found: {}", animation.metadata.has_path);
    if let Some(path_len) = path.as_ref().map(|p| p.len()) {
        println!("  Path length: {}", path_len);
    }
}
