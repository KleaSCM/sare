/**
 * Sare Terminal Emulator Library
 * 
 * This library provides a modern, feature-rich terminal emulator
 * with GPU acceleration, advanced rendering, and developer tools.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: lib.rs
 * Description: Main library module for Sare terminal emulator
 */

pub mod terminal;
pub mod tui;
pub mod gui;
pub mod history;
pub mod features;
pub mod session;
pub mod config;
pub mod unicode;
pub mod ui;
pub mod debug;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/**
 * Main terminal emulator struct
 * 
 * メインターミナルエミュレーター構造体です。
 * すべてのターミナル機能を
 * 統合管理します。
 */
pub struct SareTerminal {
	/// Terminal configuration
	config: crate::config::Config,
	/// UI manager
	ui_manager: crate::ui::UiManager,
	/// Terminal sessions
	sessions: Vec<crate::session::SessionManager>,
	/// History manager
	history: crate::history::HistoryManager,
	/// Debug manager
	debug_manager: Arc<crate::debug::DebugManager>,
	/// Profiler
	profiler: Arc<crate::debug::Profiler>,
	/// Logger
	logger: Arc<crate::debug::Logger>,
	/// Error recovery manager
	error_recovery: Arc<crate::debug::ErrorRecoveryManager>,
	/// Testing framework
	testing_framework: Arc<crate::debug::TestingFramework>,
	/// Terminal emulator
	terminal_emulator: Arc<RwLock<crate::terminal::TerminalEmulator>>,
	/// Input buffer
	input_buffer: Arc<RwLock<Vec<u8>>>,
	/// Output buffer
	output_buffer: Arc<RwLock<Vec<u8>>>,
	/// Terminal state
	terminal_state: Arc<RwLock<TerminalState>>,
	/// Running state
	running: Arc<RwLock<bool>>,
}

/**
 * Terminal state
 * 
 * ターミナル状態です。
 * ターミナルの現在の状態を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct TerminalState {
	/// Terminal size (columns, rows)
	pub size: (u16, u16),
	/// Cursor position (column, row)
	pub cursor_pos: (u16, u16),
	/// Scroll position
	pub scroll_pos: u32,
	/// Terminal mode
	pub mode: TerminalMode,
	/// Last update time
	pub last_update: Instant,
}

/**
 * Terminal mode
 * 
 * ターミナルモードです。
 * ターミナルの動作モードを
 * 定義します。
 */
#[derive(Debug, Clone)]
pub struct TerminalMode {
	/// Insert mode
	pub insert_mode: bool,
	/// Application cursor keys
	pub app_cursor_keys: bool,
	/// Application keypad
	pub app_keypad: bool,
	/// Mouse tracking
	pub mouse_tracking: bool,
	/// Bracketed paste
	pub bracketed_paste: bool,
	/// Debug mode
	pub debug_mode: bool,
}

impl Default for TerminalMode {
	fn default() -> Self {
		Self {
			insert_mode: false,
			app_cursor_keys: false,
			app_keypad: false,
			mouse_tracking: false,
			bracketed_paste: false,
			debug_mode: false,
		}
	}
}

