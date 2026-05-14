# Sand Engine

A 2D, 3D, and 4D (time-travel) falling sand simulation engine written from scratch in Rust with `wgpu`.

## Features

- **2D mode** -- 1280x960 cellular automaton grid with real-time sand physics
- **3D mode** -- 128x128x128 voxel grid with instanced cube rendering, directional lighting, and orbit camera
- **4D time travel** -- pause, rewind, scrub through simulation history, fork timelines
- **Draggable physics block** -- grab and drag a rigid block that displaces sand via BFS in both 2D and 3D
- **5 preset scenarios** per mode: triangle/pyramid, falling circle/sphere, hourglass, sand rain, block-in-pile
- **Hand-written WGSL shaders** with multi-light illumination, height-based darkening, and distance fog
- **egui HUD** with FPS counter, particle count, scenario picker, brush size slider, speed control, and time scrubber

## Controls

| Key | Action |
|-----|--------|
| `F2` / `F3` | Switch to 2D / 3D mode |
| `Space` | Pause / Resume |
| `Left` / `Right` | Scrub timeline (when paused) |
| `Shift+Left/Right` | Jump 50 frames |
| `Tab` | Next scenario |
| `R` | Reset current scenario |
| `H` | Toggle help overlay |
| `+` / `-` | Adjust simulation speed |

**2D mode:** Left-click to place sand or grab block. Right-click to erase. Scroll to change brush size.

**3D mode:** Left-click to place sand or grab block. Right-drag to orbit camera. Scroll to zoom. Middle-drag to pan.

## Building

```bash
cargo build --release
./target/release/falling-sand
```

Requires Rust 1.85+ and a GPU with Vulkan, Metal, or DX12 support.

## Tech Stack

- `wgpu` -- GPU rendering (hand-written pipelines and shaders)
- `winit` -- window and input
- `egui` -- UI overlay
- `glam` -- vector/matrix math
- `bytemuck` -- GPU buffer casting

No game engine. Physics, camera, and rendering are all implemented from scratch.
