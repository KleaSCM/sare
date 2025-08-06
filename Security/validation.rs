/**
 * Input Validation System for Sare Terminal
 * 
 * This module provides comprehensive input validation and sanitization
 * capabilities, including command validation, path sanitization, and
 * security checks to prevent malicious input from affecting the system.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: validation.rs
 * Description: Input validation and sanitization system
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashSet;
use regex::Regex;
use url::Url;

use super::{SecurityConfig, SecurityEvent, SecuritySeverity};

/**
 * Input validation configuration
 * 
 * 入力バリデーション設定を管理する構造体です。
 * コマンド検証、パス検証、セキュリティチェックなどの
 * 設定を提供します。
 */
#[derive(Debug, Clone)]
pub struct ValidationConfig {
	/// Enable command validation
	pub command_validation: bool,
	/// Enable path validation
	pub path_validation: bool,
	/// Enable host validation
	pub host_validation: bool,
	/// Enable URL validation
	pub url_validation: bool,
	/// Enable file extension validation
	pub extension_validation: bool,
	/// Enable size validation
	pub size_validation: bool,
	/// Maximum command length
	pub max_command_length: usize,
	/// Maximum path length
	pub max_path_length: usize,
	/// Maximum host length
	pub max_host_length: usize,
	/// Maximum URL length
	pub max_url_length: usize,
	/// Allowed file extensions
	pub allowed_extensions: HashSet<String>,
	/// Blocked file extensions
	pub blocked_extensions: HashSet<String>,
	/// Allowed hosts
	pub allowed_hosts: HashSet<String>,
	/// Blocked hosts
	pub blocked_hosts: HashSet<String>,
	/// Allowed protocols
	pub allowed_protocols: HashSet<String>,
	/// Blocked protocols
	pub blocked_protocols: HashSet<String>,
}

impl Default for ValidationConfig {
	fn default() -> Self {
		let mut allowed_extensions = HashSet::new();
		allowed_extensions.insert("txt".to_string());
		allowed_extensions.insert("md".to_string());
		allowed_extensions.insert("rs".to_string());
		allowed_extensions.insert("toml".to_string());
		allowed_extensions.insert("json".to_string());
		allowed_extensions.insert("yaml".to_string());
		allowed_extensions.insert("yml".to_string());
		allowed_extensions.insert("sh".to_string());
		allowed_extensions.insert("py".to_string());
		allowed_extensions.insert("js".to_string());
		allowed_extensions.insert("ts".to_string());
		allowed_extensions.insert("html".to_string());
		allowed_extensions.insert("css".to_string());
		allowed_extensions.insert("xml".to_string());
		allowed_extensions.insert("log".to_string());
		
		let mut blocked_extensions = HashSet::new();
		blocked_extensions.insert("exe".to_string());
		blocked_extensions.insert("bat".to_string());
		blocked_extensions.insert("com".to_string());
		blocked_extensions.insert("scr".to_string());
		blocked_extensions.insert("pif".to_string());
		blocked_extensions.insert("vbs".to_string());
		blocked_extensions.insert("js".to_string());
		blocked_extensions.insert("jar".to_string());
		blocked_extensions.insert("class".to_string());
		blocked_extensions.insert("dll".to_string());
		blocked_extensions.insert("so".to_string());
		blocked_extensions.insert("dylib".to_string());
		
		let mut allowed_hosts = HashSet::new();
		allowed_hosts.insert("localhost".to_string());
		allowed_hosts.insert("127.0.0.1".to_string());
		allowed_hosts.insert("::1".to_string());
		
		let mut blocked_hosts = HashSet::new();
		blocked_hosts.insert("0.0.0.0".to_string());
		blocked_hosts.insert("255.255.255.255".to_string());
		
		let mut allowed_protocols = HashSet::new();
		allowed_protocols.insert("http".to_string());
		allowed_protocols.insert("https".to_string());
		allowed_protocols.insert("ftp".to_string());
		allowed_protocols.insert("sftp".to_string());
		allowed_protocols.insert("ssh".to_string());
		
		let mut blocked_protocols = HashSet::new();
		blocked_protocols.insert("file".to_string());
		blocked_protocols.insert("data".to_string());
		blocked_protocols.insert("javascript".to_string());
		blocked_protocols.insert("vbscript".to_string());
		
		Self {
			command_validation: true,
			path_validation: true,
			host_validation: true,
			url_validation: true,
			extension_validation: true,
			size_validation: true,
			max_command_length: 8192,
			max_path_length: 4096,
			max_host_length: 255,
			max_url_length: 2048,
			allowed_extensions,
			blocked_extensions,
			allowed_hosts,
			blocked_hosts,
			allowed_protocols,
			blocked_protocols,
		}
	}
}

