use crate::{Animation, At, Op, Panel, PanelKind};
use std::collections::BTreeMap;

/// Everything visible at one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub panels: Vec<PanelState>,
    /// In the order they were first set.
    pub vars: Vec<(String, String)>,
}

/// What one panel looks like at a frame.
#[derive(Debug, Clone, PartialEq)]
pub enum PanelState {
    Grid(GridState),
    Hex(HexState),
    Array(ArrayState),
    Tree(TreeState),
    Graph(GraphState),
}

/// A tree or graph node. Indexed like the panel's node table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    /// Whether the node exists at this frame.
    pub present: bool,
    pub state: u16,
    pub label: Option<String>,
    /// Trees: the parent and which of its slots this node is in.
    pub parent: Option<(u32, u16)>,
    /// Trees: child slots, `None` where empty.
    pub children: Vec<Option<u32>>,
}

impl Node {
    fn reset(&mut self) {
        *self = Node::default();
    }
}

/// A tree panel at one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeState {
    pub nodes: Vec<Node>,
    pub markers: Vec<Option<u32>>,
}

impl TreeState {
    /// Present nodes without a parent, in node order.
    pub fn roots(&self) -> impl Iterator<Item = u32> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.present && n.parent.is_none())
            .map(|(i, _)| i as u32)
    }

    fn detach(&mut self, child: u32) {
        if let Some((p, slot)) = self.nodes[child as usize].parent.take() {
            if let Some(s) = self.nodes[p as usize].children.get_mut(slot as usize) {
                *s = None;
            }
        }
    }

    fn set_child(&mut self, parent: u32, slot: u16, child: Option<u32>) {
        let n = self.nodes.len() as u32;
        if parent >= n || child.is_some_and(|c| c >= n || c == parent) {
            return;
        }
        if let Some(c) = child {
            let mut node = parent;
            while let Some((up, _)) = self.nodes[node as usize].parent {
                if up == c {
                    self.detach(node);
                    break;
                }
                node = up;
            }
        }
        let slot_ix = slot as usize;
        if let Some(Some(old)) = self.nodes[parent as usize].children.get(slot_ix).copied() {
            if Some(old) == child {
                return;
            }
            self.nodes[old as usize].parent = None;
            self.nodes[parent as usize].children[slot_ix] = None;
        }
        if let Some(c) = child {
            self.detach(c);
            let kids = &mut self.nodes[parent as usize].children;
            if kids.len() <= slot_ix {
                kids.resize(slot_ix + 1, None);
            }
            kids[slot_ix] = Some(c);
            self.nodes[c as usize].parent = Some((parent, slot));
        }
    }

    fn remove(&mut self, node: u32) {
        self.detach(node);
        for c in std::mem::take(&mut self.nodes[node as usize].children)
            .into_iter()
            .flatten()
        {
            self.nodes[c as usize].parent = None;
        }
        self.nodes[node as usize].reset();
    }
}

/// A graph panel at one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GraphState {
    pub directed: bool,
    pub nodes: Vec<Node>,
    /// Keyed `[a, b]`; for undirected graphs `a <= b`.
    pub edges: BTreeMap<[u32; 2], Edge>,
    pub markers: Vec<Option<u32>>,
}

/// A graph edge's state and label.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Edge {
    pub state: u16,
    pub label: Option<String>,
}

impl GraphState {
    /// The key an edge between `a` and `b` is stored under.
    pub fn key(&self, a: u32, b: u32) -> [u32; 2] {
        if !self.directed && a > b {
            [b, a]
        } else {
            [a, b]
        }
    }

    /// The edge between `a` and `b`, if there is one.
    pub fn edge(&self, a: u32, b: u32) -> Option<&Edge> {
        self.edges.get(&self.key(a, b))
    }
}

fn node_mut(nodes: &mut [Node], at: At) -> Option<&mut Node> {
    match at {
        At::Index(n) => nodes.get_mut(n as usize),
        At::Cell(_) => None,
    }
}

fn node_ref(nodes: &[Node], at: At) -> Option<&Node> {
    match at {
        At::Index(n) => nodes.get(n as usize).filter(|n| n.present),
        At::Cell(_) => None,
    }
}

