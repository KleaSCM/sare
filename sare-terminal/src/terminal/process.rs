/**
 * @file process.rs
 * @brief Process management for Sare terminal
 * 
 * This module provides process management capabilities for the Sare terminal,
 * including process creation, monitoring, signal handling, and job control
 * for both internal and external processes.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file process.rs
 * @description Process management module that provides process creation,
 * monitoring, and control capabilities for the Sare terminal.
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProcessInfo, ProcessStatus};

/**
 * Process manager for terminal
 * 
 * Manages processes running in the terminal including
 * process creation, monitoring, and signal handling.
 */
pub struct ProcessManager {
	/// Active processes
	processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
	/// Process groups
	process_groups: Arc<RwLock<HashMap<u32, ProcessGroup>>>,
	/// Foreground process group
	foreground_pgid: Option<u32>,
}

/**
 * Process group information
 * 
 * Contains information about a process group including
 * the group leader and member processes.
 */
#[derive(Debug, Clone)]
pub struct ProcessGroup {
	/// Process group ID
	pub pgid: u32,
	/// Group leader process ID
	pub leader_pid: u32,
	/// Member process IDs
	pub member_pids: Vec<u32>,
	/// Group state
	pub state: GroupState,
}

/**
 * Process group state enumeration
 * 
 * Defines the different states a process group can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum GroupState {
	/// Group is running
	Running,
	/// Group is stopped
	Stopped,
	/// Group has terminated
	Terminated,
}

/**
 * Process creation options
 * 
 * Defines options for creating a new process
 * including command, arguments, and environment.
 */
#[derive(Debug, Clone)]
pub struct ProcessOptions {
	/// Command to execute
	pub command: String,
	/// Command arguments
	pub args: Vec<String>,
	/// Environment variables
	pub environment: HashMap<String, String>,
	/// Working directory
	pub working_directory: Option<String>,
	/// Process group ID (None for new group)
	pub pgid: Option<u32>,
	/// Whether to run in foreground
	pub foreground: bool,
}

impl Default for ProcessOptions {
	fn default() -> Self {
		Self {
			command: String::new(),
			args: Vec::new(),
			environment: HashMap::new(),
			working_directory: None,
			pgid: None,
			foreground: true,
		}
	}
}

impl ProcessManager {
	/**
	 * Creates a new process manager
	 * 
	 * @return ProcessManager - New process manager instance
	 */
	pub fn new() -> Self {
		Self {
			processes: Arc::new(RwLock::new(HashMap::new())),
			process_groups: Arc::new(RwLock::new(HashMap::new())),
			foreground_pgid: None,
		}
	}
	
	/**
	 * Creates a new process
	 * 
	 * Spawns a new process with the specified options and
	 * adds it to the process manager for monitoring.
	 * 
	 * @param options - Process creation options
	 * @return Result<u32> - Process ID or error
	 */
	pub async fn create_process(&mut self, options: ProcessOptions) -> Result<u32> {
		/**
		 * プロセスを作成する関数です
		 * 
		 * 指定されたコマンドとオプションに基づいて新しいプロセスを
		 * 作成し、プロセスグループとファイルディスクリプタを設定します。
		 * 
		 * fork()とexecvp()を使用してプロセスを作成し、環境変数と
		 * 作業ディレクトリを適切に設定します
		 */
		
		let pid = unsafe {
			use libc::{fork, execvp, setpgid, dup2, close, chdir};
			
			let pid = fork();
			if pid < 0 {
				return Err(anyhow::anyhow!("Failed to fork process"));
			}
			
			if pid == 0 {
				
				if let Some(pgid) = options.pgid {
					if setpgid(0, pgid as i32) < 0 {
						if setpgid(0, 0) < 0 {
							return Err(anyhow::anyhow!("Failed to set process group"));
						}
					}
				} else {
					if setpgid(0, 0) < 0 {
						return Err(anyhow::anyhow!("Failed to create process group"));
					}
				}
				
				// Set up file descriptors if provided
				// (This would be integrated with PTY system)
				
				// Change working directory if specified
				if let Some(ref dir) = options.working_directory {
					let dir_cstr = std::ffi::CString::new(dir.as_str())?;
					if chdir(dir_cstr.as_ptr()) < 0 {
						return Err(anyhow::anyhow!("Failed to change directory"));
					}
				}
				
				for (key, value) in &options.environment {
					std::env::set_var(key, value);
				}
				
				let cmd_cstr = std::ffi::CString::new(options.command.as_str())?;
				let mut args = vec![cmd_cstr.as_ptr()];
				
				for arg in &options.args {
					let arg_cstr = std::ffi::CString::new(arg.as_str())?;
					args.push(arg_cstr.as_ptr());
				}
				args.push(std::ptr::null());
				
				execvp(cmd_cstr.as_ptr(), args.as_ptr());
				
				std::process::exit(1);
			} else {
				pid as u32
			}
		};
		
		let process_info = ProcessInfo {
			pid,
			name: options.command.clone(),
			command: format!("{} {}", options.command, options.args.join(" ")),
			status: ProcessStatus::Running,
		};
		
		let mut processes = self.processes.write().await;
		processes.insert(pid, process_info);
		
		if options.foreground {
			self.foreground_pgid = Some(pid);
		}
		
		Ok(pid)
	}
	
