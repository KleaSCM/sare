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
 * Main Sare terminal application
 * 
 * Provides a real GUI terminal window with actual shell functionality,
 * command execution, and terminal emulation features.
 */
struct SareTerminal {
	/// Terminal output buffer
	output_buffer: Vec<TerminalLine>,
	/// Current input line
	current_input: String,
	/// Cursor position in input
	cursor_pos: usize,
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
 * Terminal pane
 */
#[derive(Debug, Clone)]
struct TerminalPane {
	id: String,
	output_buffer: Vec<TerminalLine>,
	current_input: String,
	cursor_pos: usize,
	working_directory: String,
	active: bool,
}

/**
 * Terminal mode
 */
#[derive(Debug, Clone, PartialEq)]
enum TerminalMode {
	Normal,
	Insert,
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
		
		// Create default pane
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
		};
		
		Self {
			output_buffer: Vec::new(), // Start with empty output
			current_input: String::new(),
			cursor_pos: 0,
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
			self.current_input.clear();
			self.cursor_pos = 0;
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
		self.current_input.clear();
		self.cursor_pos = 0;
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
	 * Adds a line to output buffer
	 */
	fn add_output_line(&mut self, content: String, color: egui::Color32, is_prompt: bool) {
		self.output_buffer.push(TerminalLine {
			content,
			color,
			is_prompt,
		});
		
		// Auto-scroll to bottom
		if self.auto_scroll {
			self.scroll_pos = self.output_buffer.len() as f32;
		}
	}
	
	/**
	 * Handles key input
	 */
	fn handle_key_input(&mut self, ctx: &egui::Context) {
		ctx.input(|input| {
			// Handle key presses
			if input.key_pressed(egui::Key::Enter) {
				let command = self.current_input.clone();
				if !command.trim().is_empty() {
					self.execute_command(&command);
				}
			} else if input.key_pressed(egui::Key::ArrowUp) {
				self.navigate_history_up();
			} else if input.key_pressed(egui::Key::ArrowDown) {
				self.navigate_history_down();
			} else if input.key_pressed(egui::Key::ArrowLeft) {
				if self.cursor_pos > 0 {
					self.cursor_pos -= 1;
				}
			} else if input.key_pressed(egui::Key::ArrowRight) {
				if self.cursor_pos < self.current_input.len() {
					self.cursor_pos += 1;
				}
			} else if input.key_pressed(egui::Key::Home) {
				self.cursor_pos = 0;
			} else if input.key_pressed(egui::Key::End) {
				self.cursor_pos = self.current_input.len();
			} else if input.key_pressed(egui::Key::Backspace) {
				if self.cursor_pos > 0 {
					self.current_input.remove(self.cursor_pos - 1);
					self.cursor_pos -= 1;
				}
			} else if input.key_pressed(egui::Key::Delete) {
				if self.cursor_pos < self.current_input.len() {
					self.current_input.remove(self.cursor_pos);
				}
			} else if input.key_pressed(egui::Key::C) && input.modifiers.ctrl {
				// Ctrl+C - clear input
				self.current_input.clear();
				self.cursor_pos = 0;
			} else if input.key_pressed(egui::Key::N) && input.modifiers.ctrl {
				// Ctrl+N - new pane
				self.create_new_pane();
			} else if input.key_pressed(egui::Key::W) && input.modifiers.ctrl {
				// Ctrl+W - close pane
				self.close_current_pane();
			} else if input.key_pressed(egui::Key::Tab) {
				// Tab - switch panes
				self.switch_to_next_pane();
			}
			
			// Handle text input
			for event in &input.events {
				if let egui::Event::Text(text) = event {
					for ch in text.chars() {
						if ch.is_ascii() && !ch.is_control() {
							self.current_input.insert(self.cursor_pos, ch);
							self.cursor_pos += 1;
						}
					}
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
			self.current_input = self.command_history[current_index - 1].clone();
			self.cursor_pos = self.current_input.len();
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
			self.current_input = self.command_history[current_index + 1].clone();
			self.cursor_pos = self.current_input.len();
		} else {
			self.history_index = None;
			self.current_input.clear();
			self.cursor_pos = 0;
		}
	}
	
	/**
	 * Creates a new pane
	 */
	fn create_new_pane(&mut self) {
		let new_pane = TerminalPane {
			id: format!("pane_{}", self.panes.len()),
			output_buffer: Vec::new(), // Start with empty output
			current_input: String::new(),
			cursor_pos: 0,
			working_directory: self.current_dir.clone(),
			active: false,
		};
		
		self.panes.push(new_pane);
		self.focused_pane = self.panes.len() - 1;
		
		// Update active states
		for (i, pane) in self.panes.iter_mut().enumerate() {
			pane.active = i == self.focused_pane;
		}
		
		// Clear current input when switching panes
		self.current_input.clear();
		self.cursor_pos = 0;
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
	 * Renders the terminal interface
	 */
	fn render_terminal(&mut self, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| {
			// Set terminal background
			ui.painter().rect_filled(
				ui.available_rect_before_wrap(),
				0.0,
				egui::Color32::from_rgb(0, 0, 0),
			);
			
			// Use vertical layout to put prompt at bottom
			ui.vertical(|ui| {
				// Output area (takes most space)
				egui::ScrollArea::vertical()
					.max_height(ui.available_height() - 40.0)
					.show(ui, |ui| {
						// Show output from current pane
						if let Some(pane) = self.panes.get(self.focused_pane) {
							for line in &pane.output_buffer {
								ui.label(egui::RichText::new(&line.content)
									.color(line.color)
									.text_style(egui::TextStyle::Monospace));
							}
						}
					});
				
				// Input area with prompt at bottom
				ui.horizontal(|ui| {
					// Terminal prompt
					let prompt = format!("sare@user:{} $ ", 
						self.current_dir
					);
					ui.label(egui::RichText::new(prompt)
						.color(egui::Color32::from_rgb(0, 255, 0))
						.text_style(egui::TextStyle::Monospace));
					
					// Input text with cursor
					let input_text = format!("{}{}", 
						&self.current_input[..self.cursor_pos],
						&self.current_input[self.cursor_pos..]
					);
					
					ui.label(egui::RichText::new(input_text)
						.color(egui::Color32::from_rgb(255, 255, 255))
						.text_style(egui::TextStyle::Monospace));
					
					// Blinking cursor
					if (ctx.input(|i| i.time) * 2.0).sin() > 0.0 {
						ui.label(egui::RichText::new("█")
							.color(egui::Color32::from_rgb(255, 255, 255))
							.text_style(egui::TextStyle::Monospace));
					}
				});
			});
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