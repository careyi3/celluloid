# Celluloid

Browser-based viewer for cellular automata and grid-based animations.

## Install

```bash
cargo install --path server
```

## Usage

```bash
cd /path/to/animations
celluloid
```

Open `http://localhost:8000`

## Update

```bash
cargo install --path server --force
```

If WASM code changed:
```bash
wasm-pack build --target web --out-dir pkg wasm/
cp -r wasm/pkg/* pkg/
cargo install --path server --force
```

## License

Licensed under MIT license.
