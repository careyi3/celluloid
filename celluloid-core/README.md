# celluloid-core

Recorder and file format for [Celluloid](https://github.com/careyi3/celluloid) animations.

Add a few calls to your code and it writes a JSON file you can step through in the `celluloid` viewer. Each frame only stores what changed, so recording every step of a big loop is fine.

You can find the [crates.io listing here](https://crates.io/crates/celluloid-core).

![CI](https://github.com/careyi3/celluloid/actions/workflows/test.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/celluloid-core.svg)](https://crates.io/crates/celluloid-core)
[![Crates.io](https://img.shields.io/crates/d/celluloid-core.svg)](https://crates.io/crates/celluloid-core)

## Setup

```bash
$ cargo add celluloid-core
$ cargo install celluloid
```

## Usage

```rust
use celluloid_core::Recorder;

let mut rec = Recorder::new("Day 12");
let g = rec.grid("map", 10, 10);
rec.state(g, "wall", "#555555");
rec.set(g, (3, 4), "wall");

for x in 0..10 {
    rec.marker(g, "me", (x, 0));
    rec.var("steps", x);
    rec.frame("step");
}

rec.save("day12.json")?;
```

Then open it:

```bash
$ celluloid day12.json
```

## Panels

| Panel | Create | Address |
| --- | --- | --- |
| Grid | `rec.grid(name, w, h)` | `(x, y)` |
| Hex | `rec.hex(name, Orientation::Pointy)` | `(q, r)` |
| Array | `rec.array(name, values)` | index |
| Tree | `rec.tree(name)` | node name |
| Graph | `rec.graph(name, directed)` | node name |

A recording can have as many panels as you like.

## Calls

| Call | What it does |
| --- | --- |
| `rec.state(p, name, "#rrggbb")` | Give a state a colour. Unset states get one from a palette |
| `rec.set(p, at, state)` | Set an element's state |
| `rec.label(p, at, text)` | Write text on an element |
| `rec.marker(p, name, at)` | Move a named dot. It glides between frames |
| `rec.var(name, value)` | Show a value beside the animation |
| `rec.bookmark(name)` | Mark this frame so you can jump to it |
| `rec.frame(message)` | End the frame |

Arrays also have `swap`, `insert`, `push`, `remove`, `pop` and `move_item`. Trees use `left`, `right` and `add_child`. Graphs use `edge` and `set_edge`. See [docs.rs](https://docs.rs/celluloid-core) for the full list.

There are runnable examples in [`examples`](examples):

```bash
$ cargo run -p celluloid-core --example dijkstra
$ celluloid dijkstra.json
```

## Reading files

`from_json` loads a file and `Timeline` gives you the state at any frame:

```rust
use celluloid_core::{from_json, Timeline};

let json = std::fs::read_to_string("day12.json")?;
let mut timeline = Timeline::new(from_json(&json)?);
let state = timeline.seek(5);
```

Files from 0.0.1 still load and are converted on the way in.

## License

MIT
