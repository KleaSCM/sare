
/**
 * Main terminal interface module for Sare terminal
 * 
 * This module provides the core terminal interface including
 * command execution, input handling, pane management, history
 * navigation, and integration with all terminal subsystems.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: terminal.rs
 * Description: Main terminal interface and command processing
 */

use anyhow::Result;
use eframe::egui;
use std::process::Command;

use super::pane::{TerminalPane, SplitDirection, TerminalMode, TerminalLine};
// TODO: Fix history import
// use super::super::history::{HistoryManager, TabCompleter};
use super::multiline::{MultilineState, MultilineProcessor};
use super::heredoc::{HeredocState, HeredocProcessor};
use super::substitution::{SubstitutionState, SubstitutionProcessor};
use super::expansion::{ExpansionState, ExpansionProcessor};

#[derive(Debug)]
pub struct SareTerminal {
	/// Command history manager
	// pub history_manager: HistoryManager,
	/// Tab completion engine
	// pub tab_completer: TabCompleter,
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
	/// Brace expansion and globbing state
	pub expansion_state: ExpansionState,
}

impl Default for SareTerminal {
	fn default() -> Self {

		
		let default_pane = TerminalPane::default();
		// TODO: Fix history manager initialization
		// let history_manager = HistoryManager::new().unwrap_or_else(|_| {
		// 	HistoryManager::with_config(1000, std::path::PathBuf::from(".sare_history"))
		// 		.unwrap_or_else(|_| HistoryManager {
		// 			history: std::collections::VecDeque::new(),
		// 			max_entries: 1000,
		// 			history_file: std::path::PathBuf::from(".sare_history"),
		// 		})
		// });
		
		// TODO: Fix tab completer initialization
		// let working_directory = std::env::current_dir()
		// 	.unwrap_or_default();
		// let tab_completer = TabCompleter::new(working_directory);
		
		Self {
			// history_manager,
			// tab_completer,
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
			expansion_state: ExpansionState::default(),
		}
	}
}

impl SareTerminal {
	pub fn new() -> Result<Self> {
		Ok(Self::default())
	}
	