/**
 * Input validation patterns
 * 
 * 入力バリデーションパターンを管理する構造体です。
 * 正規表現パターンを使用して入力の検証を
 * 実行します。
 */
#[derive(Debug)]
pub struct ValidationPatterns {
	/// Command pattern
	pub command_pattern: Regex,
	/// Path pattern
	pub path_pattern: Regex,
	/// Host pattern
	pub host_pattern: Regex,
	/// URL pattern
	pub url_pattern: Regex,
	/// File extension pattern
	pub extension_pattern: Regex,
	/// Dangerous command pattern
	pub dangerous_command_pattern: Regex,
	/// Path traversal pattern
	pub path_traversal_pattern: Regex,
	/// Shell injection pattern
	pub shell_injection_pattern: Regex,
}

impl ValidationPatterns {
	/**
	 * Creates new validation patterns
	 * 
	 * @return Result<ValidationPatterns> - New validation patterns or error
	 */
	pub fn new() -> Result<Self> {
		/**
		 * バリデーションパターンを初期化する関数です
		 * 
		 * 正規表現パターンを使用して入力検証のための
		 * パターンを作成します。
		 * 
		 * コマンド、パス、ホスト、URLなどの入力形式を
		 * 検証するためのパターンを定義します。
		 */
		
		Ok(Self {
			command_pattern: Regex::new(r"^[a-zA-Z0-9_\-\./\\\s]+$")?,
			path_pattern: Regex::new(r"^[a-zA-Z0-9_\-\./\\\s]+$")?,
			host_pattern: Regex::new(r"^[a-zA-Z0-9\-\.]+$")?,
			url_pattern: Regex::new(r"^[a-zA-Z0-9\-\.:/?=&]+$")?,
			extension_pattern: Regex::new(r"\.([a-zA-Z0-9]+)$")?,
			dangerous_command_pattern: Regex::new(r"(rm\s+-rf|dd\s+if=|:\(\)\s*\{\s*:\|:\s*&\s*\};:|forkbomb|killall|pkill|kill\s+-9)")?,
			path_traversal_pattern: Regex::new(r"\.\./|\.\.\\|%2e%2e|%2e%2e%2f|%2e%2e%5c")?,
			shell_injection_pattern: Regex::new(r"[;&|`$\(\)\{\}\[\]]")?,
		})
	}
}

/**
 * Input validator for security checks
 * 
 * セキュリティチェックのための入力バリデーターです。
 * コマンド、パス、ホスト、URLなどの入力の
 * 検証とサニタイゼーションを提供します。
 */
pub struct InputValidator {
	/// Security configuration
	config: Arc<RwLock<SecurityConfig>>,
	/// Validation configuration
	validation_config: ValidationConfig,
	/// Validation patterns
	patterns: ValidationPatterns,
	/// Active state
	active: bool,
}

impl InputValidator {
	/**
	 * Creates a new input validator
	 * 
	 * @param config - Security configuration
	 * @return Result<InputValidator> - New input validator or error
	 */
	pub async fn new(config: Arc<RwLock<SecurityConfig>>) -> Result<Self> {
		/**
		 * 入力バリデーターを初期化する関数です
		 * 
		 * 指定された設定で入力バリデーターを作成し、
		 * コマンド、パス、ホスト、URLなどの入力の
		 * 検証とサニタイゼーション機能を提供します。
		 * 
		 * 正規表現パターンを使用して入力の形式と
		 * セキュリティを検証し、悪意のある入力を
		 * 防止します。
		 */
		
		Ok(Self {
			config,
			validation_config: ValidationConfig::default(),
			patterns: ValidationPatterns::new()?,
			active: true,
		})
	}
	
