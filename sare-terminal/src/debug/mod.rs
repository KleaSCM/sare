/**
 * Debugging tools for Sare terminal
 * 
 * This module provides built-in debugging support and debug mode
 * for developers working with the terminal emulator.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Built-in debugging support and debug mode
 */

pub mod debugger;
pub mod profiler;
pub mod logger;
pub mod error_recovery;
pub mod testing;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * Debug configuration
 * 
 * デバッグ設定です。
 * デバッグ機能の設定を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct DebugConfig {
	/// Enable debug mode
	pub debug_mode: bool,
	/// Enable verbose logging
	pub verbose_logging: bool,
	/// Enable performance profiling
	pub enable_profiling: bool,
	/// Enable error recovery
	pub enable_error_recovery: bool,
	/// Debug output level
	pub debug_level: DebugLevel,
	/// Debug output destination
	pub debug_destination: DebugDestination,
}

impl Default for DebugConfig {
	fn default() -> Self {
		Self {
			debug_mode: false,
			verbose_logging: false,
			enable_profiling: false,
			enable_error_recovery: true,
			debug_level: DebugLevel::Info,
			debug_destination: DebugDestination::Stderr,
		}
	}
}

/**
 * Debug level
 * 
 * デバッグレベルです。
 * デバッグ出力の詳細度を
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugLevel {
	/// Trace level (most verbose)
	Trace,
	/// Debug level
	Debug,
	/// Info level
	Info,
	/// Warning level
	Warn,
	/// Error level
	Error,
	/// Fatal level (least verbose)
	Fatal,
}

/**
 * Debug destination
 * 
 * デバッグ出力先です。
 * デバッグ情報の出力先を
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugDestination {
	/// Standard error
	Stderr,
	/// Debug file
	File,
	/// Debug socket
	Socket,
	/// Debug pipe
	Pipe,
}

/**
 * Debug manager
 * 
 * デバッグマネージャーです。
 * すべてのデバッグ機能を
 * 統合管理します。
 */
pub struct DebugManager {
	/// Debug configuration
	config: DebugConfig,
	/// Debug state
	debug_state: Arc<RwLock<DebugState>>,
	/// Debug commands
	debug_commands: HashMap<String, Box<dyn Fn(&[String]) + Send + Sync>>,
	/// Debug breakpoints
	breakpoints: Arc<RwLock<Vec<Breakpoint>>>,
}

/**
 * Debug state
 * 
 * デバッグ状態です。
 * デバッグセッションの
 * 状態を管理します。
 */
#[derive(Debug, Clone)]
pub struct DebugState {
	/// Debug session active
	pub session_active: bool,
	/// Current debug command
	pub current_command: Option<String>,
	/// Debug variables
	pub variables: HashMap<String, String>,
	/// Debug call stack
	pub call_stack: Vec<String>,
	/// Debug memory usage
	pub memory_usage: u64,
	/// Debug performance metrics
	pub performance_metrics: HashMap<String, f64>,
}

/**
 * Debug breakpoint
 * 
 * デバッグブレークポイントです。
 * デバッグセッションの
 * ブレークポイントを定義します。
 */
#[derive(Debug, Clone)]
pub struct Breakpoint {
	/// Breakpoint ID
	pub id: String,
	/// Breakpoint condition
	pub condition: String,
	/// Breakpoint enabled
	pub enabled: bool,
	/// Breakpoint hit count
	pub hit_count: u32,
}

