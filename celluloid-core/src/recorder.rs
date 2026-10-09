use crate::color::{EMPTY, PALETTE};
use crate::format::default_frame_delay;
use crate::{
    Animation, ArrayStyle, At, Color, Frame, MarkerDef, Op, Orientation, Panel, PanelKind,
    PanelState, State, StateDef, FORMAT_VERSION,
};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// Handle to a square grid panel, returned by [`Recorder::grid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid(u16);

/// Handle to a hex panel, returned by [`Recorder::hex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hex(u16);

/// Handle to an array panel, returned by [`Recorder::array`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Array(u16);

/// Handle to a tree panel, returned by [`Recorder::tree`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tree(u16);

/// Handle to a graph panel, returned by [`Recorder::graph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Graph(u16);

/// A tree or graph handle. Their elements are nodes named by anything
/// `Display`: `"AA"`, `5`, a `String`.
pub trait NodePanel: Handle {}
impl NodePanel for Tree {}
impl NodePanel for Graph {}

/// Any panel handle. Methods like [`Recorder::set`] and
/// [`Recorder::marker`] work on every kind of panel.
pub trait Handle: Copy + sealed::Sealed {
    #[doc(hidden)]
    fn panel(self) -> u16;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Grid {}
    impl Sealed for super::Hex {}
    impl Sealed for super::Array {}
    impl Sealed for super::Tree {}
    impl Sealed for super::Graph {}
}

impl Handle for Grid {
    fn panel(self) -> u16 {
        self.0
    }
}

impl Handle for Hex {
    fn panel(self) -> u16 {
        self.0
    }
}

impl Handle for Array {
    fn panel(self) -> u16 {
        self.0
    }
}

impl Handle for Tree {
    fn panel(self) -> u16 {
        self.0
    }
}

impl Handle for Graph {
    fn panel(self) -> u16 {
        self.0
    }
}

/// A position: `(x, y)` or `[x, y]` with any integer type. Used for grid
/// cells and axial `(q, r)` hex cells.
pub trait Pos {
    /// The position as `(x, y)`.
    fn xy(self) -> (i64, i64);
}

impl<T: TryInto<i64>> Pos for (T, T) {
    fn xy(self) -> (i64, i64) {
        (to_i64(self.0), to_i64(self.1))
    }
}

impl<T: TryInto<i64>> Pos for [T; 2] {
    fn xy(self) -> (i64, i64) {
        let [x, y] = self;
        (to_i64(x), to_i64(y))
    }
}

fn to_i64<T: TryInto<i64>>(v: T) -> i64 {
    v.try_into().unwrap_or(i64::MAX)
}

/// Something that picks out an element of a panel of kind `H`: a
/// [`Pos`] for grids and hex panels, an integer index for arrays.
pub trait Target<H> {
    #[doc(hidden)]
    fn target(self) -> Raw;
}

#[doc(hidden)]
pub enum Raw {
    Cell(i64, i64),
    Index(i64),
    Key(String),
}

impl<K: Display> Target<Tree> for K {
    fn target(self) -> Raw {
        Raw::Key(self.to_string())
    }
}

impl<K: Display> Target<Graph> for K {
    fn target(self) -> Raw {
        Raw::Key(self.to_string())
    }
}

impl<P: Pos> Target<Grid> for P {
    fn target(self) -> Raw {
        let (x, y) = self.xy();
        Raw::Cell(x, y)
    }
}

impl<P: Pos> Target<Hex> for P {
    fn target(self) -> Raw {
        let (q, r) = self.xy();
        Raw::Cell(q, r)
    }
}

/// A number stored in an array.
pub trait Number: Copy {
    /// The value as `f64`.
    fn to_f64(self) -> f64;
}

macro_rules! numbers {
    ($($t:ty),*) => {$(
        impl Target<Array> for $t {
            fn target(self) -> Raw {
                Raw::Index(to_i64(self))
            }
        }

        impl Number for $t {
            fn to_f64(self) -> f64 {
                self as f64
            }
        }
    )*};
}
numbers!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl Number for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl Number for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

/// So `rec.array(name, &values)` works.
impl<T: Number> Number for &T {
    fn to_f64(self) -> f64 {
        (*self).to_f64()
    }
}

/// Builds an [`Animation`] while your solution runs.
///
/// Calls between two [`Recorder::frame`]s make up one frame. Changes that
/// don't change anything (setting a cell to the state it already has) are
/// dropped, so recording every step of a loop stays cheap.
///
/// ```
/// use celluloid_core::Recorder;
///
/// let mut rec = Recorder::new("Day 12");
/// let g = rec.grid("map", 10, 10);
/// rec.state(g, "wall", "#0a0a0a");
/// rec.set(g, (3, 4), "wall");
/// rec.marker(g, "me", (0, 0));
///
/// let values = vec![3, 1, 2];
/// let a = rec.array("queue", &values);
/// rec.swap(a, 0, 1);
/// rec.marker(a, "i", 1);
///
/// rec.var("steps", 0);
/// rec.frame("start");
/// let animation = rec.finish();
/// assert_eq!(animation.frames.len(), 1);
/// ```
pub struct Recorder {
    animation: Animation,
    shadow: State,
    pending: Frame,
    /// Per panel, node name to index in the panel's node table.
    node_ids: Vec<HashMap<String, u32>>,
}

