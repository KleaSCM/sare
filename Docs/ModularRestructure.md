# Sare Terminal - Modular Restructuring Plan

## Current Problem
Files are already getting massive (15-19KB, 400-700 lines each) and will become unmanageable as we implement the TODOs.

## Proposed Modular Structure

### Terminal Module Restructuring

#### Current: `src/terminal/io.rs` (15KB, 623 lines)
**Break into:**
- `src/terminal/io/mod.rs` - Main I/O manager
- `src/terminal/io/streams.rs` - Stream management
- `src/terminal/io/pipelines.rs` - Pipeline operations
- `src/terminal/io/redirection.rs` - File redirection
- `src/terminal/io/background.rs` - Background I/O
- `src/terminal/io/utils.rs` - I/O utilities

#### Current: `src/terminal/process.rs` (9.5KB, 380 lines)
**Break into:**
- `src/terminal/process/mod.rs` - Main process manager
- `src/terminal/process/creation.rs` - Process spawning
- `src/terminal/process/signals.rs` - Signal handling
- `src/terminal/process/groups.rs` - Process groups
- `src/terminal/process/monitoring.rs` - Process monitoring
- `src/terminal/process/utils.rs` - Process utilities

#### Current: `src/terminal/shell.rs` (9.5KB, 375 lines)
**Break into:**
- `src/terminal/shell/mod.rs` - Main shell manager
- `src/terminal/shell/sessions.rs` - Session management
- `src/terminal/shell/integration.rs` - Shell integration
- `src/terminal/shell/config.rs` - Shell configuration
- `src/terminal/shell/features.rs` - Feature detection
- `src/terminal/shell/utils.rs` - Shell utilities

#### Current: `src/terminal/pty.rs` (8.1KB, 323 lines)
**Break into:**
- `src/terminal/pty/mod.rs` - Main PTY manager
- `src/terminal/pty/creation.rs` - PTY creation
- `src/terminal/pty/io.rs` - PTY I/O operations
- `src/terminal/pty/resize.rs` - Terminal resizing
- `src/terminal/pty/setup.rs` - Slave terminal setup
- `src/terminal/pty/utils.rs` - PTY utilities

### TUI Module Restructuring

#### Current: `src/tui/panes/navigation.rs` (19KB, 699 lines)
**Break into:**
- `src/tui/panes/navigation/mod.rs` - Main navigation manager
- `src/tui/panes/navigation/modes.rs` - Navigation modes
- `src/tui/panes/navigation/shortcuts.rs` - Keyboard shortcuts
- `src/tui/panes/navigation/grid.rs` - Grid navigation
- `src/tui/panes/navigation/history.rs` - Focus history
- `src/tui/panes/navigation/actions.rs` - Navigation actions

#### Current: `src/tui/panes/session.rs` (14KB, 538 lines)
**Break into:**
- `src/tui/panes/session/mod.rs` - Main session manager
- `src/tui/panes/session/creation.rs` - Session creation
- `src/tui/panes/session/state.rs` - Session state management
- `src/tui/panes/session/events.rs` - Session events
- `src/tui/panes/session/coordination.rs` - Session coordination
- `src/tui/panes/session/utils.rs` - Session utilities

#### Current: `src/tui/panes/layout.rs` (13KB, 495 lines)
**Break into:**
- `src/tui/panes/layout/mod.rs` - Main layout manager
- `src/tui/panes/layout/algorithms.rs` - Layout algorithms
- `src/tui/panes/layout/constraints.rs` - Layout constraints
- `src/tui/panes/layout/tree.rs` - Binary tree layout
- `src/tui/panes/layout/grid.rs` - Grid layout
- `src/tui/panes/layout/utils.rs` - Layout utilities

#### Current: `src/tui/panes/mod.rs` (13KB, 548 lines)
**Break into:**
- `src/tui/panes/mod.rs` - Main pane manager
- `src/tui/panes/creation.rs` - Pane creation
- `src/tui/panes/splitting.rs` - Pane splitting
- `src/tui/panes/focus.rs` - Focus management
- `src/tui/panes/synchronization.rs` - Pane synchronization
- `src/tui/panes/utils.rs` - Pane utilities

### GPU Module Restructuring

#### Current: `src/tui/gpu/fonts.rs` (10KB, 400 lines)
**Break into:**
- `src/tui/gpu/fonts/mod.rs` - Main font manager
- `src/tui/gpu/fonts/loading.rs` - Font loading
- `src/tui/gpu/fonts/caching.rs` - Font caching
- `src/tui/gpu/fonts/metrics.rs` - Font metrics
- `src/tui/gpu/fonts/fallback.rs` - Font fallback chains
- `src/tui/gpu/fonts/utils.rs` - Font utilities

#### Current: `src/tui/gpu/skia_backend.rs` (9.7KB, 337 lines)
**Break into:**
- `src/tui/gpu/skia/mod.rs` - Main Skia backend
- `src/tui/gpu/skia/initialization.rs` - Skia initialization
- `src/tui/gpu/skia/rendering.rs` - Skia rendering
- `src/tui/gpu/skia/text.rs` - Skia text rendering
- `src/tui/gpu/skia/surfaces.rs` - Surface management
- `src/tui/gpu/skia/utils.rs` - Skia utilities

#### Current: `src/tui/gpu/text.rs` (8.7KB, 353 lines)
**Break into:**
- `src/tui/gpu/text/mod.rs` - Main text renderer
- `src/tui/gpu/text/glyphs.rs` - Glyph management
- `src/tui/gpu/text/layout.rs` - Text layout
- `src/tui/gpu/text/antialiasing.rs` - Antialiasing
- `src/tui/gpu/text/unicode.rs` - Unicode support
- `src/tui/gpu/text/utils.rs` - Text utilities

## Implementation Strategy

### Phase 1: Terminal Module Restructuring
1. Create new directory structure
2. Move existing code to appropriate modules
3. Update imports and module declarations
4. Test compilation

### Phase 2: TUI Module Restructuring
1. Break down panes module
2. Break down GPU module
3. Update imports and module declarations
4. Test compilation

### Phase 3: Implementation TODOs
1. Implement TODOs in modular structure
2. Each module focuses on specific functionality
3. Easier testing and maintenance

## Benefits
- **Maintainability**: Each file has single responsibility
- **Testability**: Smaller modules easier to test
- **Readability**: Focused, smaller files
- **Scalability**: Easy to add new features
- **Collaboration**: Multiple developers can work on different modules

## File Size Targets
- **Maximum**: 300 lines per file
- **Target**: 150-200 lines per file
- **Minimum**: 50 lines per file (unless it's a simple utility)

## Module Naming Convention
- `mod.rs` - Main module interface
- `*_manager.rs` - Core management logic
- `*_utils.rs` - Utility functions
- `*_types.rs` - Type definitions
- `*_traits.rs` - Trait definitions

Author: KleaSCM
Email: KleaSCM@gmail.com 