	/**
	 * Validates a command
	 * 
	 * @param command - Command to validate
	 * @return Result<bool> - Whether command is valid
	 */
	pub async fn validate_command(&self, command: &str) -> Result<bool> {
		/**
		 * コマンドを検証する関数です
		 * 
		 * 指定されたコマンドの形式とセキュリティを
		 * 検証し、悪意のあるコマンドを防止します。
		 * 
		 * コマンドの長さ、文字、危険なパターンなどを
		 * チェックして安全なコマンドかどうかを
		 * 判定します。
		 */
		
		if !self.validation_config.command_validation {
			return Ok(true);
		}
		
		// Check command length
		if command.len() > self.validation_config.max_command_length {
			return Ok(false);
		}
		
		// Check command pattern
		if !self.patterns.command_pattern.is_match(command) {
			return Ok(false);
		}
		
		// Check for dangerous commands
		if self.patterns.dangerous_command_pattern.is_match(command) {
			return Ok(false);
		}
		
		// Check for shell injection
		if self.patterns.shell_injection_pattern.is_match(command) {
			return Ok(false);
		}
		
		// Check blocked commands from config
		let config = self.config.read().await;
		for blocked_cmd in &config.blocked_commands {
			if command.contains(blocked_cmd) {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Validates a file path
	 * 
	 * @param path - Path to validate
	 * @return Result<bool> - Whether path is valid
	 */
	pub async fn validate_path(&self, path: &str) -> Result<bool> {
		/**
		 * ファイルパスを検証する関数です
		 * 
		 * 指定されたパスの形式とセキュリティを
		 * 検証し、パストラバーサル攻撃を防止します。
		 * 
		 * パスの長さ、文字、危険なパターンなどを
		 * チェックして安全なパスかどうかを
		 * 判定します。
		 */
		
		if !self.validation_config.path_validation {
			return Ok(true);
		}
		
		// Check path length
		if path.len() > self.validation_config.max_path_length {
			return Ok(false);
		}
		
		// Check path pattern
		if !self.patterns.path_pattern.is_match(path) {
			return Ok(false);
		}
		
		// Check for path traversal
		if self.patterns.path_traversal_pattern.is_match(path) {
			return Ok(false);
		}
		
		// Check file extension if validation is enabled
		if self.validation_config.extension_validation {
			if let Some(extension) = self.get_file_extension(path) {
				if self.validation_config.blocked_extensions.contains(&extension) {
					return Ok(false);
				}
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Validates a host name or IP address
	 * 
	 * @param host - Host to validate
	 * @return Result<bool> - Whether host is valid
	 */
	pub async fn validate_host(&self, host: &str) -> Result<bool> {
		/**
		 * ホスト名またはIPアドレスを検証する関数です
		 * 
		 * 指定されたホストの形式とセキュリティを
		 * 検証し、不正なホストへのアクセスを防止します。
		 * 
		 * ホストの長さ、文字、許可されたホストなどを
		 * チェックして安全なホストかどうかを
		 * 判定します。
		 */
		
		if !self.validation_config.host_validation {
			return Ok(true);
		}
		
		// Check host length
		if host.len() > self.validation_config.max_host_length {
			return Ok(false);
		}
		
		// Check host pattern
		if !self.patterns.host_pattern.is_match(host) {
			return Ok(false);
		}
		
		// Check blocked hosts
		if self.validation_config.blocked_hosts.contains(host) {
			return Ok(false);
		}
		
		// Check if host is in allowed list (if not empty)
		if !self.validation_config.allowed_hosts.is_empty() {
			if !self.validation_config.allowed_hosts.contains(host) {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Validates a URL
	 * 
	 * @param url - URL to validate
	 * @return Result<bool> - Whether URL is valid
	 */
	pub async fn validate_url(&self, url: &str) -> Result<bool> {
		/**
		 * URLを検証する関数です
		 * 
		 * 指定されたURLの形式とセキュリティを
		 * 検証し、不正なURLへのアクセスを防止します。
		 * 
		 * URLの長さ、プロトコル、ホストなどを
		 * チェックして安全なURLかどうかを
		 * 判定します。
		 */
		
		if !self.validation_config.url_validation {
			return Ok(true);
		}
		
		// Check URL length
		if url.len() > self.validation_config.max_url_length {
			return Ok(false);
		}
		
		// Check URL pattern
		if !self.patterns.url_pattern.is_match(url) {
			return Ok(false);
		}
		
		// Parse URL
		if let Ok(parsed_url) = Url::parse(url) {
			// Check protocol
			if let Some(scheme) = parsed_url.scheme() {
				if self.validation_config.blocked_protocols.contains(scheme) {
					return Ok(false);
				}
				
				if !self.validation_config.allowed_protocols.is_empty() {
					if !self.validation_config.allowed_protocols.contains(scheme) {
						return Ok(false);
					}
				}
			}
			
			// Check host
			if let Some(host) = parsed_url.host_str() {
				if !self.validate_host(host).await? {
					return Ok(false);
				}
			}
		} else {
			return Ok(false);
		}
		
		Ok(true)
	}
	
	/**
	 * Sanitizes input by removing dangerous characters
	 * 
	 * @param input - Input to sanitize
	 * @return String - Sanitized input
	 */
	pub fn sanitize_input(&self, input: &str) -> String {
		/**
		 * 入力から危険な文字を除去する関数です
		 * 
		 * 指定された入力から危険な文字やパターンを
		 * 除去して安全な入力に変換します。
		 * 
		 * シェルインジェクション、パストラバーサルなどの
		 * 攻撃を防止するための文字を除去します。
		 */
		
		let mut sanitized = input.to_string();
		
		// Remove shell injection characters
		sanitized = sanitized.replace(";", "");
		sanitized = sanitized.replace("&", "");
		sanitized = sanitized.replace("|", "");
		sanitized = sanitized.replace("`", "");
		sanitized = sanitized.replace("$", "");
		sanitized = sanitized.replace("(", "");
		sanitized = sanitized.replace(")", "");
		sanitized = sanitized.replace("{", "");
		sanitized = sanitized.replace("}", "");
		sanitized = sanitized.replace("[", "");
		sanitized = sanitized.replace("]", "");
		
		// Remove path traversal patterns
		sanitized = sanitized.replace("../", "");
		sanitized = sanitized.replace("..\\", "");
		sanitized = sanitized.replace("%2e%2e", "");
		sanitized = sanitized.replace("%2e%2e%2f", "");
		sanitized = sanitized.replace("%2e%2e%5c", "");
		
		// Remove null bytes
		sanitized = sanitized.replace('\0', "");
		
		// Trim whitespace
		sanitized = sanitized.trim().to_string();
		
		sanitized
	}
	
	/**
	 * Gets file extension from path
	 * 
	 * @param path - File path
	 * @return Option<String> - File extension or None
	 */
	fn get_file_extension(&self, path: &str) -> Option<String> {
		if let Some(captures) = self.patterns.extension_pattern.captures(path) {
			if let Some(extension) = captures.get(1) {
				return Some(extension.as_str().to_lowercase());
			}
		}
		None
	}
	
	/**
	 * Validates file size
	 * 
	 * @param size - File size in bytes
	 * @return Result<bool> - Whether size is valid
	 */
	pub async fn validate_size(&self, size: u64) -> Result<bool> {
		/**
		 * ファイルサイズを検証する関数です
		 * 
		 * 指定されたファイルサイズが制限内かどうかを
		 * 検証し、大きなファイルによる攻撃を防止します。
		 * 
		 * 設定された最大ファイルサイズと比較して
		 * 安全なサイズかどうかを判定します。
		 */
		
		if !self.validation_config.size_validation {
			return Ok(true);
		}
		
		let config = self.config.read().await;
		Ok(size <= config.max_file_size)
	}
	
	/**
	 * Checks if validator is active
	 * 
	 * @return bool - Whether validator is active
	 */
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	/**
	 * Updates validation configuration
	 * 
	 * @param config - New validation configuration
	 */
	pub fn update_config(&mut self, config: ValidationConfig) {
		self.validation_config = config;
	}
	
	/**
	 * Gets current validation configuration
	 * 
	 * @return ValidationConfig - Current validation configuration
	 */
	pub fn get_config(&self) -> ValidationConfig {
		self.validation_config.clone()
	}
} 