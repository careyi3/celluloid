#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CellState {
    Empty,
    Obstacle,
    Start,
    End,
    Visited,
    Path,
    Custom(u8),
}

#[derive(Clone, Debug)]
pub struct CellColor {
    pub hex: String,
}

impl CellColor {
    pub fn new(hex: &str) -> Self {
        Self {
            hex: hex.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct GridConfig {
    pub size: usize,
    pub colors: ColorScheme,
}

#[derive(Clone)]
pub struct ColorScheme {
    pub empty: CellColor,
    pub obstacle: CellColor,
    pub start: CellColor,
    pub end: CellColor,
    pub visited: CellColor,
    pub path: CellColor,
    pub background: CellColor,
    pub grid_lines: CellColor,
    pub text: CellColor,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            empty: CellColor::new("#2a2a2a"),
            obstacle: CellColor::new("#0a0a0a"),
            start: CellColor::new("#00ff00"),
            end: CellColor::new("#ff0000"),
            visited: CellColor::new("#4444ff"),
            path: CellColor::new("#ffaa00"),
            background: CellColor::new("#1a1a1a"),
            grid_lines: CellColor::new("#444444"),
            text: CellColor::new("#ffffff"),
        }
    }
}

#[derive(Clone)]
pub struct AnimationFrame {
    pub grid: Vec<Vec<CellState>>,
    pub message: String,
}

impl AnimationFrame {
    pub fn new(size: usize) -> Self {
        Self {
            grid: vec![vec![CellState::Empty; size]; size],
            message: String::new(),
        }
    }

    pub fn with_grid(grid: Vec<Vec<CellState>>, message: String) -> Self {
        Self { grid, message }
    }
}
