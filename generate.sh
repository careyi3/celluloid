#!/bin/bash

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
cd "$SCRIPT_DIR"

WIDTH=${1:-50}
HEIGHT=${2:-50}
OBSTACLE_PERCENT=${3:-0.2}
NAME=${4:-"dijkstra_${WIDTH}x${HEIGHT}"}

echo "🎬 Generating Dijkstra animation..."
echo "   Grid: ${WIDTH}x${HEIGHT}"
echo "   Obstacles: ${OBSTACLE_PERCENT}"
echo "   Output: ${NAME}"
echo ""

cargo run --manifest-path generators/dijkstra/Cargo.toml --release -- "$WIDTH" "$HEIGHT" "$OBSTACLE_PERCENT" "$NAME"

echo ""
echo "✨ Animation saved to: animations/${NAME}.json"
echo ""
echo "To view:"
echo "  1. Start server: ./run.sh"
echo "  2. Open http://127.0.0.1:8000"
echo "  3. Select animation from dropdown"
