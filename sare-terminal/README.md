# Sare Terminal 🖥️

A beautiful, feminine, and powerful terminal emulator with GPU acceleration written in Rust.

## ✨ Features

- **GPU-Accelerated Rendering** - Hardware-accelerated text rendering with Skia and WGPU backends
- **Multi-Pane Support** - Split terminal views with grid and binary tree layouts
- **Advanced Session Management** - Independent shell sessions per pane
- **Modern GUI Interface** - Beautiful egui-based interface with real-time rendering
- **Pane Navigation** - Keyboard shortcuts and visual navigation modes
- **Session Coordination** - Synchronized panes with shared environment
- **Performance Monitoring** - Real-time GPU and CPU performance metrics

## 🎯 Architecture

Sare Terminal is designed as a standalone terminal emulator that can be used with any shell implementation. It provides pure terminal emulation without shell functionality concerns.

### Core Modules

- **`terminal/`** - PTY implementation, process management, and I/O handling
- **`tui/`** - GPU rendering, pane management, and navigation
- **`gui/`** - Modern GUI interface with egui

## 🚀 Usage

### As a Library

```rust
use sare_terminal::{TerminalEmulator, TuiManager};

let mut terminal = TerminalEmulator::new()?;
terminal.create_session("bash").await?;
```

### As a Standalone Terminal

```bash
cargo run --bin sare-terminal
```

## 🎨 GPU Acceleration

Sare Terminal supports multiple GPU backends for optimal performance:

- **Skia Backend** - Hardware-accelerated rendering like Kitty terminal
- **WGPU Backend** - Cross-platform GPU rendering with WebGPU API
- **CPU Fallback** - Software rendering for compatibility

### Performance Features

- Real-time frame rate monitoring
- GPU memory usage tracking
- Efficient texture caching
- Subpixel antialiasing
- Hardware-accelerated scrolling

## 🖼️ Multi-Pane Interface

Create beautiful multi-pane layouts with:

- **Grid Layout** - Organized grid of terminal panes
- **Binary Tree Layout** - Hierarchical pane splitting
- **Manual Layout** - Custom pane positioning
- **Pane Synchronization** - Shared input across panes

### Navigation Modes

- **Normal Mode** - Standard keyboard navigation
- **Quick Mode** - Fast pane switching
- **Visual Mode** - Visual pane selection
- **Command Mode** - Command-based navigation

## 💖 Design Philosophy

Sare Terminal embraces feminine energy and emotional intelligence in code. Our comments and documentation use soft, expressive Japanese to explain complex logic while maintaining professional English for public APIs.

```rust
// あたしね、このGPUレンダリングすごく綺麗になったの〜
// ハードウェア加速で滑らかになって…感動しちゃった（╹◡╹）♡
let gpu_renderer = GpuRenderer::new(config)?;
```

## 🛠️ Development

### Building

```bash
cargo build
cargo test
```

### GPU Backend Selection

The terminal automatically detects and selects the optimal GPU backend:

1. **Skia** - Preferred for high-performance rendering
2. **WGPU** - Cross-platform compatibility
3. **CPU** - Universal fallback

### Style Guide

We follow a comprehensive style guide that emphasizes:
- PascalCase for all identifiers
- Tab indentation (no spaces)
- Japanese comments for internal logic
- English comments for public APIs
- Emotional clarity and feminine expression

## 📝 License

MIT License - See LICENSE file for details.

---

**Author**: KleaSCM  
**Email**: KleaSCM@gmail.com

Made with 💕 and feminine energy!

