#!/bin/bash
set -e

WIDTH=${1:-60}
HEIGHT=${2:-60}
PATTERN=${3:-random}
GENERATIONS=${4:-200}
NAME=${5:-game_of_life_${WIDTH}x${HEIGHT}_${PATTERN}}

echo "🎮 Generating Conway's Game of Life Animation"
echo "   Grid: ${WIDTH}x${HEIGHT}"
echo "   Pattern: ${PATTERN}"
echo "   Generations: ${GENERATIONS}"
echo "   Output: ${NAME}.json"
echo ""

cd generators/game_of_life
cargo run --release -- ${WIDTH} ${HEIGHT} ${PATTERN} ${GENERATIONS} "${NAME}"
cd ../..

echo ""
echo "✨ Animation saved to: animations/${NAME}.json"
echo ""
echo "Available patterns:"
echo "  - glider: Classic glider pattern"
echo "  - random: Random 30% density"
echo "  - sparse: Random 15% density"
echo "  - dense: Random 50% density"
