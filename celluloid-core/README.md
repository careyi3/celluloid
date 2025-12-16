# celluloid-core

Data models for [Celluloid](https://github.com/careyi3/celluloid) grid-based animations.

This crate provides the core data structures for creating grid-based animations that can be visualized with the Celluloid viewer. Perfect for Advent of Code visualizations, cellular automata, pathfinding algorithms, and other grid-based simulations.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
celluloid-core = "0.1"
```

## Example

```rust
use celluloid_core::{AnimationData, Frame, CellState};

fn main() {
    let mut animation = AnimationData::new("My Animation", 10, 10)
        .with_frame_delay(100.0);

    let mut grid = vec![vec![CellState::Empty as u8; 10]; 10];
    grid[5][5] = CellState::Start as u8;
    
    let frame = Frame::new(0, grid, "Initial state");
    animation.add_frame(frame);

    let json = serde_json::to_string_pretty(&animation).unwrap();
    std::fs::write("animation.json", json).unwrap();
}
```

## Cell States

- `Empty` (0) - Empty cell
- `Obstacle` (1) - Blocked cell  
- `Start` (2) - Starting position
- `End` (3) - Goal position
- `Visited` (4) - Visited during search
- `Path` (5) - Part of final path

## Viewing Animations

Install the Celluloid viewer:

```bash
cargo install --git https://github.com/careyi3/celluloid --path server
```

Then run it in a directory with your animation JSON files:

```bash
celluloid
```

## License

Licensed under MIT license.
