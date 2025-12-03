# WASM Animation with Rocket Server

This project consists of two parts:
1. **WASM Animation** - Rust pathfinding visualization compiled to WebAssembly
2. **Rocket Server** - A Rust web server to serve the application

## Project Structure

```
rust_wasm/
├── src/              # WASM animation source code
│   ├── lib.rs       # Main entry point
│   ├── types.rs     # Type definitions (CellState, GridConfig, etc.)
│   ├── algorithm.rs # Dijkstra's algorithm implementation
│   ├── animation.rs # Animation state management
│   └── renderer.rs  # Canvas rendering logic
├── pkg/             # Built WASM output (generated)
├── server/          # Rocket web server
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
├── index.html       # Standalone HTML (for basic-http-server)
└── Cargo.toml
```

## Quick Start

### Option 1: Using Rocket Server (Recommended)

```bash
# 1. Build the WASM module
wasm-pack build --target web

# 2. Run the Rocket server
cd server
cargo run

# 3. Open http://localhost:8000 in your browser
```

### Option 2: Using basic-http-server

```bash
# 1. Build the WASM module
wasm-pack build --target web

# 2. Install and run basic-http-server
cargo install basic-http-server
basic-http-server .

# 3. Open http://localhost:4000 in your browser
```

## What This Does

- **Visualizes Dijkstra's pathfinding algorithm** on a 50×50 grid
- **Random obstacles** block the path (20% of cells)
- **Animated exploration** shows nodes being visited in real-time
- **Path highlighting** draws the shortest path once found
- **Live statistics** display nodes explored and path length

## Development Workflow

1. Make changes to the WASM code in `src/`
2. Rebuild with `wasm-pack build --target web`
3. Refresh your browser (server auto-serves updated files)

## Customization

### Animation Speed & Grid Size
Edit `src/animation.rs`:
```rust
pub const GRID_SIZE: usize = 50;           // Grid dimensions
pub const OBSTACLE_PERCENTAGE: f64 = 0.2;  // 20% obstacles
pub const ANIMATION_SPEED_MS: f64 = 10.0;  // Milliseconds per frame
```

### Colors
Edit `src/types.rs` in `ColorScheme::default()`:
```rust
impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            empty: CellColor::new("#2a2a2a"),
            obstacle: CellColor::new("#0a0a0a"),
            start: CellColor::new("#00ff00"),
            end: CellColor::new("#ff0000"),
            visited: CellColor::new("#4444ff"),
            path: CellColor::new("#ffaa00"),
            // ... etc
        }
    }
}
```

## Architecture

The codebase is split into logical modules:

- **`types.rs`**: Core data structures (cell states, colors, grid config)
- **`algorithm.rs`**: Pathfinding algorithm trait and Dijkstra implementation
- **`renderer.rs`**: Canvas rendering logic
- **`animation.rs`**: Animation loop and state management
- **`lib.rs`**: Entry point and WASM initialization

This modular design makes it easy to add new algorithms (A*, BFS, DFS) or customize the visualization.

## Next Steps

- Add more pathfinding algorithms (A*, BFS, DFS)
- Implement user controls (pause, restart, speed control)
- Allow custom obstacle placement with mouse
- Add algorithm comparison mode
- Implement WebGL rendering for larger grids