/// A grid panel at one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct GridState {
    pub width: u32,
    pub height: u32,
    /// Row-major state indices.
    pub cells: Vec<u16>,
    pub labels: BTreeMap<[u32; 2], String>,
    /// Indexed like `Panel::markers`; `None` is hidden.
    pub markers: Vec<Option<[u32; 2]>>,
}

impl GridState {
    /// State index of a cell, or `None` if it's off the grid.
    pub fn get(&self, [x, y]: [u32; 2]) -> Option<u16> {
        self.index([x, y]).map(|i| self.cells[i])
    }

    pub(crate) fn index(&self, [x, y]: [u32; 2]) -> Option<usize> {
        (x < self.width && y < self.height).then(|| (y * self.width + x) as usize)
    }

    fn cell(&self, at: At) -> Option<[u32; 2]> {
        match at {
            At::Cell([x, y]) => {
                let c = [u32::try_from(x).ok()?, u32::try_from(y).ok()?];
                self.index(c).map(|_| c)
            }
            At::Index(_) => None,
        }
    }
}

/// Sparse hex board in axial `[q, r]` coordinates.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HexState {
    /// Cells not in `states[0]`.
    pub cells: BTreeMap<[i32; 2], u16>,
    pub labels: BTreeMap<[i32; 2], String>,
    pub markers: Vec<Option<[i32; 2]>>,
}

impl HexState {
    /// State index of a cell. Cells never touched are 0.
    pub fn get(&self, at: [i32; 2]) -> u16 {
        self.cells.get(&at).copied().unwrap_or(0)
    }

    fn set(&mut self, at: [i32; 2], state: u16) {
        if state == 0 {
            self.cells.remove(&at);
        } else {
            self.cells.insert(at, state);
        }
    }
}

/// An array panel at one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArrayState {
    pub items: Vec<Item>,
    /// Indices; may equal `items.len()`.
    pub markers: Vec<Option<u32>>,
    next_id: u32,
}

/// One array element. `id` stays with the item as it moves, so the viewer
/// can tell a swap from two values changing in place.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: u32,
    pub value: f64,
    pub state: u16,
    pub label: Option<String>,
}

impl ArrayState {
    fn new(values: &[f64], markers: usize) -> Self {
        let mut a = Self {
            markers: vec![None; markers],
            ..Self::default()
        };
        for &v in values {
            a.insert(a.items.len(), v);
        }
        a
    }

    pub(crate) fn insert(&mut self, at: usize, value: f64) {
        let item = Item {
            id: self.next_id,
            value,
            state: 0,
            label: None,
        };
        self.next_id += 1;
        self.items.insert(at, item);
    }

    fn item_mut(&mut self, at: At) -> Option<&mut Item> {
        match at {
            At::Index(i) => self.items.get_mut(i as usize),
            At::Cell(_) => None,
        }
    }
}

impl PanelState {
    /// A panel as it looks before any ops.
    pub fn new(panel: &Panel) -> Self {
        let markers = panel.markers.len();
        match &panel.kind {
            PanelKind::Grid { width, height } => PanelState::Grid(GridState {
                width: *width,
                height: *height,
                cells: vec![0; *width as usize * *height as usize],
                labels: BTreeMap::new(),
                markers: vec![None; markers],
            }),
            PanelKind::Hex { .. } => PanelState::Hex(HexState {
                markers: vec![None; markers],
                ..HexState::default()
            }),
            PanelKind::Array { values, .. } => PanelState::Array(ArrayState::new(values, markers)),
            PanelKind::Tree { nodes } => PanelState::Tree(TreeState {
                nodes: vec![Node::default(); nodes.len()],
                markers: vec![None; markers],
            }),
            PanelKind::Graph { directed, nodes } => PanelState::Graph(GraphState {
                directed: *directed,
                nodes: vec![Node::default(); nodes.len()],
                edges: BTreeMap::new(),
                markers: vec![None; markers],
            }),
        }
    }

    /// Make room for a node added to the panel's node table.
    pub(crate) fn add_node_slot(&mut self) {
        match self {
            PanelState::Tree(t) => t.nodes.push(Node::default()),
            PanelState::Graph(g) => g.nodes.push(Node::default()),
            _ => {}
        }
    }

