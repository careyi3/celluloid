# celluloid

Desktop viewer for [Celluloid](https://github.com/careyi3/celluloid) animations.

Record what your code does with [`celluloid-core`](https://crates.io/crates/celluloid-core), then scrub through it here.

You can find the [crates.io listing here](https://crates.io/crates/celluloid).

![CI](https://github.com/careyi3/celluloid/actions/workflows/test.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/celluloid.svg)](https://crates.io/crates/celluloid)
[![Crates.io](https://img.shields.io/crates/d/celluloid.svg)](https://crates.io/crates/celluloid)

## Setup

```bash
$ cargo install celluloid
```

## Running

```bash
$ celluloid                    # browse .json files here
$ celluloid day12.json         # open one; reloads when it changes
$ celluloid convert old.json   # upgrade a 0.0.1 file
```

Space plays, ← → step, `[` `]` jump between bookmarks, pinch zooms.

Note: `celluloid bundle` writes a self-contained HTML page you can share. It needs the web viewer built in, which the crates.io install doesn't have. See the [main README](https://github.com/careyi3/celluloid#share) to build it.

## License

MIT
