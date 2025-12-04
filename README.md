# Rust WASM Animation System

A modular system for creating and viewing algorithm visualizations in the browser using Rust and WebAssembly.

## Architecture

This project demonstrates a clean separation between animation generation and rendering:

```
Generators (Rust binaries) → JSON Files → Web Server → WASM Renderer
```

### Components

1. **Generators** (`generators/`): Standalone Rust binaries that run algorithms and output animation JSON files
   - `dijkstra`: Dijkstra's pathfinding algorithm with configurable grid size and obstacles
   - Easy to add more: A*, BFS, DFS, sorting algorithms, etc.

2. **Server** (`server/`): Rocket web server that serves the UI and animation files
   - REST API to list and serve animation JSON files
   - Static file serving for HTML/CSS/JS
   - Automatic WASM builds via `build.rs`

3. **WASM Renderer** (`wasm/`): WebAssembly module that renders animations in the browser
   - Consumes JSON animation data
   - Canvas-based rendering
   - Frame-by-frame playback

4. **Animations** (`animations/`): JSON files containing animation data
   - Standardized format for interoperability
   - Grid-based visualizations with cell states
   - Frame metadata and messages

## Quick Start

### Generate animations

**Dijkstra Pathfinding:**
```bash
# Generate a 50x50 grid with 20% obstacles
./generate.sh 50 50 0.2 "my_maze"

# Or use default parameters (50x50, 20% obstacles)
./generate.sh
```

**Conway's Game of Life:**
```bash
# Generate glider pattern
./generate_gol.sh 60 60 glider 150

# Random pattern with defaults (60x60, 200 generations)
./generate_gol.sh

# Dense random pattern
./generate_gol.sh 80 80 dense 300 "gol_dense"
```

### Run the viewer

```bash
./run.sh
```

Then open http://127.0.0.1:8000 and select an animation from the dropdown.

## Manual Build

```bash
# Build WASM module
cd wasm
wasm-pack build --target web --out-dir ../pkg

# Build and run server
cd ../server
cargo run --bin server
```

## Creating New Generators

1. Create a new binary in `generators/`:
```bash
cargo new --bin generators/my_algorithm
```

2. Add to workspace in root `Cargo.toml`:
```toml
[workspace]
members = ["wasm", "server", "generators/dijkstra", "generators/my_algorithm"]
```

3. Generate JSON in the format defined in `animations/README.md`

4. Output to `animations/your_animation.json`

## JSON Animation Format

See `animations/README.md` for the complete format specification.

```json
{
  "name": "My Animation",
  "algorithm": "Algorithm Name",
  "created_at": "2024-01-01T00:00:00Z",
  "grid_config": {
    "width": 50,
    "height": 50
  },
  "metadata": {
    "total_frames": 100,
    "has_path": true
  },
  "frames": [
    {
      "step": 0,
      "grid": [[0, 1, 2, ...], ...],
      "message": "Frame description",
      "highlighted": [[x, y], ...]
    }
  ]
}
```

### Cell States
- `0`: Empty
- `1`: Obstacle
- `2`: Start
- `3`: End
- `4`: Visited
- `5`: Path

## API Endpoints

- `GET /api/animations` - List all available animations
- `GET /api/animation/:name` - Get animation JSON by name
- `POST /api/animations/refresh` - Refresh the animations list

## Project Structure

```
rust_wasm/
├── wasm/               # WASM renderer
│   ├── src/
│   │   ├── lib.rs     # Main WASM entry point
│   │   └── types.rs   # Color scheme and types
│   └── Cargo.toml
├── server/             # Rocket web server
│   ├── src/
│   │   └── main.rs    # Server routes and logic
│   ├── templates/
│   │   └── index.html # Main UI
│   ├── static/
│   │   ├── css/
│   │   └── js/
│   ├── build.rs       # Auto-build WASM
│   └── Cargo.toml
├── generators/         # Animation generators
│   ├── dijkstra/      # Dijkstra pathfinding
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── game_of_life/  # Conway's Game of Life
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
├── animations/         # Generated animation files
│   └── README.md
├── pkg/               # Built WASM output
├── generate.sh        # Helper script to generate animations
├── run.sh             # Helper script to run the server
└── Cargo.toml         # Workspace configuration
```

## Dependencies

### WASM
- `wasm-bindgen`: JavaScript interop
- `web-sys`: Browser APIs
- `serde`, `serde_json`: JSON parsing

### Server
- `rocket`: Web framework
- `serde`, `serde_json`: JSON handling

### Generators
- `serde`, `serde_json`: JSON output
- `rand`: Random grid generation
- `chrono`: Timestamps

## Available Generators

### Dijkstra Pathfinding
Visualizes Dijkstra's shortest path algorithm finding a route through obstacles.

**Usage:**
```bash
./generate.sh [width] [height] [obstacle_%] [name]
./generate.sh 50 50 0.2 "my_maze"
```

**Parameters:**
- Width: Grid width (default: 50)
- Height: Grid height (default: 50)
- Obstacle %: Percentage of obstacles (default: 0.2 = 20%)
- Name: Output filename (default: auto-generated)

### Conway's Game of Life
Simulates Conway's Game of Life cellular automaton with various starting patterns.

**Usage:**
```bash
./generate_gol.sh [width] [height] [pattern] [generations] [name]
./generate_gol.sh 60 60 glider 150
```

**Parameters:**
- Width: Grid width (default: 60)
- Height: Grid height (default: 60)
- Pattern: Initial pattern - `glider`, `random`, `sparse`, `dense` (default: random)
- Generations: Number of generations to simulate (default: 200)
- Name: Output filename (default: auto-generated)

**Patterns:**
- `glider`: Classic glider pattern that moves across the grid
- `random`: 30% initial cell density
- `sparse`: 15% initial cell density
- `dense`: 50% initial cell density

## Tips

- Generate animations with different parameters to see various behaviors
- The dropdown refreshes when you click "Refresh"
- Animations play frame-by-frame automatically
- Game of Life uses 100ms frame delay (slower) vs pathfinding's 50ms
- Check the browser console for debug information

## Extending

To add a new algorithm:

1. Create a generator in `generators/`
2. Output JSON matching the format
3. Run `./generate.sh` or build manually
4. Animation appears in dropdown automatically

You can write generators in any language - just output the correct JSON format!