	pub fn execute_command(&mut self, command: &str) {

		
		let processed_command = match self.substitution_mode.process_substitutions(command) {
			Ok(processed) => processed,
			Err(_) => command.to_string(),
		};
		
		let final_command = match self.expansion_state.process_expansions(&processed_command) {
			Ok(expanded) => expanded,
			Err(_) => processed_command,
		};
		
		if !final_command.trim().is_empty() {
			// TODO: Fix history manager usage
			// self.history_manager.add_command(final_command.clone(), None);
			// self.tab_completer.add_command(final_command.clone());
			self.history_index = None;
			self.history_search_mode = false;
			self.history_search_query.clear();
		}
		
		let output = self.run_command(&final_command);
		
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			pane.add_output_line(output, egui::Color32::from_rgb(255, 255, 255), false);
		}
	}
	
	pub fn run_command(&mut self, command: &str) -> String {
		if command.trim().is_empty() {
			return String::new();
		}
		
		match command.trim() {
			"clear" => {
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
	
	fn get_history_display(&self) -> String {
		// TODO: Fix history manager usage
		// let history = self.history_manager.get_history();
		// let mut display = String::new();
		// 
		// for (i, entry) in history.iter().enumerate() {
		// 	display.push_str(&format!("{:4}  {}\n", i + 1, entry.command));
		// }
		// 
		// display
		"History display temporarily disabled".to_string()
	}
	
	pub fn add_output_line(&mut self, content: String, color: egui::Color32, is_prompt: bool) {
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			pane.add_output_line(content, color, is_prompt);
		}
	}
	
	pub fn handle_key_input(&mut self, ctx: &egui::Context) {

		
		ctx.input(|input| {
			// Handle key presses
			for event in &input.events {
				if let egui::Event::Key { key, pressed, modifiers, .. } = event {
					if *pressed {
						match *key {
							egui::Key::ArrowUp => {
								if modifiers.ctrl {
									self.navigate_history_up();
								} else {
									self.navigate_history_up();
								}
							}
							egui::Key::ArrowDown => {
								if modifiers.ctrl {
									self.navigate_history_down();
								} else {
									self.navigate_history_down();
								}
							}
							egui::Key::Tab => {
								if let Some(pane) = self.panes.get_mut(self.focused_pane) {
									let input = &pane.current_input;
									let cursor_pos = pane.cursor_pos;
									
									// TODO: Fix tab completer usage
									// if let Ok(Some(completion)) = self.tab_completer.complete(input, cursor_pos) {
									// 	pane.current_input = completion.completed_text;
									// 	pane.cursor_pos = pane.current_input.len();
									// 	
									// 	if completion.is_partial && !completion.alternatives.is_empty() {
									// 		println!("Available completions: {:?}", completion.alternatives);
									// 	}
									// }
								}
							}
							egui::Key::Tab if modifiers.shift => {
								println!("Shift+Tab detected - switching panes");
								self.switch_to_next_pane();
							}
							egui::Key::D => {
								if modifiers.ctrl {
									println!("Ctrl+D detected - closing pane");
									self.close_current_pane();
								}
							}
							egui::Key::N => {
								if modifiers.ctrl {
									println!("Ctrl+N detected - creating new pane");
									self.split_pane(SplitDirection::Horizontal);
								}
							}
							egui::Key::H => {
								if modifiers.ctrl {
									println!("Ctrl+H detected - creating new pane");
									self.split_pane(SplitDirection::Vertical);
								}
							}
							egui::Key::R => {
								if modifiers.ctrl {
									self.start_reverse_search();
								}
							}
							egui::Key::Escape => {
								if self.history_search_mode {
									self.exit_history_search();
								}
							}
							egui::Key::Enter => {
								if let Some(pane) = self.panes.get(self.focused_pane) {
									let command = pane.current_input.clone();
									
									if self.multiline_state.is_multiline() {
										let mut new_input = command.clone();
										new_input.push('\n');
										
										let is_heredoc_delimiter = self.heredoc_state.is_heredoc() && HeredocProcessor::is_heredoc_delimiter(&self.heredoc_state, &command);
										
										if is_heredoc_delimiter {
											let full_command = format!("{}\n{}", new_input, self.heredoc_state.get_heredoc_content());
											self.execute_command(&full_command);
											
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
											
											self.multiline_state.update(&new_input);
										}
									} else if !command.trim().is_empty() {
										self.execute_command(&command);
										
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
								if let Some(c) = Self::key_to_char(*key) {
									if self.history_search_mode {
										self.history_search_query.push(c);
										self.perform_reverse_search();
									} else {
										if let Some(pane) = self.panes.get_mut(self.focused_pane) {
											pane.add_char(c);
											
											let updated_input = pane.current_input.clone();
											
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
	
	pub fn navigate_history_up(&mut self) {
		// TODO: Fix history manager usage
		// let history = self.history_manager.get_history();
		// if history.is_empty() {
		// 	return;
		// }
		// 
		// let current_index = self.history_index.unwrap_or(history.len());
		// 
		// if current_index > 0 {
		// 	self.history_index = Some(current_index - 1);
		// 	if let Some(entry) = history.get(current_index - 1) {
		// 		if self.original_input.is_empty() {
		// 			if let Some(pane) = self.panes.get(self.focused_pane) {
		// 				self.original_input = pane.current_input.clone();
		// 			}
		// 		}
		// 		
		// 		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
		// 			pane.current_input = entry.command.clone();
		// 		}
		// 	}
		// }
	}
	
	pub fn navigate_history_down(&mut self) {
		// TODO: Fix history manager usage
		// let history = self.history_manager.get_history();
		// if history.is_empty() {
		// 	return;
		// }
		// 
		// let current_index = self.history_index.unwrap_or(history.len());
		// 
		// if current_index < history.len() - 1 {
		// 	self.history_index = Some(current_index + 1);
		// 	if let Some(entry) = history.get(current_index + 1) {
		// 		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
		// 			pane.current_input = entry.command.clone();
		// 		}
		// 	}
		// } else {
		// 	// Reached the end, restore original input
		// 	self.history_index = None;
		// 	if let Some(pane) = self.panes.get_mut(self.focused_pane) {
		// 		pane.current_input = self.original_input.clone();
		// 	}
		// 	self.original_input.clear();
		// }
	}
	
	pub fn start_reverse_search(&mut self) {
		self.history_search_mode = true;
		self.history_search_query.clear();
		self.history_index = None;
		
		// Save current input
		if let Some(pane) = self.panes.get(self.focused_pane) {
			self.original_input = pane.current_input.clone();
		}
	}
	
	pub fn perform_reverse_search(&mut self) {
		// TODO: Fix history manager usage
		// let history = self.history_manager.get_history();
		// let query = &self.history_search_query;
		// 
		// if query.is_empty() {
		// 	return;
		// }
		// 
		// // Search backwards through history
		// for (i, entry) in history.iter().enumerate().rev() {
		// 	if entry.command.contains(query) {
		// 		self.history_index = Some(i);
		// 		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
		// 			pane.current_input = entry.command.clone();
		// 		}
		// 		break;
		// 	}
		// }
	}
	
	pub fn exit_history_search(&mut self) {
		self.history_search_mode = false;
		self.history_search_query.clear();
		self.history_index = None;
		
		if let Some(pane) = self.panes.get_mut(self.focused_pane) {
			pane.current_input = self.original_input.clone();
		}
		self.original_input.clear();
	}
	
	pub fn split_pane(&mut self, direction: SplitDirection) {
		let focused_pane_index = self.focused_pane;
		let current_layout = self.panes[focused_pane_index].layout;
		let current_id = self.panes[focused_pane_index].id.clone();
		let (x, y, width, height) = current_layout;
		
		// Create new pane
		let new_pane_id = format!("pane_{}", self.panes.len());
		let mut new_pane = TerminalPane::default();
		new_pane.id = new_pane_id.clone();
		new_pane.working_directory = self.current_dir.clone();
		
		match direction {
			SplitDirection::Vertical => {
				let new_width = width / 2.0;
				new_pane.layout = (x + new_width, y, new_width, height);
				
				self.panes[focused_pane_index].layout = (x, y, new_width, height);
			}
			SplitDirection::Horizontal => {
				let new_height = height / 2.0;
				new_pane.layout = (x, y + new_height, width, new_height);
				
				self.panes[focused_pane_index].layout = (x, y, width, new_height);
			}
		}
		
		new_pane.split_direction = Some(direction.clone());
		self.panes[focused_pane_index].split_direction = Some(direction);
		
		new_pane.parent_id = Some(current_id);
		self.panes[focused_pane_index].child_ids.push(new_pane_id.clone());
		
		self.panes.push(new_pane);
		
		self.focused_pane = self.panes.len() - 1;
		
		for (i, pane) in self.panes.iter_mut().enumerate() {
			pane.active = i == self.focused_pane;
		}
	}
	
	pub fn create_new_pane(&mut self) {
		self.split_pane(SplitDirection::Vertical);
	}
	
	pub fn close_current_pane(&mut self) {
		if self.panes.len() > 1 {
			self.panes.remove(self.focused_pane);
			if self.focused_pane >= self.panes.len() {
				self.focused_pane = self.panes.len() - 1;
			}
			
			for (i, pane) in self.panes.iter_mut().enumerate() {
				pane.active = i == self.focused_pane;
			}
		}
	}
	
	pub fn switch_to_next_pane(&mut self) {
		if self.panes.len() > 1 {
			self.focused_pane = (self.focused_pane + 1) % self.panes.len();
			
			for (i, pane) in self.panes.iter_mut().enumerate() {
				pane.active = i == self.focused_pane;
			}
		}
	}
}

impl eframe::App for SareTerminal {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.handle_key_input(ctx);
		super::renderer::TerminalRenderer::render_terminal(self, ctx);		
		ctx.request_repaint();
	}
	
	fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
		// Dark background color
		[0.1, 0.1, 0.1, 1.0]
	}
} 