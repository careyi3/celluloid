# Celluloid

Rust visualisation tool and library for algorithmic problem solving.

Add a few calls to your code and it writes a small JSON file. Open that in the viewer and step through it frame by frame, forwards and backwards. Built for Advent of Code style grids and graphs, but anything you can draw as a grid, array, tree or graph works.

You can find the crates.io listings here: [celluloid](https://crates.io/crates/celluloid), [celluloid-core](https://crates.io/crates/celluloid-core) and [celluloid-view](https://crates.io/crates/celluloid-view).

![CI](https://github.com/careyi3/celluloid/actions/workflows/test.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/celluloid-core.svg)](https://crates.io/crates/celluloid-core)
[![Crates.io](https://img.shields.io/crates/d/celluloid-core.svg)](https://crates.io/crates/celluloid-core)

## Setup

Install the viewer:

```bash
$ cargo install celluloid
```

Add the recorder to your project:

```bash
$ cargo add celluloid-core
```

## Record

```rust
use celluloid_core::Recorder;

let mut rec = Recorder::new("Day 12");
let g = rec.grid("map", width, height);
rec.set(g, (x, y), "wall");
rec.marker(g, "me", (x, y));
rec.var("steps", steps);
rec.frame("step");
rec.save("day12.json")?;
```

| Panel | Create | Address |
| --- | --- | --- |
| Grid | `rec.grid(name, w, h)` | `(x, y)` |
| Hex | `rec.hex(name, Orientation::Pointy)` | `(q, r)` |
| Array | `rec.array(name, values)` | index |
| Tree | `rec.tree(name)` | node name |
| Graph | `rec.graph(name, directed)` | node name |

Grids and hex panels have a camera: `rec.camera(g, (x, y), 3.0)` eases the view to that cell at 3× zoom, and `rec.reset_camera(g)` shows the whole panel again. Call it each frame to follow something.

Grids can also have lines and notes drawn over them. Points are `(x, y)` in cells, with whole numbers at cell centres, so `(2.5, 3)` is on the edge between `(2, 3)` and `(3, 3)` and `(-0.5, -0.5)` is the top left corner:

```rust
rec.line(g, (0.5, 0.0), (1.0, 0.5), "contour");      // a straight line
rec.path(g, [(0, 0), (3, 0), (3, 2)], "route");     // joined-up lines
rec.line_color(g, "route", "#e15759");
rec.note(g, (0.5, 0.5), "11");                      // small text anywhere
rec.clear_lines(g, "contour");                       // or erase_line, clear_all_lines
```

There are runnable examples in [`celluloid-core/examples`](celluloid-core/examples):

```bash
$ cargo run -p celluloid-core --example avl
$ celluloid avl.json
```

## View

```bash
$ celluloid                # browse .json files here
$ celluloid day12.json     # open one; reloads when it changes
```

Space plays, ← → step, `[` `]` jump between bookmarks, pinch zooms.

## Share

```bash
$ celluloid bundle day12.json      # self-contained day12.html
$ celluloid convert old.json       # upgrade a 0.0.1 file
```

Note: `bundle` needs the web viewer built in, which the crates.io install doesn't have. Build it from this repo:

```bash
$ cargo xtask web
$ cargo install --path celluloid
```

To embed the viewer in a page, serve `celluloid/web/*` and:

```js
import init, { Player } from "/celluloid_web.js";
await init();
const player = await Player.start(canvas);
player.load(json);
```

## Crates

| Crate | What it is |
| --- | --- |
| [`celluloid-core`](celluloid-core) | The recorder and file format |
| [`celluloid-view`](celluloid-view) | The egui viewer, to embed in your own app |
| [`celluloid`](celluloid) | The desktop viewer |
| `celluloid-web` | The viewer built for the browser (not published) |

## Development

This repo uses [`just`](https://github.com/casey/just) as a task runner. Run `just` with no arguments to list the recipes:

| Recipe | What it does |
| --- | --- |
| `just build` | Build everything, examples included |
| `just lint` | Format code, then run clippy (warnings denied) |
| `just test` | Run the test suite |
| `just web` | Build the web viewer into `celluloid/web` |

You don't need `just` installed; every recipe is a thin wrapper over a `cargo` command you can run directly.

## License

MIT