impl Recorder {
    /// Start a recording. `name` is shown as its title.
    pub fn new(name: impl Into<String>) -> Self {
        let animation = Animation {
            format: FORMAT_VERSION,
            name: name.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            frame_delay_ms: default_frame_delay(),
            panels: Vec::new(),
            frames: Vec::new(),
        };
        let shadow = State::initial(&animation);
        Self {
            animation,
            shadow,
            pending: Frame::default(),
            node_ids: Vec::new(),
        }
    }

    /// Default playback speed, in milliseconds per frame.
    pub fn frame_delay(&mut self, ms: f64) -> &mut Self {
        self.animation.frame_delay_ms = ms;
        self
    }

    /// Add a `width` x `height` grid panel. Every cell starts as `"empty"`.
    pub fn grid(&mut self, name: impl Into<String>, width: usize, height: usize) -> Grid {
        let kind = PanelKind::Grid {
            width: u32::try_from(width).expect("grid too wide"),
            height: u32::try_from(height).expect("grid too tall"),
        };
        Grid(self.add_panel(name.into(), kind, "empty", EMPTY))
    }

    /// Add a hex panel addressed by axial `(q, r)` coordinates (see
    /// [`crate::hex`] to convert from offset coordinates). It has no fixed
    /// size: the board grows to fit every cell you touch. Every cell
    /// starts as `"empty"`.
    pub fn hex(&mut self, name: impl Into<String>, orientation: Orientation) -> Hex {
        Hex(self.add_panel(name.into(), PanelKind::Hex { orientation }, "empty", EMPTY))
    }

    /// Add an array panel holding `values`, drawn as bars. Every item
    /// starts as `"default"`.
    pub fn array<N: Number>(
        &mut self,
        name: impl Into<String>,
        values: impl IntoIterator<Item = N>,
    ) -> Array {
        let values = values.into_iter().map(finite).collect();
        let kind = PanelKind::Array {
            style: ArrayStyle::Bars,
            values,
        };
        Array(self.add_panel(name.into(), kind, "default", ARRAY_DEFAULT))
    }

    /// Draw an array as boxes with the values written inside. Good for ids
    /// or letters where bar heights mean nothing.
    pub fn array_style(&mut self, array: Array, style: ArrayStyle) -> &mut Self {
        if let PanelKind::Array { style: s, .. } = &mut self.panel_mut(array).kind {
            *s = style;
        }
        self
    }

    /// Add a tree panel. Nodes appear when first mentioned; link them with
    /// [`Recorder::left`], [`Recorder::right`] or [`Recorder::add_child`].
    /// Every node starts as `"default"`.
    pub fn tree(&mut self, name: impl Into<String>) -> Tree {
        let kind = PanelKind::Tree { nodes: Vec::new() };
        Tree(self.add_panel(name.into(), kind, "default", ARRAY_DEFAULT))
    }

    /// Add a graph panel. Nodes appear when first mentioned; join them with
    /// [`Recorder::edge`]. The viewer lays the graph out once over every
    /// node and edge it ever has, so nothing jumps between frames. Every
    /// node and edge starts as `"default"`.
    pub fn graph(&mut self, name: impl Into<String>, directed: bool) -> Graph {
        let kind = PanelKind::Graph {
            directed,
            nodes: Vec::new(),
        };
        Graph(self.add_panel(name.into(), kind, "default", ARRAY_DEFAULT))
    }

    fn add_panel(&mut self, name: String, kind: PanelKind, first: &str, color: Color) -> u16 {
        let id = u16::try_from(self.animation.panels.len()).expect("too many panels");
        let panel = Panel {
            name,
            kind,
            states: vec![StateDef {
                name: first.into(),
                color,
            }],
            markers: Vec::new(),
        };
        self.shadow.panels.push(PanelState::new(&panel));
        self.animation.panels.push(panel);
        self.node_ids.push(HashMap::new());
        id
    }

    /// Define a state's colour (`"#rrggbb"`), or change it. States you use
    /// without defining get a colour from a built-in palette.
    pub fn state<H: Handle>(&mut self, panel: H, name: &str, color: &str) -> &mut Self {
        let color = parse_color(color);
        let i = self.state_index(panel, name);
        self.panel_mut(panel).states[i as usize].color = color;
        self
    }

    /// Set one element's state.
    pub fn set<H: Handle>(&mut self, panel: H, at: impl Target<H>, state: &str) {
        self.set_all(panel, [at], state);
    }

    /// Set many elements to the same state.
    pub fn set_all<H: Handle, T: Target<H>>(
        &mut self,
        panel: H,
        targets: impl IntoIterator<Item = T>,
        state: &str,
    ) {
        let s = self.state_index(panel, state);
        let p = panel.panel();
        let mut changed = Vec::new();
        for t in targets {
            let at = self.resolve(p, t.target(), false);
            if self.shadow.panels[p as usize].state_at(at) != Some(s) {
                changed.push(at);
            }
        }
        if changed.is_empty() {
            return;
        }
        let op = Op::Set {
            panel: p,
            state: s,
            cells: changed,
        };
        self.shadow.apply(&op);
        if let (
            Some(Op::Set {
                panel: lp,
                state: ls,
                cells: lc,
            }),
            Op::Set { cells, .. },
        ) = (self.pending.ops.last_mut(), &op)
        {
            if *lp == p && *ls == s {
                lc.extend(cells);
                return;
            }
        }
        self.pending.ops.push(op);
    }

