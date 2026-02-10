# Mandelbrot Adaptive

A high-performance, GPU-accelerated Mandelbrot set explorer written in Rust.

This project uses **WGPU** for rendering and **Rug** (GMP/MPFR) for arbitrary precision arithmetic, allowing for extremely deep zooms beyond standard floating-point limits.

## Features

-   **Deep Zooming**: Capable of zooming up to **10^38** magnification using 128-bit high-precision floats.
-   **GPU Acceleration**: Utilizes WGPU for efficient rendering.
-   **Adaptive Iterations**: Automatically adjusts iteration counts based on zoom level to maintain detail.
-   **Perturbation Theory**: Uses reference orbits to accelerate high-precision calculations on the GPU. Based on the method described at [mandelbrot.site](https://mandelbrot.site/).

## ⚠️ Photosensitivity / Strobe Warning

**Please Read Before Running:**

This application can produce rapid changes in contrast and color patterns, especially when zooming in or out quickly. If you have a history of photosensitive epilepsy or are sensitive to flashing lights/patterns, please exercise caution or avoid using the zoom features aggressively.

## Prerequisites

To build this project, you need Rust installed. You also need the system development libraries for GMP and MPFR, as the `rug` crate depends on them.

### Linux (Debian/Ubuntu)
```bash
sudo apt-get install libgmp-dev libmpfr-dev libmpc-dev
```

### macOS
```bash
brew install gmp mpfr libmpc
```

## Installation & Running

1.  Clone the repository:
    ```bash
    git clone https://github.com/mterenzi/mandelbrot_adaptive.git
    cd mandelbrot_adaptive
    ```

2.  Run with Cargo:
    ```bash
    cargo run --release
    ```
    *Note: Release mode is highly recommended for performance.*

## Controls

-   **Mouse Scroll**: Zoom in and out.
-   **Mouse Position**: The zoom centers on the mouse cursor.
