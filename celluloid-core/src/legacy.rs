//! The pre-0.1 format, where every frame stored the whole grid as `u8`s.
//! Kept so old files still load; [`crate::from_json`] converts them.

use crate::{Animation, Recorder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationData {
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    pub grid_config: GridConfig,
    pub metadata: Metadata,
    pub frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    #[serde(default)]
    pub total_frames: usize,
    #[serde(default)]
    pub has_path: bool,
    #[serde(default = "crate::format::default_frame_delay")]
    pub frame_delay_ms: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    #[serde(default)]
    pub step: usize,
    pub grid: Vec<Vec<u8>>,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub highlighted: Vec<(usize, usize)>,
}

/// The old fixed states, with the colours the old viewer drew them in.
const STATES: [(&str, &str); 6] = [
    ("empty", "#2a2a2a"),
    ("obstacle", "#0a0a0a"),
    ("start", "#00ff00"),
    ("end", "#ff0000"),
    ("visited", "#4444ff"),
    ("path", "#ffaa00"),
];
const HIGHLIGHT: (&str, &str) = ("highlight", "#ffff00");

pub fn convert(old: &AnimationData) -> Animation {
    let (width, height) = (old.grid_config.width, old.grid_config.height);

    let mut rec = Recorder::new(old.name.clone());
    rec.frame_delay(old.metadata.frame_delay_ms);
    let g = rec.grid("grid", width, height);
    for (name, color) in STATES.into_iter().chain([HIGHLIGHT]) {
        rec.state(g, name, color);
    }

    for frame in &old.frames {
        let highlighted: HashSet<_> = frame.highlighted.iter().copied().collect();
        rec.redraw(g, |x, y| {
            if highlighted.contains(&(x, y)) {
                return HIGHLIGHT.0;
            }
            let value = frame.grid.get(y).and_then(|row| row.get(x)).copied();
            STATES.get(value.unwrap_or(0) as usize).unwrap_or(&STATES[0]).0
        });
        rec.frame(frame.message.clone());
    }

    let mut animation = rec.finish();
    animation.created_at = old.created_at.clone();
    animation
}

#[cfg(test)]
mod tests {
    use crate::{PanelState, Timeline};

    #[test]
    fn converts_old_files() {
        let json = r#"{
            "name": "old", "created_at": "2025-12-01T00:00:00Z",
            "grid_config": {"width": 3, "height": 2},
            "metadata": {"total_frames": 2, "has_path": false, "frame_delay_ms": 80.0},
            "frames": [
                {"step": 0, "grid": [[0,1,2],[3,4,5]], "message": "a", "highlighted": []},
                {"step": 1, "grid": [[0,1,2],[3,4,9]], "message": "b", "highlighted": [[0,0]]}
            ]
        }"#;
        let anim = crate::from_json(json).unwrap();
        assert_eq!(anim.frame_delay_ms, 80.0);
        assert_eq!(anim.created_at, "2025-12-01T00:00:00Z");
        assert_eq!(anim.frames[1].message, "b");

        let names: Vec<_> = anim.panels[0].states.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            ["empty", "obstacle", "start", "end", "visited", "path", "highlight"]
        );

        let mut tl = Timeline::new(anim);
        let PanelState::Grid(g) = &tl.seek(0).panels[0] else { panic!() };
        assert_eq!(g.cells, vec![0, 1, 2, 3, 4, 5]);
        let PanelState::Grid(g) = &tl.seek(1).panels[0] else { panic!() };
        assert_eq!(g.cells, vec![6, 1, 2, 3, 4, 0]);
    }
}