    /// Set every element to `state`. On a hex panel this means every cell
    /// that isn't empty; `fill(h, "empty")` clears the board.
    pub fn fill<H: Handle>(&mut self, panel: H, state: &str) {
        let s = self.state_index(panel, state);
        let unchanged = match &self.shadow.panels[panel.panel() as usize] {
            PanelState::Grid(g) => g.cells.iter().all(|&c| c == s),
            PanelState::Hex(h) => h.cells.values().all(|&c| c == s),
            PanelState::Array(a) => a.items.iter().all(|it| it.state == s),
            PanelState::Tree(crate::TreeState { nodes, .. })
            | PanelState::Graph(crate::GraphState { nodes, .. }) => {
                nodes.iter().filter(|n| n.present).all(|n| n.state == s)
            }
        };
        if !unchanged {
            self.emit(Op::Fill {
                panel: panel.panel(),
                state: s,
            });
        }
    }

    /// Show text on an element. Array labels travel with their item.
    pub fn label<H: Handle>(&mut self, panel: H, at: impl Target<H>, text: impl Display) {
        self.set_label(panel, at, Some(text.to_string()));
    }

    /// Remove an element's label.
    pub fn clear_label<H: Handle>(&mut self, panel: H, at: impl Target<H>) {
        self.set_label(panel, at, None);
    }

    fn set_label<H: Handle>(&mut self, panel: H, at: impl Target<H>, text: Option<String>) {
        let p = panel.panel();
        let at = self.resolve(p, at.target(), false);
        if self.shadow.panels[p as usize].label_at(at) == text.as_deref() {
            return;
        }
        self.emit(Op::Label { panel: p, at, text });
    }

    /// Move a named marker to an element, showing it if hidden. Markers
    /// glide between positions in the viewer. On arrays they're pointers
    /// drawn under the items, and may sit one past the end.
    pub fn marker<H: Handle>(&mut self, panel: H, name: &str, at: impl Target<H>) {
        let at = self.resolve(panel.panel(), at.target(), true);
        self.set_marker(panel, name, Some(at));
    }

    /// Hide a marker until it's moved again.
    pub fn hide_marker<H: Handle>(&mut self, panel: H, name: &str) {
        self.set_marker(panel, name, None);
    }

    /// Set a marker's colour (`"#rrggbb"`).
    pub fn marker_color<H: Handle>(&mut self, panel: H, name: &str, color: &str) -> &mut Self {
        let color = parse_color(color);
        let i = self.marker_index(panel, name);
        self.panel_mut(panel).markers[i as usize].color = color;
        self
    }

    fn set_marker<H: Handle>(&mut self, panel: H, name: &str, at: Option<At>) {
        let m = self.marker_index(panel, name);
        let p = panel.panel();
        if self.shadow.panels[p as usize].marker_at(m as usize) == at {
            return;
        }
        self.emit(Op::Marker {
            panel: p,
            marker: m,
            at,
        });
    }

    /// Set every cell to whatever `state_at(x, y)` returns. Handy when your
    /// solution already keeps a whole grid; only the changed cells are recorded.
    pub fn redraw<S: AsRef<str>>(
        &mut self,
        grid: Grid,
        mut state_at: impl FnMut(usize, usize) -> S,
    ) {
        let (width, height) = match &self.shadow.panels[grid.0 as usize] {
            PanelState::Grid(g) => (g.width as usize, g.height as usize),
            _ => unreachable!("Grid handle on a non-grid panel"),
        };
        for y in 0..height {
            for x in 0..width {
                self.set(grid, (x, y), state_at(x, y).as_ref());
            }
        }
    }

    /// Change the value of the item at `index`.
    pub fn value(&mut self, array: Array, index: impl Target<Array>, value: impl Number) {
        let at = self.index(array, index, false);
        let value = finite(value);
        if self.array_state(array).items[at as usize].value == value {
            return;
        }
        self.emit(Op::Value {
            panel: array.0,
            at,
            value,
        });
    }

    /// Exchange two items. The viewer shows them trading places.
    pub fn swap(&mut self, array: Array, a: impl Target<Array>, b: impl Target<Array>) {
        let (a, b) = (self.index(array, a, false), self.index(array, b, false));
        if a != b {
            self.emit(Op::Swap {
                panel: array.0,
                a,
                b,
            });
        }
    }

    /// Insert a new item before `index`; `index` may equal the length.
    pub fn insert(&mut self, array: Array, index: impl Target<Array>, value: impl Number) {
        let at = self.index(array, index, true);
        self.emit(Op::Insert {
            panel: array.0,
            at,
            value: finite(value),
        });
    }

    /// Append an item.
    pub fn push(&mut self, array: Array, value: impl Number) {
        let len = self.array_state(array).items.len();
        self.insert(array, len, value);
    }

