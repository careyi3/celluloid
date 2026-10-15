use crate::{Color, Error};
use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Bumped whenever the file format changes incompatibly.
pub const FORMAT_VERSION: u32 = 1;

/// A whole recording: its panels and frames. This is what gets saved as JSON.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Animation {
    pub format: u32,
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    /// Default playback speed, in milliseconds per frame.
    #[serde(default = "default_frame_delay")]
    pub frame_delay_ms: f64,
    pub panels: Vec<Panel>,
    pub frames: Vec<Frame>,
}

pub(crate) fn default_frame_delay() -> f64 {
    50.0
}

/// One thing being animated, like a grid or an array.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Panel {
    pub name: String,
    #[serde(flatten)]
    pub kind: PanelKind,
    /// Every element starts in `states[0]`.
    pub states: Vec<StateDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<MarkerDef>,
    /// Grids: the styles lines are drawn in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<LineDef>,
}

/// What kind of panel this is, with its size or starting contents.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PanelKind {
    /// Cells addressed `[x, y]`, `0 <= x < width`, `0 <= y < height`.
    Grid { width: u32, height: u32 },
    /// Hexagons addressed by axial `[q, r]`. Unbounded: the board grows to
    /// fit every cell the animation touches.
    Hex {
        #[serde(default)]
        orientation: Orientation,
    },
    /// A list of numbers addressed by index. Items keep their identity
    /// through swaps and moves, so the viewer can animate them travelling.
    Array {
        #[serde(default)]
        style: ArrayStyle,
        #[serde(default)]
        values: Vec<f64>,
    },
    /// Nodes addressed by index into `nodes`, each with numbered child
    /// slots. Slot 0 and 1 are left and right in a binary tree.
    Tree {
        #[serde(default)]
        nodes: Vec<NodeDef>,
    },
    /// Nodes addressed by index into `nodes`, joined by edges.
    Graph {
        #[serde(default)]
        directed: bool,
        #[serde(default)]
        nodes: Vec<NodeDef>,
    },
}

impl PanelKind {
    /// The node table of a tree or graph panel.
    pub fn nodes(&self) -> Option<&[NodeDef]> {
        match self {
            PanelKind::Tree { nodes } | PanelKind::Graph { nodes, .. } => Some(nodes),
            _ => None,
        }
    }
}

/// Every node a tree or graph panel ever has. Whether it exists at a given
/// frame is up to `add_node` / `remove_node` ops.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NodeDef {
    pub name: String,
    /// A fixed position, for graphs whose nodes have real coordinates.
    /// Otherwise the viewer lays the graph out itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pos: Option<[f32; 2]>,
}

/// Which way hex panel hexagons point.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    /// A corner at the top.
    #[default]
    Pointy,
    /// A flat edge at the top.
    Flat,
}

/// How an array panel is drawn.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArrayStyle {
    /// Bar heights show the values.
    #[default]
    Bars,
    /// Equal boxes with the value written inside.
    Boxes,
}

/// A named state and the colour it's drawn in.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StateDef {
    pub name: String,
    pub color: Color,
}

/// A named marker and the colour it's drawn in.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MarkerDef {
    pub name: String,
    pub color: Color,
}

/// A named line style and the colour it's drawn in.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LineDef {
    pub name: String,
    pub color: Color,
}

/// A point on a grid, `[x, y]` in cells. Whole numbers are cell centres,
/// so `[0.5, 0.0]` is on the edge between cells `(0, 0)` and `(1, 0)` and
/// `[-0.5, -0.5]` is the grid's top left corner.
pub type Point = [f32; 2];

/// One step of the animation.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Frame {
    /// Changes applied when this frame is reached.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<Op>,
    /// Shown while this frame is on screen.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
    /// Name the viewer lists this frame under, to jump to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

/// Where in a panel an op applies: `[x, y]` on grids, `[q, r]` on hex
/// panels, a bare index on arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum At {
    /// A grid or hex cell.
    Cell([i32; 2]),
    /// An array item, or a tree or graph node.
    Index(u32),
}