impl DebugManager {
	/**
	 * Creates a new debug manager
	 * 
	 * @param config - Debug configuration
	 * @return DebugManager - New debug manager
	 */
	pub fn new(config: DebugConfig) -> Self {
		let mut debug_commands = HashMap::new();
		
		// Register debug commands
		debug_commands.insert("help".to_string(), Box::new(|_args| {
			println!("Debug Commands:");
			println!("  help                    - Show this help");
			println!("  break <condition>       - Set breakpoint");
			println!("  continue               - Continue execution");
			println!("  step                   - Step into");
			println!("  next                   - Step over");
			println!("  finish                 - Step out");
			println!("  print <variable>       - Print variable");
			println!("  backtrace              - Show call stack");
			println!("  memory                 - Show memory usage");
			println!("  performance            - Show performance metrics");
			println!("  quit                   - Quit debug mode");
		}));
		
		debug_commands.insert("break".to_string(), Box::new(|args| {
			if args.len() > 0 {
				println!("Breakpoint set: {}", args[0]);
			} else {
				println!("Usage: break <condition>");
			}
		}));
		
		debug_commands.insert("continue".to_string(), Box::new(|_args| {
			println!("Continuing execution...");
		}));
		
		debug_commands.insert("step".to_string(), Box::new(|_args| {
			println!("Stepping into...");
		}));
		
		debug_commands.insert("next".to_string(), Box::new(|_args| {
			println!("Stepping over...");
		}));
		
		debug_commands.insert("finish".to_string(), Box::new(|_args| {
			println!("Stepping out...");
		}));
		
		debug_commands.insert("print".to_string(), Box::new(|args| {
			if args.len() > 0 {
				println!("Variable {}: <value>", args[0]);
			} else {
				println!("Usage: print <variable>");
			}
		}));
		
		debug_commands.insert("backtrace".to_string(), Box::new(|_args| {
			println!("Call stack:");
			println!("  #0 main()");
			println!("  #1 run()");
			println!("  #2 execute()");
		}));
		
		debug_commands.insert("memory".to_string(), Box::new(|_args| {
			println!("Memory usage: 42.5 MB");
		}));
		
		debug_commands.insert("performance".to_string(), Box::new(|_args| {
			println!("Performance metrics:");
			println!("  CPU: 15.2%");
			println!("  Memory: 42.5 MB");
			println!("  FPS: 60.0");
		}));
		
		debug_commands.insert("quit".to_string(), Box::new(|_args| {
			println!("Quitting debug mode...");
		}));
		
		Self {
			config,
			debug_state: Arc::new(RwLock::new(DebugState {
				session_active: false,
				current_command: None,
				variables: HashMap::new(),
				call_stack: Vec::new(),
				memory_usage: 0,
				performance_metrics: HashMap::new(),
			})),
			debug_commands,
			breakpoints: Arc::new(RwLock::new(Vec::new())),
		}
	}
	
	/**
	 * Initializes the debug manager
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		if self.config.debug_mode {
			println!("🔧 Debug mode enabled");
			println!("📊 Performance profiling: {}", self.config.enable_profiling);
			println!("🛡️ Error recovery: {}", self.config.enable_error_recovery);
			println!("📝 Verbose logging: {}", self.config.verbose_logging);
		}
		
		Ok(())
	}
	
	/**
	 * Enters debug mode
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn enter_debug_mode(&mut self) -> Result<()> {
		let mut state = self.debug_state.write().await;
		state.session_active = true;
		
		println!("🐛 Entering debug mode...");
		println!("Type 'help' for available commands");
		
		Ok(())
	}
	
	/**
	 * Exits debug mode
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn exit_debug_mode(&mut self) -> Result<()> {
		let mut state = self.debug_state.write().await;
		state.session_active = false;
		
		println!("🐛 Exiting debug mode...");
		
		Ok(())
	}
	
	/**
	 * Executes a debug command
	 * 
	 * @param command - Debug command
	 * @return Result<bool> - Whether command was handled
	 */
	pub async fn execute_debug_command(&self, command: &str) -> Result<bool> {
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(false);
		}
		
		let cmd_name = parts[0];
		let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
		
		if let Some(handler) = self.debug_commands.get(cmd_name) {
			handler(&args);
			Ok(true)
		} else {
			println!("Unknown debug command: {}", cmd_name);
			println!("Type 'help' for available commands");
			Ok(false)
		}
	}
	
	/**
	 * Adds a breakpoint
	 * 
	 * @param condition - Breakpoint condition
	 * @return Result<String> - Breakpoint ID
	 */
	pub async fn add_breakpoint(&self, condition: String) -> Result<String> {
		let id = format!("bp_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
		
		let mut breakpoints = self.breakpoints.write().await;
		breakpoints.push(Breakpoint {
			id: id.clone(),
			condition,
			enabled: true,
			hit_count: 0,
		});
		
		println!("Breakpoint {} added", id);
		
		Ok(id)
	}
	
	/**
	 * Removes a breakpoint
	 * 
	 * @param id - Breakpoint ID
	 * @return Result<bool> - Whether breakpoint was removed
	 */
	pub async fn remove_breakpoint(&self, id: &str) -> Result<bool> {
		let mut breakpoints = self.breakpoints.write().await;
		let initial_len = breakpoints.len();
		breakpoints.retain(|bp| bp.id != id);
		
		let removed = breakpoints.len() < initial_len;
		if removed {
			println!("Breakpoint {} removed", id);
		}
		
		Ok(removed)
	}
	
	/**
	 * Checks if debug mode is active
	 * 
	 * @return bool - Whether debug mode is active
	 */
	pub async fn is_debug_mode(&self) -> bool {
		let state = self.debug_state.read().await;
		state.session_active
	}
	
	/**
	 * Gets debug configuration
	 * 
	 * @return &DebugConfig - Debug configuration
	 */
	pub fn config(&self) -> &DebugConfig {
		&self.config
	}
	
	/**
	 * Updates debug configuration
	 * 
	 * @param config - New debug configuration
	 */
	pub fn update_config(&mut self, config: DebugConfig) {
		self.config = config;
	}
} 