impl SareTerminal {
	/**
	 * Creates a new terminal emulator instance
	 * 
	 * @param config - Terminal configuration
	 * @return Result<SareTerminal> - New terminal instance or error
	 */
	pub async fn new(config: crate::config::Config) -> Result<Self> {
		// Initialize debug components
		let debug_config = crate::debug::DebugConfig::default();
		let debug_manager = Arc::new(crate::debug::DebugManager::new(debug_config));
		
		let profiler_config = crate::debug::ProfilerConfig::default();
		let profiler = Arc::new(crate::debug::Profiler::new(profiler_config));
		
		let logger_config = crate::debug::LoggerConfig::default();
		let logger = Arc::new(crate::debug::Logger::new(logger_config));
		
		let error_recovery_config = crate::debug::ErrorRecoveryConfig::default();
		let error_recovery = Arc::new(crate::debug::ErrorRecoveryManager::new(error_recovery_config));
		
		let test_config = crate::debug::TestConfig::default();
		let testing_framework = Arc::new(crate::debug::TestingFramework::new(test_config));
		
		// Initialize UI manager
		let ui_config = crate::ui::UiConfig::default();
		let ui_manager = crate::ui::UiManager::new(ui_config);
		
		// Initialize terminal emulator
		let terminal_config = crate::terminal::TerminalConfig::default();
		let terminal_emulator = Arc::new(RwLock::new(crate::terminal::TerminalEmulator::new(terminal_config)?));
		
		// Initialize buffers and state
		let input_buffer = Arc::new(RwLock::new(Vec::new()));
		let output_buffer = Arc::new(RwLock::new(Vec::new()));
		let terminal_state = Arc::new(RwLock::new(TerminalState {
			size: (80, 24),
			cursor_pos: (0, 0),
			scroll_pos: 0,
			mode: TerminalMode::default(),
			last_update: Instant::now(),
		}));
		let running = Arc::new(RwLock::new(true));
		
		Ok(Self {
			config,
			ui_manager,
			sessions: Vec::new(),
			history: crate::history::HistoryManager::new()?,
			debug_manager,
			profiler,
			logger,
			error_recovery,
			testing_framework,
			terminal_emulator,
			input_buffer,
			output_buffer,
			terminal_state,
			running,
		})
	}
	
