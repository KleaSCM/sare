
use eframe::egui;

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

#[derive(Debug, Clone)]
pub struct TerminalLine {
	/// Line content
	pub content: String,
	/// Line color
	pub color: egui::Color32,
	/// Whether this is a prompt line
	pub is_prompt: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitDirection {
	/// Vertical split (left/right)
	Vertical,
	/// Horizontal split (top/bottom)
	Horizontal,
}

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

impl TerminalPane {
	pub fn add_output_line(&mut self, content: String, color: egui::Color32, is_prompt: bool) {
		self.output_buffer.push(TerminalLine {
			content,
			color,
			is_prompt,
		});
	}
	
	pub fn add_char(&mut self, c: char) {
		self.current_input.push(c);
		self.cursor_pos = self.current_input.len();
	}
}