	/**
	 * Gets process information
	 * 
	 * @param pid - Process ID
	 * @return Option<ProcessInfo> - Process information if found
	 */
	pub async fn get_process(&self, pid: u32) -> Option<ProcessInfo> {
		let processes = self.processes.read().await;
		processes.get(&pid).cloned()
	}
	
	/**
	 * Lists all processes
	 * 
	 * @return Vec<ProcessInfo> - List of all processes
	 */
	pub async fn list_processes(&self) -> Vec<ProcessInfo> {
		let processes = self.processes.read().await;
		processes.values().cloned().collect()
	}
	
	/**
	 * Sends a signal to a process
	 * 
	 * @param pid - Process ID
	 * @param signal - Signal to send
	 * @return Result<()> - Success or error status
	 */
	pub async fn send_signal(&self, pid: u32, signal: i32) -> Result<()> {
		/**
		 * プロセスにシグナルを送信する関数です
		 * 
		 * 指定されたプロセスIDにシグナルを送信し、
		 * プロセスの状態を適切に更新します。
		 * 
		 * kill()システムコールを使用してシグナルを送信し、
		 * プロセスステータスをシグナルに応じて変更します
		 */
		
		// Implement actual signal sending with kill() system call
		unsafe {
			use libc::kill;
			
			if kill(pid as i32, signal) < 0 {
				return Err(anyhow::anyhow!("Failed to send signal {} to process {}", signal, pid));
			}
		}
		
		// Update process status based on signal
		let mut processes = self.processes.write().await;
		if let Some(process) = processes.get_mut(&pid) {
			match signal {
				libc::SIGTERM | libc::SIGKILL => {
					process.status = ProcessStatus::Terminated(0);
				}
				libc::SIGSTOP | libc::SIGTSTP => {
					process.status = ProcessStatus::Suspended;
				}
				libc::SIGCONT => {
					process.status = ProcessStatus::Running;
				}
				_ => {}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Terminates a process
	 * 
	 * @param pid - Process ID to terminate
	 * @return Result<()> - Success or error status
	 */
	pub async fn terminate_process(&mut self, pid: u32) -> Result<()> {
		// Send SIGTERM first
		self.send_signal(pid, libc::SIGTERM).await?;
		
		// Implement graceful termination waiting
		unsafe {
			use libc::{waitpid, WNOHANG, WIFEXITED, WIFSIGNALED};
			
			let mut status = 0;
			let mut attempts = 0;
			const MAX_ATTEMPTS: i32 = 10; // 10 seconds timeout
			
			while attempts < MAX_ATTEMPTS {
				let result = waitpid(pid as i32, &mut status, WNOHANG);
				
				if result == pid as i32 {
					if WIFEXITED(status) || WIFSIGNALED(status) {
						break;
					}
				} else if result < 0 {
					break;
				}
				
				std::thread::sleep(std::time::Duration::from_millis(100));
				attempts += 1;
			}
			
			if attempts >= MAX_ATTEMPTS {
				self.send_signal(pid, libc::SIGKILL).await?;
				
				std::thread::sleep(std::time::Duration::from_millis(500));
				waitpid(pid as i32, &mut status, WNOHANG);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Kills a process
	 * 
	 * @param pid - Process ID to kill
	 * @return Result<()> - Success or error status
	 */
	pub async fn kill_process(&mut self, pid: u32) -> Result<()> {
		self.send_signal(pid, libc::SIGKILL).await?;
		
		let mut processes = self.processes.write().await;
		processes.remove(&pid);
		
		Ok(())
	}
	
	/**
	 * Suspends a process
	 * 
	 * @param pid - Process ID to suspend
	 * @return Result<()> - Success or error status
	 */
	pub async fn suspend_process(&mut self, pid: u32) -> Result<()> {
		self.send_signal(pid, libc::SIGTSTP).await?;
		
		if let Some(process) = self.get_process(pid).await {
			let mut processes = self.processes.write().await;
			if let Some(process_info) = processes.get_mut(&pid) {
				process_info.status = ProcessStatus::Stopped;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Resumes a process
	 * 
	 * @param pid - Process ID to resume
	 * @return Result<()> - Success or error status
	 */
	pub async fn resume_process(&mut self, pid: u32) -> Result<()> {
		self.send_signal(pid, libc::SIGCONT).await?;
		
		if let Some(process) = self.get_process(pid).await {
			let mut processes = self.processes.write().await;
			if let Some(process_info) = processes.get_mut(&pid) {
				process_info.status = ProcessStatus::Running;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets the foreground process group
	 * 
	 * @return Option<u32> - Foreground process group ID
	 */
	pub fn foreground_pgid(&self) -> Option<u32> {
		self.foreground_pgid
	}
	
	/**
	 * Sets the foreground process group
	 * 
	 * @param pgid - Process group ID to set as foreground
	 * @return Result<()> - Success or error status
	 */
	pub async fn set_foreground_pgid(&mut self, pgid: u32) -> Result<()> {
		/**
		 * フォアグラウンドプロセスグループを設定する関数です
		 * 
		 * 指定されたプロセスグループをフォアグラウンドに設定し、
		 * ターミナル制御を適切に管理します。
		 * 
		 * tcsetpgrp()システムコールを使用してフォアグラウンド
		 * プロセスグループを設定し、内部状態を更新します
		 */
		
		// Implement actual foreground group setting with tcsetpgrp()
		unsafe {
			use libc::tcsetpgrp;
			
			if tcsetpgrp(0, pgid as i32) < 0 {
				return Err(anyhow::anyhow!("Failed to set foreground process group"));
			}
		}
		
		self.foreground_pgid = Some(pgid);
		
		Ok(())
	}
	
	/**
	 * Creates a new process group
	 * 
	 * @param leader_pid - Process group leader PID
	 * @return Result<u32> - Process group ID or error
	 */
	pub async fn create_process_group(&mut self, leader_pid: u32) -> Result<u32> {
		/**
		 * プロセスグループを作成する関数です
		 * 
		 * 指定されたリーダープロセスIDを使用して新しい
		 * プロセスグループを作成し、グループ管理を設定します。
		 * 
		 * setpgid()システムコールを使用してプロセスグループを作成し、
		 * グループ情報を内部管理システムに追加します
		 */
		
		// Implement actual process group creation with setpgid()
		unsafe {
			use libc::setpgid;
			
			// Create new process group with leader_pid as the group leader
			if setpgid(leader_pid as i32, leader_pid as i32) < 0 {
				return Err(anyhow::anyhow!("Failed to create process group"));
			}
		}
		
		let pgid = leader_pid; // Use leader PID as group ID
		
		let process_group = ProcessGroup {
			pgid,
			leader_pid,
			member_pids: vec![leader_pid],
			state: GroupState::Running,
		};
		
		let mut groups = self.process_groups.write().await;
		groups.insert(pgid, process_group);
		
		Ok(pgid)
	}
	
	/**
	 * Gets process group information
	 * 
	 * @param pgid - Process group ID
	 * @return Option<ProcessGroup> - Process group if found
	 */
	pub async fn get_process_group(&self, pgid: u32) -> Option<ProcessGroup> {
		let groups = self.process_groups.read().await;
		groups.get(&pgid).cloned()
	}
	
	/**
	 * Lists all process groups
	 * 
	 * @return Vec<ProcessGroup> - List of all process groups
	 */
	pub async fn list_process_groups(&self) -> Vec<ProcessGroup> {
		let groups = self.process_groups.read().await;
		groups.values().cloned().collect()
	}
} 