	/**
	 * Initializes the terminal emulator
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		println!("🚀 Initializing Sare Terminal Emulator...");
		
		// Initialize debug components
		self.debug_manager.initialize().await?;
		self.profiler.initialize().await?;
		self.logger.initialize().await?;
		self.error_recovery.initialize().await?;
		self.testing_framework.initialize().await?;
		
		// Initialize UI manager
		self.ui_manager.initialize().await?;
		
		// Initialize history manager
		self.history.initialize().await?;
		
		// Initialize terminal emulator
		{
			let mut emulator = self.terminal_emulator.write().await;
			emulator.start_session(None).await?;
		}
		
		println!("✅ Sare Terminal Emulator initialized successfully!");
		
		Ok(())
	}
	
	/**
	 * Runs the terminal emulator
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn run(&mut self) -> Result<()> {
		println!("🎯 Starting Sare Terminal Emulator main loop...");
		
		let start_time = Instant::now();
		
		// Main terminal loop
		while *self.running.read().await {
			let loop_start = Instant::now();
			
			// Process input with error recovery
			let input_result = self.error_recovery.execute_with_recovery(
				|| self.process_input().await,
				"terminal",
				"process_input"
			).await;
			
			if let Err(e) = input_result {
				log_error!(self.logger, "terminal", "Input processing error: {}", e);
			}
			
			// Process output with error recovery
			let output_result = self.error_recovery.execute_with_recovery(
				|| self.process_output().await,
				"terminal",
				"process_output"
			).await;
			
			if let Err(e) = output_result {
				log_error!(self.logger, "terminal", "Output processing error: {}", e);
			}
			
			// Update UI with error recovery
			let ui_result = self.error_recovery.execute_with_recovery(
				|| self.update_ui().await,
				"terminal",
				"update_ui"
			).await;
			
			if let Err(e) = ui_result {
				log_error!(self.logger, "terminal", "UI update error: {}", e);
			}
			
			// Update terminal state
			self.update_terminal_state().await?;
			
			// Record performance metrics
			let loop_duration = loop_start.elapsed();
			self.profiler.record_sample(
				"main_loop".to_string(),
				"terminal".to_string(),
				loop_duration
			).await?;
			
			// Maintain 60 FPS (16.67ms per frame)
			if loop_duration < Duration::from_millis(16) {
				tokio::time::sleep(Duration::from_millis(16) - loop_duration).await;
			}
		}
		
		let total_runtime = start_time.elapsed();
		println!("🛑 Sare Terminal Emulator stopped after {:?}", total_runtime);
		
		Ok(())
	}
	
	/**
	 * Processes input from various sources
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn process_input(&self) -> Result<()> {
		// Process keyboard input
		self.process_keyboard_input().await?;
		
		// Process mouse input
		self.process_mouse_input().await?;
		
		// Process terminal input
		self.process_terminal_input().await?;
		
		// Process debug commands
		self.process_debug_commands().await?;
		
		Ok(())
	}
	
	/**
	 * Processes keyboard input
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn process_keyboard_input(&self) -> Result<()> {
		// Simulate keyboard input processing
		// In a real implementation, this would handle actual keyboard events
		
		let mut input_buffer = self.input_buffer.write().await;
		
		// Process special keys
		if input_buffer.contains(&0x1b) { // ESC
			self.handle_escape_sequence(&input_buffer).await?;
		}
		
		// Process regular input
		if !input_buffer.is_empty() {
			let input = input_buffer.clone();
			input_buffer.clear();
			
			// Send input to terminal emulator
			{
				let mut emulator = self.terminal_emulator.write().await;
				emulator.send_input(&input).await?;
			}
			
			// Add to history
			if let Ok(input_str) = String::from_utf8(input.clone()) {
				self.history.add_command(&input_str).await?;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Handles escape sequences
	 * 
	 * @param input - Input buffer
	 * @return Result<()> - Success or error status
	 */
	async fn handle_escape_sequence(&self, input: &[u8]) -> Result<()> {
		// Parse ANSI escape sequences
		let mut parser = crate::terminal::protocol::AnsiParser::new();
		let commands = parser.process_input(input)?;
		
		for command in commands {
			match command {
				crate::terminal::protocol::AnsiCommand::CursorUp(count) => {
					let mut state = self.terminal_state.write().await;
					state.cursor_pos.1 = state.cursor_pos.1.saturating_sub(count as u16);
				}
				crate::terminal::protocol::AnsiCommand::CursorDown(count) => {
					let mut state = self.terminal_state.write().await;
					state.cursor_pos.1 = (state.cursor_pos.1 + count as u16).min(state.size.1 - 1);
				}
				crate::terminal::protocol::AnsiCommand::CursorForward(count) => {
					let mut state = self.terminal_state.write().await;
					state.cursor_pos.0 = (state.cursor_pos.0 + count as u16).min(state.size.0 - 1);
				}
				crate::terminal::protocol::AnsiCommand::CursorBackward(count) => {
					let mut state = self.terminal_state.write().await;
					state.cursor_pos.0 = state.cursor_pos.0.saturating_sub(count as u16);
				}
				crate::terminal::protocol::AnsiCommand::CursorPosition(row, col) => {
					let mut state = self.terminal_state.write().await;
					state.cursor_pos = (col as u16, row as u16);
				}
				_ => {
					// Handle other commands
					log_debug!(self.logger, "terminal", "Unhandled ANSI command: {:?}", command);
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Processes mouse input
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn process_mouse_input(&self) -> Result<()> {
		// Simulate mouse input processing
		// In a real implementation, this would handle actual mouse events
		
		let state = self.terminal_state.read().await;
		if state.mode.mouse_tracking {
			// Handle mouse events
			log_debug!(self.logger, "terminal", "Mouse tracking enabled");
		}
		
		Ok(())
	}
	
	/**
	 * Processes terminal input
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn process_terminal_input(&self) -> Result<()> {
		// Read output from terminal emulator
		{
			let mut emulator = self.terminal_emulator.write().await;
			let output = emulator.read_output().await?;
			
			if !output.is_empty() {
				let mut output_buffer = self.output_buffer.write().await;
				output_buffer.extend(output);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Processes debug commands
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn process_debug_commands(&self) -> Result<()> {
		// Check for debug mode activation
		let state = self.terminal_state.read().await;
		if state.mode.debug_mode {
			// Process debug commands
			log_debug!(self.logger, "terminal", "Debug mode active");
		}
		
		Ok(())
	}
	
	/**
	 * Processes output and renders it
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn process_output(&self) -> Result<()> {
		let output_buffer = self.output_buffer.read().await;
		
		if !output_buffer.is_empty() {
			// Process output through terminal renderer
			{
				let mut emulator = self.terminal_emulator.write().await;
				let renderer = emulator.renderer_mut();
				renderer.process_input(&output_buffer).await?;
			}
			
			// Clear output buffer
			{
				let mut output_buffer = self.output_buffer.write().await;
				output_buffer.clear();
			}
		}
		
		Ok(())
	}
	
	/**
	 * Updates the UI
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn update_ui(&self) -> Result<()> {
		// Update UI manager
		if self.ui_manager.update().await? {
			// Render UI
			let ui_content = self.ui_manager.render().await?;
			print!("{}", ui_content);
		}
		
		Ok(())
	}
	
	/**
	 * Updates terminal state
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn update_terminal_state(&self) -> Result<()> {
		let mut state = self.terminal_state.write().await;
		state.last_update = Instant::now();
		
		// Update terminal size if needed
		{
			let emulator = self.terminal_emulator.read().await;
			let emulator_state = emulator.state();
			state.size = emulator_state.size;
			state.cursor_pos = emulator_state.cursor_pos;
		}
		
		Ok(())
	}
	
	/**
	 * Stops the terminal emulator
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn stop(&self) -> Result<()> {
		println!("🛑 Stopping Sare Terminal Emulator...");
		
		// Stop the main loop
		{
			let mut running = self.running.write().await;
			*running = false;
		}
		
		// Stop terminal emulator
		{
			let mut emulator = self.terminal_emulator.write().await;
			emulator.stop_session().await?;
		}
		
		// Generate final reports
		self.generate_final_reports().await?;
		
		Ok(())
	}
	
	/**
	 * Generates final reports
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn generate_final_reports(&self) -> Result<()> {
		// Generate profiling report
		let profiler_report = self.profiler.generate_report().await?;
		println!("📊 Profiling Report:\n{}", profiler_report);
		
		// Generate test report
		let test_stats = self.testing_framework.get_statistics().await;
		println!("🧪 Test Statistics: {:?}", test_stats);
		
		// Generate error recovery report
		let recovery_stats = self.error_recovery.get_statistics().await;
		println!("🛡️ Recovery Statistics: {:?}", recovery_stats);
		
		Ok(())
	}
	
	/**
	 * Gets the UI manager
	 * 
	 * @return &crate::ui::UiManager - UI manager reference
	 */
	pub fn ui_manager(&self) -> &crate::ui::UiManager {
		&self.ui_manager
	}
	
	/**
	 * Gets a mutable UI manager
	 * 
	 * @return &mut crate::ui::UiManager - Mutable UI manager reference
	 */
	pub fn ui_manager_mut(&mut self) -> &mut crate::ui::UiManager {
		&mut self.ui_manager
	}
	
	/**
	 * Gets the history manager
	 * 
	 * @return &crate::history::HistoryManager - History manager reference
	 */
	pub fn history_manager(&self) -> &crate::history::HistoryManager {
		&self.history
	}
	
	/**
	 * Gets a mutable history manager
	 * 
	 * @return &mut crate::history::HistoryManager - Mutable history manager reference
	 */
	pub fn history_manager_mut(&mut self) -> &mut crate::history::HistoryManager {
		&mut self.history
	}
	
	/**
	 * Gets the debug manager
	 * 
	 * @return &Arc<crate::debug::DebugManager> - Debug manager reference
	 */
	pub fn debug_manager(&self) -> &Arc<crate::debug::DebugManager> {
		&self.debug_manager
	}
	
	/**
	 * Gets the profiler
	 * 
	 * @return &Arc<crate::debug::Profiler> - Profiler reference
	 */
	pub fn profiler(&self) -> &Arc<crate::debug::Profiler> {
		&self.profiler
	}
	
	/**
	 * Gets the logger
	 * 
	 * @return &Arc<crate::debug::Logger> - Logger reference
	 */
	pub fn logger(&self) -> &Arc<crate::debug::Logger> {
		&self.logger
	}
	
	/**
	 * Gets the error recovery manager
	 * 
	 * @return &Arc<crate::debug::ErrorRecoveryManager> - Error recovery manager reference
	 */
	pub fn error_recovery(&self) -> &Arc<crate::debug::ErrorRecoveryManager> {
		&self.error_recovery
	}
	
	/**
	 * Gets the testing framework
	 * 
	 * @return &Arc<crate::debug::TestingFramework> - Testing framework reference
	 */
	pub fn testing_framework(&self) -> &Arc<crate::debug::TestingFramework> {
		&self.testing_framework
	}
} 