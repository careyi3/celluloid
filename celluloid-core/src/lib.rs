//! Record what your code does, frame by frame, and save it for the
//! [Celluloid](https://github.com/careyi3/celluloid) viewer.
//!
//! Make a [`Recorder`], add a panel and change things in it. Each call to
//! [`Recorder::frame`] ends a step. Frames only store what changed, so
//! recording every step of a big loop is fine.
//!
//! ```no_run
//! use celluloid_core::Recorder;
//!
//! let mut rec = Recorder::new("Day 12");
//! let g = rec.grid("map", 10, 10);
//! rec.state(g, "wall", "#555555");
//! rec.set(g, (3, 4), "wall");
//!
//! for x in 0..10 {
//!     rec.marker(g, "me", (x, 0));
//!     rec.var("steps", x);
//!     rec.frame("step");
//! }
//!
//! rec.save("day12.json")?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Open the file with `celluloid day12.json`. To read a file back in your
//! own code, use [`from_json`] and step through it with [`Timeline`].

mod color;
mod format;
pub mod hex;
pub mod legacy;
mod recorder;
mod replay;

pub use color::Color;
pub use format::{
    Animation, ArrayStyle, At, Camera, Frame, MarkerDef, NodeDef, Op, Orientation, Panel,
    PanelKind, StateDef, FORMAT_VERSION,
};
pub use recorder::{
    Array, CellPanel, Graph, Grid, Handle, Hex, NodePanel, Number, Pos, Recorder, Target, Tree,
};
pub use replay::{
    ArrayState, Edge, GraphState, GridState, HexState, Item, Node, PanelState, State, Timeline,
    TreeState,
};

use std::fmt;

/// What went wrong loading or checking an animation.
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
