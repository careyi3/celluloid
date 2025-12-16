use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationData {
    pub name: String,
    pub created_at: String,
    pub grid_config: GridConfig,
    pub metadata: Metadata,
    pub frames: Vec<Frame>,
}

impl AnimationData {
    pub fn new(name: impl Into<String>, width: usize, height: usize) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name: name.into(),
            created_at: now,
            grid_config: GridConfig { width, height },
            metadata: Metadata {
                total_frames: 0,
                has_path: false,
                frame_delay_ms: 50.0,
            },
            frames: Vec::new(),
        }
    }

    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
        self.metadata.total_frames = self.frames.len();
    }

    pub fn with_frame_delay(mut self, delay_ms: f64) -> Self {
        self.metadata.frame_delay_ms = delay_ms;
        self
    }

    pub fn with_has_path(mut self, has_path: bool) -> Self {
        self.metadata.has_path = has_path;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub total_frames: usize,
    pub has_path: bool,
    #[serde(default = "default_frame_delay")]
    pub frame_delay_ms: f64,
}

fn default_frame_delay() -> f64 {
    50.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    pub step: usize,
    pub grid: Vec<Vec<u8>>,
    pub message: String,
    pub highlighted: Vec<(usize, usize)>,
}

impl Frame {
    pub fn new(step: usize, grid: Vec<Vec<u8>>, message: impl Into<String>) -> Self {
        Self {
            step,
            grid,
            message: message.into(),
            highlighted: Vec::new(),
        }
    }

    pub fn with_highlighted(mut self, coords: Vec<(usize, usize)>) -> Self {
        self.highlighted = coords;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CellState {
    Empty = 0,
    Obstacle = 1,
    Start = 2,
    End = 3,
    Visited = 4,
    Path = 5,
}

impl From<CellState> for u8 {
    fn from(state: CellState) -> u8 {
        state as u8
    }
}

impl From<u8> for CellState {
    fn from(value: u8) -> Self {
        match value {
            0 => CellState::Empty,
            1 => CellState::Obstacle,
            2 => CellState::Start,
            3 => CellState::End,
            4 => CellState::Visited,
            5 => CellState::Path,
            _ => CellState::Empty,
        }
    }
}