    /// Remove the item at `index`.
    pub fn remove(&mut self, array: Array, index: impl Target<Array>) {
        let at = self.index(array, index, false);
        self.emit(Op::Remove { panel: array.0, at });
    }

    /// Remove the last item, if there is one.
    pub fn pop(&mut self, array: Array) {
        let len = self.array_state(array).items.len();
        if len > 0 {
            self.remove(array, len - 1);
        }
    }

    /// Take the item at `from` out and reinsert it at `to`, like
    /// `v.insert(to, v.remove(from))`. The item keeps its identity.
    pub fn move_item(&mut self, array: Array, from: impl Target<Array>, to: impl Target<Array>) {
        let (from, to) = (self.index(array, from, false), self.index(array, to, false));
        if from != to {
            self.emit(Op::Move {
                panel: array.0,
                from,
                to,
            });
        }
    }

    /// Make sure a node exists. Nodes are also created by mentioning them
    /// anywhere else, so this is only needed for nodes with nothing else
    /// going on.
    pub fn node<H: NodePanel>(&mut self, panel: H, key: impl Display) {
        self.ensure_node(panel.panel(), &key.to_string());
    }

    /// Remove a node with its edges, state and label. A tree node's
    /// children become roots.
    pub fn remove_node<H: NodePanel>(&mut self, panel: H, key: impl Display) {
        let p = panel.panel();
        let n = self.node_index(p, &key.to_string());
        if self.shadow.panels[p as usize].has_node(n) {
            self.emit(Op::RemoveNode { panel: p, node: n });
        }
    }

    /// Put `child` in `parent`'s left slot (slot 0), moving it from
    /// wherever it was. A rotation is just a few of these in one frame,
    /// in any order: if `child` is currently above `parent`, `parent`'s
    /// branch is cut away from it first.
    pub fn left(&mut self, tree: Tree, parent: impl Display, child: impl Display) {
        self.child(tree, parent, 0, child);
    }

    /// Put `child` in `parent`'s right slot (slot 1).
    pub fn right(&mut self, tree: Tree, parent: impl Display, child: impl Display) {
        self.child(tree, parent, 1, child);
    }

    /// Put `child` in a numbered slot of `parent`.
    pub fn child(&mut self, tree: Tree, parent: impl Display, slot: usize, child: impl Display) {
        let parent = self.ensure_node(tree.0, &parent.to_string());
        let child = self.ensure_node(tree.0, &child.to_string());
        let slot = u16::try_from(slot).expect("child slot too large");
        self.set_child(tree, parent, slot, Some(child));
    }

    /// Append `child` after `parent`'s existing children.
    pub fn add_child(&mut self, tree: Tree, parent: impl Display, child: impl Display) {
        let parent_ix = self.ensure_node(tree.0, &parent.to_string());
        let slot = self.tree_state(tree).nodes[parent_ix as usize]
            .children
            .len();
        self.child(tree, parent, slot, child);
    }

    /// Empty one of `parent`'s slots.
    pub fn clear_child(&mut self, tree: Tree, parent: impl Display, slot: usize) {
        let parent = self.node_index(tree.0, &parent.to_string());
        let slot = u16::try_from(slot).expect("child slot too large");
        self.set_child(tree, parent, slot, None);
    }

    /// Take a node away from its parent, making it a root.
    pub fn detach(&mut self, tree: Tree, child: impl Display) {
        let child = self.node_index(tree.0, &child.to_string());
        if let Some((parent, slot)) = self.tree_state(tree).nodes[child as usize].parent {
            self.set_child(tree, parent, slot, None);
        }
    }

    fn set_child(&mut self, tree: Tree, parent: u32, slot: u16, child: Option<u32>) {
        let current = self.tree_state(tree).nodes[parent as usize]
            .children
            .get(slot as usize)
            .copied()
            .flatten();
        if current != child {
            self.emit(Op::Child {
                panel: tree.0,
                parent,
                slot,
                child,
            });
        }
    }

    /// Pin a graph node to a position, e.g. real coordinates from the
    /// puzzle. Units don't matter; the viewer scales to fit. Unpinned
    /// nodes are laid out around pinned ones.
    pub fn place(&mut self, graph: Graph, key: impl Display, x: f32, y: f32) {
        let n = self.node_index(graph.0, &key.to_string());
        if let PanelKind::Graph { nodes, .. } = &mut self.panel_mut(graph).kind {
            nodes[n as usize].pos = Some([x, y]);
        }
    }

    /// Join two nodes, creating them if needed.
    pub fn edge(&mut self, graph: Graph, a: impl Display, b: impl Display) {
        let (a, b) = (
            self.ensure_node(graph.0, &a.to_string()),
            self.ensure_node(graph.0, &b.to_string()),
        );
        self.ensure_edge(graph, a, b);
    }

    /// Remove the edge between two nodes.
    pub fn remove_edge(&mut self, graph: Graph, a: impl Display, b: impl Display) {
        let (a, b) = (
            self.node_index(graph.0, &a.to_string()),
            self.node_index(graph.0, &b.to_string()),
        );
        if self.graph_state(graph).edge(a, b).is_some() {
            self.emit(Op::RemoveEdge {
                panel: graph.0,
                a,
                b,
            });
        }
    }

