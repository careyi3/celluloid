use crate::types::{AnimationFrame, CellState};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub trait PathfindingAlgorithm {
    fn generate_frames(
        &self,
        grid: &[Vec<CellState>],
        start: (usize, usize),
        end: (usize, usize),
    ) -> Vec<AnimationFrame>;
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct DijkstraState {
    cost: usize,
    position: (usize, usize),
}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct DijkstraAlgorithm;

impl PathfindingAlgorithm for DijkstraAlgorithm {
    fn generate_frames(
        &self,
        grid: &[Vec<CellState>],
        start: (usize, usize),
        end: (usize, usize),
    ) -> Vec<AnimationFrame> {
        let mut frames = Vec::new();

        if grid.is_empty() {
            return frames;
        }

        let height = grid.len();
        let width = grid[0].len();

        if start.0 >= width || start.1 >= height || end.0 >= width || end.1 >= height {
            return frames;
        }

        let mut dist = vec![vec![usize::MAX; width]; height];
        let mut prev = vec![vec![None; width]; height];
        let mut heap = BinaryHeap::new();

        let mut current_grid = grid.to_vec();

        dist[start.1][start.0] = 0;
        heap.push(DijkstraState {
            cost: 0,
            position: start,
        });

        while let Some(DijkstraState { cost, position }) = heap.pop() {
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

            frames.push(AnimationFrame::with_grid(
                current_grid.clone(),
                format!("Searching... Nodes explored: {}", frames.len()),
            ));

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
                    heap.push(DijkstraState {
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
            if let Some(row) = prev.get(pos.1) {
                if let Some(cell) = row.get(pos.0) {
                    current = *cell;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        path.reverse();

        if path.is_empty() || path[0] != start {
            frames.push(AnimationFrame::with_grid(
                current_grid.clone(),
                format!("No path found! Nodes explored: {}", frames.len()),
            ));
            return frames;
        }

        for (i, &(x, y)) in path.iter().enumerate() {
            if y < current_grid.len() && x < current_grid[y].len() {
                if current_grid[y][x] != CellState::Start && current_grid[y][x] != CellState::End {
                    current_grid[y][x] = CellState::Path;
                }

                frames.push(AnimationFrame::with_grid(
                    current_grid.clone(),
                    format!(
                        "Path found! Nodes explored: {} | Path length: {} | Drawing: {}/{}",
                        frames.len() - i,
                        path.len(),
                        i + 1,
                        path.len()
                    ),
                ));
            }
        }

        frames.push(AnimationFrame::with_grid(
            current_grid,
            format!(
                "Complete! Nodes explored: {} | Path length: {}",
                frames.len() - path.len(),
                path.len()
            ),
        ));

        frames
    }
}