    /// Whether node `n` exists at this frame.
    pub fn has_node(&self, n: u32) -> bool {
        match self {
            PanelState::Tree(t) => t.nodes.get(n as usize).is_some_and(|n| n.present),
            PanelState::Graph(g) => g.nodes.get(n as usize).is_some_and(|n| n.present),
            _ => false,
        }
    }

    /// Number of elements, for sizing snapshots.
    fn size(&self) -> usize {
        match self {
            PanelState::Grid(g) => g.cells.len(),
            PanelState::Hex(h) => h.cells.len(),
            PanelState::Array(a) => a.items.len(),
            PanelState::Tree(t) => t.nodes.len(),
            PanelState::Graph(g) => g.nodes.len() + g.edges.len(),
        }
    }

    pub(crate) fn add_marker(&mut self) {
        match self {
            PanelState::Grid(g) => g.markers.push(None),
            PanelState::Hex(h) => h.markers.push(None),
            PanelState::Array(a) => a.markers.push(None),
            PanelState::Tree(t) => t.markers.push(None),
            PanelState::Graph(g) => g.markers.push(None),
        }
    }

    /// State index of the element at `at`, if it exists.
    pub fn state_at(&self, at: At) -> Option<u16> {
        match (self, at) {
            (PanelState::Grid(g), at) => g.cell(at).and_then(|c| g.get(c)),
            (PanelState::Hex(h), At::Cell(c)) => Some(h.get(c)),
            (PanelState::Array(a), At::Index(i)) => a.items.get(i as usize).map(|it| it.state),
            (PanelState::Tree(t), at) => node_ref(&t.nodes, at).map(|n| n.state),
            (PanelState::Graph(g), at) => node_ref(&g.nodes, at).map(|n| n.state),
            _ => None,
        }
    }

    /// Label of the element at `at`, if it has one.
    pub fn label_at(&self, at: At) -> Option<&str> {
        match (self, at) {
            (PanelState::Grid(g), at) => g.cell(at).and_then(|c| g.labels.get(&c)),
            (PanelState::Hex(h), At::Cell(c)) => h.labels.get(&c),
            (PanelState::Array(a), At::Index(i)) => {
                a.items.get(i as usize).and_then(|it| it.label.as_ref())
            }
            (PanelState::Tree(t), at) => node_ref(&t.nodes, at).and_then(|n| n.label.as_ref()),
            (PanelState::Graph(g), at) => node_ref(&g.nodes, at).and_then(|n| n.label.as_ref()),
            _ => None,
        }
        .map(String::as_str)
    }

    /// Where a marker is, or `None` if it's hidden.
    pub fn marker_at(&self, marker: usize) -> Option<At> {
        match self {
            PanelState::Grid(g) => g
                .markers
                .get(marker)
                .copied()
                .flatten()
                .map(|[x, y]| At::Cell([x as i32, y as i32])),
            PanelState::Hex(h) => h.markers.get(marker).copied().flatten().map(At::Cell),
            PanelState::Array(a) => a.markers.get(marker).copied().flatten().map(At::Index),
            PanelState::Tree(t) => t.markers.get(marker).copied().flatten().map(At::Index),
            PanelState::Graph(g) => g.markers.get(marker).copied().flatten().map(At::Index),
        }
    }

