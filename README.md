# Celluloid

Record what your Rust puzzle solution does, then scrub through it.

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