impl Serialize for At {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            At::Index(i) => serializer.serialize_u32(*i),
            At::Cell([a, b]) => {
                let mut t = serializer.serialize_tuple(2)?;
                t.serialize_element(a)?;
                t.serialize_element(b)?;
                t.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for At {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AtVisitor;

        impl<'de> Visitor<'de> for AtVisitor {
            type Value = At;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an index or an [x, y] pair")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<At, E> {
                u32::try_from(v).map(At::Index).map_err(E::custom)
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<At, E> {
                u32::try_from(v).map(At::Index).map_err(E::custom)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<At, A::Error> {
                let a = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let b = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }
                Ok(At::Cell([a, b]))
            }
        }

        deserializer.deserialize_any(AtVisitor)
    }
}

impl fmt::Display for At {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            At::Cell([a, b]) => write!(f, "({a}, {b})"),
            At::Index(i) => write!(f, "[{i}]"),
        }
    }
}

/// Where a grid or hex panel's camera points.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// The cell in the middle of the view: `[x, y]` on grids, axial
    /// `[q, r]` on hex panels. Fractions fall between cells.
    pub center: [f32; 2],
    /// How much closer than fitting the whole panel. 1 shows it all, 4
    /// shows a quarter of its width.
    pub zoom: f32,
}

/// A change applied when a frame is reached. Indices refer to
/// `Animation::panels` and the panel's `states` / `markers`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Set the state of some elements.
    Set {
        panel: u16,
        state: u16,
        cells: Vec<At>,
    },
    /// Grids and arrays: every element. Hex: every cell that isn't in
    /// `states[0]`; filling with state 0 clears the board.
    Fill { panel: u16, state: u16 },
    /// Show text on an element.
    Label {
        panel: u16,
        at: At,
        /// `None` clears the label.
        text: Option<String>,
    },
    /// Move or hide a marker.
    Marker {
        panel: u16,
        marker: u16,
        /// `None` hides the marker. On arrays a marker may sit one past
        /// the end, like an exclusive upper bound.
        at: Option<At>,
    },
    /// Grids and hex panels: point the camera. The viewer eases to it and
    /// holds it until the next camera op.
    Camera {
        panel: u16,
        /// `None` goes back to showing the whole panel.
        view: Option<Camera>,
    },
    /// Grids: draw straight lines between pairs of points, or erase them.
    /// A line is the same whichever end comes first; drawing one that's
    /// already there changes its style.
    Line {
        panel: u16,
        /// Index into `Panel::lines`; `None` erases the lines.
        style: Option<u16>,
        lines: Vec<[Point; 2]>,
    },
    /// Grids: erase every line, or every line in one style.
    ClearLines { panel: u16, style: Option<u16> },
    /// Grids: show small text at a point, anywhere on the grid.
    Note {
        panel: u16,
        at: Point,
        /// `None` removes the note.
        text: Option<String>,
    },
    /// Grids: remove every note.
    ClearNotes { panel: u16 },
    /// Set a named value shown beside the animation.
    Var {
        name: String,
        /// `None` removes the variable.
        value: Option<String>,
    },
    /// Arrays: change the value of the item at an index.
    Value { panel: u16, at: u32, value: f64 },
    /// Arrays: exchange two items.
    Swap { panel: u16, a: u32, b: u32 },
    /// Arrays: insert a new item; `at` may equal the length to append.
    Insert { panel: u16, at: u32, value: f64 },
    /// Arrays: remove an item.
    Remove { panel: u16, at: u32 },
    /// Arrays: take the item at `from` out and put it back in at `to`.
    Move { panel: u16, from: u32, to: u32 },
    /// Trees and graphs: make a node exist.
    AddNode { panel: u16, node: u32 },
    /// Trees and graphs: remove a node, its edges and its state. A tree
    /// node's children become roots.
    RemoveNode { panel: u16, node: u32 },
    /// Trees: put `child` in `parent`'s slot, detaching it from wherever
    /// it was. `None` empties the slot. If `child` is an ancestor of
    /// `parent`, the branch leading down to `parent` is cut off it first,
    /// so rotations can be recorded in any order.
    Child {
        panel: u16,
        parent: u32,
        slot: u16,
        child: Option<u32>,
    },
    /// Graphs: join two nodes.
    AddEdge { panel: u16, a: u32, b: u32 },
    /// Graphs: unjoin two nodes.
    RemoveEdge { panel: u16, a: u32, b: u32 },
    /// Graphs: set the state of edges.
    EdgeSet {
        panel: u16,
        state: u16,
        edges: Vec<[u32; 2]>,
    },
    /// Graphs: show text on an edge, e.g. a weight. `None` clears it.
    EdgeLabel {
        panel: u16,
        a: u32,
        b: u32,
        text: Option<String>,
    },
}

