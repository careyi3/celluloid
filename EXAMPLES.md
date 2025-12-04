# Animation Generator Examples

This document shows examples of how to create different types of animations.

## Dijkstra Pathfinding

### Basic maze (default settings)
```bash
./generate.sh
```

### Large sparse maze
```bash
./generate.sh 80 80 0.1 "large_sparse_maze"
```

### Small dense maze
```bash
./generate.sh 30 30 0.4 "dense_challenge"
```

### Rectangular maze
```bash
./generate.sh 100 50 0.25 "wide_maze"
```

## Conway's Game of Life

### Classic glider pattern
```bash
./generate_gol.sh 60 60 glider 150 "glider_demo"
```

### Random chaos
```bash
./generate_gol.sh 80 80 random 300 "chaos"
```

### Sparse evolution
```bash
./generate_gol.sh 100 100 sparse 400 "sparse_world"
```

### Dense patterns (lots of activity)
```bash
./generate_gol.sh 70 70 dense 250 "dense_activity"
```

### Long evolution
```bash
./generate_gol.sh 60 60 random 500 "long_evolution"
```

## Tips

- **File sizes**: Larger grids and more frames = larger files
  - 30x30 grid, 200 frames ≈ 500KB-1MB
  - 60x60 grid, 300 frames ≈ 2-4MB
  - 100x100 grid, 500 frames ≈ 8-15MB

- **Performance**: Very large animations (>10MB) may take a few seconds to load

- **Frame rates**:
  - Dijkstra: 50ms (20 FPS) - good for watching pathfinding
  - Game of Life: 100ms (10 FPS) - good for cellular automaton evolution

- **Patterns**:
  - Glider patterns travel diagonally and loop indefinitely
  - Random patterns stabilize after ~100-200 generations
  - Dense patterns create more interesting initial chaos
  - Sparse patterns often die out quickly
