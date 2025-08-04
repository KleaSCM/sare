/**
 * Main application entry point for Sare terminal emulator
 * 
 * This module provides the main GUI application that opens in its own window,
 * featuring a modern terminal interface with multi-pane support, GPU acceleration,
 * and developer-focused features.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: main.rs
 * Description: GUI application entry point for Sare terminal emulator
 */

use eframe::egui;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

mod shell;
mod tui;
mod terminal;
mod config;
mod history;

/**
 * Split direction for pane splitting
 */
#[derive(Debug, Clone, PartialEq)]
enum SplitDirection {
	/// Vertical split (left/right)
	Vertical,
	/// Horizontal split (top/bottom)
	Horizontal,
}

/**
 * Terminal pane with layout information
 */
#[derive(Debug, Clone)]
struct TerminalPane {
	/// Pane ID
	id: String,
	/// Output buffer for this pane
	output_buffer: Vec<TerminalLine>,
	/// Current input for this pane
	current_input: String,
	/// Cursor position in input
	cursor_pos: usize,
	/// Working directory for this pane
	working_directory: String,
	/// Whether this pane is active
	active: bool,
	/// Pane position and size (x, y, width, height)
	layout: (f32, f32, f32, f32),
	/// Parent pane ID (if split)
	parent_id: Option<String>,
	/// Child pane IDs (if split)
	child_ids: Vec<String>,
	/// Split direction (if this pane was created by splitting)
	split_direction: Option<SplitDirection>,
}

/**
 * Terminal line for output
 */
#[derive(Debug, Clone)]
struct TerminalLine {
	content: String,
	color: egui::Color32,
	is_prompt: bool,
}

/**
 * Terminal mode
 */
#[derive(Debug, Clone)]
enum TerminalMode {
	Normal,
	Insert,
}

/**
 * Main Sare terminal application
 * 
 * Provides a real GUI terminal window with actual shell functionality,
 * command execution, and terminal emulation features.
 */
struct SareTerminal {
	/// Command history
	command_history: Vec<String>,
	/// History index for navigation
	history_index: Option<usize>,
	/// Current working directory
	current_dir: String,
	/// Terminal size
	terminal_size: (u32, u32),
	/// Scroll position
	scroll_pos: f32,
	/// Auto-scroll enabled
	auto_scroll: bool,
	/// Terminal mode (normal/insert)
	mode: TerminalMode,
	/// Active panes
	panes: Vec<TerminalPane>,
	/// Currently focused pane
	focused_pane: usize,
	/// Current split direction for new panes
	current_split_direction: SplitDirection,
}

impl Default for SareTerminal {
	fn default() -> Self {
		/**
		 * Sareターミナル初期化の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なターミナル初期化を行います。
		 * デフォルト設定とパネ管理が難しい部分なので、
	 * 適切なエラーハンドリングで実装しています (◕‿◕)
	 */
	
		let default_pane = TerminalPane {
			id: "pane_0".to_string(),
			output_buffer: Vec::new(), // Start with empty output
			current_input: String::new(),
			cursor_pos: 0,
			working_directory: std::env::current_dir()
				.unwrap_or_default()
				.to_string_lossy()
				.to_string(),
			active: true,
			layout: (0.0, 0.0, 1.0, 1.0), // Default to full screen
			parent_id: None,
			child_ids: Vec::new(),
			split_direction: None,
		};
		
		Self {
			command_history: Vec::new(),
			history_index: None,
			current_dir: std::env::current_dir()
				.unwrap_or_default()
				.to_string_lossy()
				.to_string(),
			terminal_size: (80, 24),
			scroll_pos: 0.0,
			auto_scroll: true,
			mode: TerminalMode::Normal,
			panes: vec![default_pane],
			focused_pane: 0,
			current_split_direction: SplitDirection::Vertical,
		}
	}
}

