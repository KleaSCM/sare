/**
 * Input validation and sanitization module
 * 
 * This module provides comprehensive input validation and sanitization
 * for commands, file paths, hosts, and URLs to prevent injection attacks.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: validation.rs
 * Description: Input validation and sanitization with regex patterns
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use regex::Regex;
use url::Url;

use super::{SecurityConfig, SecurityEvent, SecuritySeverity};

/**
 * Validation configuration
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
	/// Enable command validation
	pub command_validation: bool,
	/// Enable path validation
	pub path_validation: bool,
	/// Enable host validation
	pub host_validation: bool,
	/// Enable URL validation
	pub url_validation: bool,
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
	/// Maximum file size (bytes)
	pub max_file_size: u64,
	/// Allowed file extensions
	pub allowed_extensions: Vec<String>,
	/// Blocked patterns
	pub blocked_patterns: Vec<String>,
}

impl Default for ValidationConfig {
	fn default() -> Self {
		Self {
			command_validation: true,
			path_validation: true,
			host_validation: true,
			url_validation: true,
			size_validation: true,
			max_command_length: 1024,
			max_path_length: 4096,
			max_host_length: 253,
			max_url_length: 2048,
			max_file_size: 100 * 1024 * 1024, // 100MB
			allowed_extensions: vec![
				"txt".to_string(), "md".to_string(), "rs".to_string(),
				"toml".to_string(), "json".to_string(), "yaml".to_string(),
				"yml".to_string(), "sh".to_string(), "py".to_string(),
				"js".to_string(), "ts".to_string(), "html".to_string(),
				"css".to_string(), "xml".to_string(), "log".to_string(),
			],
			blocked_patterns: vec![
				r"rm\s+-rf\s+/".to_string(),
				r"dd\s+if=/dev/zero".to_string(),
				r":\(\)\s*\{\s*:\|\s*:\s*&\s*\};\s*:".to_string(),
				r"forkbomb".to_string(),
				r"mkfs".to_string(),
				r"fdisk".to_string(),
				r"dd\s+if=".to_string(),
			],
		}
	}
}

/**
 * Validation patterns
 */
#[derive(Debug, Clone)]
pub struct ValidationPatterns {
	/// Command validation regex
	pub command_regex: Regex,
	/// Path validation regex
	pub path_regex: Regex,
	/// Host validation regex
	pub host_regex: Regex,
	/// URL validation regex
	pub url_regex: Regex,
	/// Blocked patterns
	pub blocked_patterns: Vec<Regex>,
	/// Dangerous characters
	pub dangerous_chars: Regex,
}

impl ValidationPatterns {
	/**
	 * Creates new validation patterns
	 */
	pub fn new() -> Result<Self> {
		Ok(Self {
			command_regex: Regex::new(r"^[a-zA-Z0-9_\-\./\\\s]+$")?,
			path_regex: Regex::new(r"^[a-zA-Z0-9_\-\./\\\s]+$")?,
			host_regex: Regex::new(r"^[a-zA-Z0-9\-\.]+$")?,
			url_regex: Regex::new(r"^https?://[a-zA-Z0-9\-\.]+(:\d+)?(/[a-zA-Z0-9\-\./]*)?$")?,
			blocked_patterns: vec![
				Regex::new(r"rm\s+-rf\s+/")?,
				Regex::new(r"dd\s+if=/dev/zero")?,
				Regex::new(r":\(\)\s*\{\s*:\|\s*:\s*&\s*\};\s*:")?,
				Regex::new(r"forkbomb")?,
				Regex::new(r"mkfs")?,
				Regex::new(r"fdisk")?,
				Regex::new(r"dd\s+if=")?,
			],
			dangerous_chars: Regex::new(r"[;&|`$(){}[\]<>]")?,
		})
	}
}

