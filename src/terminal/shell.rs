/**
 * @file shell.rs
 * @brief External shell integration for Sare terminal
 * 
 * This module provides integration with external shells (bash, zsh, fish, etc.)
 * through PTY sessions, enabling Sare to host any shell with full functionality
 * including command history, completion, and shell-specific features.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file shell.rs
 * @description Shell integration module that provides external shell
 * hosting capabilities for the Sare terminal.
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProcessInfo, ProcessStatus};

/**
 * Shell integration manager
 * 
 * Manages external shell sessions and provides
 * integration with various shell features.
 */
pub struct ShellManager {
	/// Active shell sessions
	sessions: Arc<RwLock<HashMap<String, ShellSession>>>,
	/// Shell configurations
	configurations: HashMap<String, ShellConfig>,
	/// Default shell
	default_shell: String,
}

/**
 * Shell session information
 * 
 * Contains information about an active shell session
 * including process details and session state.
 */
#[derive(Debug, Clone)]
pub struct ShellSession {
	/// Session ID
	pub session_id: String,
	/// Shell type (bash, zsh, fish, etc.)
	pub shell_type: String,
	/// Process information
	pub process: ProcessInfo,
	/// Session state
	pub state: SessionState,
	/// Working directory
	pub working_directory: String,
}

/**
 * Session state enumeration
 * 
 * Defines the different states a shell session can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
	/// Session is active and ready
	Active,
	/// Session is starting up
	Starting,
	/// Session is shutting down
	ShuttingDown,
	/// Session has terminated
	Terminated(i32),
}

/**
 * Shell configuration
 * 
 * Contains configuration options for different shell types
 * including startup commands and environment setup.
 */
#[derive(Debug, Clone)]
pub struct ShellConfig {
	/// Shell executable path
	pub executable: String,
	/// Shell name
	pub name: String,
	/// Startup commands
	pub startup_commands: Vec<String>,
	/// Environment variables
	pub environment: HashMap<String, String>,
	/// Shell-specific features
	pub features: ShellFeatures,
}

/**
 * Shell features
 * 
 * Defines features supported by different shell types
 * including completion, history, and syntax highlighting.
 */
#[derive(Debug, Clone)]
pub struct ShellFeatures {
	/// Command completion support
	pub completion: bool,
	/// Command history support
	pub history: bool,
	/// Syntax highlighting support
	pub syntax_highlighting: bool,
	/// Directory stack support
	pub directory_stack: bool,
	/// Job control support
	pub job_control: bool,
}

impl Default for ShellFeatures {
	fn default() -> Self {
		Self {
			completion: true,
			history: true,
			syntax_highlighting: false,
			directory_stack: true,
			job_control: true,
		}
	}
}

impl ShellManager {
	/**
	 * Creates a new shell manager
	 * 
	 * @param default_shell - Default shell to use
	 * @return ShellManager - New shell manager instance
	 */
	pub fn new(default_shell: String) -> Self {
		/**
		 * シェルマネージャーを初期化する関数です
		 * 
		 * 複数のシェルタイプ（bash、zsh、fish）の設定を管理し、
		 * 各シェルの実行可能ファイルと機能を設定します。
		 * 
		 * 各シェルの設定、環境変数、起動コマンドを定義して
		 * シェルセッション管理を準備します
		 */
		
		let mut configurations = HashMap::new();
		
		// Configure bash
		configurations.insert("bash".to_string(), ShellConfig {
			executable: "/bin/bash".to_string(),
			name: "Bash".to_string(),
			startup_commands: vec![
				"export HISTSIZE=10000".to_string(),
				"export HISTFILESIZE=20000".to_string(),
			],
			environment: HashMap::new(),
			features: ShellFeatures {
				completion: true,
				history: true,
				syntax_highlighting: false,
				directory_stack: true,
				job_control: true,
			},
		});
		
		// Configure zsh
		configurations.insert("zsh".to_string(), ShellConfig {
			executable: "/bin/zsh".to_string(),
			name: "Zsh".to_string(),
			startup_commands: vec![
				"export HISTSIZE=10000".to_string(),
				"export SAVEHIST=10000".to_string(),
			],
			environment: HashMap::new(),
			features: ShellFeatures {
				completion: true,
				history: true,
				syntax_highlighting: true,
				directory_stack: true,
				job_control: true,
			},
		});
		
		// Configure fish
		configurations.insert("fish".to_string(), ShellConfig {
			executable: "/bin/fish".to_string(),
			name: "Fish".to_string(),
			startup_commands: vec![
				"set -g fish_history_size 10000".to_string(),
			],
			environment: HashMap::new(),
			features: ShellFeatures {
				completion: true,
				history: true,
				syntax_highlighting: true,
				directory_stack: true,
				job_control: true,
			},
		});
		
		Self {
			sessions: Arc::new(RwLock::new(HashMap::new())),
			configurations,
			default_shell,
		}
	}
	
