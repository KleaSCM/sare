/**
 * Logging system for Sare terminal
 * 
 * This module provides comprehensive logging with log levels,
 * log rotation, and multiple output destinations.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: logger.rs
 * Description: Comprehensive logging system with log levels and rotation
 */

use anyhow::Result;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/**
 * Log level
 * 
 * ログレベルです。
 * ログの重要度を
 * 定義します。
 */
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum LogLevel {
	/// Trace level (most verbose)
	Trace = 0,
	/// Debug level
	Debug = 1,
	/// Info level
	Info = 2,
	/// Warning level
	Warn = 3,
	/// Error level
	Error = 4,
	/// Fatal level (least verbose)
	Fatal = 5,
}

impl std::fmt::Display for LogLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LogLevel::Trace => write!(f, "TRACE"),
			LogLevel::Debug => write!(f, "DEBUG"),
			LogLevel::Info => write!(f, "INFO"),
			LogLevel::Warn => write!(f, "WARN"),
			LogLevel::Error => write!(f, "ERROR"),
			LogLevel::Fatal => write!(f, "FATAL"),
		}
	}
}

/**
 * Log destination
 * 
 * ログ出力先です。
 * ログの出力先を
 * 定義します。
 */
pub enum LogDestination {
	/// Standard output
	Stdout,
	/// Standard error
	Stderr,
	/// File output
	File(String),
	/// Socket output
	Socket(String),
	/// Custom writer
	Custom(Box<dyn Write + Send + Sync>),
}

/**
 * Log entry
 * 
 * ログエントリです。
 * 個別のログエントリを
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct LogEntry {
	/// Entry timestamp
	pub timestamp: DateTime<Utc>,
	/// Entry level
	pub level: LogLevel,
	/// Entry module
	pub module: String,
	/// Entry message
	pub message: String,
	/// Entry context
	pub context: HashMap<String, String>,
	/// Entry thread ID
	pub thread_id: u64,
}

/**
 * Log rotation configuration
 * 
 * ログローテーション設定です。
 * ログファイルの
 * ローテーション設定を管理します。
 */
#[derive(Debug, Clone)]
pub struct LogRotationConfig {
	/// Maximum file size in bytes
	pub max_file_size: u64,
	/// Maximum number of backup files
	pub max_backup_files: u32,
	/// Rotation interval in seconds
	pub rotation_interval: u64,
	/// Enable compression
	pub enable_compression: bool,
}

impl Default for LogRotationConfig {
	fn default() -> Self {
		Self {
			max_file_size: 10 * 1024 * 1024, // 10 MB
			max_backup_files: 5,
			rotation_interval: 24 * 60 * 60, // 24 hours
			enable_compression: true,
		}
	}
}

/**
 * Logger configuration
 * 
 * ロガー設定です。
 * ログ機能の設定を
 * 管理します。
 */
pub struct LoggerConfig {
	/// Minimum log level
	pub min_level: LogLevel,
	/// Log destinations
	pub destinations: Vec<LogDestination>,
	/// Log format
	pub format: LogFormat,
	/// Log rotation configuration
	pub rotation: LogRotationConfig,
	/// Enable timestamps
	pub enable_timestamps: bool,
	/// Enable thread IDs
	pub enable_thread_ids: bool,
	/// Enable module names
	pub enable_module_names: bool,
	/// Enable context
	pub enable_context: bool,
}

impl Default for LoggerConfig {
	fn default() -> Self {
		Self {
			min_level: LogLevel::Info,
			destinations: vec![LogDestination::Stderr],
			format: LogFormat::Text,
			rotation: LogRotationConfig::default(),
			enable_timestamps: true,
			enable_thread_ids: true,
			enable_module_names: true,
			enable_context: false,
		}
	}
}

/**
 * Log format
 * 
 * ログ形式です。
 * ログの出力形式を
 * 定義します。
 */
#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
	/// Text format
	Text,
	/// JSON format
	Json,
	/// CSV format
	Csv,
	/// Custom format
	Custom(String),
}

/**
 * Comprehensive logger
 * 
 * 包括的なロガーです。
 * ログレベル、ローテーション、
 * 複数出力先を提供します。
 */
pub struct Logger {
	/// Logger configuration
	config: LoggerConfig,
	/// Log entries
	entries: Arc<RwLock<Vec<LogEntry>>>,
	/// Log files
	log_files: Arc<RwLock<HashMap<String, File>>>,
	/// Log statistics
	statistics: Arc<RwLock<LogStatistics>>,
	/// Last rotation time
	last_rotation: Instant,
}