impl Op {
    /// The panel this op changes. `None` for vars.
    pub fn panel(&self) -> Option<u16> {
        match self {
            Op::Set { panel, .. }
            | Op::Fill { panel, .. }
            | Op::Label { panel, .. }
            | Op::Marker { panel, .. }
            | Op::Camera { panel, .. }
            | Op::Value { panel, .. }
            | Op::Swap { panel, .. }
            | Op::Insert { panel, .. }
            | Op::Remove { panel, .. }
            | Op::Move { panel, .. }
            | Op::AddNode { panel, .. }
            | Op::RemoveNode { panel, .. }
            | Op::Child { panel, .. }
            | Op::AddEdge { panel, .. }
            | Op::RemoveEdge { panel, .. }
            | Op::EdgeSet { panel, .. }
            | Op::EdgeLabel { panel, .. }
            | Op::Line { panel, .. }
            | Op::ClearLines { panel, .. }
            | Op::Note { panel, .. }
            | Op::ClearNotes { panel } => Some(*panel),
            Op::Var { .. } => None,
        }
    }
}

impl Animation {
    /// Check that every op refers to panels, states, markers and elements
    /// that exist at that point.
    pub fn validate(&self) -> Result<(), Error> {
        for (i, panel) in self.panels.iter().enumerate() {
            if panel.states.is_empty() {
                return Err(Error(format!("panel {i} ({:?}) has no states", panel.name)));
            }
        }

        let mut lens: Vec<usize> = self
            .panels
            .iter()
            .map(|p| match &p.kind {
                PanelKind::Array { values, .. } => values.len(),
                _ => 0,
            })
            .collect();

        for (f, frame) in self.frames.iter().enumerate() {
            for op in &frame.ops {
                self.validate_op(op, &mut lens)
                    .map_err(|e| Error(format!("frame {f}: {e}")))?;
            }
        }
        Ok(())
    }

