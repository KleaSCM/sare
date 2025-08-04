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

use super::pane::{TerminalPane, SplitDirection, TerminalMode, TerminalLine};
use crate::history::{HistoryManager, TabCompleter};

/**
 * Main terminal interface
 * 
 * Contains the main terminal state including panes,
 * command history, and terminal configuration.
 */
#[derive(Debug)]
pub struct SareTerminal {
	/// Command history manager
	pub history_manager: HistoryManager,
	/// Tab completion engine
	pub tab_completer: TabCompleter,
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
	/// History search mode
	pub history_search_mode: bool,
	/// History search query
	pub history_search_query: String,
	/// Original input before history navigation
	pub original_input: String,
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
		let history_manager = HistoryManager::new().unwrap_or_else(|_| {
			// Create a basic history manager with fallback config
			HistoryManager::with_config(1000, std::path::PathBuf::from(".sare_history"))
				.unwrap_or_else(|_| HistoryManager {
					history: std::collections::VecDeque::new(),
					max_entries: 1000,
					history_file: std::path::PathBuf::from(".sare_history"),
				})
		});
		
		let working_directory = std::env::current_dir()
			.unwrap_or_default();
		let tab_completer = TabCompleter::new(working_directory);
		
		Self {
			history_manager,
			tab_completer,
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
			history_search_mode: false,
			history_search_query: String::new(),
			original_input: String::new(),
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
			self.history_manager.add_command(command.to_string(), None);
			self.tab_completer.add_command(command.to_string());
			self.history_index = None;
			self.history_search_mode = false;
			self.history_search_query.clear();
		}
		
		// Execute the command first
		let output = self.run_command(command);
		
