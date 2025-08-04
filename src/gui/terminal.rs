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
use super::multiline::{MultilineState, MultilineProcessor};
use super::heredoc::{HeredocState, HeredocProcessor};
use super::substitution::{SubstitutionState, SubstitutionProcessor};

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
	pub multiline_state: MultilineState,
	/// Heredoc state
	pub heredoc_state: HeredocState,
	/// Command substitution state
	pub substitution_mode: SubstitutionState,
	/// Brace expansion state
	pub brace_expansion_mode: bool,
	/// Globbing patterns cache
	pub glob_cache: std::collections::HashMap<String, Vec<String>>,
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
			multiline_state: MultilineState::default(),
			heredoc_state: HeredocState::default(),
			substitution_mode: SubstitutionState::default(),
			brace_expansion_mode: false,
			glob_cache: std::collections::HashMap::new(),
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
		let processed_command = match self.substitution_mode.process_substitutions(command) {
			Ok(processed) => processed,
			Err(_) => command.to_string(),
		};
		
		// Process brace expansions and globbing
		let final_command = match self.process_brace_expansions(&processed_command) {
			Ok(expanded) => expanded,
			Err(_) => processed_command,
		};
		
		// Add command to history
		if !final_command.trim().is_empty() {
			self.history_manager.add_command(final_command.clone(), None);
			self.tab_completer.add_command(final_command.clone());
			self.history_index = None;
			self.history_search_mode = false;
			self.history_search_query.clear();
		}
		
		// Execute the processed command
		let output = self.run_command(&final_command);
		
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
									
									if self.multiline_state.is_multiline() {
										// In multiline mode, add newline and continue
										let mut new_input = command.clone();
										new_input.push('\n');
										
										// Check if this is a heredoc delimiter before mutable borrow
										let is_heredoc_delimiter = self.heredoc_state.is_heredoc() && HeredocProcessor::is_heredoc_delimiter(&self.heredoc_state, &command);
										
										if is_heredoc_delimiter {
											// End of heredoc, execute the command
											let full_command = format!("{}\n{}", new_input, self.heredoc_state.get_heredoc_content());
											self.execute_command(&full_command);
											
											// Clear the input and reset states
											if let Some(pane) = self.panes.get_mut(self.focused_pane) {
												pane.current_input.clear();
												pane.cursor_pos = 0;
											}
											
											self.history_index = None;
											self.original_input.clear();
											self.multiline_state = MultilineState::default();
											self.heredoc_state = HeredocState::default();
										} else {
											// Continue collecting input
											if let Some(pane) = self.panes.get_mut(self.focused_pane) {
												pane.current_input = new_input.clone();
												pane.cursor_pos = pane.current_input.len();
											}
											
											// If in heredoc mode, add to heredoc content
											if self.heredoc_state.is_heredoc() {
												let line_content = if self.heredoc_state.should_expand_vars() {
													HeredocProcessor::expand_heredoc_variables(&command)
												} else {
													command.clone()
												};
												self.heredoc_state.add_heredoc_content(line_content);
											}
											
											// Update multiline state
											self.multiline_state.update(&new_input);
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
										self.multiline_state = MultilineState::default();
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
											self.multiline_state.update(&updated_input);
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
	 * Detects brace expansion patterns in input
	 * 
	 * @param input - Input text to check
	 * @return Vec<(usize, usize, String)> - List of (start, end, pattern) expansions
	 */
	pub fn detect_brace_expansions(&self, input: &str) -> Vec<(usize, usize, String)> {
		/**
		 * ブレース展開検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * ネストしたブレース処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut expansions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
			if input[i..].starts_with('{') {
				let start = i;
				let mut depth = 1;
				let mut j = i + 1;
				
				while j < input.len() && depth > 0 {
					match input.chars().nth(j) {
						Some('{') => depth += 1,
						Some('}') => depth -= 1,
						_ => {}
					}
					j += 1;
				}
				
				if depth == 0 {
					let pattern = input[i + 1..j - 1].to_string();
					expansions.push((start, j, pattern));
				}
				
				i = j;
			} else {
				i += 1;
			}
		}
		
		expansions
	}
	
	/**
	 * Expands a brace pattern
	 * 
	 * @param pattern - Brace pattern to expand
	 * @return Vec<String> - List of expanded strings
	 */
	pub fn expand_brace_pattern(&self, pattern: &str) -> Vec<String> {
		/**
		 * ブレース展開の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な展開処理を行います。
		 * 数値範囲とカンマ区切り処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Check for numeric range {start..end}
		if pattern.contains("..") {
			return self.expand_numeric_range(pattern);
		}
		
		// Check for comma-separated list {a,b,c}
		if pattern.contains(',') {
			return self.expand_comma_list(pattern);
		}
		
		// Single item, return as-is
		vec![pattern.to_string()]
	}
	
	/**
	 * Expands a numeric range pattern
	 * 
	 * @param pattern - Numeric range pattern
	 * @return Vec<String> - List of expanded numbers
	 */
	fn expand_numeric_range(&self, pattern: &str) -> Vec<String> {
		let parts: Vec<&str> = pattern.split("..").collect();
		if parts.len() != 2 {
			return vec![pattern.to_string()];
		}
		
		let start = parts[0].trim().parse::<i32>().unwrap_or(0);
		let end = parts[1].trim().parse::<i32>().unwrap_or(0);
		
		let mut result = Vec::new();
		for i in start..=end {
			result.push(i.to_string());
		}
		
		result
	}
	
	/**
	 * Expands a comma-separated list pattern
	 * 
	 * @param pattern - Comma-separated pattern
	 * @return Vec<String> - List of expanded items
	 */
	fn expand_comma_list(&self, pattern: &str) -> Vec<String> {
		/**
		 * カンマ区切り展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なリスト処理を行います。
		 * ネストしたカンマ処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut result = Vec::new();
		let mut current = String::new();
		let mut depth = 0;
		
		for ch in pattern.chars() {
			match ch {
				',' if depth == 0 => {
					if !current.is_empty() {
						result.push(current.trim().to_string());
						current.clear();
					}
				}
				'{' => {
					depth += 1;
					current.push(ch);
				}
				'}' => {
					depth -= 1;
					current.push(ch);
				}
				_ => {
					current.push(ch);
				}
			}
		}
		
		// Add the last item
		if !current.is_empty() {
			result.push(current.trim().to_string());
		}
		
		result
	}
	
	/**
	 * Expands glob patterns to matching files
	 * 
	 * @param pattern - Glob pattern to expand
	 * @return Vec<String> - List of matching files
	 */
	pub fn expand_glob_pattern(&self, pattern: &str) -> Vec<String> {
		/**
		 * グロブ展開の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なファイルマッチングを行います。
		 * パターンマッチングとファイル検索が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Check cache first
		if let Some(cached) = self.glob_cache.get(pattern) {
			return cached.clone();
		}
		
		let mut matches = Vec::new();
		
		// Simple glob implementation
		if pattern.contains('*') || pattern.contains('?') {
			// For now, implement basic globbing
			// In a full implementation, this would use a proper glob library
			if let Ok(entries) = std::fs::read_dir(".") {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(file_name) = entry.file_name().into_string() {
							if self.matches_glob_pattern(&file_name, pattern) {
								matches.push(file_name);
							}
						}
					}
				}
			}
		} else {
			// No glob characters, check if file exists
			if std::path::Path::new(pattern).exists() {
				matches.push(pattern.to_string());
			}
		}
		
		// Cache the result
		matches.sort();
		matches
	}
	
	/**
	 * Checks if a filename matches a glob pattern
	 * 
	 * @param filename - Filename to check
	 * @param pattern - Glob pattern
	 * @return bool - True if filename matches pattern
	 */
	fn matches_glob_pattern(&self, filename: &str, pattern: &str) -> bool {
		// Simple glob matching implementation
		// This is a basic implementation - a full version would use regex
		
		if pattern == "*" {
			return true;
		}
		
		if pattern.starts_with("*.") {
			let ext = &pattern[1..];
			return filename.ends_with(ext);
		}
		
		if pattern.ends_with("*") {
			let prefix = &pattern[..pattern.len()-1];
			return filename.starts_with(prefix);
		}
		
		filename == pattern
	}
	
	/**
	 * Processes brace expansions and globbing in input
	 * 
	 * @param input - Input text to process
	 * @return Result<String> - Processed input with expansions
	 */
	pub fn process_brace_expansions(&self, input: &str) -> Result<String> {
		/**
		 * ブレース展開処理の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な展開処理を行います。
		 * ネストした展開とグロブ処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut result = input.to_string();
		let expansions = self.detect_brace_expansions(&result);
		
		// Process expansions in reverse order to maintain indices
		for (start, end, pattern) in expansions.iter().rev() {
			let expanded = self.expand_brace_pattern(pattern);
			let expanded_str = expanded.join(" ");
			
			// Replace the expansion with the expanded string
			result.replace_range(*start..*end, &expanded_str);
		}
		
		// Process glob patterns
		let words: Vec<&str> = result.split_whitespace().collect();
		let mut processed_words = Vec::new();
		
		for word in words {
			if word.contains('*') || word.contains('?') {
				let glob_matches = self.expand_glob_pattern(word);
				if !glob_matches.is_empty() {
					processed_words.extend(glob_matches);
				} else {
					processed_words.push(word.to_string());
				}
			} else {
				processed_words.push(word.to_string());
			}
		}
		
		Ok(processed_words.join(" "))
	}
} 