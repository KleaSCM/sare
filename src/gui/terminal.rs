/**
 * Main terminal interface for Sare GUI
 * 
 * This module contains the main SareTerminal struct and
 * its implementation for the GUI terminal interface.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: terminal.rs
 * Description: Main terminal interface implementation
 */

use anyhow::Result;
use eframe::egui;
use std::process::Command;

use super::pane::{TerminalPane, SplitDirection, TerminalMode};

/**
 * Main terminal interface
 * 
 * Contains the main terminal state including panes,
 * command history, and terminal configuration.
 */
#[derive(Debug)]
pub struct SareTerminal {
	/// Command history
	pub command_history: Vec<String>,
	/// History index for navigation
	pub history_index: Option<usize>,
	/// Current working directory
	pub current_dir: String,
	/// Terminal size
	pub terminal_size: (u32, u32),
	/// Scroll position
	pub scroll_pos: f32,
	/// Auto-scroll enabled
	pub auto_scroll: bool,
	/// Terminal mode (normal/insert)
	pub mode: TerminalMode,
	/// Active panes
	pub panes: Vec<TerminalPane>,
	/// Currently focused pane
	pub focused_pane: usize,
	/// Current split direction for new panes
	pub current_split_direction: SplitDirection,
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
		
		let default_pane = TerminalPane::default();
		
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
	 * Creates a new terminal instance
	 */
	pub fn new() -> Result<Self> {
		Ok(Self::default())
	}
	
	/**
	 * Executes a command
	 */
	pub fn execute_command(&mut self, command: &str) {
		// Add command to history
		if !command.trim().is_empty() {
			self.command_history.push(command.to_string());
			self.history_index = None;
		}
		
		// Execute the command first
		let output = self.run_command(command);
		
		// Get current pane and add output
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			// Add command to pane output
			pane.output_buffer.push(super::pane::TerminalLine {
				content: format!("sare@user:{} $ {}", self.current_dir, command),
				color: egui::Color32::from_rgb(0, 255, 0),
				is_prompt: true,
			});
			
			// Add output to pane
			pane.output_buffer.push(super::pane::TerminalLine {
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
	 * Runs a command and returns the output
	 */
	pub fn run_command(&mut self, command: &str) -> String {
		let trimmed = command.trim();
		
		match trimmed {
			"clear" => {
				// Clear the output buffer
				self.panes[self.focused_pane].output_buffer.clear();
				return String::new();
			}
			"pwd" => {
				return self.current_dir.clone();
			}
			"exit" => {
				std::process::exit(0);
			}
			_ => {}
		}
		
		// Handle ping command with arguments
		if trimmed.starts_with("ping ") {
			let args = trimmed[5..].trim(); // Remove "ping " prefix
			let output = Command::new("ping")
				.arg("-c")
				.arg("5")
				.arg(args)
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
					
					return result;
				}
				Err(e) => {
					return format!("Error executing command: {}", e);
				}
			}
		}
		
		// Handle cd command specially
		if trimmed.starts_with("cd ") {
			let new_dir = trimmed[3..].trim();
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
			.arg(trimmed)
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
	pub fn add_output_line(&mut self, content: String, color: egui::Color32, is_prompt: bool) {
		let line = super::pane::TerminalLine {
			content,
			color,
			is_prompt,
		};
		
		self.panes[self.focused_pane].output_buffer.push(line);
	}
	
	/**
	 * Handles key input
	 */
	pub fn handle_key_input(&mut self, ctx: &egui::Context) {
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
	 * Navigates up in command history
	 */
	pub fn navigate_history_up(&mut self) {
		if self.history_index.is_none() {
			self.history_index = Some(self.command_history.len());
		}
		
		if let Some(ref mut index) = self.history_index {
			if *index > 0 {
				*index -= 1;
				if let Some(command) = self.command_history.get(*index) {
					self.panes[self.focused_pane].current_input = command.clone();
					self.panes[self.focused_pane].cursor_pos = command.len();
				}
			}
		}
	}
	
	/**
	 * Navigates down in command history
	 */
	pub fn navigate_history_down(&mut self) {
		if let Some(ref mut index) = self.history_index {
			if *index < self.command_history.len() - 1 {
				*index += 1;
				if let Some(command) = self.command_history.get(*index) {
					self.panes[self.focused_pane].current_input = command.clone();
					self.panes[self.focused_pane].cursor_pos = command.len();
				}
			} else {
				self.history_index = None;
				self.panes[self.focused_pane].current_input.clear();
				self.panes[self.focused_pane].cursor_pos = 0;
			}
		}
	}
	
	/**
	 * Splits the current pane
	 */
	pub fn split_pane(&mut self, direction: SplitDirection) {
		// Extract necessary data before mutable borrow
		let focused_pane_index = self.focused_pane;
		let current_layout = self.panes[focused_pane_index].layout;
		let current_id = self.panes[focused_pane_index].id.clone();
		let (x, y, width, height) = current_layout;
		
		// Create new pane
		let new_pane_id = format!("pane_{}", self.panes.len());
		let mut new_pane = TerminalPane::default();
		new_pane.id = new_pane_id.clone();
		new_pane.working_directory = self.current_dir.clone();
		
		// Calculate new layout based on split direction
		match direction {
			SplitDirection::Vertical => {
				// Current pane takes left half, new pane takes right half
				let new_width = width / 2.0;
				new_pane.layout = (x + new_width, y, new_width, height);
				
				// Update current pane layout
				self.panes[focused_pane_index].layout = (x, y, new_width, height);
			}
			SplitDirection::Horizontal => {
				// Current pane takes top half, new pane takes bottom half
				let new_height = height / 2.0;
				new_pane.layout = (x, y + new_height, width, new_height);
				
				// Update current pane layout
				self.panes[focused_pane_index].layout = (x, y, width, new_height);
			}
		}
		
		// Set split direction for both panes
		new_pane.split_direction = Some(direction.clone());
		self.panes[focused_pane_index].split_direction = Some(direction);
		
		// Update parent/child relationships
		new_pane.parent_id = Some(current_id);
		self.panes[focused_pane_index].child_ids.push(new_pane_id.clone());
		
		// Add new pane
		self.panes.push(new_pane);
		
		// Focus the new pane
		self.focused_pane = self.panes.len() - 1;
		
		// Update active states
		for (i, pane) in self.panes.iter_mut().enumerate() {
			pane.active = i == self.focused_pane;
		}
	}
	
	/**
	 * Creates a new pane
	 */
	pub fn create_new_pane(&mut self) {
		self.split_pane(SplitDirection::Vertical);
	}
	
	/**
	 * Closes the current pane
	 */
	pub fn close_current_pane(&mut self) {
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
	pub fn switch_to_next_pane(&mut self) {
		if self.panes.len() > 1 {
			self.focused_pane = (self.focused_pane + 1) % self.panes.len();
			
			// Update active states
			for (i, pane) in self.panes.iter_mut().enumerate() {
				pane.active = i == self.focused_pane;
			}
		}
	}
} 