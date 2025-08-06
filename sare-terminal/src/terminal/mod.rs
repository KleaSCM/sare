/**
 * @file mod.rs
 * @brief Terminal emulator module for Sare
 * 
 * This module provides PTY (Pseudo-Terminal) capabilities for the Sare terminal,
 * enabling it to host external shells and provide a full terminal emulator experience
 * with proper process management, I/O redirection, and signal handling.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description Terminal emulator module that provides PTY capabilities
 * for hosting external shells in the Sare terminal.
 */

pub mod pty;
pub mod shell;
pub mod process;
pub mod io;
pub mod protocol;
pub mod renderer;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use protocol::AnsiParser;
use renderer::{TerminalRenderer, RendererConfig};
use crate::session::SessionSystem;
use crate::features::TerminalFeatures;

/**
 * Terminal emulator configuration
 * 
 * Configuration options for the terminal emulator including
 * shell preferences, PTY settings, and display options.
 */
#[derive(Debug, Clone)]
pub struct TerminalConfig {
	/// Default shell to use
	pub default_shell: String,
	/// Terminal type (e.g., "xterm-256color")
	pub term_type: String,
	/// Terminal size (columns, rows)
	pub size: (u16, u16),
	/// Enable color support
	pub color_support: bool,
	/// Enable mouse support
	pub mouse_support: bool,
	/// Enable bracketed paste mode
	pub bracketed_paste: bool,
}

impl Default for TerminalConfig {
	fn default() -> Self {
		Self {
			default_shell: "/bin/bash".to_string(),
			term_type: "xterm-256color".to_string(),
			size: (80, 24),
			color_support: true,
			mouse_support: true,
			bracketed_paste: true,
		}
	}
}

/**
 * Terminal emulator instance
 * 
 * Manages a single terminal session with PTY capabilities,
 * process management, and I/O handling for external shells.
 */
pub struct TerminalEmulator {
	/// Terminal configuration
	config: TerminalConfig,
	/// Current PTY session
	pty_session: Option<Arc<RwLock<PtySession>>>,
	/// Active processes
	processes: Arc<RwLock<Vec<ProcessInfo>>>,
	/// Terminal state
	state: TerminalState,
	/// ANSI parser for escape sequence processing
	ansi_parser: AnsiParser,
	/// Terminal renderer for content display
	renderer: TerminalRenderer,
	/// Session management system
	session_system: SessionSystem,
	/// Terminal features system
	terminal_features: TerminalFeatures,
}

/**
 * PTY session information
 * 
 * Contains information about an active PTY session
 * including the master and slave file descriptors.
 */
#[derive(Debug, Clone)]
pub struct PtySession {
	/// Master PTY file descriptor
	pub master_fd: i32,
	/// Slave PTY file descriptor
	pub slave_fd: i32,
	/// PTY device path
	pub pty_path: String,
	/// Terminal size
	pub size: (u16, u16),
}

/**
 * Process information
 * 
 * Contains information about processes running
 * in the terminal session.
 */
#[derive(Debug, Clone)]
pub struct ProcessInfo {
	/// Process ID
	pub pid: u32,
	/// Process name
	pub name: String,
	/// Process command line
	pub command: String,
	/// Process status
	pub status: ProcessStatus,
}

/**
 * Process status enumeration
 * 
 * Defines the different states a process can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
	/// Process is running
	Running,
	/// Process is stopped
	Stopped,
	/// Process has terminated
	Terminated(i32),
	/// Process is suspended
	Suspended,
	/// Process has exited
	Exited(i32),
	/// Process has been killed
	Killed(i32),
}

/**
 * Terminal state information
 * 
 * Contains the current state of the terminal
 * including cursor position, selection, and mode.
 */
#[derive(Debug, Clone)]
pub struct TerminalState {
	/// Cursor position (column, row)
	pub cursor_pos: (u16, u16),
	/// Terminal size (columns, rows)
	pub size: (u16, u16),
	/// Scroll position
	pub scroll_pos: u32,
	/// Selection start position
	pub selection_start: Option<(u16, u16)>,
	/// Selection end position
	pub selection_end: Option<(u16, u16)>,
	/// Terminal mode flags
	pub mode_flags: TerminalMode,
}

/**
 * Terminal mode flags
 * 
 * Defines various terminal mode settings
 * that control terminal behavior.
 */
#[derive(Debug, Clone)]
pub struct TerminalMode {
	/// Insert mode enabled
	pub insert_mode: bool,
	/// Application cursor keys enabled
	pub app_cursor_keys: bool,
	/// Application keypad enabled
	pub app_keypad: bool,
	/// Mouse tracking enabled
	pub mouse_tracking: bool,
	/// Bracketed paste mode enabled
	pub bracketed_paste: bool,
}

impl Default for TerminalMode {
	fn default() -> Self {
		Self {
			insert_mode: false,
			app_cursor_keys: false,
			app_keypad: false,
			mouse_tracking: false,
			bracketed_paste: false,
		}
	}
}

