/**
 * Sare Terminal Library
 * 
 * This library provides the core terminal emulation functionality
 * including GPU-accelerated rendering, multi-pane support, and
 * terminal session management. Can be used as a library in other
 * applications or as a standalone terminal emulator.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: lib.rs
 * Description: Library interface for the Sare terminal emulator
 */

pub mod terminal;
pub mod tui;
pub mod gui;
pub mod history;
pub mod features;

pub use terminal::TerminalEmulator;
pub use tui::TuiManager;
pub use gui::SareTerminal; 