		// Get current pane and add output
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			pane.add_output_line(output, egui::Color32::from_rgb(255, 255, 255), false);
		}
	}
	
	/**
	 * Runs a command and returns the output
	 */
	pub fn run_command(&mut self, command: &str) -> String {
		if command.trim().is_empty() {
			return String::new();
		}
		
		// Handle built-in commands first
		match command.trim() {
			"clear" => {
				// Clear all panes
				for pane in &mut self.panes {
					pane.output_buffer.clear();
				}
				return String::new();
			}
			"pwd" => {
				return format!("{}\n", self.current_dir);
			}
			"history" => {
				return self.get_history_display();
			}
			_ => {
				// Try to execute as external command
				let parts: Vec<&str> = command.split_whitespace().collect();
				if parts.is_empty() {
					return String::new();
				}
				
				match Command::new(parts[0])
					.args(&parts[1..])
					.current_dir(&self.current_dir)
					.output() {
					Ok(output) => {
						let stdout = String::from_utf8_lossy(&output.stdout);
						let stderr = String::from_utf8_lossy(&output.stderr);
						
						let mut result = String::new();
						if !stdout.is_empty() {
							result.push_str(&stdout);
						}
						if !stderr.is_empty() {
							result.push_str(&stderr);
						}
						
						if result.is_empty() {
							result.push('\n');
						}
						
						result
					}
					Err(e) => {
						format!("Error executing command: {}\n", e)
					}
				}
			}
		}
	}
	
	/**
	 * Gets history display for history command
	 */
	fn get_history_display(&self) -> String {
		let history = self.history_manager.get_history();
		let mut display = String::new();
		
		for (i, entry) in history.iter().enumerate() {
			display.push_str(&format!("{:4}  {}\n", i + 1, entry.command));
		}
		
		display
	}
	
	/**
	 * Adds an output line to the current pane
	 */
	pub fn add_output_line(&mut self, content: String, color: egui::Color32, is_prompt: bool) {
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			pane.add_output_line(content, color, is_prompt);
		}
	}
	
	/**
	 * Handles key input with advanced history navigation
	 */
	pub fn handle_key_input(&mut self, ctx: &egui::Context) {
		/**
		 * キー入力処理の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なキー入力処理を行います。
		 * 履歴ナビゲーションとショートカット処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		ctx.input(|input| {
			// Handle key presses
			for event in &input.events {
				if let egui::Event::Key { key, pressed, modifiers, .. } = event {
					if *pressed {
						match *key {
							egui::Key::ArrowUp => {
								if modifiers.ctrl {
									// Ctrl+Up: Navigate history up
									self.navigate_history_up();
								} else {
									// Up: Navigate history up
									self.navigate_history_up();
								}
							}
							egui::Key::ArrowDown => {
								if modifiers.ctrl {
									// Ctrl+Down: Navigate history down
									self.navigate_history_down();
								} else {
									// Down: Navigate history down
									self.navigate_history_down();
								}
							}
							egui::Key::Tab => {
								// Tab: Perform tab completion
								if let Some(pane) = self.panes.get_mut(self.focused_pane) {
									let input = &pane.current_input;
									let cursor_pos = pane.cursor_pos;
									
									if let Ok(Some(completion)) = self.tab_completer.complete(input, cursor_pos) {
										// Apply the completion
										pane.current_input = completion.completed_text;
										pane.cursor_pos = pane.current_input.len();
										
										// If partial completion, show alternatives
										if completion.is_partial && !completion.alternatives.is_empty() {
											println!("Available completions: {:?}", completion.alternatives);
										}
									}
								}
							}
							egui::Key::Tab if modifiers.shift => {
								// Shift+Tab: Switch between panes
								println!("Shift+Tab detected - switching panes");
								self.switch_to_next_pane();
							}
							egui::Key::D => {
								if modifiers.ctrl {
									// Ctrl+D: Close current pane
									println!("Ctrl+D detected - closing pane");
									self.close_current_pane();
								}
							}
							egui::Key::N => {
								if modifiers.ctrl {
									// Ctrl+N: Create new pane
									println!("Ctrl+N detected - creating new pane");
									self.split_pane(SplitDirection::Horizontal);
								}
							}
							egui::Key::H => {
								if modifiers.ctrl {
									// Ctrl+H: Create new pane
									println!("Ctrl+H detected - creating new pane");
									self.split_pane(SplitDirection::Vertical);
								}
							}
							egui::Key::R => {
								if modifiers.ctrl {
									// Ctrl+R: Reverse incremental search
									self.start_reverse_search();
								}
							}
							egui::Key::Escape => {
								// Escape: Exit history search mode
								if self.history_search_mode {
									self.exit_history_search();
								}
							}
							egui::Key::Enter => {
								// Enter: Execute current command
								if let Some(pane) = self.panes.get(self.focused_pane) {
									let command = pane.current_input.clone();
									if !command.trim().is_empty() {
										// Execute the command
										self.execute_command(&command);
										
										// Clear the input and reset history navigation
										if let Some(pane) = self.panes.get_mut(self.focused_pane) {
											pane.current_input.clear();
											pane.cursor_pos = 0;
										}
										
										self.history_index = None;
										self.original_input.clear();
									}
								}
							}
							_ => {
								// Handle regular character input
								if let Some(c) = Self::key_to_char(*key) {
									if self.history_search_mode {
										// Add to search query
										self.history_search_query.push(c);
										self.perform_reverse_search();
									} else {
										// Add to current pane input
										if let Some(pane) = self.panes.get_mut(self.focused_pane) {
											pane.add_char(c);
										}
									}
								}
							}
						}
					}
				}
			}
		});
	}
	
	/**
	 * Converts a key to a character
	 */
	fn key_to_char(key: egui::Key) -> Option<char> {
		match key {
			egui::Key::A => Some('a'),
			egui::Key::B => Some('b'),
			egui::Key::C => Some('c'),
			egui::Key::D => Some('d'),
			egui::Key::E => Some('e'),
			egui::Key::F => Some('f'),
			egui::Key::G => Some('g'),
			egui::Key::H => Some('h'),
			egui::Key::I => Some('i'),
			egui::Key::J => Some('j'),
			egui::Key::K => Some('k'),
			egui::Key::L => Some('l'),
			egui::Key::M => Some('m'),
			egui::Key::N => Some('n'),
			egui::Key::O => Some('o'),
			egui::Key::P => Some('p'),
			egui::Key::Q => Some('q'),
			egui::Key::R => Some('r'),
			egui::Key::S => Some('s'),
			egui::Key::T => Some('t'),
			egui::Key::U => Some('u'),
			egui::Key::V => Some('v'),
			egui::Key::W => Some('w'),
			egui::Key::X => Some('x'),
			egui::Key::Y => Some('y'),
			egui::Key::Z => Some('z'),
			egui::Key::Space => Some(' '),
			_ => None,
		}
	}
	
	/**
	 * Navigates history up (older commands)
	 */
	pub fn navigate_history_up(&mut self) {
		/**
		 * 履歴ナビゲーション上移動の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な履歴ナビゲーションを行います。
		 * 履歴インデックス管理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let history = self.history_manager.get_history();
		if history.is_empty() {
			return;
		}
		
		let current_index = self.history_index.unwrap_or(history.len());
		
		if current_index > 0 {
			self.history_index = Some(current_index - 1);
			if let Some(entry) = history.get(current_index - 1) {
				// Save original input if this is the first navigation
				if self.original_input.is_empty() {
					if let Some(pane) = self.panes.get(self.focused_pane) {
						self.original_input = pane.current_input.clone();
					}
				}
				
				// Set the command from history
				if let Some(pane) = self.panes.get_mut(self.focused_pane) {
					pane.current_input = entry.command.clone();
				}
			}
		}
	}
	
	/**
	 * Navigates history down (newer commands)
	 */
	pub fn navigate_history_down(&mut self) {
		let history = self.history_manager.get_history();
		if history.is_empty() {
			return;
		}
		
		let current_index = self.history_index.unwrap_or(history.len());
		
		if current_index < history.len() - 1 {
			self.history_index = Some(current_index + 1);
			if let Some(entry) = history.get(current_index + 1) {
				if let Some(pane) = self.panes.get_mut(self.focused_pane) {
					pane.current_input = entry.command.clone();
				}
			}
		} else {
			// Reached the end, restore original input
			self.history_index = None;
			if let Some(pane) = self.panes.get_mut(self.focused_pane) {
				pane.current_input = self.original_input.clone();
			}
			self.original_input.clear();
		}
	}
	
	/**
	 * Starts reverse incremental search (Ctrl+R)
	 */
	pub fn start_reverse_search(&mut self) {
		self.history_search_mode = true;
		self.history_search_query.clear();
		self.history_index = None;
		
		// Save current input
		if let Some(pane) = self.panes.get(self.focused_pane) {
			self.original_input = pane.current_input.clone();
		}
	}
	
	/**
	 * Performs reverse incremental search
	 */
	pub fn perform_reverse_search(&mut self) {
		let history = self.history_manager.get_history();
		let query = &self.history_search_query;
		
		if query.is_empty() {
			return;
		}
		
		// Search backwards through history
		for (i, entry) in history.iter().enumerate().rev() {
			if entry.command.contains(query) {
				self.history_index = Some(i);
				if let Some(pane) = self.panes.get_mut(self.focused_pane) {
					pane.current_input = entry.command.clone();
				}
				break;
			}
		}
	}
	
	/**
	 * Exits history search mode
	 */
	pub fn exit_history_search(&mut self) {
		self.history_search_mode = false;
		self.history_search_query.clear();
		self.history_index = None;
		
		// Restore original input
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			pane.current_input = self.original_input.clone();
		}
		self.original_input.clear();
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
	 * Switches to the next pane
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