impl SareTerminal {
	/**
	 * Creates a new Sare terminal
	 */
	fn new() -> anyhow::Result<Self> {
		Ok(Self::default())
	}
	
	/**
	 * Executes a command
	 */
	fn execute_command(&mut self, command: &str) {
		// Add command to history
		if !command.trim().is_empty() {
			self.command_history.push(command.to_string());
			self.history_index = None;
		}
		
		// Handle clear command specially
		if command.trim() == "clear" {
			if let Some(pane) = self.panes.get_mut(self.focused_pane) {
				pane.output_buffer.clear();
			}
			self.panes[self.focused_pane].current_input.clear();
			self.panes[self.focused_pane].cursor_pos = 0;
			return;
		}
		
		// Execute the command first
		let output = self.run_command(command);
		
		// Get current pane and add output
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			// Add command to pane output
			pane.output_buffer.push(TerminalLine {
				content: format!("sare@user:{} $ {}", self.current_dir, command),
				color: egui::Color32::from_rgb(0, 255, 0),
				is_prompt: true,
			});
			
			// Add output to pane
			pane.output_buffer.push(TerminalLine {
				content: output,
				color: egui::Color32::from_rgb(255, 255, 255),
				is_prompt: false,
			});
		}
		
		// Clear input
		self.panes[self.focused_pane].current_input.clear();
		self.panes[self.focused_pane].cursor_pos = 0;
	}
	
	/**
	 * Runs a command using std::process::Command
	 */
	fn run_command(&mut self, command: &str) -> String {
		use std::process::Command;
		
		// Handle built-in commands
		match command.trim() {
			"pwd" => {
				return self.current_dir.clone();
			}
			"exit" => {
				std::process::exit(0);
			}
			_ => {}
		}
		
		// Handle cd command specially
		if command.starts_with("cd ") {
			let new_dir = command[3..].trim();
			if let Ok(_) = std::env::set_current_dir(new_dir) {
				if let Ok(current_dir) = std::env::current_dir() {
					let new_path = current_dir.to_string_lossy().to_string();
					// Update current directory
					self.current_dir = new_path.clone();
					return format!("Changed directory to: {}", new_path);
				}
			} else {
				return format!("Error: Cannot change to directory '{}'", new_dir);
			}
		}
		
		// Execute system command
		let output = Command::new("sh")
			.arg("-c")
			.arg(command)
			.current_dir(&self.current_dir)
			.output();
		
		match output {
			Ok(output) => {
				let mut result = String::new();
				
				// Add stdout
				if !output.stdout.is_empty() {
					let stdout = String::from_utf8_lossy(&output.stdout);
					result.push_str(&stdout);
				}
				
				// Add stderr
				if !output.stderr.is_empty() {
					let stderr = String::from_utf8_lossy(&output.stderr);
					if !result.is_empty() {
						result.push('\n');
					}
					result.push_str(&stderr);
				}
				
				// Add exit status if not successful
				if !output.status.success() {
					if !result.is_empty() {
						result.push('\n');
					}
					result.push_str(&format!("Exit code: {}", output.status));
				}
				
				result
			}
			Err(e) => {
				format!("Error executing command: {}", e)
			}
		}
	}
	
	/**
	 * Adds a line to the output buffer
	 */
	fn add_output_line(&mut self, content: String, color: egui::Color32, is_prompt: bool) {
		let line = TerminalLine {
			content,
			color,
			is_prompt,
		};
		
		self.panes[self.focused_pane].output_buffer.push(line);
	}
	
	/**
	 * Handles key input
	 */
	fn handle_key_input(&mut self, ctx: &egui::Context) {
		ctx.input(|input| {
			// Debug: Print all key events
			for event in &input.events {
				match event {
					egui::Event::Key { key, pressed, modifiers, .. } => {
						if *pressed {
							println!("Key pressed: {:?} with modifiers: {:?}", key, modifiers);
							
							// Handle Ctrl combinations
							if modifiers.ctrl {
								match key {
									egui::Key::N => {
										println!("Ctrl+N detected - creating vertical split");
										self.split_pane(SplitDirection::Vertical);
									}
									egui::Key::H => {
										println!("Ctrl+H detected - creating horizontal split");
										self.split_pane(SplitDirection::Horizontal);
									}
									egui::Key::D => {
										println!("Ctrl+D detected - closing pane");
										self.close_current_pane();
									}
									egui::Key::C => {
										println!("Ctrl+C detected - clearing input");
										self.panes[self.focused_pane].current_input.clear();
										self.panes[self.focused_pane].cursor_pos = 0;
									}
									_ => {}
								}
						} else {
								// Handle single keys
								match key {
									egui::Key::Enter => {
										let command = self.panes[self.focused_pane].current_input.clone();
										if !command.trim().is_empty() {
											self.execute_command(&command);
										}
									}
									egui::Key::ArrowUp => {
										self.navigate_history_up();
									}
									egui::Key::ArrowDown => {
										self.navigate_history_down();
									}
									egui::Key::ArrowLeft => {
										if self.panes[self.focused_pane].cursor_pos > 0 {
											self.panes[self.focused_pane].cursor_pos -= 1;
										}
									}
									egui::Key::ArrowRight => {
										let current_input_len = self.panes[self.focused_pane].current_input.len();
										if self.panes[self.focused_pane].cursor_pos < current_input_len {
											self.panes[self.focused_pane].cursor_pos += 1;
										}
									}
									egui::Key::Home => {
										self.panes[self.focused_pane].cursor_pos = 0;
									}
									egui::Key::End => {
										self.panes[self.focused_pane].cursor_pos = self.panes[self.focused_pane].current_input.len();
									}
									egui::Key::Backspace => {
										if self.panes[self.focused_pane].cursor_pos > 0 {
											let cursor_pos = self.panes[self.focused_pane].cursor_pos;
											self.panes[self.focused_pane].current_input.remove(cursor_pos - 1);
											self.panes[self.focused_pane].cursor_pos -= 1;
										}
									}
									egui::Key::Delete => {
										let cursor_pos = self.panes[self.focused_pane].cursor_pos;
										let current_input_len = self.panes[self.focused_pane].current_input.len();
										if cursor_pos < current_input_len {
											self.panes[self.focused_pane].current_input.remove(cursor_pos);
										}
									}
									egui::Key::Tab => {
										println!("Tab detected - switching panes");
										self.switch_to_next_pane();
									}
									_ => {}
								}
							}
						}
					}
					egui::Event::Text(text) => {
						for ch in text.chars() {
							if ch.is_ascii() && !ch.is_control() {
								let cursor_pos = self.panes[self.focused_pane].cursor_pos;
								self.panes[self.focused_pane].current_input.insert(cursor_pos, ch);
								self.panes[self.focused_pane].cursor_pos += 1;
							}
						}
					}
					_ => {}
				}
			}
		});
	}
	
	/**
	 * Navigate up in command history
	 */
	fn navigate_history_up(&mut self) {
		if self.command_history.is_empty() {
			return;
		}
		
		let history_len = self.command_history.len();
		let current_index = self.history_index.unwrap_or(history_len);
		
		if current_index > 0 {
			self.history_index = Some(current_index - 1);
			self.panes[self.focused_pane].current_input = self.command_history[current_index - 1].clone();
			self.panes[self.focused_pane].cursor_pos = self.panes[self.focused_pane].current_input.len();
		}
	}
	
	/**
	 * Navigate down in command history
	 */
	fn navigate_history_down(&mut self) {
		if self.command_history.is_empty() {
			return;
		}
		
		let history_len = self.command_history.len();
		let current_index = self.history_index.unwrap_or(history_len);
		
		if current_index < history_len - 1 {
			self.history_index = Some(current_index + 1);
			self.panes[self.focused_pane].current_input = self.command_history[current_index + 1].clone();
			self.panes[self.focused_pane].cursor_pos = self.panes[self.focused_pane].current_input.len();
		} else {
			self.history_index = None;
			self.panes[self.focused_pane].current_input.clear();
			self.panes[self.focused_pane].cursor_pos = 0;
		}
	}
	
	/**
	 * Splits the current pane in the specified direction
	 */
	fn split_pane(&mut self, direction: SplitDirection) {
		// Get current pane layout and working directory
		let current_pane = &self.panes[self.focused_pane];
		let (x, y, width, height) = current_pane.layout;
		let working_directory = current_pane.working_directory.clone();
		let current_pane_id = current_pane.id.clone();
		let new_pane_id = format!("pane_{}", self.panes.len());
		
		// Calculate new pane layout based on split direction
		let (new_x, new_y, new_width, new_height) = match direction {
			SplitDirection::Vertical => {
				// Split vertically: current pane gets left half, new pane gets right half
				(x + width * 0.5, y, width * 0.5, height)
			}
			SplitDirection::Horizontal => {
				// Split horizontally: current pane gets top half, new pane gets bottom half
				(x, y + height * 0.5, width, height * 0.5)
			}
		};
		
		// Update current pane layout
		let current_pane = &mut self.panes[self.focused_pane];
		match direction {
			SplitDirection::Vertical => {
				current_pane.layout = (x, y, width * 0.5, height);
			}
			SplitDirection::Horizontal => {
				current_pane.layout = (x, y, width, height * 0.5);
			}
		}
		
		// Create new pane
		let new_pane = TerminalPane {
			id: new_pane_id,
			output_buffer: Vec::new(),
			current_input: String::new(),
			cursor_pos: 0,
			working_directory,
			active: false,
			layout: (new_x, new_y, new_width, new_height),
			parent_id: Some(current_pane_id),
			child_ids: Vec::new(),
			split_direction: Some(direction),
		};
		
		// Add new pane
		self.panes.push(new_pane);
		self.focused_pane = self.panes.len() - 1;
		
		// Update active states
		for (i, pane) in self.panes.iter_mut().enumerate() {
			pane.active = i == self.focused_pane;
		}
	}
	
	/**
	 * Creates a new pane (legacy function, now uses split_pane)
	 */
	fn create_new_pane(&mut self) {
		self.split_pane(SplitDirection::Vertical);
	}
	
	/**
	 * Closes the current pane
	 */
	fn close_current_pane(&mut self) {
		if self.panes.len() > 1 {
			self.panes.remove(self.focused_pane);
			if self.focused_pane >= self.panes.len() {
				self.focused_pane = self.panes.len() - 1;
			}
			
			// Update active states
			for (i, pane) in self.panes.iter_mut().enumerate() {
				pane.active = i == self.focused_pane;
			}
		}
	}
	
	/**
	 * Switches to next pane
	 */
	fn switch_to_next_pane(&mut self) {
		if self.panes.len() > 1 {
			self.focused_pane = (self.focused_pane + 1) % self.panes.len();
			
			// Update active states
			for (i, pane) in self.panes.iter_mut().enumerate() {
				pane.active = i == self.focused_pane;
			}
		}
	}
	
	/**
	 * Renders the terminal interface with multiple panes
	 */
	fn render_terminal(&mut self, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| {
			// Set terminal background
			ui.painter().rect_filled(
				ui.available_rect_before_wrap(),
				0.0,
				egui::Color32::from_rgb(0, 0, 0),
			);
			
			// Get the available area for rendering
			let available_rect = ui.available_rect_before_wrap();
			let total_width = available_rect.width();
			let total_height = available_rect.height();
			
			// Render each pane according to its layout
			for (pane_index, pane) in self.panes.iter().enumerate() {
				let (x, y, width, height) = pane.layout;
				
				// Convert relative coordinates to absolute pixel coordinates
				let pane_x = available_rect.min.x + x * total_width;
				let pane_y = available_rect.min.y + y * total_height;
				let pane_width = width * total_width;
				let pane_height = height * total_height;
				
				// Create a clip rect for this pane
				let pane_rect = egui::Rect::from_min_size(
					egui::pos2(pane_x, pane_y),
					egui::vec2(pane_width, pane_height),
				);
				
				// Render pane background (different color for focused pane)
				let bg_color = if pane_index == self.focused_pane {
					egui::Color32::from_rgb(20, 20, 20) // Slightly lighter for focused pane
				} else {
					egui::Color32::from_rgb(10, 10, 10) // Darker for unfocused panes
				};
				
				ui.painter().rect_filled(pane_rect, 0.0, bg_color);
				
				// Draw pane border
				let border_color = if pane_index == self.focused_pane {
					egui::Color32::from_rgb(100, 100, 100) // Highlight focused pane
				} else {
					egui::Color32::from_rgb(50, 50, 50) // Subtle border for unfocused
				};
				ui.painter().rect_stroke(pane_rect, 1.0, (1.0, border_color));
				
				// Render pane content in a clipped area
				ui.set_clip_rect(pane_rect);
				
				// Create a child UI for this pane with unique ID
				ui.allocate_ui_at_rect(pane_rect, |ui| {
					// Push unique ID for this pane to prevent ID clashes
					ui.push_id(format!("pane_{}", pane_index), |ui| {
						// Vertical layout for this pane
						ui.vertical(|ui| {
							// Output area (most of the pane)
							let output_height = pane_height - 30.0; // Leave space for input
							
							egui::ScrollArea::vertical()
								.max_height(output_height)
								.show(ui, |ui| {
									// Show output from this pane
									for line in &pane.output_buffer {
										ui.label(egui::RichText::new(&line.content)
											.color(line.color)
											.text_style(egui::TextStyle::Monospace));
									}
								});
							
							// Input area at bottom of pane
							ui.horizontal(|ui| {
								// Terminal prompt
								let prompt = format!("sare@user:{} $ ", pane.working_directory);
								ui.label(egui::RichText::new(prompt)
									.color(egui::Color32::from_rgb(0, 255, 0))
									.text_style(egui::TextStyle::Monospace));
								
								// Input text with cursor (only for focused pane)
								if pane_index == self.focused_pane {
									let input_text = format!("{}{}", 
										&pane.current_input[..pane.cursor_pos],
										&pane.current_input[pane.cursor_pos..]
									);
									
									ui.label(egui::RichText::new(input_text)
										.color(egui::Color32::from_rgb(255, 255, 255))
										.text_style(egui::TextStyle::Monospace));
									
									// Blinking cursor (only for focused pane)
									if (ctx.input(|i| i.time) * 2.0).sin() > 0.0 {
										ui.label(egui::RichText::new("█")
											.color(egui::Color32::from_rgb(255, 255, 255))
											.text_style(egui::TextStyle::Monospace));
									}
								}
							});
						});
					});
				});
				
				// Reset clip rect
				ui.set_clip_rect(ui.available_rect_before_wrap());
			}
		});
	}
}

impl eframe::App for SareTerminal {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Handle key input
		self.handle_key_input(ctx);
		
		// Render the terminal interface
		self.render_terminal(ctx);
		
		// Request continuous updates for real-time terminal behavior
		ctx.request_repaint();
	}
	
	fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
		// Set background color to terminal black
		[0.0, 0.0, 0.0, 1.0]
	}
}

fn main() -> anyhow::Result<()> {
	// Initialize logging
	env_logger::init();
	
	// Create the Sare terminal application
	let app = SareTerminal::new()?;
	
	// Configure native options for terminal window
	let native_options = eframe::NativeOptions::default();
	
	// Run the terminal application
	eframe::run_native(
		"Sare Terminal Emulator",
		native_options,
		Box::new(|_cc| Box::new(app)),
	).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
} 