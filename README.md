# WASM Wave Function Collapse (WFC)

A high-performance implementation of the Wave Function Collapse algorithm running entirely in the browser using Rust and WebAssembly.

## Features

- **Extreme Performance:** Core logic implemented in Rust, compiled to WASM.
- **Progressive Backtracking:** Intelligently recovers from contradictions by resetting local areas.
- **128-bit Bitmasks:** Supports up to 128 unique patterns for high-complexity drawings.
- **Mobile Optimized:** Responsive UI with a toggleable editor for smaller screens.
- **Pattern Sharing:** Share your creations via URL-encoded patterns.

## Local Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A simple local web server (like `python3 -m http.server`)

### Building

To compile the Rust code to WebAssembly:

```bash
wasm-pack build --target web --out-dir www/pkg
```

*Alternatively, if you don't have `wasm-pack` installed, you can use the provided script:* `./build.sh`

### Running

Navigate to the `www` directory and start a server:

```bash
cd www
python3 -m http.server 8000
```

Open `http://localhost:8000` in your browser.

## Deployment on Render.com

This project can be hosted as a **Static Site** on Render.

### Configuration

1. **Service Type:** Static Site
2. **Build Command:**
   ```bash
   ./build.sh
   ```
3. **Publish Directory:** `www` (or `wave_wa/www` if your repo root is the parent directory)
4. **Environment Variables:**
   - No specific variables required, but ensure the build environment has access to the internet to download the `wasm-pack` binary via the build script.

### Why this Build Command?

Since the standard Render environment doesn't include `wasm-pack` and has a read-only global filesystem, `build.sh` downloads a pre-compiled `wasm-pack` binary to a local directory and uses it to build the project.