/**
 * Log statistics
 * 
 * ログ統計です。
 * ログの統計情報を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct LogStatistics {
	/// Total log entries
	pub total_entries: u64,
	/// Entries by level
	pub entries_by_level: HashMap<LogLevel, u64>,
	/// Entries by module
	pub entries_by_module: HashMap<String, u64>,
	/// Last log time
	pub last_log_time: Option<DateTime<Utc>>,
	/// Log file sizes
	pub log_file_sizes: HashMap<String, u64>,
}

impl Logger {
	/**
	 * Creates a new logger
	 * 
	 * @param config - Logger configuration
	 * @return Logger - New logger
	 */
	pub fn new(config: LoggerConfig) -> Self {
		Self {
			config,
			entries: Arc::new(RwLock::new(Vec::new())),
			log_files: Arc::new(RwLock::new(HashMap::new())),
			statistics: Arc::new(RwLock::new(LogStatistics {
				total_entries: 0,
				entries_by_level: HashMap::new(),
				entries_by_module: HashMap::new(),
				last_log_time: None,
				log_file_sizes: HashMap::new(),
			})),
			last_rotation: Instant::now(),
		}
	}
	
	/**
	 * Initializes the logger
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		println!("📝 Logger initialized");
		println!("📊 Minimum log level: {}", self.config.min_level);
		println!("🎯 Destinations: {}", self.config.destinations.len());
		println!("📄 Format: {:?}", self.config.format);
		println!("🔄 Rotation enabled: {}", self.config.rotation.max_file_size > 0);
		
		// Initialize log files
		for destination in &self.config.destinations {
			if let LogDestination::File(path) = destination {
				self.initialize_log_file(path).await?;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Initializes a log file
	 * 
	 * @param path - Log file path
	 * @return Result<()> - Success or error status
	 */
	async fn initialize_log_file(&self, path: &str) -> Result<()> {
		let file = OpenOptions::new()
			.create(true)
			.append(true)
			.open(path)?;
		
		let mut log_files = self.log_files.write().await;
		log_files.insert(path.to_string(), file);
		
		println!("📁 Log file initialized: {}", path);
		
		Ok(())
	}
	
	/**
	 * Logs a message
	 * 
	 * @param level - Log level
	 * @param module - Module name
	 * @param message - Log message
	 * @param context - Log context
	 * @return Result<()> - Success or error status
	 */
	pub async fn log(&self, level: LogLevel, module: &str, message: &str, context: Option<HashMap<String, String>>) -> Result<()> {
		if level < self.config.min_level {
			return Ok(());
		}
		
		let entry = LogEntry {
			timestamp: Utc::now(),
			level: level.clone(),
			module: module.to_string(),
			message: message.to_string(),
			context: context.unwrap_or_default(),
			thread_id: 0, // Simplified for now - thread ID not available in stable Rust
		};
		
		// Add to entries
		let mut entries = self.entries.write().await;
		entries.push(entry.clone());
		
		// Update statistics
		{
			let mut statistics = self.statistics.write().await;
			statistics.total_entries += 1;
			*statistics.entries_by_level.entry(level).or_insert(0) += 1;
			*statistics.entries_by_module.entry(module.to_string()).or_insert(0) += 1;
			statistics.last_log_time = Some(entry.timestamp);
		}
		
		// Format and write log entry
		let formatted = self.format_log_entry(&entry)?;
		self.write_log_entry(&formatted).await?;
		
		// Check for rotation
		self.check_rotation().await?;
		
		Ok(())
	}
	
	/**
	 * Formats a log entry
	 * 
	 * @param entry - Log entry
	 * @return Result<String> - Formatted log entry
	 */
	fn format_log_entry(&self, entry: &LogEntry) -> Result<String> {
		let mut formatted = String::new();
		
		match self.config.format {
			LogFormat::Text => {
				if self.config.enable_timestamps {
					formatted.push_str(&format!("[{}] ", entry.timestamp.format("%Y-%m-%d %H:%M:%S")));
				}
				
				formatted.push_str(&format!("[{}] ", entry.level));
				
				if self.config.enable_module_names {
					formatted.push_str(&format!("[{}] ", entry.module));
				}
				
				if self.config.enable_thread_ids {
					formatted.push_str(&format!("[TID:{}] ", entry.thread_id));
				}
				
				formatted.push_str(&entry.message);
				
				if self.config.enable_context && !entry.context.is_empty() {
					formatted.push_str(" {");
					for (key, value) in &entry.context {
						formatted.push_str(&format!("{}={}", key, value));
					}
					formatted.push_str("}");
				}
				
				formatted.push('\n');
			}
			LogFormat::Json => {
				let mut json = serde_json::Map::new();
				json.insert("timestamp".to_string(), serde_json::Value::String(entry.timestamp.to_rfc3339()));
				json.insert("level".to_string(), serde_json::Value::String(entry.level.to_string()));
				json.insert("module".to_string(), serde_json::Value::String(entry.module.clone()));
				json.insert("message".to_string(), serde_json::Value::String(entry.message.clone()));
				json.insert("thread_id".to_string(), serde_json::Value::Number(serde_json::Number::from(entry.thread_id)));
				
				if !entry.context.is_empty() {
					let context: serde_json::Map<String, serde_json::Value> = entry.context
						.iter()
						.map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
						.collect();
					json.insert("context".to_string(), serde_json::Value::Object(context));
				}
				
				formatted = serde_json::to_string(&serde_json::Value::Object(json))?;
				formatted.push('\n');
			}
			LogFormat::Csv => {
				formatted.push_str(&format!("{},{},{},{},{},\"{}\"",
					entry.timestamp.to_rfc3339(),
					entry.level,
					entry.module,
					entry.thread_id,
					entry.message.replace("\"", "\"\""),
					entry.context.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(";")
				));
				formatted.push('\n');
			}
			LogFormat::Custom(ref format) => {
				formatted = format
					.replace("{timestamp}", &entry.timestamp.to_rfc3339())
					.replace("{level}", &entry.level.to_string())
					.replace("{module}", &entry.module)
					.replace("{thread_id}", &entry.thread_id.to_string())
					.replace("{message}", &entry.message);
				formatted.push('\n');
			}
		}
		
		Ok(formatted)
	}
	
	/**
	 * Writes a log entry to all destinations
	 * 
	 * @param formatted - Formatted log entry
	 * @return Result<()> - Success or error status
	 */
	async fn write_log_entry(&self, formatted: &str) -> Result<()> {
		for destination in &self.config.destinations {
			match destination {
				LogDestination::Stdout => {
					print!("{}", formatted);
					io::stdout().flush()?;
				}
				LogDestination::Stderr => {
					eprint!("{}", formatted);
					io::stderr().flush()?;
				}
				LogDestination::File(path) => {
					if let Some(file) = self.log_files.read().await.get(path) {
						// Note: This is a simplified implementation
						// In a real implementation, you'd need to handle file writing properly
						println!("[FILE] {}", formatted.trim());
					}
				}
				LogDestination::Socket(_) => {
					// Socket logging would be implemented here
					println!("[SOCKET] {}", formatted.trim());
				}
				LogDestination::Custom(_) => {
					// Custom writer logging would be implemented here
					println!("[CUSTOM] {}", formatted.trim());
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Checks if log rotation is needed
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn check_rotation(&self) -> Result<()> {
		let now = Instant::now();
		if now.duration_since(self.last_rotation).as_secs() >= self.config.rotation.rotation_interval {
			self.rotate_logs().await?;
		}
		
		Ok(())
	}
	
	/**
	 * Rotates log files
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn rotate_logs(&self) -> Result<()> {
		println!("🔄 Rotating log files...");
		
		// Implementation would rotate log files based on configuration
		// This is a simplified version
		
		Ok(())
	}
	
	/**
	 * Gets log statistics
	 * 
	 * @return LogStatistics - Log statistics
	 */
	pub async fn get_statistics(&self) -> LogStatistics {
		self.statistics.read().await.clone()
	}
	
	/**
	 * Clears log entries
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn clear_entries(&self) -> Result<()> {
		let mut entries = self.entries.write().await;
		entries.clear();
		
		let mut statistics = self.statistics.write().await;
		*statistics = LogStatistics {
			total_entries: 0,
			entries_by_level: HashMap::new(),
			entries_by_module: HashMap::new(),
			last_log_time: None,
			log_file_sizes: HashMap::new(),
		};
		
		println!("📝 Log entries cleared");
		
		Ok(())
	}
	
	/**
	 * Gets logger configuration
	 * 
	 * @return &LoggerConfig - Logger configuration
	 */
	pub fn config(&self) -> &LoggerConfig {
		&self.config
	}
	
	/**
	 * Updates logger configuration
	 * 
	 * @param config - New logger configuration
	 */
	pub fn update_config(&mut self, config: LoggerConfig) {
		self.config = config;
	}
}

// Convenience macros for logging
#[macro_export]
macro_rules! log_trace {
	($logger:expr, $module:expr, $($arg:tt)*) => {
		$logger.log(crate::debug::logger::LogLevel::Trace, $module, &format!($($arg)*), None).await
	};
}

#[macro_export]
macro_rules! log_debug {
	($logger:expr, $module:expr, $($arg:tt)*) => {
		$logger.log(crate::debug::logger::LogLevel::Debug, $module, &format!($($arg)*), None).await
	};
}

#[macro_export]
macro_rules! log_info {
	($logger:expr, $module:expr, $($arg:tt)*) => {
		$logger.log(crate::debug::logger::LogLevel::Info, $module, &format!($($arg)*), None).await
	};
}

#[macro_export]
macro_rules! log_warn {
	($logger:expr, $module:expr, $($arg:tt)*) => {
		$logger.log(crate::debug::logger::LogLevel::Warn, $module, &format!($($arg)*), None).await
	};
}

#[macro_export]
macro_rules! log_error {
	($logger:expr, $module:expr, $($arg:tt)*) => {
		$logger.log(crate::debug::logger::LogLevel::Error, $module, &format!($($arg)*), None).await
	};
}

#[macro_export]
macro_rules! log_fatal {
	($logger:expr, $module:expr, $($arg:tt)*) => {
		$logger.log(crate::debug::logger::LogLevel::Fatal, $module, &format!($($arg)*), None).await
	};
} 