    fn validate_op(&self, op: &Op, lens: &mut [usize]) -> Result<(), String> {
        let Some(p) = op.panel() else {
            return Ok(());
        };
        let panel = self
            .panels
            .get(p as usize)
            .ok_or_else(|| format!("unknown panel {p}"))?;
        let len = &mut lens[p as usize];
        let name = &panel.name;

        let state = |s: u16| {
            if (s as usize) < panel.states.len() {
                Ok(())
            } else {
                Err(format!("unknown state {s} in panel {name:?}"))
            }
        };
        let at = |a: At, len: usize, extra: usize| match (&panel.kind, a) {
            (PanelKind::Grid { width, height }, At::Cell([x, y]))
                if x >= 0 && y >= 0 && (x as u32) < *width && (y as u32) < *height =>
            {
                Ok(())
            }
            (PanelKind::Grid { width, height }, At::Cell([x, y])) => Err(format!(
                "cell ({x}, {y}) outside {width}x{height} grid {name:?}"
            )),
            (PanelKind::Hex { .. }, At::Cell(_)) => Ok(()),
            (PanelKind::Array { .. }, At::Index(i)) if (i as usize) < len + extra => Ok(()),
            (PanelKind::Array { .. }, At::Index(i)) => {
                Err(format!("index {i} outside array {name:?} of length {len}"))
            }
            (PanelKind::Array { .. }, At::Cell(_)) => {
                Err(format!("array {name:?} takes indices, not [x, y] cells"))
            }
            (PanelKind::Tree { nodes } | PanelKind::Graph { nodes, .. }, At::Index(n))
                if (n as usize) < nodes.len() =>
            {
                Ok(())
            }
            (PanelKind::Tree { .. } | PanelKind::Graph { .. }, At::Index(n)) => {
                Err(format!("unknown node {n} in {name:?}"))
            }
            (PanelKind::Tree { .. } | PanelKind::Graph { .. }, At::Cell(_)) => Err(format!(
                "panel {name:?} takes node indices, not [x, y] cells"
            )),
            (_, At::Index(_)) => Err(format!("panel {name:?} takes [x, y] cells, not indices")),
        };
        let index = |i: u32, len: usize, extra: usize| at(At::Index(i), len, extra);
        let array_only = |what: &str| match panel.kind {
            PanelKind::Array { .. } => Ok(()),
            _ => Err(format!("{what} only applies to arrays, not {name:?}")),
        };
        let node = |n: u32| match panel.kind.nodes() {
            Some(nodes) if (n as usize) < nodes.len() => Ok(()),
            Some(_) => Err(format!("unknown node {n} in {name:?}")),
            None => Err(format!("{name:?} has no nodes")),
        };
        let graph_only = |what: &str| match panel.kind {
            PanelKind::Graph { .. } => Ok(()),
            _ => Err(format!("{what} only applies to graphs, not {name:?}")),
        };
        let line_style = |s: Option<u16>| match s {
            Some(s) if s as usize >= panel.lines.len() => {
                Err(format!("unknown line style {s} in panel {name:?}"))
            }
            _ => Ok(()),
        };
        let grid_only = || match panel.kind {
            PanelKind::Grid { width, height } => Ok((width, height)),
            _ => Err(format!("lines and notes only apply to grids, not {name:?}")),
        };
        let point = |p: Point| {
            let (width, height) = grid_only()?;
            if on_grid(p, width, height) {
                Ok(())
            } else {
                Err(format!(
                    "point ({}, {}) is off the {width}x{height} grid {name:?}",
                    p[0], p[1]
                ))
            }
        };

        match op {
            Op::Set {
                state: s, cells, ..
            } => {
                state(*s)?;
                cells.iter().try_for_each(|&c| at(c, *len, 0))
            }
            Op::Fill { state: s, .. } => state(*s),
            Op::Label { at: a, .. } => at(*a, *len, 0),
            Op::Marker { marker, at: a, .. } => {
                if *marker as usize >= panel.markers.len() {
                    return Err(format!("unknown marker {marker} in panel {name:?}"));
                }
                a.map_or(Ok(()), |a| at(a, *len, 1))
            }
            Op::Camera { view, .. } => {
                if !matches!(panel.kind, PanelKind::Grid { .. } | PanelKind::Hex { .. }) {
                    return Err(format!(
                        "camera only applies to grids and hex panels, not {name:?}"
                    ));
                }
                match view {
                    Some(Camera { center, zoom })
                        if !center.iter().all(|c| c.is_finite())
                            || !zoom.is_finite()
                            || *zoom <= 0.0 =>
                    {
                        Err(format!(
                            "camera in {name:?} needs a finite centre and a positive zoom"
                        ))
                    }
                    _ => Ok(()),
                }
            }
            Op::Value { at: i, value, .. } => {
                array_only("value")?;
                if !value.is_finite() {
                    return Err(format!("value {value} is not a finite number"));
                }
                index(*i, *len, 0)
            }
            Op::Swap { a, b, .. } => {
                array_only("swap")?;
                index(*a, *len, 0)?;
                index(*b, *len, 0)
            }
            Op::Insert { at: i, value, .. } => {
                array_only("insert")?;
                if !value.is_finite() {
                    return Err(format!("value {value} is not a finite number"));
                }
                index(*i, *len, 1)?;
                *len += 1;
                Ok(())
            }
            Op::Remove { at: i, .. } => {
                array_only("remove")?;
                index(*i, *len, 0)?;
                *len -= 1;
                Ok(())
            }
            Op::Move { from, to, .. } => {
                array_only("move")?;
                index(*from, *len, 0)?;
                index(*to, *len, 0)
            }
            Op::AddNode { node: n, .. } | Op::RemoveNode { node: n, .. } => node(*n),
            Op::Child {
                parent,
                slot,
                child,
                ..
            } => {
                if !matches!(panel.kind, PanelKind::Tree { .. }) {
                    return Err(format!("child only applies to trees, not {name:?}"));
                }
                if *slot >= 1024 {
                    return Err(format!("child slot {slot} is too large"));
                }
                node(*parent)?;
                child.map_or(Ok(()), node)
            }
            Op::AddEdge { a, b, .. } | Op::RemoveEdge { a, b, .. } => {
                graph_only("edges")?;
                node(*a)?;
                node(*b)
            }
            Op::EdgeSet {
                state: s, edges, ..
            } => {
                graph_only("edge states")?;
                state(*s)?;
                edges.iter().try_for_each(|[a, b]| node(*a).and(node(*b)))
            }
            Op::EdgeLabel { a, b, .. } => {
                graph_only("edge labels")?;
                node(*a)?;
                node(*b)
            }
            Op::Line { style, lines, .. } => {
                grid_only()?;
                line_style(*style)?;
                lines.iter().flatten().try_for_each(|&p| point(p))
            }
            Op::ClearLines { style, .. } => {
                grid_only()?;
                line_style(*style)
            }
            Op::Note { at, .. } => point(*at),
            Op::ClearNotes { .. } => grid_only().map(|_| ()),
            Op::Var { .. } => Ok(()),
        }
    }