    fn apply(&mut self, op: &Op) {
        match (self, op) {
            (PanelState::Grid(g), Op::Set { state, cells, .. }) => {
                for &at in cells {
                    if let Some(i) = g.cell(at).and_then(|c| g.index(c)) {
                        g.cells[i] = *state;
                    }
                }
            }
            (PanelState::Hex(h), Op::Set { state, cells, .. }) => {
                for &at in cells {
                    if let At::Cell(c) = at {
                        h.set(c, *state);
                    }
                }
            }
            (PanelState::Array(a), Op::Set { state, cells, .. }) => {
                for &at in cells {
                    if let Some(item) = a.item_mut(at) {
                        item.state = *state;
                    }
                }
            }

            (PanelState::Grid(g), Op::Fill { state, .. }) => g.cells.fill(*state),
            (PanelState::Hex(h), Op::Fill { state, .. }) => {
                if *state == 0 {
                    h.cells.clear();
                } else {
                    h.cells.values_mut().for_each(|s| *s = *state);
                }
            }
            (PanelState::Array(a), Op::Fill { state, .. }) => {
                a.items.iter_mut().for_each(|it| it.state = *state);
            }

            (PanelState::Grid(g), Op::Label { at, text, .. }) => {
                if let Some(c) = g.cell(*at) {
                    set_label(&mut g.labels, c, text);
                }
            }
            (
                PanelState::Hex(h),
                Op::Label {
                    at: At::Cell(c),
                    text,
                    ..
                },
            ) => {
                set_label(&mut h.labels, *c, text);
            }
            (PanelState::Array(a), Op::Label { at, text, .. }) => {
                if let Some(item) = a.item_mut(*at) {
                    item.label = text.clone();
                }
            }

            (PanelState::Grid(g), Op::Marker { marker, at, .. }) => {
                if let Some(m) = g.markers.get_mut(*marker as usize) {
                    *m = at.and_then(|at| g_cell(g.width, g.height, at));
                }
            }
            (PanelState::Hex(h), Op::Marker { marker, at, .. }) => {
                if let Some(m) = h.markers.get_mut(*marker as usize) {
                    *m = match at {
                        Some(At::Cell(c)) => Some(*c),
                        _ => None,
                    };
                }
            }
            (PanelState::Array(a), Op::Marker { marker, at, .. }) => {
                if let Some(m) = a.markers.get_mut(*marker as usize) {
                    *m = match at {
                        Some(At::Index(i)) => Some(*i),
                        _ => None,
                    };
                }
            }

            (PanelState::Array(a), Op::Value { at, value, .. }) => {
                if let Some(item) = a.items.get_mut(*at as usize) {
                    item.value = *value;
                }
            }
            (PanelState::Array(a), Op::Swap { a: i, b: j, .. }) => {
                let len = a.items.len();
                if (*i as usize) < len && (*j as usize) < len {
                    a.items.swap(*i as usize, *j as usize);
                }
            }
            (PanelState::Array(a), Op::Insert { at, value, .. }) => {
                if *at as usize <= a.items.len() {
                    a.insert(*at as usize, *value);
                }
            }
            (PanelState::Array(a), Op::Remove { at, .. }) => {
                if (*at as usize) < a.items.len() {
                    a.items.remove(*at as usize);
                }
            }
            (PanelState::Array(a), Op::Move { from, to, .. }) => {
                let len = a.items.len();
                if (*from as usize) < len && (*to as usize) < len {
                    let item = a.items.remove(*from as usize);
                    a.items.insert(*to as usize, item);
                }
            }
            (PanelState::Tree(t), op) => apply_nodes(&mut t.nodes, &mut t.markers, op)
                .unwrap_or_else(|| match op {
                    Op::RemoveNode { node, .. } if (*node as usize) < t.nodes.len() => {
                        t.remove(*node)
                    }
                    Op::Child {
                        parent,
                        slot,
                        child,
                        ..
                    } => t.set_child(*parent, *slot, *child),
                    _ => {}
                }),
            (PanelState::Graph(g), op) => apply_nodes(&mut g.nodes, &mut g.markers, op)
                .unwrap_or_else(|| match op {
                    Op::RemoveNode { node, .. } if (*node as usize) < g.nodes.len() => {
                        g.edges.retain(|[a, b], _| a != node && b != node);
                        g.nodes[*node as usize].reset();
                    }
                    Op::AddEdge { a, b, .. } => {
                        let k = g.key(*a, *b);
                        g.edges.entry(k).or_default();
                    }
                    Op::RemoveEdge { a, b, .. } => {
                        let k = g.key(*a, *b);
                        g.edges.remove(&k);
                    }
                    Op::EdgeSet { state, edges, .. } => {
                        for &[a, b] in edges {
                            let k = g.key(a, b);
                            if let Some(e) = g.edges.get_mut(&k) {
                                e.state = *state;
                            }
                        }
                    }
                    Op::EdgeLabel { a, b, text, .. } => {
                        let k = g.key(*a, *b);
                        if let Some(e) = g.edges.get_mut(&k) {
                            e.label = text.clone();
                        }
                    }
                    _ => {}
                }),
            _ => {}
        }
    }
}

