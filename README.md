# Celluloid

Record what your Rust puzzle solution does, then scrub through it.

## Install

```bash
cargo xtask web                  # optional, for `celluloid bundle`
cargo install --path celluloid
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

Examples: `cargo run -p celluloid-core --example avl` (also `game_of_life`,
`insertion_sort`, `hex_life`, `dijkstra`, `filesystem`).

## View

```bash
celluloid                # browse .json files here
celluloid day12.json     # open one; reloads when it changes
```

Space plays, ← → step, `[` `]` jump between bookmarks, pinch zooms.

## Share

```bash
celluloid bundle day12.json      # self-contained day12.html
celluloid convert old.json       # upgrade a 0.0.1 file
```

To embed, serve `celluloid/web/*` and:

```js
import init, { Player } from "/celluloid_web.js";
await init();
const player = await Player.start(canvas);
player.load(json);
```

## License

MIT
