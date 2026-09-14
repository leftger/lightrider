# Performance, Optimization & Benchmarking

**LIGHTRIDER** is engineered with **Rust + Bevy 0.19** to render tens of thousands of filesystem nodes and fast-paced 3D arcade geometry smoothly. Because the engine is GPU-raster bound, rendering settings can be dialed in for everything from high-end discrete GPUs to lightweight integrated graphics.

---

## 1. Quick Presets & Options

```bash
# Recommended release run
cargo run --release

# Fast preset (ideal for laptops and integrated GPUs)
cargo run --release -- --fast

# Windowed resolution tuning (fill-rate optimization)
cargo run --release -- --window 1280x720

# Disable bloom or scanlines
cargo run --release -- --no-bloom --no-scanlines
```

---

## 2. Benchmarking Mode

LIGHTRIDER includes a built-in benchmarking system to profile real frame-time statistics on your machine:

```bash
# Run a 12-second automated camera flight benchmark and print statistics
cargo run --release -- --bench

# Run a static camera benchmark for fair A/B comparison of settings
cargo run --release -- --bench --bench-static

# Benchmark directly inside a Lightcycle ride
cargo run --release -- --bench --bench-ride
```

### Benchmark Comparison (Intel Iris 6100, 1280x720, 3,000 entries)

| Configuration | Frame Time | Average Frame Rate | Notes |
| :--- | :--- | :--- | :--- |
| **Bevy Default (4x MSAA + Bloom)** | 59.0 ms | 17 fps | Heavy fill-rate cost |
| **LIGHTRIDER Default (No MSAA + Bloom)** | **36.1 ms** | **28 fps** | Balanced glow & speed |
| **Fast Preset (`--fast`)** | **23.1 ms** | **43 fps** | Maximum smoothness |

---

## 3. Cost Breakdown of Knobs

* **MSAA (Multisampling)**: 4x MSAA costs ~20 ms per frame on integrated GPUs; disabled by default in favor of crisp cyber wireframes.
* **Bloom**: Costs ~15 ms because it requires an HDR rendering pipeline pass. Enabled by default for neon aesthetics, disabled by `--fast` or `--no-bloom`.
* **Scanlines**: Highly efficient post-processing CRT scanline overlay (~1 ms cost).
* **Fill-Rate & Resolution**: 960x540 saves ~13 ms compared to 1280x720 on bandwidth-constrained GPUs.

---

## 4. Large-Directory Architecture

* **Hierarchical View Frustum Culling**: When displaying folders with up to 30,000 nodes, labels and block meshes are culled and distance-pooled.
* **Instance Batching**: Road meshes, district buildings, and trace markers are merged into instanced drawing batches.
* **Background Scanning**: Filesystem IO runs asynchronously in background tasks, keeping the frame rate butter-smooth while traversing directory hierarchies.