/// Ops that work the same on tree and graph nodes. Returns `None` for
/// ops it doesn't handle.
fn apply_nodes(nodes: &mut [Node], markers: &mut [Option<u32>], op: &Op) -> Option<()> {
    match op {
        Op::Set { state, cells, .. } => {
            for &at in cells {
                if let Some(n) = node_mut(nodes, at) {
                    n.state = *state;
                }
            }
        }
        Op::Fill { state, .. } => nodes
            .iter_mut()
            .filter(|n| n.present)
            .for_each(|n| n.state = *state),
        Op::Label { at, text, .. } => {
            if let Some(n) = node_mut(nodes, *at) {
                n.label = text.clone();
            }
        }
        Op::Marker { marker, at, .. } => {
            if let Some(m) = markers.get_mut(*marker as usize) {
                *m = match at {
                    Some(At::Index(n)) => Some(*n),
                    _ => None,
                };
            }
        }
        Op::AddNode { node, .. } => {
            if let Some(n) = nodes.get_mut(*node as usize) {
                n.present = true;
            }
        }
        _ => return None,
    }
    Some(())
}

fn g_cell(width: u32, height: u32, at: At) -> Option<[u32; 2]> {
    match at {
        At::Cell([x, y]) if x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height => {
            Some([x as u32, y as u32])
        }
        _ => None,
    }
}

fn set_label<K: Ord + Copy>(labels: &mut BTreeMap<K, String>, at: K, text: &Option<String>) {
    match text {
        Some(t) => {
            labels.insert(at, t.clone());
        }
        None => {
            labels.remove(&at);
        }
    }
}

impl State {
    /// The state before frame 0's ops run.
    pub fn initial(animation: &Animation) -> Self {
        Self {
            panels: animation.panels.iter().map(PanelState::new).collect(),
            vars: Vec::new(),
        }
    }

    /// Current value of a var.
    pub fn var(&self, name: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Apply one op. Ops that refer to things that don't exist are ignored;
    /// use [`Animation::validate`] to report them.
    pub fn apply(&mut self, op: &Op) {
        if let Op::Var { name, value } = op {
            let existing = self.vars.iter().position(|(n, _)| n == name);
            match (existing, value) {
                (Some(i), Some(v)) => self.vars[i].1 = v.clone(),
                (None, Some(v)) => self.vars.push((name.clone(), v.clone())),
                (Some(i), None) => {
                    self.vars.remove(i);
                }
                (None, None) => {}
            }
        } else if let Some(p) = op.panel().and_then(|p| self.panels.get_mut(p as usize)) {
            p.apply(op);
        }
    }
}

/// Random access to the state at any frame.
///
/// Keeps a full snapshot every `interval` frames, so seeking costs at most
/// one snapshot clone plus `interval` frames of ops, and stepping forward
/// costs one frame.
pub struct Timeline {
    animation: Animation,
    snapshots: Vec<State>,
    interval: usize,
    cursor: usize,
    current: State,
}

/// Roughly how many elements all snapshots together may hold.
const SNAPSHOT_BUDGET: usize = 32 * 1024 * 1024;
const MIN_INTERVAL: usize = 16;

impl Timeline {
    /// Replay `animation` once to build snapshots. Starts at frame 0.
    pub fn new(animation: Animation) -> Self {
        let mut state = State::initial(&animation);
        let mut snapshots = Vec::new();
        let mut interval = MIN_INTERVAL;
        let mut stored = 0usize;
        for (i, frame) in animation.frames.iter().enumerate() {
            frame.ops.iter().for_each(|op| state.apply(op));
            if i % interval == 0 {
                stored += state
                    .panels
                    .iter()
                    .map(PanelState::size)
                    .sum::<usize>()
                    .max(1);
                snapshots.push(state.clone());
                if stored > SNAPSHOT_BUDGET {
                    snapshots = snapshots.into_iter().step_by(2).collect();
                    stored /= 2;
                    interval *= 2;
                }
            }
        }

        let current = snapshots
            .first()
            .cloned()
            .unwrap_or_else(|| State::initial(&animation));
        Self {
            animation,
            snapshots,
            interval,
            cursor: 0,
            current,
        }
    }

