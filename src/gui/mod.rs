/**
 * GUI module for Sare terminal
 * 
 * This module contains all GUI-related components including
 * the main terminal interface, pane management, and rendering.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Main GUI module that exports all GUI components
 */

pub mod terminal;
pub mod pane;
pub mod renderer;
pub mod multiline;
pub mod heredoc;

pub use terminal::SareTerminal;
pub use pane::{TerminalPane, SplitDirection};
pub use renderer::TerminalRenderer; 