    /// Set an edge's state, adding the edge if it isn't there.
    pub fn set_edge(&mut self, graph: Graph, a: impl Display, b: impl Display, state: &str) {
        let s = self.state_index(graph, state);
        let (a, b) = (
            self.ensure_node(graph.0, &a.to_string()),
            self.ensure_node(graph.0, &b.to_string()),
        );
        self.ensure_edge(graph, a, b);
        if self.graph_state(graph).edge(a, b).map(|e| e.state) == Some(s) {
            return;
        }
        let op = Op::EdgeSet {
            panel: graph.0,
            state: s,
            edges: vec![[a, b]],
        };
        self.shadow.apply(&op);
        if let Some(Op::EdgeSet {
            panel,
            state,
            edges,
            ..
        }) = self.pending.ops.last_mut()
        {
            if *panel == graph.0 && *state == s {
                edges.push([a, b]);
                return;
            }
        }
        self.pending.ops.push(op);
    }

    /// Show text on an edge, such as its weight. Adds the edge if needed.
    pub fn edge_label(
        &mut self,
        graph: Graph,
        a: impl Display,
        b: impl Display,
        text: impl Display,
    ) {
        let (a, b) = (
            self.ensure_node(graph.0, &a.to_string()),
            self.ensure_node(graph.0, &b.to_string()),
        );
        self.ensure_edge(graph, a, b);
        let text = Some(text.to_string());
        if self
            .graph_state(graph)
            .edge(a, b)
            .and_then(|e| e.label.as_ref())
            != text.as_ref()
        {
            self.emit(Op::EdgeLabel {
                panel: graph.0,
                a,
                b,
                text,
            });
        }
    }

    fn ensure_edge(&mut self, graph: Graph, a: u32, b: u32) {
        if self.graph_state(graph).edge(a, b).is_none() {
            self.emit(Op::AddEdge {
                panel: graph.0,
                a,
                b,
            });
        }
    }

    /// Show a named value alongside the animation. It keeps its value in
    /// later frames until set again.
    pub fn var(&mut self, name: &str, value: impl Display) {
        let value = value.to_string();
        if self.shadow.var(name) == Some(value.as_str()) {
            return;
        }
        self.emit(Op::Var {
            name: name.into(),
            value: Some(value),
        });
    }

    /// Stop showing a var.
    pub fn remove_var(&mut self, name: &str) {
        if self.shadow.var(name).is_none() {
            return;
        }
        self.emit(Op::Var {
            name: name.into(),
            value: None,
        });
    }

    /// Mark the frame being recorded so the viewer can jump to it.
    pub fn bookmark(&mut self, name: impl Into<String>) {
        self.pending.bookmark = Some(name.into());
    }

    /// End the current frame, with a message shown while it's on screen.
    pub fn frame(&mut self, message: impl Into<String>) {
        let mut frame = std::mem::take(&mut self.pending);
        frame.message = message.into();
        self.animation.frames.push(frame);
    }

    /// Number of frames recorded so far.
    pub fn frames(&self) -> usize {
        self.animation.frames.len()
    }

    /// The finished animation. Changes made since the last
    /// [`Recorder::frame`] become one final frame.
    pub fn finish(mut self) -> Animation {
        if self.pending != Frame::default() {
            self.frame("");
        }
        self.animation
    }

    /// Finish and write the animation as JSON.
    pub fn save(self, path: impl AsRef<Path>) -> io::Result<()> {
        let animation = self.finish();
        let mut out = BufWriter::new(std::fs::File::create(path)?);
        serde_json::to_writer(&mut out, &animation)?;
        out.flush()
    }

    fn emit(&mut self, op: Op) {
        self.shadow.apply(&op);
        self.pending.ops.push(op);
    }

    fn panel_mut<H: Handle>(&mut self, panel: H) -> &mut Panel {
        &mut self.animation.panels[panel.panel() as usize]
    }

    fn array_state(&self, array: Array) -> &crate::ArrayState {
        match &self.shadow.panels[array.0 as usize] {
            PanelState::Array(a) => a,
            _ => unreachable!("Array handle on a non-array panel"),
        }
    }

    fn tree_state(&self, tree: Tree) -> &crate::TreeState {
        match &self.shadow.panels[tree.0 as usize] {
            PanelState::Tree(t) => t,
            _ => unreachable!("Tree handle on a non-tree panel"),
        }
    }

    fn graph_state(&self, graph: Graph) -> &crate::GraphState {
        match &self.shadow.panels[graph.0 as usize] {
            PanelState::Graph(g) => g,
            _ => unreachable!("Graph handle on a non-graph panel"),
        }
    }

    /// Index of a node in the panel's table, adding it to the table (but
    /// not to the current frame) if new.
    fn node_index(&mut self, p: u16, name: &str) -> u32 {
        if let Some(&n) = self.node_ids[p as usize].get(name) {
            return n;
        }
        let (PanelKind::Tree { nodes } | PanelKind::Graph { nodes, .. }) =
            &mut self.animation.panels[p as usize].kind
        else {
            unreachable!("node on a panel without nodes")
        };
        let n = u32::try_from(nodes.len()).expect("too many nodes");
        nodes.push(crate::NodeDef {
            name: name.into(),
            pos: None,
        });
        self.shadow.panels[p as usize].add_node_slot();
        self.node_ids[p as usize].insert(name.into(), n);
        n
    }

