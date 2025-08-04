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
	/// Input buffer for commands
	input_buffer: String,
	/// Terminal output history
	output_history: Vec<String>,
	/// Current working directory
	current_dir: String,
	/// Terminal state
	terminal_state: TerminalState,
	/// Command history
	command_history: Vec<String>,
	/// History index for navigation
	history_index: Option<usize>,
}

/**
 * Terminal state
 * 
 * Tracks the current state of the terminal including
 * cursor position, selection, and input mode.
 */
#[derive(Debug, Clone)]
struct TerminalState {
	/// Cursor position in input
	cursor_pos: usize,
	/// Whether text is selected
	selection_active: bool,
	/// Selection start position
	selection_start: usize,
	/// Selection end position
	selection_end: usize,
	/// Terminal size
	terminal_size: (u32, u32),
	/// Scroll position
	scroll_pos: f32,
	/// Auto-scroll enabled
	auto_scroll: bool,
}

impl Default for TerminalState {
	fn default() -> Self {
		Self {
			cursor_pos: 0,
			selection_active: false,
			selection_start: 0,
			selection_end: 0,
			terminal_size: (80, 24),
			scroll_pos: 0.0,
			auto_scroll: true,
		}
	}
}

impl SareTerminal {
	/**
	 * Creates a new Sare terminal
	 * 
	 * Initializes the GUI terminal with real shell functionality
	 * and command execution capabilities.
	 * 
	 * @return Result<SareTerminal> - New terminal instance or error
	 */
	fn new() -> Result<Self> {
		/**
		 * Sareターミナル初期化の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なターミナル初期化を行います。
		 * シェル統合とコマンド実行が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		// Get current directory
		let current_dir = std::env::current_dir()?
			.to_string_lossy()
			.to_string();
		
		// Initialize output with welcome message
		let mut output_history = Vec::new();
		output_history.push("🌸 Welcome to Sare Terminal Emulator 🌸".to_string());
		output_history.push("".to_string());
		output_history.push(format!("Current directory: {}", current_dir));
		output_history.push("Type 'help' for available commands".to_string());
		output_history.push("".to_string());
		
		Ok(Self {
			input_buffer: String::new(),
			output_history,
			current_dir,
			terminal_state: TerminalState::default(),
			command_history: Vec::new(),
			history_index: None,
		})
	}
	
	/**
	 * Executes a shell command
	 * 
	 * Executes the given command using the system shell
	 * and captures the output for display.
	 * 
	 * @param command - Command to execute
	 */
	fn execute_command(&mut self, command: &str) {
		/**
		 * コマンド実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なシェルコマンド実行を行います。
		 * プロセス作成と出力キャプチャが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		if command.trim().is_empty() {
			return;
		}
		
		// Add command to history
		self.command_history.push(command.to_string());
		self.output_history.push(format!("$ {}", command));
		
		// Handle built-in commands
		match command.trim() {
			"clear" => {
				self.output_history.clear();
				self.output_history.push("🌸 Sare Terminal Emulator 🌸".to_string());
				return;
			}
			"help" => {
				self.output_history.push("Available commands:".to_string());
				self.output_history.push("  clear - Clear terminal".to_string());
				self.output_history.push("  help - Show this help".to_string());
				self.output_history.push("  pwd - Show current directory".to_string());
				self.output_history.push("  ls - List files".to_string());
				self.output_history.push("  cd <dir> - Change directory".to_string());
				self.output_history.push("  exit - Exit terminal".to_string());
				return;
			}
			"pwd" => {
				self.output_history.push(self.current_dir.clone());
				return;
			}
			"exit" => {
				std::process::exit(0);
			}
			_ => {}
		}
		
		// Handle cd command specially
		if command.starts_with("cd ") {
			let new_dir = command[3..].trim();
			if let Ok(new_path) = std::env::set_current_dir(new_dir) {
				if let Ok(current_dir) = std::env::current_dir() {
					self.current_dir = current_dir.to_string_lossy().to_string();
				}
			} else {
				self.output_history.push(format!("Error: Cannot change to directory '{}'", new_dir));
			}
			return;
		}
		
		// Execute system command
		let output = Command::new("sh")
			.arg("-c")
			.arg(command)
			.current_dir(&self.current_dir)
			.output();
		
		match output {
			Ok(output) => {
				// Add stdout
				if !output.stdout.is_empty() {
					let stdout = String::from_utf8_lossy(&output.stdout);
					for line in stdout.lines() {
						self.output_history.push(line.to_string());
					}
				}
				
				// Add stderr
				if !output.stderr.is_empty() {
					let stderr = String::from_utf8_lossy(&output.stderr);
					for line in stderr.lines() {
						self.output_history.push(format!("ERROR: {}", line));
					}
				}
				
				// Add exit status
				if !output.status.success() {
					self.output_history.push(format!("Exit code: {}", output.status));
				}
			}
			Err(e) => {
				self.output_history.push(format!("Error executing command: {}", e));
			}
		}
		
		// Clear input buffer
		self.input_buffer.clear();
		self.terminal_state.cursor_pos = 0;
	}
	
	/**
	 * Renders the terminal interface
	 * 
	 * Renders the terminal GUI with output history, command input,
	 * and proper terminal styling.
	 * 
	 * @param ctx - Egui context
	 */
	fn render_terminal(&mut self, ctx: &egui::Context) {
		/**
		 * ターミナルレンダリングの複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なGUIレンダリングを行います。
		 * リアルタイム出力表示とユーザーインターフェースが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		egui::CentralPanel::default().show(ctx, |ui| {
			// Set terminal background
			ui.painter().rect_filled(
				ui.available_rect_before_wrap(),
				0.0,
				egui::Color32::from_rgb(0, 0, 0),
			);
			
			// Configure terminal text style
			let text_style = egui::TextStyle::Monospace;
			
			// Render output area
			ui.group(|ui| {
				ui.set_enabled(false);
				
				// Scrollable output area
				egui::ScrollArea::vertical()
					.max_height(ui.available_height() - 60.0)
					.show(ui, |ui| {
						for output in &self.output_history {
							ui.label(egui::RichText::new(output)
								.color(egui::Color32::from_rgb(255, 255, 255))
								.text_style(text_style));
						}
					});
			});
			
			// Render command input area
			ui.group(|ui| {
				ui.horizontal(|ui| {
					// Terminal prompt
					ui.label(egui::RichText::new(format!("sare@{}:{} $ ", 
						whoami::username(), 
						self.current_dir
					)).color(egui::Color32::from_rgb(0, 255, 0))
					.text_style(text_style));
					
					// Command input field
					let response = ui.add_sized(
						[ui.available_width(), 20.0],
						egui::TextEdit::singleline(&mut self.input_buffer)
							.text_style(text_style)
							.desired_width(f32::INFINITY)
							.hint_text("Enter command...")
							.cursor_at_end(true)
					);
					
					// Handle Enter key for command execution
					if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
						let command = self.input_buffer.clone();
						if !command.trim().is_empty() {
							self.execute_command(&command);
						}
					}
					
					// Handle Up/Down for command history
					if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
						self.navigate_history_up();
					}
					
					if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
						self.navigate_history_down();
					}
					
					// Handle Ctrl+C to clear input
					if ui.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.ctrl) {
						self.input_buffer.clear();
						self.terminal_state.cursor_pos = 0;
					}
				});
			});
			
			// Render status bar
			ui.group(|ui| {
				ui.horizontal(|ui| {
					ui.label(egui::RichText::new("Ready").color(egui::Color32::from_rgb(0, 255, 0)));
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						ui.label(egui::RichText::new("Ctrl+C: Clear | Ctrl+D: Exit | ↑↓: History").color(egui::Color32::from_rgb(128, 128, 128)));
					});
				});
			});
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
			self.input_buffer = self.command_history[current_index - 1].clone();
			self.terminal_state.cursor_pos = self.input_buffer.len();
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
			self.input_buffer = self.command_history[current_index + 1].clone();
			self.terminal_state.cursor_pos = self.input_buffer.len();
		} else {
			self.history_index = None;
			self.input_buffer.clear();
			self.terminal_state.cursor_pos = 0;
		}
	}
}

impl eframe::App for SareTerminal {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Handle global keyboard shortcuts
		if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
			self.input_buffer.clear();
			self.terminal_state.cursor_pos = 0;
		}
		
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

fn main() -> Result<(), eframe::Error> {
	/**
	 * メインアプリケーション初期化の複雑な処理です (｡◕‿◕｡)
	 * 
	 * この関数は複雑なGUI初期化を行います。
	 * ウィンドウ作成とアプリケーション設定が難しい部分なので、
	 * 適切なエラーハンドリングで実装しています (◕‿◕)
	 */
	
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
	)
} 