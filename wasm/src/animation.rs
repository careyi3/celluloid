use crate::algorithm::PathfindingAlgorithm;
use crate::renderer::Renderer;
use crate::types::{AnimationFrame, CellState, GridConfig};
use web_sys::CanvasRenderingContext2d;

pub const GRID_SIZE: usize = 50;
pub const OBSTACLE_PERCENTAGE: f64 = 0.2;
pub const ANIMATION_SPEED_MS: f64 = 10.0;

pub struct AnimationState {
    renderer: Renderer,
    frames: Vec<AnimationFrame>,
    current_frame: usize,
    last_step: f64,
}

impl AnimationState {
    pub fn new(
        context: CanvasRenderingContext2d,
        config: GridConfig,
        algorithm: Box<dyn PathfindingAlgorithm>,
    ) -> Self {
        let mut grid = vec![vec![CellState::Empty; GRID_SIZE]; GRID_SIZE];

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                if js_sys::Math::random() < OBSTACLE_PERCENTAGE {
                    grid[y][x] = CellState::Obstacle;
                }
            }
        }

        let start = Self::find_empty_cell(&grid);
        let end = Self::find_empty_cell(&grid);

        grid[start.1][start.0] = CellState::Start;
        grid[end.1][end.0] = CellState::End;

        let frames = algorithm.generate_frames(&grid, start, end);

        let renderer = Renderer::new(context, config);

        Self {
            renderer,
            frames,
            current_frame: 0,
            last_step: 0.0,
        }
    }

    fn find_empty_cell(grid: &[Vec<CellState>]) -> (usize, usize) {
        let height = grid.len();
        if height == 0 {
            return (0, 0);
        }
        let width = grid[0].len();
        if width == 0 {
            return (0, 0);
        }

        let mut attempts = 0;
        loop {
            let x = (js_sys::Math::random() * width as f64).floor() as usize;
            let y = (js_sys::Math::random() * height as f64).floor() as usize;

            if y < grid.len() && x < grid[y].len() && grid[y][x] == CellState::Empty {
                return (x, y);
            }

            attempts += 1;
            if attempts > 10000 {
                for y in 0..height {
                    for x in 0..width {
                        if grid[y][x] == CellState::Empty {
                            return (x, y);
                        }
                    }
                }
                return (0, 0);
            }
        }
    }

    pub fn render(&mut self, time: f64) {
        if time - self.last_step > ANIMATION_SPEED_MS && self.current_frame < self.frames.len() {
            self.current_frame += 1;
            self.last_step = time;
        }

        let canvas = self.renderer.context.canvas().unwrap();
        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        if self.current_frame < self.frames.len() {
            let frame = &self.frames[self.current_frame];
            self.renderer.render_frame(frame, width, height);
        } else {
            if let Some(last_frame) = self.frames.last() {
                self.renderer.render_frame(last_frame, width, height);
            }
        }
    }
}