    /// Index of a node, making sure it exists in the current frame.
    fn ensure_node(&mut self, p: u16, name: &str) -> u32 {
        let n = self.node_index(p, name);
        if !self.shadow.panels[p as usize].has_node(n) {
            self.emit(Op::AddNode { panel: p, node: n });
        }
        n
    }

    fn index(&mut self, array: Array, index: impl Target<Array>, past_end: bool) -> u32 {
        match self.resolve(array.0, index.target(), past_end) {
            At::Index(i) => i,
            At::Cell(_) => unreachable!(),
        }
    }

    /// Check a position against the panel and turn it into an [`At`].
    /// `past_end` allows an array index equal to the length.
    fn resolve(&mut self, p: u16, raw: Raw, past_end: bool) -> At {
        if let Raw::Key(name) = raw {
            return At::Index(self.ensure_node(p, &name));
        }
        let panel = &self.animation.panels[p as usize];
        let name = &panel.name;
        match (&self.shadow.panels[p as usize], raw) {
            (PanelState::Grid(g), Raw::Cell(x, y)) => {
                if x >= 0 && y >= 0 && x < g.width as i64 && y < g.height as i64 {
                    At::Cell([x as i32, y as i32])
                } else {
                    panic!(
                        "celluloid: cell ({x}, {y}) is outside the {}x{} grid {name:?}",
                        g.width, g.height
                    )
                }
            }
            (PanelState::Hex(_), Raw::Cell(q, r)) => match (i32::try_from(q), i32::try_from(r)) {
                (Ok(q), Ok(r)) => At::Cell([q, r]),
                _ => panic!("celluloid: hex cell ({q}, {r}) in {name:?} is too far out"),
            },
            (PanelState::Array(a), Raw::Index(i)) => {
                let len = a.items.len() as i64;
                if i >= 0 && (i < len || (past_end && i == len)) {
                    At::Index(i as u32)
                } else {
                    panic!("celluloid: index {i} is outside array {name:?} of length {len}")
                }
            }
            _ => unreachable!("handle and position kinds are checked by Target"),
        }
    }

    fn state_index<H: Handle>(&mut self, panel: H, name: &str) -> u16 {
        let states = &mut self.panel_mut(panel).states;
        if let Some(i) = states.iter().position(|s| s.name == name) {
            return i as u16;
        }
        let color = PALETTE[(states.len() - 1) % PALETTE.len()];
        states.push(StateDef {
            name: name.into(),
            color,
        });
        u16::try_from(states.len() - 1).expect("too many states")
    }

    fn marker_index<H: Handle>(&mut self, panel: H, name: &str) -> u16 {
        let markers = &mut self.panel_mut(panel).markers;
        if let Some(i) = markers.iter().position(|m| m.name == name) {
            return i as u16;
        }
        let color = PALETTE[(markers.len() + 4) % PALETTE.len()];
        markers.push(MarkerDef {
            name: name.into(),
            color,
        });
        let index = u16::try_from(markers.len() - 1).expect("too many markers");
        self.shadow.panels[panel.panel() as usize].add_marker();
        index
    }
}

/// Neutral colour for array items that haven't been given a state.
const ARRAY_DEFAULT: Color = Color::rgb(0x6b, 0x72, 0x80);

fn finite(n: impl Number) -> f64 {
    let v = n.to_f64();
    assert!(
        v.is_finite(),
        "celluloid: array values must be finite, got {v}"
    );
    v
}

