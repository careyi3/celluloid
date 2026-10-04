# celluloid-core

Recorder and file format for [Celluloid](https://github.com/careyi3/celluloid)
animations: animated debugging for grid puzzles, cellular automata,
path-finding and the like.

```rust
use celluloid_core::Recorder;

let mut rec = Recorder::new("Game of Life");
let g = rec.grid("life", 40, 40);
rec.state(g, "alive", "#59a14f");
for generation in 0..200 {
    rec.redraw(g, |x, y| if alive[y][x] { "alive" } else { "empty" });
    rec.var("generation", generation);
    rec.frame(format!("generation {generation}"));
    step(&mut alive);
}
rec.save("life.json")?;
```

View the result with the `celluloid` binary from the repository.

## Concepts

- **Panels**: a recording holds one or more square grids (`rec.grid`), hex
  grids (`rec.hex`, axial coordinates, unbounded), arrays (`rec.array`),
  trees (`rec.tree`) and graphs (`rec.graph`).
- **States**: named element states with colours. Grid and hex cells start as
  `"empty"`, array items as `"default"`. States used without
  `rec.state(...)` get a colour from a built-in palette.
- **Labels**: short text drawn on an element. On arrays they travel with
  the item.
- **Markers**: named dots that glide between cells, or pointers under an
  array.
- **Array ops**: `value`, `swap`, `insert`, `push`, `remove`, `pop` and
  `move_item`. Items keep their identity, so the viewer animates them
  moving.
- **Nodes**: tree and graph nodes are named by anything `Display` and
  created when first mentioned. Trees link nodes through child slots
  (`left`, `right`, `add_child`); moving a node detaches it from its old
  parent, so rotations can be recorded in any order. Graphs have `edge`,
  `set_edge`, `edge_label` and `place` for fixed positions.
- **Vars**: named values shown next to the animation, kept until changed.
- **Bookmarks**: named frames you can jump to.

Frames store small ops ("these cells are now `seen`") rather than whole
grids, and the viewer keeps periodic snapshots so scrubbing stays instant.
`Timeline` gives the same random access to the state at any frame in
your own code.

Files from 0.0.1 (a full grid per frame) are still read by `from_json`
and converted automatically; their types live in `celluloid_core::legacy`.

## License

Licensed under MIT license.