impl TerminalEmulator {
	/**
	 * Creates a new terminal emulator instance
	 * 
	 * Initializes the terminal emulator with the specified configuration
	 * and sets up the PTY session management.
	 * 
	 * @param config - Terminal configuration
	 * @return Result<TerminalEmulator> - New terminal emulator instance or error
	 */
	pub fn new(config: TerminalConfig) -> Result<Self> {
		/**
		 * ターミナルエミュレーターを初期化する関数です
		 * 
		 * 指定された設定を使用してターミナルエミュレーターを初期化し、
		 * プロセス管理、PTYセッション、ターミナル状態を設定します。
		 * 
		 * プロセスリストを空のベクターで初期化し、ターミナル状態を
		 * デフォルト値（カーソル位置(0,0)、指定サイズ、スクロール位置0）
		 * で設定します。
		 * 
		 * PTYセッションは後でstart_session()で作成されます。
		 */
		
		let renderer_config = RendererConfig {
			size: config.size,
			max_scrollback: 1000,
			color_support: config.color_support,
			mouse_support: config.mouse_support,
			default_fg_color: protocol::Color::default(),
			default_bg_color: protocol::Color { r: 0, g: 0, b: 0, color_type: protocol::ColorType::Default },
		};
		let session_system = SessionSystem::new()?;
		let terminal_features = TerminalFeatures::new()?;
		
		Ok(Self {
			config: config.clone(),
			pty_session: None,
			processes: Arc::new(RwLock::new(Vec::new())),
			state: TerminalState {
				cursor_pos: (0, 0),
				size: config.size.clone(),
				scroll_pos: 0,
				selection_start: None,
				selection_end: None,
				mode_flags: TerminalMode::default(),
			},
			ansi_parser: AnsiParser::new(),
			renderer: TerminalRenderer::new(renderer_config),
			session_system,
			terminal_features,
		})
	}
	
	/**
	 * Starts a new PTY session
	 * 
	 * Creates a new pseudo-terminal session and launches
	 * the specified shell or command in it.
	 * 
	 * @param command - Command to run (defaults to shell if None)
	 * @return Result<()> - Success or error status
	 */
	pub async fn start_session(&mut self, command: Option<&str>) -> Result<()> {
		/**
		 * 新しいPTYセッションを開始する関数です
		 * 
		 * PtyManagerを使用してPTYセッションを作成し、指定された
		 * コマンドまたはデフォルトシェルを起動します。
		 * 
		 * ターミナルサイズ、端末タイプ、環境変数を設定して
		 * PtyOptionsを作成し、PtyManager::create_session()で
		 * セッションを開始します。
		 * 
		 * 作成されたセッションを内部に保存し、後でコマンドまたは
		 * シェルの起動を実装する予定です。
		 */
		
		use crate::terminal::pty::{PtyManager, PtyOptions};
		let mut pty_manager = PtyManager::new(self.config.clone());
		let options = PtyOptions {
			size: self.state.size,
			term_type: self.config.term_type.clone(),
			environment: Vec::new(),
			working_directory: None,
			command: command.map(|s| s.to_string()),
		};
		let session = pty_manager.create_session(options).await?;
		self.pty_session = Some(Arc::new(RwLock::new(session)));
		if let Some(cmd) = command {
			// Launch specific command
			self.launch_command(cmd).await?;
		} else {
			// Launch default shell
			self.launch_shell().await?;
		}
		
		Ok(())
	}
	
