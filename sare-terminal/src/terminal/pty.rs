/**
 * @file pty.rs
 * @brief PTY (Pseudo-Terminal) implementation for Sare terminal
 * 
 * This module provides PTY creation and management capabilities,
 * enabling the Sare terminal to host external shells and processes
 * with proper terminal emulation and I/O handling.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file pty.rs
 * @description PTY module that provides pseudo-terminal creation
 * and management for hosting external shells in Sare.
 */

use anyhow::Result;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{PtySession, TerminalConfig};

/**
 * PTY manager for terminal emulation
 * 
 * Manages PTY creation, configuration, and lifecycle
 * for hosting external shells and processes.
 */
pub struct PtyManager {
	/// Current PTY session
	session: Option<Arc<RwLock<PtySession>>>,
	/// Terminal configuration
	config: TerminalConfig,
}

/**
 * PTY creation options
 * 
 * Defines options for creating a new PTY session
 * including terminal settings and process configuration.
 */
#[derive(Debug, Clone)]
pub struct PtyOptions {
	/// Terminal size (columns, rows)
	pub size: (u16, u16),
	/// Terminal type
	pub term_type: String,
	/// Environment variables to set
	pub environment: Vec<(String, String)>,
	/// Working directory
	pub working_directory: Option<String>,
	/// Command to run (None for default shell)
	pub command: Option<String>,
}

impl Default for PtyOptions {
	fn default() -> Self {
		Self {
			size: (80, 24),
			term_type: "xterm-256color".to_string(),
			environment: Vec::new(),
			working_directory: None,
			command: None,
		}
	}
}

impl PtyManager {
	/**
	 * Creates a new PTY manager
	 * 
	 * @param config - Terminal configuration
	 * @return PtyManager - New PTY manager instance
	 */
	pub fn new(config: TerminalConfig) -> Self {
		Self {
			session: None,
			config,
		}
	}
	
	/**
	 * Creates a new PTY session
	 * 
	 * Creates a new pseudo-terminal pair and configures
	 * it for hosting external shells or processes.
	 * 
	 * @param options - PTY creation options
	 * @return Result<PtySession> - New PTY session or error
	 */
	pub async fn create_session(&mut self, options: PtyOptions) -> Result<PtySession> {
		/**
		 * 新しいPTYセッションを作成する関数です
		 * 
		 * posix_openpt()を使用してマスターファイルディスクリプタを作成し、
		 * grantpt()とunlockpt()でスレーブ端末へのアクセスを設定します。
		 * 
		 * ptsname()でスレーブ端末のパスを取得し、open()でスレーブ
		 * ファイルディスクリプタを作成します。最後にPtyUtils::setup_slave_terminal()
		 * でスレーブ端末の初期化を行います。
		 * 
		 * 各ステップでエラーが発生した場合は適切にファイルディスクリプタを
		 * クローズしてエラーを返します。
		 */
		
		let master_fd = unsafe {
			use libc::{posix_openpt, O_RDWR, O_NOCTTY};
			let flags = O_RDWR | O_NOCTTY;
			let fd = posix_openpt(flags);
			if fd < 0 {
				return Err(anyhow::anyhow!("Failed to create PTY master"));
			}
			fd
		};
		
		unsafe {
			use libc::grantpt;
			if grantpt(master_fd) != 0 {
				use libc::close;
				close(master_fd);
				return Err(anyhow::anyhow!("Failed to grant PTY access"));
			}
		}
		
		unsafe {
			use libc::unlockpt;
			if unlockpt(master_fd) != 0 {
				use libc::close;
				close(master_fd);
				return Err(anyhow::anyhow!("Failed to unlock PTY"));
			}
		}
		
		let pty_path = unsafe {
			use libc::ptsname;
			let path_ptr = ptsname(master_fd);
			if path_ptr.is_null() {
				use libc::close;
				close(master_fd);
				return Err(anyhow::anyhow!("Failed to get PTY path"));
			}
			let path_str = std::ffi::CStr::from_ptr(path_ptr);
			path_str.to_string_lossy().to_string()
		};
		
		let slave_fd = unsafe {
			use libc::{open, O_RDWR, O_NOCTTY};
			let path = std::ffi::CString::new(pty_path.as_str())?;
			let flags = O_RDWR | O_NOCTTY;
			let fd = open(path.as_ptr(), flags);
			if fd < 0 {
				use libc::close;
				close(master_fd);
				return Err(anyhow::anyhow!("Failed to open PTY slave"));
			}
			fd
		};
		
		PtyUtils::setup_slave_terminal(slave_fd)?;
		
		let session = PtySession {
			master_fd,
			slave_fd,
			pty_path,
			size: options.size,
		};
		
		self.session = Some(Arc::new(RwLock::new(session.clone())));
		
		Ok(session)
	}
	