    /// The animation being replayed.
    pub fn animation(&self) -> &Animation {
        &self.animation
    }

    /// Number of frames.
    pub fn len(&self) -> usize {
        self.animation.frames.len()
    }

    /// Whether there are no frames.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Index of the frame [`Self::state`] belongs to.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// State at the current frame.
    pub fn state(&self) -> &State {
        &self.current
    }

    /// Move to `frame` (clamped to the last frame) and return its state.
    pub fn seek(&mut self, frame: usize) -> &State {
        if self.is_empty() {
            return &self.current;
        }
        let frame = frame.min(self.len() - 1);

        let from = if frame >= self.cursor && frame - self.cursor <= self.interval {
            self.cursor
        } else {
            let k = frame / self.interval;
            self.current = self.snapshots[k].clone();
            k * self.interval
        };
        for f in &self.animation.frames[from + 1..=frame] {
            f.ops.iter().for_each(|op| self.current.apply(op));
        }
        self.cursor = frame;
        &self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Recorder;

    fn counting(frames: usize) -> Animation {
        let mut rec = Recorder::new("count");
        let g = rec.grid("g", frames, 1);
        let a = rec.array("a", [0]);
        for i in 0..frames {
            rec.set(g, (i, 0), "on");
            rec.push(a, i);
            if i % 3 == 0 {
                rec.swap(a, 0, i);
            }
            rec.var("i", i);
            rec.frame(format!("{i}"));
        }
        rec.finish()
    }

    fn naive(anim: &Animation, frame: usize) -> State {
        let mut s = State::initial(anim);
        for f in &anim.frames[..=frame] {
            f.ops.iter().for_each(|op| s.apply(op));
        }
        s
    }

    #[test]
    fn seek_matches_naive_replay_in_any_order() {
        let anim = counting(100);
        let mut tl = Timeline::new(anim.clone());
        for &f in &[0, 1, 2, 50, 49, 17, 99, 16, 15, 0, 98, 99, 33, 34, 70] {
            assert_eq!(tl.seek(f), &naive(&anim, f), "frame {f}");
            assert_eq!(tl.cursor(), f);
        }
        assert_eq!(tl.seek(1000).var("i"), Some("99"));
    }

    #[test]
    fn snapshot_interval_grows_past_budget() {
        let mut rec = Recorder::new("big");
        let g = rec.grid("g", 1024, 1024);
        for i in 0..800 {
            rec.set(g, (i, 0), "on");
            rec.frame("");
        }
        let anim = rec.finish();
        let mut tl = Timeline::new(anim.clone());
        assert!(tl.interval > MIN_INTERVAL);
        assert!(tl.snapshots.len() * (1 << 20) <= SNAPSHOT_BUDGET + (1 << 20));
        for &f in &[799, 3, 200, 201, 64, 517] {
            assert_eq!(tl.seek(f), &naive(&anim, f), "frame {f}");
        }
    }

    #[test]
    fn array_items_keep_identity() {
        let mut rec = Recorder::new("ids");
        let a = rec.array("a", [10, 20, 30]);
        rec.swap(a, 0, 2);
        rec.move_item(a, 0, 1);
        rec.insert(a, 0, 5);
        rec.remove(a, 3);
        let anim = rec.finish();
        let state = naive(&anim, 0);
        let PanelState::Array(arr) = &state.panels[0] else {
            panic!()
        };
        let ids: Vec<_> = arr.items.iter().map(|it| (it.id, it.value)).collect();
        assert_eq!(ids, vec![(3, 5.0), (1, 20.0), (2, 30.0)]);
    }

    #[test]
    fn empty_animation() {
        let mut tl = Timeline::new(Recorder::new("empty").finish());
        assert!(tl.is_empty());
        assert!(tl.seek(5).panels.is_empty());
    }
}