	/**
	 * Launches a specific command in the PTY session
	 * 
	 * @param command - Command to launch
	 * @return Result<()> - Success or error status
	 */
	async fn launch_command(&mut self, command: &str) -> Result<()> {
		use std::process::Command;
		use std::os::unix::io::{AsRawFd, FromRawFd};
		
		if let Some(pty_session) = &self.pty_session {
			let session = pty_session.read().await;
			
			// Parse command and arguments
			let parts: Vec<&str> = command.split_whitespace().collect();
			if parts.is_empty() {
				return Err(anyhow::anyhow!("Empty command"));
			}
			
			let (cmd, args) = (parts[0], &parts[1..]);
			
			// Create command with PTY slave as stdin/stdout/stderr
			let mut child = Command::new(cmd);
			child.args(args);
			
			// Set up PTY slave file descriptor
			unsafe {
				use libc::{dup2, close};
				let slave_fd = session.slave_fd;
				
				// Redirect stdin, stdout, stderr to PTY slave
				dup2(slave_fd, 0); // stdin
				dup2(slave_fd, 1); // stdout
				dup2(slave_fd, 2); // stderr
				
				// Close the original slave fd since we've duplicated it
				close(slave_fd);
			}
			
			// Set environment variables
			child.env("TERM", &self.config.term_type);
			child.env("COLUMNS", self.state.size.0.to_string());
			child.env("LINES", self.state.size.1.to_string());
			
			// Spawn the process
			match child.spawn() {
				Ok(child) => {
					// Add process to tracking
					let mut processes = self.processes.write().await;
					processes.push(ProcessInfo {
						pid: child.id(),
						name: cmd.to_string(),
						command: command.to_string(),
						status: ProcessStatus::Running,
					});
				}
				Err(e) => {
					return Err(anyhow::anyhow!("Failed to spawn command: {}", e));
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Launches the default shell in the PTY session
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn launch_shell(&mut self) -> Result<()> {
		use std::process::Command;
		use std::os::unix::io::{AsRawFd, FromRawFd};
		
		if let Some(pty_session) = &self.pty_session {
			let session = pty_session.read().await;
			
			// Get default shell
			let shell = &self.config.default_shell;
			
			// Create command with PTY slave as stdin/stdout/stderr
			let mut child = Command::new(shell);
			
			// Set up PTY slave file descriptor
			unsafe {
				use libc::{dup2, close};
				let slave_fd = session.slave_fd;
				
				// Redirect stdin, stdout, stderr to PTY slave
				dup2(slave_fd, 0); // stdin
				dup2(slave_fd, 1); // stdout
				dup2(slave_fd, 2); // stderr
				
				// Close the original slave fd since we've duplicated it
				close(slave_fd);
			}
			
			// Set environment variables
			child.env("TERM", &self.config.term_type);
			child.env("COLUMNS", self.state.size.0.to_string());
			child.env("LINES", self.state.size.1.to_string());
			
			// Spawn the shell
			match child.spawn() {
				Ok(child) => {
					// Add process to tracking
					let mut processes = self.processes.write().await;
					processes.push(ProcessInfo {
						pid: child.id(),
						name: shell.clone(),
						command: shell.clone(),
						status: ProcessStatus::Running,
					});
				}
				Err(e) => {
					return Err(anyhow::anyhow!("Failed to spawn shell: {}", e));
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Resizes the terminal
	 * 
	 * Updates the terminal size and notifies the PTY
	 * session of the size change.
	 * 
	 * @param columns - Number of columns
	 * @param rows - Number of rows
	 * @return Result<()> - Success or error status
	 */
	pub async fn resize(&mut self, columns: u16, rows: u16) -> Result<()> {
		self.state.size = (columns, rows);
		self.renderer.resize(columns, rows);
		if let Some(pty_session) = &self.pty_session {
			use crate::terminal::pty::PtyManager;
			let pty_manager = PtyManager::new(self.config.clone());
			pty_manager.resize_session((columns, rows)).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Sends input to the terminal
	 * 
	 * Processes input and sends it to the PTY session
	 * for handling by the shell or running process.
	 * 
	 * @param input - Input data to send
	 * @return Result<()> - Success or error status
	 */
	pub async fn send_input(&mut self, input: &[u8]) -> Result<()> {
		self.renderer.process_input(input)?;
		if let Some(pty_session) = &self.pty_session {
			use crate::terminal::pty::PtyManager;
			let pty_manager = PtyManager::new(self.config.clone());
			pty_manager.write_to_pty(input).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Reads output from the terminal
	 * 
	 * Reads output data from the PTY session and
	 * processes it for display.
	 * 
	 * @return Result<Vec<u8>> - Output data or error
	 */
	pub async fn read_output(&mut self) -> Result<Vec<u8>> {
		if let Some(pty_session) = &self.pty_session {
			// Read output from PTY master
			use crate::terminal::pty::PtyManager;
			
			let pty_manager = PtyManager::new(self.config.clone());
			let mut buffer = vec![0u8; 4096]; // 4KB buffer
			let bytes_read = pty_manager.read_from_pty(&mut buffer).await?;
			
			// Process output through ANSI parser and renderer
			let output_data = buffer[..bytes_read].to_vec();
			self.renderer.process_input(&output_data)?;
			
			return Ok(output_data);
		}
		
		Ok(Vec::new())
	}
	
	/**
	 * Gets the current terminal state
	 * 
	 * @return &TerminalState - Current terminal state
	 */
	pub fn state(&self) -> &TerminalState {
		&self.state
	}
	
	/**
	 * Gets the terminal configuration
	 * 
	 * @return &TerminalConfig - Terminal configuration
	 */
	pub fn config(&self) -> &TerminalConfig {
		&self.config
	}
	
	/**
	 * Gets the terminal renderer
	 * 
	 * @return &TerminalRenderer - Terminal renderer
	 */
	pub fn renderer(&self) -> &TerminalRenderer {
		&self.renderer
	}
	
	/**
	 * Gets the terminal renderer (mutable)
	 * 
	 * @return &mut TerminalRenderer - Terminal renderer
	 */
	pub fn renderer_mut(&mut self) -> &mut TerminalRenderer {
		&mut self.renderer
	}
	
	/**
	 * Gets active processes
	 * 
	 * @return Vec<ProcessInfo> - List of active processes
	 */
	pub async fn get_processes(&self) -> Vec<ProcessInfo> {
		self.processes.read().await.clone()
	}
	
	/**
	 * Stops the current session
	 * 
	 * Terminates the current PTY session and cleans up
	 * associated processes and resources.
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn stop_session(&mut self) -> Result<()> {
		if let Some(pty_session) = &self.pty_session {
			use crate::terminal::pty::PtyManager;
			
			let mut pty_manager = PtyManager::new(self.config.clone());
			pty_manager.close_session().await?;
		}
		
		self.pty_session = None;
		
		Ok(())
	}
} 