	/**
	 * Resizes the PTY session
	 * 
	 * Updates the terminal size and notifies the
	 * running process of the size change.
	 * 
	 * @param size - New terminal size (columns, rows)
	 * @return Result<()> - Success or error status
	 */
	pub async fn resize_session(&self, size: (u16, u16)) -> Result<()> {
		/**
		 * PTYセッションのサイズを変更する関数です
		 * 
		 * TIOCSWINSZ ioctlを使用してターミナルサイズを更新し、
		 * 実行中のプロセスにサイズ変更を通知します。
		 * 
		 * winsize構造体に新しい列数と行数を設定し、
		 * マスターとスレーブの両方のファイルディスクリプタに
		 * ioctlを実行してサイズ変更を反映します。
		 */
		
		if let Some(session) = &self.session {
			let mut session = session.write().await;
			session.size = size;
			
			// Implement actual resize with ioctl TIOCSWINSZ
			unsafe {
				use libc::{ioctl, TIOCSWINSZ, winsize};
				let mut ws = winsize {
					ws_row: size.1 as u16,
					ws_col: size.0 as u16,
					ws_xpixel: 0,
					ws_ypixel: 0,
				};
				
				if ioctl(session.master_fd, TIOCSWINSZ, &mut ws) < 0 {
					return Err(anyhow::anyhow!("Failed to resize PTY"));
				}
			}
			
			// Send SIGWINCH to notify the process of size change
			self.send_sigwinch_to_process(&session).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Sends SIGWINCH signal to the process group
	 * 
	 * @param session - PTY session
	 * @return Result<()> - Success or error status
	 */
	async fn send_sigwinch_to_process(&self, session: &PtySession) -> Result<()> {
		use std::process::Command;
		use std::os::unix::process::CommandExt;
		
		// Get the process group ID from the session
		let pgid = unsafe {
			use libc::{tcgetpgrp, getpgid};
			let pgid = tcgetpgrp(session.master_fd);
			if pgid < 0 {
				// Fallback to getting PGID from master FD
				getpgid(session.master_fd)
			} else {
				pgid
			}
		};
		
		if pgid > 0 {
			// Send SIGWINCH to the process group
			unsafe {
				use libc::{kill, SIGWINCH};
				if kill(-pgid, SIGWINCH) < 0 {
					// If sending to process group fails, try sending to individual processes
					self.send_sigwinch_to_individual_processes(session).await?;
				}
			}
		} else {
			// Fallback: send SIGWINCH to individual processes
			self.send_sigwinch_to_individual_processes(session).await?;
		}
		
		Ok(())
	}
	
	/**
	 * Sends SIGWINCH to individual processes in the session
	 * 
	 * @param session - PTY session
	 * @return Result<()> - Success or error status
	 */
	async fn send_sigwinch_to_individual_processes(&self, session: &PtySession) -> Result<()> {
		use std::process::Command;
		
		// Get list of processes using this PTY
		if let Ok(output) = Command::new("lsof")
			.args(&["-t", &format!("+D{}", session.pty_path)])
			.output() {
			if let Ok(process_list) = String::from_utf8(output.stdout) {
				for pid_str in process_list.lines() {
					if let Ok(pid) = pid_str.trim().parse::<i32>() {
						// Send SIGWINCH to each process
						unsafe {
							use libc::{kill, SIGWINCH};
							kill(pid, SIGWINCH);
						}
					}
				}
			}
		}
		
		// Alternative: use ps to find processes
		if let Ok(output) = Command::new("ps")
			.args(&["-o", "pid", "-t", &session.pty_path])
			.output() {
			if let Ok(process_list) = String::from_utf8(output.stdout) {
				for line in process_list.lines().skip(1) { // Skip header
					if let Ok(pid) = line.trim().parse::<i32>() {
						// Send SIGWINCH to each process
						unsafe {
							use libc::{kill, SIGWINCH};
							kill(pid, SIGWINCH);
						}
					}
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Writes data to the PTY
	 * 
	 * Sends input data to the PTY master for processing
	 * by the shell or running process.
	 * 
	 * @param data - Data to write
	 * @return Result<usize> - Number of bytes written or error
	 */
	pub async fn write_to_pty(&self, data: &[u8]) -> Result<usize> {
		if let Some(session) = &self.session {
			let session = session.read().await;
			
			// Implement actual writing to master file descriptor
			let bytes_written = unsafe {
				use libc::{write, c_void};
				let fd = session.master_fd;
				let ptr = data.as_ptr() as *const c_void;
				let len = data.len();
				write(fd, ptr, len)
			};
			
			if bytes_written < 0 {
				return Err(anyhow::anyhow!("Failed to write to PTY"));
		}
		
			Ok(bytes_written as usize)
		} else {
			Err(anyhow::anyhow!("No PTY session available"))
		}
	}
	
	/**
	 * Reads data from the PTY
	 * 
	 * Reads output data from the PTY master for
	 * processing and display.
	 * 
	 * @param buffer - Buffer to read into
	 * @return Result<usize> - Number of bytes read or error
	 */
	pub async fn read_from_pty(&self, buffer: &mut [u8]) -> Result<usize> {
		if let Some(session) = &self.session {
			let session = session.read().await;
			
			// Implement actual reading from master file descriptor
			let bytes_read = unsafe {
				use libc::{read, c_void};
				let fd = session.master_fd;
				let ptr = buffer.as_mut_ptr() as *mut c_void;
				let len = buffer.len();
				read(fd, ptr, len)
			};
			
			if bytes_read < 0 {
				return Err(anyhow::anyhow!("Failed to read from PTY"));
		}
		
			Ok(bytes_read as usize)
		} else {
			Err(anyhow::anyhow!("No PTY session available"))
		}
	}
	
	/**
	 * Gets the current session
	 * 
	 * @return Option<Arc<RwLock<PtySession>>> - Current session if available
	 */
	pub fn get_session(&self) -> Option<Arc<RwLock<PtySession>>> {
		self.session.clone()
	}
	
	/**
	 * Closes the current session
	 * 
	 * Terminates the PTY session and cleans up
	 * associated resources.
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn close_session(&mut self) -> Result<()> {
		if let Some(session) = &self.session {
			let session = session.read().await;
			
			// Implement session cleanup with proper file descriptor closing
			unsafe {
				use libc::close;
				
				// Close slave file descriptor
				if session.slave_fd >= 0 {
					close(session.slave_fd);
				}
				
				// Close master file descriptor
				if session.master_fd >= 0 {
					close(session.master_fd);
				}
			}
		}
		
		self.session = None;
		
		Ok(())
	}
	
	/**
	 * Gets the terminal configuration
	 * 
	 * @return &TerminalConfig - Terminal configuration
	 */
	pub fn config(&self) -> &TerminalConfig {
		&self.config
	}
}

/**
 * PTY utilities
 * 
 * Provides utility functions for PTY operations
 * including terminal setup and process management.
 */
pub struct PtyUtils;

impl PtyUtils {
	/**
	 * Sets up a slave terminal
	 * 
	 * Configures the slave terminal with appropriate
	 * attributes and settings for shell operation.
	 * 
	 * @param slave_fd - Slave file descriptor
	 * @return Result<()> - Success or error status
	 */
	pub fn setup_slave_terminal(slave_fd: RawFd) -> Result<()> {
		/**
		 * スレーブターミナルを設定する関数です
		 * 
		 * tcgetattr()で現在のターミナル属性を取得し、
		 * シェル操作に適した設定に変更します。
		 * 
		 * 入力モード（ICRNL無効化）、出力モード（OPOST無効化）、
		 * ローカルモード（ECHO、ICANON、ISIG無効化）を設定して
		 * 生モード（raw mode）にします。
		 * 
		 * tcsetattr()で変更された属性を適用し、エラー時は
		 * 適切なエラーメッセージを返します。
		 */
		
		// Implement slave terminal setup with actual terminal attributes
		unsafe {
			use libc::{tcgetattr, tcsetattr, TCSANOW, termios, ECHO, ICANON, ISIG, ICRNL, OPOST};
			
			// Get current terminal attributes
			let mut termios: termios = std::mem::zeroed();
			if tcgetattr(slave_fd, &mut termios) < 0 {
				return Err(anyhow::anyhow!("Failed to get terminal attributes"));
			}
			
			// Configure input modes
			termios.c_iflag &= !(ICRNL); // Don't translate CR to NL
			
			// Configure output modes
			termios.c_oflag &= !(OPOST); // Don't post-process output
			
			termios.c_lflag &= !(ECHO | ICANON | ISIG); // Raw mode
			
			if tcsetattr(slave_fd, TCSANOW, &termios) < 0 {
				return Err(anyhow::anyhow!("Failed to set terminal attributes"));
			}
		}
		
		Ok(())
	}
	
	/**
	 * Creates environment for PTY session
	 * 
	 * Sets up environment variables for the PTY session
	 * including terminal type and other necessary variables.
	 * 
	 * @param term_type - Terminal type
	 * @param size - Terminal size
	 * @return Vec<(String, String)> - Environment variables
	 */
	pub fn create_environment(term_type: &str, size: (u16, u16)) -> Vec<(String, String)> {
		let mut env = Vec::new();
		
		env.push(("TERM".to_string(), term_type.to_string()));
		
		env.push(("COLUMNS".to_string(), size.0.to_string()));
		env.push(("LINES".to_string(), size.1.to_string()));
		
		env.push(("TERM_PROGRAM".to_string(), "sare".to_string()));
		env.push(("TERM_PROGRAM_VERSION".to_string(), "0.1.0".to_string()));
		
		env
	}
	
	/**
	 * Gets the default shell
	 * 
	 * Determines the default shell to use based on
	 * system configuration and user preferences.
	 * 
	 * @return String - Default shell path
	 */
	pub fn get_default_shell() -> String {
		// Try to get shell from environment
		if let Ok(shell) = std::env::var("SHELL") {
			return shell;
		}
		
		let common_shells = ["/bin/bash", "/bin/zsh", "/bin/fish", "/bin/sh"];
		
		for shell in &common_shells {
			if std::path::Path::new(shell).exists() {
				return shell.to_string();
			}
		}
		
		"/bin/sh".to_string()
	}
} 