/**
 * Input validator
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
	 */
	pub async fn new(config: Arc<RwLock<SecurityConfig>>) -> Result<Self> {
		let validation_config = ValidationConfig::default();
		let patterns = ValidationPatterns::new()?;
		
		Ok(Self {
			config,
			validation_config,
			patterns,
			active: true,
		})
	}
	
	/**
	 * Validates a command
	 */
	pub async fn validate_command(&self, command: &str) -> Result<bool> {
		if !self.active || !self.validation_config.command_validation {
			return Ok(true);
		}
		
		// Check length
		if command.len() > self.validation_config.max_command_length {
			return Ok(false);
		}
		
		// Check for blocked patterns
		for pattern in &self.patterns.blocked_patterns {
			if pattern.is_match(command) {
				return Ok(false);
			}
		}
		
		// Check for dangerous characters
		if self.patterns.dangerous_chars.is_match(command) {
			return Ok(false);
		}
		
		// Validate command format
		if !self.patterns.command_regex.is_match(command) {
			return Ok(false);
		}
		
		// Check for command injection attempts
		if command.contains(";") || command.contains("|") || command.contains("&") {
			return Ok(false);
		}
		
		// Check for path traversal attempts
		if command.contains("../") || command.contains("..\\") {
			return Ok(false);
		}
		
		Ok(true)
	}
	
	/**
	 * Validates a file path
	 */
	pub async fn validate_path(&self, path: &str) -> Result<bool> {
		if !self.active || !self.validation_config.path_validation {
			return Ok(true);
		}
		
		// Check length
		if path.len() > self.validation_config.max_path_length {
			return Ok(false);
		}
		
		// Check for path traversal
		if path.contains("../") || path.contains("..\\") {
			return Ok(false);
		}
		
		// Check for absolute paths to sensitive directories
		let sensitive_dirs = vec!["/etc", "/var", "/sys", "/proc", "/dev"];
		for dir in sensitive_dirs {
			if path.starts_with(dir) {
				return Ok(false);
			}
		}
		
		// Validate path format
		if !self.patterns.path_regex.is_match(path) {
			return Ok(false);
		}
		
		// Check file extension if specified
		if let Some(extension) = self.get_file_extension(path) {
			if !self.validation_config.allowed_extensions.contains(&extension) {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Validates a host
	 */
	pub async fn validate_host(&self, host: &str) -> Result<bool> {
		if !self.active || !self.validation_config.host_validation {
			return Ok(true);
		}
		
		// Check length
		if host.len() > self.validation_config.max_host_length {
			return Ok(false);
		}
		
		// Check for localhost or private IPs
		if host == "localhost" || host == "127.0.0.1" || host == "::1" {
			return Ok(false);
		}
		
		// Check for private IP ranges
		if host.starts_with("192.168.") || host.starts_with("10.") || host.starts_with("172.") {
			return Ok(false);
		}
		
		// Validate host format
		if !self.patterns.host_regex.is_match(host) {
			return Ok(false);
		}
		
		// Check for IP address format
		if self.is_ip_address(host) {
			return self.validate_ip_address(host);
		}
		
		// Check for domain name format
		if self.is_domain_name(host) {
			return self.validate_domain_name(host);
		}
		
		Ok(false)
	}
	
	/**
	 * Validates a URL
	 */
	pub async fn validate_url(&self, url: &str) -> Result<bool> {
		if !self.active || !self.validation_config.url_validation {
			return Ok(true);
		}
		
		// Check length
		if url.len() > self.validation_config.max_url_length {
			return Ok(false);
		}
		
		// Parse URL
		let parsed_url = match Url::parse(url) {
			Ok(url) => url,
			Err(_) => return Ok(false),
		};
		
		// Check scheme
		let scheme = parsed_url.scheme();
		if scheme != "http" && scheme != "https" {
			return Ok(false);
		}
		
		// Check host
		if let Some(host) = parsed_url.host_str() {
			if !self.validate_host(host).await? {
				return Ok(false);
			}
		}
		
		// Validate URL format
		if !self.patterns.url_regex.is_match(url) {
			return Ok(false);
		}
		
		Ok(true)
	}
	
	/**
	 * Sanitizes input by removing dangerous characters
	 */
	pub fn sanitize_input(&self, input: &str) -> String {
		// Remove dangerous characters
		let sanitized = self.patterns.dangerous_chars.replace_all(input, "");
		
		// Remove path traversal attempts
		let sanitized = sanitized.replace("../", "").replace("..\\", "");
		
		// Remove command injection attempts
		let sanitized = sanitized.replace(";", "").replace("|", "").replace("&", "");
		
		// Remove backticks
		let sanitized = sanitized.replace("`", "");
		
		// Remove dollar signs
		let sanitized = sanitized.replace("$", "");
		
		// Remove parentheses
		let sanitized = sanitized.replace("(", "").replace(")", "");
		
		// Remove brackets
		let sanitized = sanitized.replace("[", "").replace("]", "");
		let sanitized = sanitized.replace("{", "").replace("}", "");
		
		// Remove angle brackets
		let sanitized = sanitized.replace("<", "").replace(">", "");
		
		sanitized.to_string()
	}
	
	/**
	 * Gets file extension from path
	 */
	fn get_file_extension(&self, path: &str) -> Option<String> {
		path.split('.')
			.last()
			.map(|ext| ext.to_lowercase())
	}
	
	/**
	 * Validates file size
	 */
	pub async fn validate_size(&self, size: u64) -> Result<bool> {
		if !self.active || !self.validation_config.size_validation {
			return Ok(true);
		}
		
		Ok(size <= self.validation_config.max_file_size)
	}
	
	/**
	 * Checks if sandbox is active
	 */
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	/**
	 * Updates validation configuration
	 */
	pub fn update_config(&mut self, config: ValidationConfig) {
		self.validation_config = config;
	}
	
	/**
	 * Gets current configuration
	 */
	pub fn get_config(&self) -> ValidationConfig {
		self.validation_config.clone()
	}
	
	/**
	 * Checks if string is an IP address
	 */
	fn is_ip_address(&self, host: &str) -> bool {
		// Simple IP address check
		host.split('.').count() == 4 && host.chars().all(|c| c.is_ascii_digit() || c == '.')
	}
	
	/**
	 * Validates IP address
	 */
	fn validate_ip_address(&self, ip: &str) -> Result<bool> {
		// Check for private IP ranges
		if ip.starts_with("192.168.") || ip.starts_with("10.") || ip.starts_with("172.") {
			return Ok(false);
		}
		
		// Check for loopback
		if ip == "127.0.0.1" || ip == "::1" {
			return Ok(false);
		}
		
		// Parse IP address
		let parts: Vec<&str> = ip.split('.').collect();
		if parts.len() != 4 {
			return Ok(false);
		}
		
		for part in parts {
			if let Ok(num) = part.parse::<u8>() {
				if num > 255 {
					return Ok(false);
				}
			} else {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Checks if string is a domain name
	 */
	fn is_domain_name(&self, host: &str) -> bool {
		// Simple domain name check
		host.contains('.') && !host.starts_with('.') && !host.ends_with('.')
	}
	
	/**
	 * Validates domain name
	 */
	fn validate_domain_name(&self, domain: &str) -> Result<bool> {
		// Check for localhost
		if domain == "localhost" {
			return Ok(false);
		}
		
		// Check for valid characters
		if !domain.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
			return Ok(false);
		}
		
		// Check for consecutive dots
		if domain.contains("..") {
			return Ok(false);
		}
		
		// Check for valid TLD
		let parts: Vec<&str> = domain.split('.').collect();
		if parts.len() < 2 {
			return Ok(false);
		}
		
		// Check each part
		for part in parts {
			if part.is_empty() || part.len() > 63 {
				return Ok(false);
			}
			
			if part.starts_with('-') || part.ends_with('-') {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
} 