fn parse_color(color: &str) -> Color {
    match Color::hex(color) {
        Some(c) => c,
        None => panic!("celluloid: invalid colour {color:?}, expected \"#rrggbb\""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Timeline;

    fn cell(x: i32, y: i32) -> At {
        At::Cell([x, y])
    }

    #[test]
    fn merges_and_dedupes_sets() {
        let mut rec = Recorder::new("t");
        let g = rec.grid("g", 4, 4);
        rec.set(g, (0, 0), "seen");
        rec.set(g, (1usize, 0usize), "seen");
        rec.set(g, [2i64, 0], "seen");
        rec.set(g, (0, 0), "seen");
        rec.frame("one");
        rec.set(g, (0, 0), "seen");
        rec.frame("two");
        let anim = rec.finish();

        assert_eq!(
            anim.frames[0].ops,
            vec![Op::Set {
                panel: 0,
                state: 1,
                cells: vec![cell(0, 0), cell(1, 0), cell(2, 0)],
            }]
        );
        assert!(anim.frames[1].ops.is_empty());
        assert_eq!(anim.panels[0].states[1].color, PALETTE[0]);
    }

    #[test]
    fn redraw_records_only_changes() {
        let mut rec = Recorder::new("t");
        let g = rec.grid("g", 3, 3);
        rec.redraw(g, |x, y| if x == y { "on" } else { "empty" });
        rec.frame("");
        rec.redraw(g, |x, y| if x == y || x == 0 { "on" } else { "empty" });
        rec.frame("");
        let anim = rec.finish();
        let cells = |f: usize| match &anim.frames[f].ops[..] {
            [Op::Set { cells, .. }] => cells.clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(cells(0), vec![cell(0, 0), cell(1, 1), cell(2, 2)]);
        assert_eq!(cells(1), vec![cell(0, 1), cell(0, 2)]);
    }

    #[test]
    fn replay_matches_recorder() {
        let mut rec = Recorder::new("t");
        let g = rec.grid("g", 5, 5);
        rec.state(g, "wall", "#000000");
        for i in 0..5 {
            rec.set(g, (i, 2), "wall");
            rec.marker(g, "me", (i, 0));
            rec.label(g, (i, 4), i);
            rec.var("i", i);
            if i == 3 {
                rec.bookmark("three");
                rec.fill(g, "seen");
            }
            rec.frame(format!("step {i}"));
        }
        rec.hide_marker(g, "me");
        rec.remove_var("i");
        let anim = rec.finish();
        assert_eq!(anim.frames.len(), 6);
        assert_eq!(anim.bookmarks().collect::<Vec<_>>(), vec![(3, "three")]);

        let mut tl = Timeline::new(anim);
        let s = tl.seek(3).clone();
        let PanelState::Grid(grid) = &s.panels[0] else {
            panic!()
        };
        assert!(grid.cells.iter().all(|&c| c == 2));
        assert_eq!(grid.markers, vec![Some([3, 0])]);
        assert_eq!(grid.labels.get(&[3, 4]).map(String::as_str), Some("3"));
        assert_eq!(s.var("i"), Some("3"));

        let s = tl.seek(5);
        let PanelState::Grid(grid) = &s.panels[0] else {
            panic!()
        };
        assert_eq!(grid.get([4, 2]), Some(1));
        assert_eq!(grid.markers, vec![None]);
        assert_eq!(s.var("i"), None);
    }

    #[test]
    fn hex_is_sparse() {
        let mut rec = Recorder::new("t");
        let h = rec.hex("h", Orientation::Pointy);
        rec.set(h, (-3, 2), "alive");
        rec.set(h, (5, -1), "alive");
        rec.label(h, (-3, 2), "a");
        rec.marker(h, "me", (100, 100));
        rec.frame("");
        rec.set(h, (5, -1), "empty");
        rec.set(h, (7, 7), "empty");
        rec.frame("");
        rec.fill(h, "empty");
        let anim = rec.finish();
        assert_eq!(
            anim.frames[1].ops.len(),
            1,
            "setting an empty cell empty is a no-op"
        );
        assert_eq!(anim.validate(), Ok(()));

        let mut tl = Timeline::new(anim);
        let PanelState::Hex(hex) = &tl.seek(1).panels[0] else {
            panic!()
        };
        assert_eq!(hex.cells.len(), 1);
        assert_eq!(hex.get([-3, 2]), 1);
        assert_eq!(hex.markers, vec![Some([100, 100])]);
        let PanelState::Hex(hex) = &tl.seek(2).panels[0] else {
            panic!()
        };
        assert!(hex.cells.is_empty());
    }

    #[test]
    fn array_ops() {
        let mut rec = Recorder::new("t");
        let a = rec.array("a", [3, 1, 2]);
        rec.array_style(a, ArrayStyle::Boxes);
        rec.set(a, 0, "pivot");
        rec.marker(a, "hi", 3);
        rec.swap(a, 0, 2);
        rec.swap(a, 1, 1);
        rec.value(a, 1, 1);
        rec.value(a, 1, 1.5);
        rec.push(a, 9);
        rec.pop(a);
        rec.move_item(a, 2, 0);
        rec.label(a, 0, "moved");
        let anim = rec.finish();
        assert_eq!(anim.validate(), Ok(()));
        let ops = &anim.frames[0].ops;
        assert!(!ops
            .iter()
            .any(|op| matches!(op, Op::Swap { a: 1, b: 1, .. })));
        assert_eq!(
            ops.iter()
                .filter(|op| matches!(op, Op::Value { .. }))
                .count(),
            1,
            "setting a value to itself is a no-op"
        );

        let mut tl = Timeline::new(anim);
        let PanelState::Array(arr) = &tl.seek(0).panels[0] else {
            panic!()
        };
        let items: Vec<_> = arr
            .items
            .iter()
            .map(|it| (it.value, it.state, it.label.as_deref()))
            .collect();
        assert_eq!(
            items,
            vec![(3.0, 1, Some("moved")), (2.0, 0, None), (1.5, 0, None)]
        );
        assert_eq!(arr.markers, vec![Some(3)]);
    }

    fn tree_of(anim: &Animation, frame: usize) -> crate::TreeState {
        let mut tl = Timeline::new(anim.clone());
        match &tl.seek(frame).panels[0] {
            PanelState::Tree(t) => t.clone(),
            _ => panic!(),
        }
    }

    #[test]
    fn tree_rotation_reparents() {
        let mut rec = Recorder::new("t");
        let t = rec.tree("bst");
        rec.right(t, 1, 2);
        rec.right(t, 2, 3);
        rec.frame("chain");
        rec.left(t, 2, 1);
        rec.frame("rotated");
        rec.left(t, 2, 1);
        rec.frame("");
        let anim = rec.finish();
        assert_eq!(anim.validate(), Ok(()));
        assert!(anim.frames[2].ops.is_empty());

        let names = |anim: &Animation| -> Vec<String> {
            anim.panels[0]
                .kind
                .nodes()
                .unwrap()
                .iter()
                .map(|n| n.name.clone())
                .collect()
        };
        assert_eq!(names(&anim), ["1", "2", "3"]);

        let before = tree_of(&anim, 0);
        assert_eq!(before.roots().collect::<Vec<_>>(), vec![0]);
        let after = tree_of(&anim, 1);
        assert_eq!(after.roots().collect::<Vec<_>>(), vec![1]);
        assert_eq!(after.nodes[1].children, vec![Some(0), Some(2)]);
        assert_eq!(after.nodes[0].parent, Some((1, 0)));
        assert_eq!(
            after.nodes[0].children,
            vec![None, None],
            "1 lost its right child"
        );
    }

    #[test]
    fn tree_reparents_ancestors_and_removes_cleanly() {
        let mut rec = Recorder::new("t");
        let t = rec.tree("t");
        rec.add_child(t, "root", "a");
        rec.add_child(t, "root", "b");
        rec.add_child(t, "a", "c");
        rec.frame("");
        rec.add_child(t, "c", "root");
        rec.frame("");
        rec.remove_node(t, "a");
        rec.detach(t, "b");
        rec.frame("");
        let anim = rec.finish();
        assert_eq!(anim.validate(), Ok(()));

        let moved = tree_of(&anim, 1);
        assert_eq!(
            moved.roots().collect::<Vec<_>>(),
            vec![1],
            "a is the new root"
        );
        assert_eq!(moved.nodes[3].children, vec![Some(0)]);
        assert_eq!(moved.nodes[0].children, vec![None, Some(2)]);

        let after = tree_of(&anim, 2);
        assert!(!after.nodes[1].present);
        assert_eq!(after.roots().collect::<Vec<_>>(), vec![2, 3]);
        assert_eq!(after.nodes[0].children, vec![None, None]);
    }

    #[test]
    fn graph_edges() {
        let mut rec = Recorder::new("t");
        let g = rec.graph("g", false);
        rec.edge(g, "AA", "BB");
        rec.edge(g, "BB", "AA");
        rec.edge_label(g, "AA", "BB", 7);
        rec.set_edge(g, "BB", "CC", "open");
        rec.set_edge(g, "CC", "DD", "open");
        rec.set(g, "AA", "start");
        rec.marker(g, "me", "CC");
        rec.place(g, "AA", 1.0, 2.0);
        rec.frame("");
        rec.set_edge(g, "AA", "BB", "path");
        rec.set_edge(g, "BB", "CC", "path");
        rec.frame("");
        rec.remove_node(g, "CC");
        rec.frame("");
        let anim = rec.finish();
        assert_eq!(anim.validate(), Ok(()));
        assert_eq!(
            anim.frames[1].ops,
            vec![Op::EdgeSet {
                panel: 0,
                state: 3,
                edges: vec![[0, 1], [1, 2]],
            }],
            "consecutive edge states merge"
        );
        assert_eq!(
            anim.panels[0].kind.nodes().unwrap()[0].pos,
            Some([1.0, 2.0])
        );

        let json = serde_json::to_string(&anim).unwrap();
        let anim = crate::from_json(&json).unwrap();
        let mut tl = Timeline::new(anim);
        let PanelState::Graph(gs) = tl.seek(0).clone().panels[0].clone() else {
            panic!()
        };
        assert_eq!(gs.edges.len(), 3);
        assert_eq!(gs.edge(1, 0).unwrap().label.as_deref(), Some("7"));
        assert_eq!(gs.edge(2, 3).unwrap().state, 1);
        assert_eq!(gs.nodes[0].state, 2);
        assert_eq!(gs.markers, vec![Some(2)]);

        let PanelState::Graph(gs) = tl.seek(2).clone().panels[0].clone() else {
            panic!()
        };
        assert_eq!(gs.edges.keys().collect::<Vec<_>>(), vec![&[0, 1]]);
        assert!(!gs.nodes[2].present);
    }

    #[test]
    fn directed_edges_are_distinct() {
        let mut rec = Recorder::new("t");
        let g = rec.graph("g", true);
        rec.edge(g, 1, 2);
        rec.edge(g, 2, 1);
        let anim = rec.finish();
        let mut tl = Timeline::new(anim);
        let PanelState::Graph(gs) = &tl.seek(0).panels[0] else {
            panic!()
        };
        assert_eq!(gs.edges.len(), 2);
    }

    #[test]
    #[should_panic(expected = "cell (5, 0) is outside the 5x5 grid")]
    fn out_of_bounds_panics() {
        let mut rec = Recorder::new("t");
        let g = rec.grid("g", 5, 5);
        rec.set(g, (5, 0), "x");
    }

    #[test]
    #[should_panic(expected = "index 4 is outside array \"a\" of length 3")]
    fn array_out_of_bounds_panics() {
        let mut rec = Recorder::new("t");
        let a = rec.array("a", [1, 2, 3]);
        rec.marker(a, "end", 3);
        rec.set(a, 4, "x");
    }
}