    /// Indices of frames that carry a bookmark.
    pub fn bookmarks(&self) -> impl Iterator<Item = (usize, &str)> {
        self.frames
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.bookmark.as_deref().map(|b| (i, b)))
    }
}

/// Whether `p` is on a `width` x `height` grid, edges included.
pub(crate) fn on_grid(p: Point, width: u32, height: u32) -> bool {
    let within = |v: f32, n: u32| v.is_finite() && v >= -0.5 && v <= n as f32 - 0.5;
    width > 0 && height > 0 && within(p[0], width) && within(p[1], height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(kind: PanelKind) -> Panel {
        Panel {
            name: "p".into(),
            kind,
            states: vec![StateDef {
                name: "empty".into(),
                color: Color::rgb(0, 0, 0),
            }],
            markers: vec![],
            lines: vec![],
        }
    }

    fn animation(kind: PanelKind, ops: Vec<Op>) -> Animation {
        Animation {
            format: FORMAT_VERSION,
            name: "t".into(),
            created_at: String::new(),
            frame_delay_ms: 50.0,
            panels: vec![panel(kind)],
            frames: vec![Frame {
                ops,
                ..Default::default()
            }],
        }
    }

    #[test]
    fn op_json_shape() {
        let op = Op::Set {
            panel: 0,
            state: 2,
            cells: vec![At::Cell([3, 4]), At::Index(7)],
        };
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(
            json,
            r#"{"op":"set","panel":0,"state":2,"cells":[[3,4],7]}"#
        );
        assert_eq!(serde_json::from_str::<Op>(&json).unwrap(), op);
        assert!(serde_json::from_str::<At>("[1,2,3]").is_err());
        assert!(serde_json::from_str::<At>("-1").is_err());
        assert_eq!(
            serde_json::from_str::<At>("[-1,2]").unwrap(),
            At::Cell([-1, 2])
        );
    }

    #[test]
    fn panel_kind_json_shape() {
        let p = panel(PanelKind::Array {
            style: ArrayStyle::Boxes,
            values: vec![1.0, 2.5],
        });
        let json = serde_json::to_string(&p).unwrap();
        assert!(
            json.contains(r#""kind":"array","style":"boxes","values":[1.0,2.5]"#),
            "{json}"
        );
        let hex: Panel = serde_json::from_str(
            r##"{"name":"h","kind":"hex","states":[{"name":"e","color":"#000000"}]}"##,
        )
        .unwrap();
        assert_eq!(
            hex.kind,
            PanelKind::Hex {
                orientation: Orientation::Pointy
            }
        );
    }

    #[test]
    fn validate_rejects_out_of_bounds() {
        let grid = PanelKind::Grid {
            width: 2,
            height: 2,
        };
        let set = |at| Op::Set {
            panel: 0,
            state: 0,
            cells: vec![at],
        };
        let err = animation(grid.clone(), vec![set(At::Cell([2, 0]))])
            .validate()
            .unwrap_err();
        assert!(err.0.starts_with("frame 0: cell (2, 0)"), "{err}");
        assert!(animation(grid, vec![set(At::Index(0))]).validate().is_err());
        assert!(animation(
            PanelKind::Hex {
                orientation: Orientation::Flat
            },
            vec![set(At::Cell([-50, 9]))]
        )
        .validate()
        .is_ok());
    }

    #[test]
    fn validate_checks_cameras() {
        let camera = |zoom| Op::Camera {
            panel: 0,
            view: Some(Camera {
                center: [1.0, 1.0],
                zoom,
            }),
        };
        let grid = PanelKind::Grid {
            width: 2,
            height: 2,
        };
        assert_eq!(
            animation(grid.clone(), vec![camera(2.0)]).validate(),
            Ok(())
        );
        let err = animation(grid, vec![camera(0.0)]).validate().unwrap_err();
        assert!(err.0.contains("positive zoom"), "{err}");
        let tree = PanelKind::Tree { nodes: vec![] };
        let err = animation(tree, vec![camera(2.0)]).validate().unwrap_err();
        assert!(err.0.contains("only applies to grids and hex"), "{err}");
    }

    #[test]
    fn validate_tracks_array_length() {
        let array = PanelKind::Array {
            style: ArrayStyle::Bars,
            values: vec![1.0],
        };
        let ok = vec![
            Op::Insert {
                panel: 0,
                at: 1,
                value: 2.0,
            },
            Op::Swap {
                panel: 0,
                a: 0,
                b: 1,
            },
            Op::Remove { panel: 0, at: 0 },
            Op::Value {
                panel: 0,
                at: 0,
                value: 3.0,
            },
        ];
        assert_eq!(animation(array.clone(), ok).validate(), Ok(()));

        let bad = vec![
            Op::Remove { panel: 0, at: 0 },
            Op::Value {
                panel: 0,
                at: 0,
                value: 3.0,
            },
        ];
        let err = animation(array, bad).validate().unwrap_err();
        assert!(err.0.contains("index 0 outside array"), "{err}");
    }

    #[test]
    fn validate_checks_lines_and_notes() {
        let grid = PanelKind::Grid {
            width: 2,
            height: 2,
        };
        let line = |style, to| Op::Line {
            panel: 0,
            style,
            lines: vec![[[-0.5, -0.5], to]],
        };
        let mut anim = animation(grid.clone(), vec![line(Some(0), [1.5, 1.5])]);
        anim.panels[0].lines.push(LineDef {
            name: "l".into(),
            color: Color::rgb(0, 0, 0),
        });
        assert_eq!(anim.validate(), Ok(()));

        anim.frames[0].ops = vec![line(Some(1), [0.0, 0.0])];
        let err = anim.validate().unwrap_err();
        assert!(err.0.contains("unknown line style 1"), "{err}");

        anim.frames[0].ops = vec![line(None, [1.6, 0.0])];
        let err = anim.validate().unwrap_err();
        assert!(
            err.0.contains("point (1.6, 0) is off the 2x2 grid"),
            "{err}"
        );

        let note = Op::Note {
            panel: 0,
            at: [f32::NAN, 0.0],
            text: None,
        };
        assert!(animation(grid, vec![note]).validate().is_err());

        let hex = PanelKind::Hex {
            orientation: Orientation::Pointy,
        };
        let clear = Op::ClearLines {
            panel: 0,
            style: None,
        };
        let err = animation(hex, vec![clear]).validate().unwrap_err();
        assert!(err.0.contains("only apply to grids"), "{err}");

        let json = serde_json::to_string(&line(None, [1.0, 0.5])).unwrap();
        assert_eq!(
            json,
            r#"{"op":"line","panel":0,"style":null,"lines":[[[-0.5,-0.5],[1.0,0.5]]]}"#
        );
    }
}
