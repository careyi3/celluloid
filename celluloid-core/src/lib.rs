//! Recorder and file format for Celluloid animations.
//!
//! An animation is a set of panels (square grids, hex grids, arrays,
//! trees, graphs) plus
//! a list of frames. Each frame holds
//! small ops ("these cells are now `seen`", "marker `me` moved to (3, 4)")
//! rather than a copy of the whole grid. [`Recorder`] produces animations,
//! [`Timeline`] replays them.

mod color;
mod format;
pub mod hex;
pub mod legacy;
mod recorder;
mod replay;

pub use color::Color;
pub use format::{
    Animation, ArrayStyle, At, Frame, MarkerDef, NodeDef, Op, Orientation, Panel, PanelKind,
    StateDef, FORMAT_VERSION,
};
pub use recorder::{Array, Graph, Grid, Handle, Hex, NodePanel, Number, Pos, Recorder, Target, Tree};
pub use replay::{
    ArrayState, Edge, GraphState, GridState, HexState, Item, Node, PanelState, State, Timeline,
    TreeState,
};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// Parse an animation file, converting the pre-0.1 full-grid format if needed.
pub fn from_json(json: &str) -> Result<Animation, Error> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| Error(format!("invalid JSON: {e}")))?;

    let animation = if value.get("format").is_some() {
        serde_json::from_value::<Animation>(value)
            .map_err(|e| Error(format!("invalid animation: {e}")))?
    } else if value.get("grid_config").is_some() {
        let old = serde_json::from_value::<legacy::AnimationData>(value)
            .map_err(|e| Error(format!("invalid legacy animation: {e}")))?;
        legacy::convert(&old)
    } else {
        return Err(Error("not a celluloid animation".into()));
    };

    if animation.format > FORMAT_VERSION {
        return Err(Error(format!(
            "animation uses format {} but this build only understands up to {}",
            animation.format, FORMAT_VERSION
        )));
    }
    animation.validate()?;
    Ok(animation)
}