	/**
	 * Creates a new shell session
	 * 
	 * Starts a new shell session with the specified shell type
	 * and configuration options.
	 * 
	 * @param shell_type - Type of shell to start
	 * @param working_directory - Working directory for the session
	 * @return Result<String> - Session ID or error
	 */
	pub async fn create_session(&mut self, shell_type: &str, working_directory: &str) -> Result<String> {
		/**
		 * シェルセッションを作成する関数です
		 * 
		 * 指定されたシェルタイプと作業ディレクトリを使用して
		 * 新しいシェルセッションを開始します。
		 * 
		 * シェル設定を取得し、プロセスマネージャーを使用して
		 * シェルプロセスを作成してセッションを初期化します
		 */
		
		let session_id = uuid::Uuid::new_v4().to_string();
		
		// Get shell configuration
		let config = self.configurations.get(shell_type)
			.ok_or_else(|| anyhow::anyhow!("Unknown shell type: {}", shell_type))?;
		
		// Create process info with actual process ID
		use crate::terminal::process::{ProcessManager, ProcessOptions};
		
		let mut process_manager = ProcessManager::new();
		let options = ProcessOptions {
			command: config.executable.clone(),
			args: Vec::new(),
			environment: config.environment.clone(),
			working_directory: Some(working_directory.to_string()),
			pgid: None,
			foreground: true,
		};
		let pid = process_manager.create_process(options).await?;
		
		let process = ProcessInfo {
			pid,
			name: config.name.clone(),
			command: config.executable.clone(),
			status: ProcessStatus::Running,
		};
		
		// Create session
		let session = ShellSession {
			session_id: session_id.clone(),
			shell_type: shell_type.to_string(),
			process,
			state: SessionState::Starting,
			working_directory: working_directory.to_string(),
		};
		
		// Add session to manager
		let mut sessions = self.sessions.write().await;
		sessions.insert(session_id.clone(), session);
		
		Ok(session_id)
	}
	
	/**
	 * Gets a shell session
	 * 
	 * @param session_id - Session ID
	 * @return Option<ShellSession> - Session if found
	 */
	pub async fn get_session(&self, session_id: &str) -> Option<ShellSession> {
		let sessions = self.sessions.read().await;
		sessions.get(session_id).cloned()
	}
	
	/**
	 * Lists all active sessions
	 * 
	 * @return Vec<ShellSession> - List of active sessions
	 */
	pub async fn list_sessions(&self) -> Vec<ShellSession> {
		let sessions = self.sessions.read().await;
		sessions.values().cloned().collect()
	}
	
	/**
	 * Terminates a shell session
	 * 
	 * @param session_id - Session ID to terminate
	 * @return Result<()> - Success or error status
	 */
	pub async fn terminate_session(&mut self, session_id: &str) -> Result<()> {
		/**
		 * シェルセッションを終了する関数です
		 * 
		 * 指定されたセッションIDのシェルセッションを終了し、
		 * 関連するプロセスとリソースをクリーンアップします。
		 * 
		 * セッション状態を更新し、プロセスに終了シグナルを
		 * 送信してリソースを適切に解放します
		 */
		
		let mut sessions = self.sessions.write().await;
		
		if let Some(session) = sessions.get_mut(session_id) {
			session.state = SessionState::ShuttingDown;
			
			// Terminate the process using ProcessManager
			use crate::terminal::process::ProcessManager;
			
			let mut process_manager = ProcessManager::new();
			process_manager.terminate_process(session.process.pid).await?;
			
			// Update session state
			session.state = SessionState::Terminated(0);
		}
		
		Ok(())
	}
	
	/**
	 * Gets shell configuration
	 * 
	 * @param shell_type - Shell type
	 * @return Option<&ShellConfig> - Configuration if found
	 */
	pub fn get_config(&self, shell_type: &str) -> Option<&ShellConfig> {
		self.configurations.get(shell_type)
	}
	
	/**
	 * Gets available shell types
	 * 
	 * @return Vec<String> - List of available shell types
	 */
	pub fn get_available_shells(&self) -> Vec<String> {
		self.configurations.keys().cloned().collect()
	}
	
	/**
	 * Gets the default shell
	 * 
	 * @return &str - Default shell
	 */
	pub fn default_shell(&self) -> &str {
		&self.default_shell
	}
	
	/**
	 * Detects available shells
	 * 
	 * Scans the system for available shells and
	 * updates the configuration accordingly.
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub fn detect_available_shells(&mut self) -> Result<()> {
		/**
		 * システムで利用可能なシェルを検出する関数です
		 * 
		 * 一般的なシェル（bash、zsh、fish、sh、dash）の
		 * 実行可能ファイルの存在をチェックし、見つかった
		 * シェルを設定に追加します。
		 * 
		 * 各シェルのパスを確認し、存在する場合はShellConfigを
		 * 作成してconfigurationsに追加します。デフォルトの
		 * 機能設定も適用されます。
		 */
		
		let common_shells = [
			("/bin/bash", "bash"),
			("/bin/zsh", "zsh"),
			("/bin/fish", "fish"),
			("/bin/sh", "sh"),
			("/bin/dash", "dash"),
		];
		
		for (path, name) in &common_shells {
			if std::path::Path::new(path).exists() {
				if !self.configurations.contains_key(*name) {
					self.configurations.insert(name.to_string(), ShellConfig {
						executable: path.to_string(),
						name: name.to_string(),
						startup_commands: Vec::new(),
						environment: HashMap::new(),
						features: ShellFeatures::default(),
					});
				}
			}
		}
		
		Ok(())
	}
} 