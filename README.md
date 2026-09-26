# ⚡ LIGHTRIDER

### *Realtime Abstracted Path Tree Observer & Cyber Grid Rider Navigator*

*A dangerously cool 3D filesystem explorer and cyber-arcade built with Rust + Bevy, inspired by Jurassic Park's iconic FNS scene.*

[![GitHub Repo](https://img.shields.io/badge/GitHub-leftger%2Flightrider-00f0ff?style=flat&logo=github)](https://github.com/leftger/lightrider)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![Demo](./assets/demo.gif)

> *"It's a UNIX system... I know this!"* — Lex Murphy, *Jurassic Park (1993)*

---

## 🌟 What is LIGHTRIDER?

**[LIGHTRIDER](https://github.com/leftger/lightrider)** transforms your filesystem into an interactive, luminous neon-grid cyber-city. Seamlessly navigate directories in 3D, inspect and open files, ride a high-speed grid rider through city streets, and battle inside source code files transformed into 15+ retro arcade minigames!

### Key Highlights
* 🏙️ **3D Filesystem City**: Every directory renders as an explorable 3D grid with raycast picking, smooth orbit camera, live telemetry, and breadcrumbs.
* 🏍️ **Grid Rider Mode**: Press `M` to seamlessly zoom from 3D Explorer into a high-speed grid rider ride across procedural streets leaving liquid-glass light trails.
* 🕹️ **15+ File-Based Arcade Games**: Ride into source code towers to launch arena battles generated from file bytes — **Disc Wars** (`.rs`), **Asteroid Field** (`.c`), **Snake** (`.py`), **Platformer** (`.slint`), **Brick Breaker** (`.lua`), **Stealth** (`.sh`), **River Surfer** (`.toml`), **Swarm Shooter** (`.json`), **Byte Muncher** (`.go`), **Falling Gems** (`.rb`), **Block Fall** (`.yaml`), **Grid Hopper** (`.js`), **Cube Hopper** (`.zig`), **Grid Bomber** (`.php`), and more!
* 🎵 **Synthesized Soundtrack & Title Theme**: Features an energetic 128 BPM Techno / Frutiger-Aero title theme with Daft Punk sidechain ducking, plus a deterministic procedural synthesizer powered by [Glicol](https://glicol.org) that adapts dynamically to your directory.
* 🚀 **High-Performance**: Engineered with Rust and Bevy 0.19 to render up to 30,000 files at high frame rates with zero allocations in the render loop.

---

## 📸 Gameplay & Minigames Showcase

| **Grid Rider Grid Ride** | **Disc Wars (`.rs`, `.cpp`)** |
| :---: | :---: |
| ![Grid Rider Grid](./assets/screenshots/grid_rider.png) | ![Disc Wars](./assets/screenshots/disc_wars.png) |
| *Stream glowing liquid-glass trails across downtown boulevards* | *Battle Sentinels with auto-aiming discs and bullet time* |

| **Stealth Espionage (`.sh`)** | **Asteroid Field (`.c`, `.h`)** |
| :---: | :---: |
| ![Stealth](./assets/screenshots/stealth.png) | ![Asteroid Field](./assets/screenshots/asteroid_field.png) |
| *Infiltrate security rooms and sneak past patrol vision cones* | *Pivot and blast splitting rock fields before they reach you* |

---

## 📚 Documentation & Guides

Detailed guides have been organized into dedicated documentation:

| Guide | Description |
| :--- | :--- |
| 🎮 **[Controls Cheat Sheet](docs/controls.md)** | Full keybindings for Menu, Explorer, Grid Rider, and Minigames. |
| 🏍️ **[Grid Rider Mode & Cyber-City](docs/grid rider.md)** | Driving mechanics, satellite transitions, city generation, and machine metaphors. |
| 🕹️ **[Arcade Minigames Guide](docs/games.md)** | Breakdown of all 15+ code-generated minigames and combat rules. |
| 🎵 **[Audio & Procedural Music](docs/audio.md)** | Title menu track details, Glicol synthesizer architecture, and audio controls. |
| ⚡ **[Performance & Benchmarking](docs/performance.md)** | Raster tuning, MSAA/Bloom options, `--bench` flags, and low-spec optimization. |
| 🐧 **[WSL2 & Linux Setup Guide](docs/wsl2.md)** | Audio configuration, Wayland/X11 crash fixes, and software rendering tips for WSL2. |
| 📱 **[Mobile Port Plan (Android & iOS)](docs/mobile-port-plan.md)** | Architecture, touch UX, sandbox storage, and store deployment roadmap. |

---

## 🚀 Quick Start

### 1. Clone the Repository
```bash
git clone https://github.com/leftger/lightrider.git
cd lightrider
```

### 2. Prerequisites
* **Rust**: Stable toolchain (`rustup update`)
* **Linux Dependencies** (for audio synthesis and windowing):
  ```bash
  sudo apt install libasound2-dev pkg-config
  ```
* **WSL2 Users**: If running in Windows Subsystem for Linux, install `libasound2-plugins` for audio and launch with `WAYLAND_DISPLAY=""` to avoid Wayland socket crashes. See **[docs/wsl2.md](docs/wsl2.md)**.

### 3. Running

```bash
# Run in release mode (recommended)
cargo run --release

# Fast dev builds (uses dynamic linking)
cargo run --features dev

# Launch directly into a specific folder
cargo run --release -- /path/to/folder

# Fast preset for integrated GPUs / laptops
cargo run --release -- --fast
```

### 4. Building for Windows
* **GitHub Actions**: Automated Windows release binaries are built automatically on tags and PRs ([`.github/workflows/build.yml`](.github/workflows/build.yml)).
* **Local Cross-Compilation (Linux to Windows)**:
  ```bash
  ./tools/build-windows.sh
  ```

---

## 🎮 Essential Controls

| Key | Action |
| :--- | :--- |
| **`Right Mouse Drag`** | Rotate 3D Camera / Free look in Grid Rider |
| **`Scroll Wheel`** | Zoom in / out |
| **`h` / `j` / `k` / `l`** | Move selection (Left / Down / Up / Right) |
| **`o` / `Enter`** | Open directory / Launch file in default system app |
| **`u` / `-`** | Navigate to parent directory |
| **`f`** | Reveal selected file in OS file manager |
| **`M`** | Toggle between **3D Explorer** and **Grid Rider Mode** |
| **`A` / `D`** | Steer grid rider (in Grid Rider mode) |
| **`Shift`** | Engage **Bullet Time** (in minigames) |
| **`P` / `Esc`** | Pause menu & Minigame Warper |
| **`N`** | Toggle music on / off (`[` and `]` for volume) |

👉 *See [docs/controls.md](docs/controls.md) for the complete list of controls.*

---

## 🛠️ Command-Line Options

```bash
cargo run --release -- [OPTIONS] [PATH]

Arguments:
  [PATH]  Initial directory path to open [default: current directory]

Options:
      --hidden           Show hidden files
      --no-labels        Hide text labels
      --no-fps           Hide FPS counter
      --no-music         Start with music disabled
      --music-volume <V> Set volume (0.0 to 1.0) [default: 0.7]
      --fast             Disable MSAA and bloom for maximum framerate
      --bench            Run 12-second automated performance benchmark
  -h, --help             Print help information
```

---

## 📜 Credits

* **Repository**: [https://github.com/leftger/lightrider](https://github.com/leftger/lightrider)
* **Grid Rider 3D Model**: ["Cyber Grid Bike"](https://sketchfab.com/3d-models/light-cycle-tron-1982-54fedda920094ef09d87a17d42b282af) by [arabinowitz](https://sketchfab.com/arabinowitz) (Sketchfab Standard license).
* **Runner 3D Model**: ["Cyber Runner Character"](https://sketchfab.com/3d-models/tron-male-character-b7b2dd24bf6e495e9a729d7d271c52db) by [dehariyalokesh1998](https://sketchfab.com/dehariyalokesh1998) (CC-BY-4.0). Rigged with 19 joints and custom walk cycle.
* **Audio Engine**: Powered by [Glicol](https://glicol.org) and `cpal`.

---

## 🦖 License

Licensed under the MIT License.
