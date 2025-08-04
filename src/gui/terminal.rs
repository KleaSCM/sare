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
	/// Multiline input state
	pub multiline_mode: bool,
	/// Multiline continuation character
	pub continuation_char: Option<char>,
	/// Multiline prompt prefix
	pub multiline_prompt: String,
	/// Heredoc state
	pub heredoc_mode: bool,
	/// Heredoc delimiter
	pub heredoc_delimiter: String,
	/// Heredoc content being collected
	pub heredoc_content: String,
	/// Whether heredoc content should expand variables
	pub heredoc_expand_vars: bool,
	/// Command substitution state
	pub substitution_mode: bool,
	/// Current substitution depth
	pub substitution_depth: usize,
	/// Substitution buffer for nested commands
	pub substitution_buffer: String,
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
			multiline_mode: false,
			continuation_char: None,
			multiline_prompt: String::new(),
			heredoc_mode: false,
			heredoc_delimiter: String::new(),
			heredoc_content: String::new(),
			heredoc_expand_vars: false,
			substitution_mode: false,
			substitution_depth: 0,
			substitution_buffer: String::new(),
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
	 * Executes a command with substitution processing
	 * 
	 * @param command - Command to execute
	 */
	pub fn execute_command(&mut self, command: &str) {
		/**
		 * コマンド実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なコマンド処理を行います。
		 * 置換処理とコマンド実行が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Process command substitutions first
		let processed_command = match self.process_command_substitutions(command) {
			Ok(processed) => processed,
			Err(_) => command.to_string(),
		};
		
		// Add command to history
		if !processed_command.trim().is_empty() {
			self.history_manager.add_command(processed_command.clone(), None);
			self.tab_completer.add_command(processed_command.clone());
			self.history_index = None;
			self.history_search_mode = false;
			self.history_search_query.clear();
		}
		
		// Execute the processed command
		let output = self.run_command(&processed_command);
		
		// Add output to the focused pane
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
								// Enter: Execute current command or continue multiline
								if let Some(pane) = self.panes.get(self.focused_pane) {
									let command = pane.current_input.clone();
									
									if self.multiline_mode {
										// In multiline mode, add newline and continue
										let mut new_input = command.clone();
										new_input.push('\n');
										
										// Check if this is a heredoc delimiter before mutable borrow
										let is_heredoc_delimiter = self.heredoc_mode && self.is_heredoc_delimiter(&command);
										
										if is_heredoc_delimiter {
											// End of heredoc, execute the command
											let full_command = format!("{}\n{}", new_input, self.heredoc_content);
											self.execute_command(&full_command);
											
											// Clear the input and reset states
											if let Some(pane) = self.panes.get_mut(self.focused_pane) {
												pane.current_input.clear();
												pane.cursor_pos = 0;
											}
											
											self.history_index = None;
											self.original_input.clear();
											self.multiline_mode = false;
											self.continuation_char = None;
											self.multiline_prompt.clear();
											self.heredoc_mode = false;
											self.heredoc_delimiter.clear();
											self.heredoc_content.clear();
											self.heredoc_expand_vars = false;
										} else {
											// Continue collecting input
											if let Some(pane) = self.panes.get_mut(self.focused_pane) {
												pane.current_input = new_input.clone();
												pane.cursor_pos = pane.current_input.len();
											}
											
											// If in heredoc mode, add to heredoc content
											if self.heredoc_mode {
												let line_content = if self.heredoc_expand_vars {
													self.expand_heredoc_variables(&command)
												} else {
													command.clone()
												};
												self.heredoc_content.push_str(&line_content);
												self.heredoc_content.push('\n');
											}
											
											// Update multiline state
											self.update_multiline_state(&new_input);
										}
									} else if !command.trim().is_empty() {
										// Execute the command
										self.execute_command(&command);
										
										// Clear the input and reset history navigation
										if let Some(pane) = self.panes.get_mut(self.focused_pane) {
											pane.current_input.clear();
											pane.cursor_pos = 0;
										}
										
										self.history_index = None;
										self.original_input.clear();
										self.multiline_mode = false;
										self.continuation_char = None;
										self.multiline_prompt.clear();
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
											
											// Get the updated input for multiline state update
											let updated_input = pane.current_input.clone();
											
											// Update multiline state after character input
											self.update_multiline_state(&updated_input);
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
	
	/**
	 * Checks if input needs multiline continuation
	 * 
	 * @param input - Input text to check
	 * @return (bool, Option<char>) - (Needs continuation, continuation character)
	 */
	pub fn check_multiline_continuation(&self, input: &str) -> (bool, Option<char>) {
		/**
		 * マルチライン継続チェックの複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * 継続文字とクォート処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let trimmed = input.trim();
		
		// Check for backslash continuation
		if trimmed.ends_with('\\') {
			return (true, Some('\\'));
		}
		
		// Check for pipe continuation
		if trimmed.ends_with('|') {
			return (true, Some('|'));
		}
		
		// Check for unclosed quotes
		let mut in_single_quotes = false;
		let mut in_double_quotes = false;
		let mut escaped = false;
		
		for ch in input.chars() {
			if escaped {
				escaped = false;
				continue;
			}
			
			match ch {
				'\\' => escaped = true,
				'\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
				'"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
				_ => {}
			}
		}
		
		if in_single_quotes {
			return (true, Some('\''));
		}
		
		if in_double_quotes {
			return (true, Some('"'));
		}
		
		// Check for unclosed parentheses
		let mut paren_count = 0;
		let mut brace_count = 0;
		let mut bracket_count = 0;
		
		for ch in input.chars() {
			match ch {
				'(' => paren_count += 1,
				')' => paren_count -= 1,
				'{' => brace_count += 1,
				'}' => brace_count -= 1,
				'[' => bracket_count += 1,
				']' => bracket_count -= 1,
				_ => {}
			}
		}
		
		if paren_count > 0 {
			return (true, Some('('));
		}
		
		if brace_count > 0 {
			return (true, Some('{'));
		}
		
		if bracket_count > 0 {
			return (true, Some('['));
		}
		
		// No continuation needed
		(false, None)
	}
	
	/**
	 * Updates multiline state based on input
	 * 
	 * @param input - Input text to check
	 */
	pub fn update_multiline_state(&mut self, input: &str) {
		// Check for heredoc first
		if let Some((delimiter, expand_vars)) = self.detect_heredoc(input) {
			self.heredoc_mode = true;
			self.heredoc_delimiter = delimiter;
			self.heredoc_expand_vars = expand_vars;
			self.multiline_mode = true;
			self.multiline_prompt = format!("heredoc> ");
			return;
		}
		
		// Check for regular multiline continuation
		let (needs_continuation, continuation_char) = self.check_multiline_continuation(input);
		
		self.multiline_mode = needs_continuation;
		self.continuation_char = continuation_char;
		
		// Reset heredoc state if not in heredoc mode
		if !self.heredoc_mode {
			self.heredoc_delimiter.clear();
			self.heredoc_content.clear();
			self.heredoc_expand_vars = false;
		}
		
		// Set appropriate prompt
		if needs_continuation {
			match continuation_char {
				Some('\\') => self.multiline_prompt = "> ".to_string(),
				Some('|') => self.multiline_prompt = "| ".to_string(),
				Some('\'') => self.multiline_prompt = "'> ".to_string(),
				Some('"') => self.multiline_prompt = "\"> ".to_string(),
				Some('(') => self.multiline_prompt = "(> ".to_string(),
				Some('{') => self.multiline_prompt = "{> ".to_string(),
				Some('[') => self.multiline_prompt = "[> ".to_string(),
				_ => self.multiline_prompt = "> ".to_string(),
			}
		} else {
			self.multiline_prompt.clear();
		}
	}
	
	/**
	 * Checks if input contains heredoc syntax
	 * 
	 * @param input - Input text to check
	 * @return Option<(String, bool)> - (Delimiter, expand variables) if heredoc found
	 */
	pub fn detect_heredoc(&self, input: &str) -> Option<(String, bool)> {
		/**
		 * ヒアドキュメント検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * ヒアドキュメント構文の検出が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let words: Vec<&str> = input.split_whitespace().collect();
		
		for (i, word) in words.iter().enumerate() {
			if word.starts_with("<<") {
				// Check for quoted delimiter (no variable expansion)
				if word.starts_with("<<'") || word.starts_with("<<\"") {
					let quote_char = word.chars().nth(2).unwrap();
					let delimiter = word[3..].to_string();
					return Some((delimiter, false));
				}
				
				// Regular heredoc (with variable expansion)
				if word.len() > 2 {
					let delimiter = word[2..].to_string();
					return Some((delimiter, true));
				}
			}
		}
		
		None
	}
	
	/**
	 * Checks if current line matches heredoc delimiter
	 * 
	 * @param line - Current line to check
	 * @return bool - True if line matches delimiter
	 */
	pub fn is_heredoc_delimiter(&self, line: &str) -> bool {
		if !self.heredoc_mode {
			return false;
		}
		
		let trimmed = line.trim();
		trimmed == self.heredoc_delimiter
	}
	
	/**
	 * Expands variables in heredoc content
	 * 
	 * @param content - Content to expand
	 * @return String - Content with variables expanded
	 */
	pub fn expand_heredoc_variables(&self, content: &str) -> String {
		/**
		 * ヒアドキュメント変数展開の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な変数展開を行います。
		 * 環境変数の置換が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut result = String::new();
		let mut i = 0;
		
		while i < content.len() {
			if content[i..].starts_with('$') {
				// Found variable reference
				let var_start = i + 1;
				let mut var_end = var_start;
				
				// Find variable name
				while var_end < content.len() {
					let ch = content.chars().nth(var_end).unwrap();
					if ch.is_alphanumeric() || ch == '_' {
						var_end += 1;
					} else {
						break;
					}
				}
				
				if var_end > var_start {
					let var_name = &content[var_start..var_end];
					
					// Get environment variable
					if let Ok(var_value) = std::env::var(var_name) {
						result.push_str(&var_value);
					} else {
						// Variable not found, keep original
						result.push_str(&content[i..var_end]);
					}
					
					i = var_end;
				} else {
					// Just a $, keep it
					result.push('$');
					i += 1;
				}
			} else {
				// Regular character
				result.push(content.chars().nth(i).unwrap());
				i += 1;
			}
		}
		
		result
	}
	
	/**
	 * Detects command substitution in input
	 * 
	 * @param input - Input text to check
	 * @return Vec<(usize, usize, String)> - List of (start, end, command) substitutions
	 */
	pub fn detect_command_substitutions(&self, input: &str) -> Vec<(usize, usize, String)> {
		/**
		 * コマンド置換検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * ネストした置換処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut substitutions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
			// Check for $(command) syntax
			if input[i..].starts_with("$(") {
				let start = i;
				let mut depth = 1;
				let mut j = i + 2;
				
				while j < input.len() && depth > 0 {
					match input.chars().nth(j) {
						Some('(') => depth += 1,
						Some(')') => depth -= 1,
						_ => {}
					}
					j += 1;
				}
				
				if depth == 0 {
					let command = input[i + 2..j - 1].to_string();
					substitutions.push((start, j, command));
				}
				
				i = j;
			}
			// Check for `command` syntax
			else if input[i..].starts_with('`') {
				let start = i;
				let mut j = i + 1;
				
				while j < input.len() {
					if input.chars().nth(j) == Some('`') {
						break;
					}
					j += 1;
				}
				
				if j < input.len() {
					let command = input[i + 1..j].to_string();
					substitutions.push((start, j + 1, command));
				}
				
				i = j + 1;
			} else {
				i += 1;
			}
		}
		
		substitutions
	}
	
	/**
	 * Executes a command and returns its output
	 * 
	 * @param command - Command to execute
	 * @return Result<String> - Command output or error
	 */
	pub fn execute_substitution_command(&self, command: &str) -> Result<String> {
		/**
		 * コマンド置換実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なコマンド実行を行います。
		 * 子プロセス実行と出力取得が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		use std::process::Command;
		
		// Split command into parts
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(String::new());
		}
		
		// Execute the command
		let output = Command::new(parts[0])
			.args(&parts[1..])
			.output()?;
		
		// Convert output to string
		let stdout = String::from_utf8(output.stdout)?;
		let stderr = String::from_utf8(output.stderr)?;
		
		// Combine stdout and stderr, trim whitespace
		let mut result = stdout;
		if !stderr.is_empty() {
			result.push_str(&stderr);
		}
		
		Ok(result.trim().to_string())
	}
	
	/**
	 * Processes command substitutions in input
	 * 
	 * @param input - Input text to process
	 * @return Result<String> - Processed input with substitutions
	 */
	pub fn process_command_substitutions(&self, input: &str) -> Result<String> {
		/**
		 * コマンド置換処理の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な置換処理を行います。
		 * ネストした置換とエラーハンドリングが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut result = input.to_string();
		let substitutions = self.detect_command_substitutions(&result);
		
		// Process substitutions in reverse order to maintain indices
		for (start, end, command) in substitutions.iter().rev() {
			match self.execute_substitution_command(command) {
				Ok(output) => {
					// Replace the substitution with the output
					result.replace_range(*start..*end, &output);
				}
				Err(_) => {
					// On error, replace with empty string
					result.replace_range(*start..*end, "");
				}
			}
		}
		
		Ok(result)
	}
} 