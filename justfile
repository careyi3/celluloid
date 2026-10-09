# List available recipes
default:
    @just --list

# Build everything, examples included
build:
    cargo build --workspace --all-targets

# Format code and lint with clippy, treating warnings as errors
lint:
    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy -p celluloid-web --target wasm32-unknown-unknown -- -D warnings

# Run the test suite
test:
    cargo test --workspace

# Build the web viewer into celluloid/web
web:
    cargo xtask web
