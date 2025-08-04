/**
 * Pane management for Sare terminal GUI
 * 
 * This module contains structures and enums for managing
 * terminal panes, including pane state, layout, and splitting.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: pane.rs
 * Description: Pane management structures and enums
 */

use eframe::egui;

/**
 * Terminal pane structure
 * 
 * Represents a single terminal pane with its own
 * input, output, and state management.
 */
#[derive(Debug, Clone)]
pub struct TerminalPane {
	/// Pane ID
	pub id: String,
	/// Output buffer for this pane
	pub output_buffer: Vec<TerminalLine>,
	/// Current input for this pane
	pub current_input: String,
	/// Cursor position in input
	pub cursor_pos: usize,
	/// Working directory for this pane
	pub working_directory: String,
	/// Whether this pane is active
	pub active: bool,
	/// Pane position and size (x, y, width, height)
	pub layout: (f32, f32, f32, f32),
	/// Parent pane ID (if split)
	pub parent_id: Option<String>,
	/// Child pane IDs (if split)
	pub child_ids: Vec<String>,
	/// Split direction (if this pane was created by splitting)
	pub split_direction: Option<SplitDirection>,
}

/**
 * Terminal line structure
 * 
 * Represents a single line of terminal output
 * with content, color, and prompt status.
 */
#[derive(Debug, Clone)]
pub struct TerminalLine {
	/// Line content
	pub content: String,
	/// Line color
	pub color: egui::Color32,
	/// Whether this is a prompt line
	pub is_prompt: bool,
}

/**
 * Split direction enumeration
 * 
 * Defines the direction for pane splitting.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum SplitDirection {
	/// Vertical split (left/right)
	Vertical,
	/// Horizontal split (top/bottom)
	Horizontal,
}

/**
 * Terminal mode enumeration
 * 
 * Defines the different modes the terminal can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalMode {
	/// Normal mode
	Normal,
	/// Insert mode
	Insert,
}

impl Default for TerminalPane {
	fn default() -> Self {
		Self {
			id: "pane_0".to_string(),
			output_buffer: Vec::new(),
			current_input: String::new(),
			cursor_pos: 0,
			working_directory: std::env::current_dir()
				.unwrap_or_default()
				.to_string_lossy()
				.to_string(),
			active: true,
			layout: (0.0, 0.0, 1.0, 1.0), // Full screen by default
			parent_id: None,
			child_ids: Vec::new(),
			split_direction: None,
		}
	}
} 