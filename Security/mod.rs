/**
 * Advanced Security System for Sare Terminal
 * 
 * This module provides comprehensive security features for the terminal emulator,
 * including process sandboxing, input validation, path sanitization, audit logging,
 * and permission management. All components are designed for full integration
 * with the terminal system.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Main security module orchestrating all security components
 */

pub mod sandbox;
pub mod validation;
pub mod audit;
pub mod permissions;
pub mod encryption;
pub mod isolation;
pub mod monitoring;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/**
 * Security configuration for the terminal
 * 
 * セキュリティ設定を管理する構造体です。
 * サンドボックス、監査、暗号化などの
 * セキュリティ機能の設定を提供します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
	/// Enable process sandboxing
	pub sandbox_enabled: bool,
	/// Enable input validation
	pub validation_enabled: bool,
	/// Enable audit logging
	pub audit_enabled: bool,
	/// Enable permission system
	pub permissions_enabled: bool,
	/// Enable encryption for sensitive data
	pub encryption_enabled: bool,
	/// Enable process isolation
	pub isolation_enabled: bool,
	/// Enable security monitoring
	pub monitoring_enabled: bool,
	/// Maximum allowed file size for operations (bytes)
	pub max_file_size: u64,
	/// Allowed file extensions
	pub allowed_extensions: Vec<String>,
	/// Blocked commands
	pub blocked_commands: Vec<String>,
	/// Allowed network ports
	pub allowed_ports: Vec<u16>,
	/// Security log level
	pub log_level: SecurityLogLevel,
	/// Audit log path
	pub audit_log_path: String,
	/// Encryption key path
	pub encryption_key_path: String,
}

/**
 * Security log levels
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLogLevel {
	/// Minimal logging
	Minimal,
	/// Standard logging
	Standard,
	/// Verbose logging
	Verbose,
	/// Debug logging
	Debug,
}

impl Default for SecurityConfig {
	fn default() -> Self {
		Self {
			sandbox_enabled: true,
			validation_enabled: true,
			audit_enabled: true,
			permissions_enabled: true,
			encryption_enabled: false,
			isolation_enabled: true,
			monitoring_enabled: true,
			max_file_size: 100 * 1024 * 1024, // 100MB
			allowed_extensions: vec![
				"txt".to_string(), "md".to_string(), "rs".to_string(),
				"toml".to_string(), "json".to_string(), "yaml".to_string(),
				"yml".to_string(), "sh".to_string(), "py".to_string(),
				"js".to_string(), "ts".to_string(), "html".to_string(),
				"css".to_string(), "xml".to_string(), "log".to_string(),
			],
			blocked_commands: vec![
				"rm -rf /".to_string(), "dd if=/dev/zero".to_string(),
				":(){ :|:& };:".to_string(), "forkbomb".to_string(),
			],
			allowed_ports: vec![
				22, 80, 443, 8080, 3000, 5000, 8000, 9000,
			],
			log_level: SecurityLogLevel::Standard,
			audit_log_path: "/tmp/sare_security_audit.log".to_string(),
			encryption_key_path: "/tmp/sare_encryption.key".to_string(),
		}
	}
}

/**
 * Security event types
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
	/// Command execution
	CommandExecution {
		command: String,
		user: String,
		timestamp: u64,
		success: bool,
	},
	/// File access
	FileAccess {
		path: String,
		operation: String,
		user: String,
		timestamp: u64,
		success: bool,
	},
	/// Network access
	NetworkAccess {
		host: String,
		port: u16,
		protocol: String,
		user: String,
		timestamp: u64,
		success: bool,
	},
	/// Permission violation
	PermissionViolation {
		resource: String,
		operation: String,
		user: String,
		timestamp: u64,
		reason: String,
	},
	/// Security alert
	SecurityAlert {
		alert_type: String,
		description: String,
		severity: SecuritySeverity,
		timestamp: u64,
	},
}

/**
 * Security severity levels
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
	/// Low severity
	Low,
	/// Medium severity
	Medium,
	/// High severity
	High,
	/// Critical severity
	Critical,
}

/**
 * Main security manager for the terminal
 * 
 * ターミナルのセキュリティを管理するメインクラスです。
 * すべてのセキュリティコンポーネントを統合し、
 * 包括的なセキュリティ保護を提供します。
 */
pub struct SecurityManager {
	/// Security configuration
	config: Arc<RwLock<SecurityConfig>>,
	/// Sandbox manager
	sandbox: Arc<sandbox::SandboxManager>,
	/// Input validator
	validator: Arc<validation::InputValidator>,
	/// Audit logger
	auditor: Arc<audit::AuditLogger>,
	/// Permission manager
	permissions: Arc<permissions::PermissionManager>,
	/// Encryption manager
	encryption: Arc<encryption::EncryptionManager>,
	/// Isolation manager
	isolation: Arc<isolation::IsolationManager>,
	/// Security monitor
	monitor: Arc<monitoring::SecurityMonitor>,
}

impl SecurityManager {
	/**
	 * Creates a new security manager
	 * 
	 * @param config - Security configuration
	 * @return Result<SecurityManager> - New security manager or error
	 */
	pub async fn new(config: SecurityConfig) -> Result<Self> {
		/**
		 * セキュリティマネージャーを初期化する関数です
		 * 
		 * 指定された設定でセキュリティマネージャーを作成し、
		 * すべてのセキュリティコンポーネントを初期化します。
		 * 
		 * サンドボックス、バリデーション、監査、権限管理などの
		 * セキュリティ機能を統合して包括的な保護を提供します。
		 */
		
		let config_arc = Arc::new(RwLock::new(config.clone()));
		
		let sandbox = Arc::new(sandbox::SandboxManager::new(config_arc.clone()).await?);
		let validator = Arc::new(validation::InputValidator::new(config_arc.clone()).await?);
		let auditor = Arc::new(audit::AuditLogger::new(config_arc.clone()).await?);
		let permissions = Arc::new(permissions::PermissionManager::new(config_arc.clone()).await?);
		let encryption = Arc::new(encryption::EncryptionManager::new(config_arc.clone()).await?);
		let isolation = Arc::new(isolation::IsolationManager::new(config_arc.clone()).await?);
		let monitor = Arc::new(monitoring::SecurityMonitor::new(config_arc.clone()).await?);
		
		Ok(Self {
			config: config_arc,
			sandbox,
			validator,
			auditor,
			permissions,
			encryption,
			isolation,
			monitor,
		})
	}
	
	/**
	 * Validates and executes a command securely
	 * 
	 * @param command - Command to execute
	 * @param user - User executing the command
	 * @return Result<bool> - Whether command is allowed
	 */
	pub async fn validate_command(&self, command: &str, user: &str) -> Result<bool> {
		// Validate input
		if !self.validator.validate_command(command).await? {
			self.auditor.log_event(SecurityEvent::PermissionViolation {
				resource: command.to_string(),
				operation: "execute".to_string(),
				user: user.to_string(),
				timestamp: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				reason: "Invalid command format".to_string(),
			}).await?;
			return Ok(false);
		}
		
		// Check permissions
		if !self.permissions.can_execute_command(command, user).await? {
			self.auditor.log_event(SecurityEvent::PermissionViolation {
				resource: command.to_string(),
				operation: "execute".to_string(),
				user: user.to_string(),
				timestamp: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				reason: "Insufficient permissions".to_string(),
			}).await?;
			return Ok(false);
		}
		
		// Log command execution
		self.auditor.log_event(SecurityEvent::CommandExecution {
			command: command.to_string(),
			user: user.to_string(),
			timestamp: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)?
				.as_secs(),
			success: true,
		}).await?;
		
		Ok(true)
	}
	
	/**
	 * Validates file access
	 * 
	 * @param path - File path
	 * @param operation - File operation
	 * @param user - User accessing the file
	 * @return Result<bool> - Whether access is allowed
	 */
	pub async fn validate_file_access(&self, path: &str, operation: &str, user: &str) -> Result<bool> {
		// Validate path
		if !self.validator.validate_path(path).await? {
			self.auditor.log_event(SecurityEvent::PermissionViolation {
				resource: path.to_string(),
				operation: operation.to_string(),
				user: user.to_string(),
				timestamp: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				reason: "Invalid path".to_string(),
			}).await?;
			return Ok(false);
		}
		
		// Check file permissions
		if !self.permissions.can_access_file(path, operation, user).await? {
			self.auditor.log_event(SecurityEvent::PermissionViolation {
				resource: path.to_string(),
				operation: operation.to_string(),
				user: user.to_string(),
				timestamp: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				reason: "Insufficient file permissions".to_string(),
			}).await?;
			return Ok(false);
		}
		
		// Log file access
		self.auditor.log_event(SecurityEvent::FileAccess {
			path: path.to_string(),
			operation: operation.to_string(),
			user: user.to_string(),
			timestamp: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)?
				.as_secs(),
			success: true,
		}).await?;
		
		Ok(true)
	}
	
	/**
	 * Validates network access
	 * 
	 * @param host - Target host
	 * @param port - Target port
	 * @param protocol - Network protocol
	 * @param user - User making the request
	 * @return Result<bool> - Whether access is allowed
	 */
	pub async fn validate_network_access(&self, host: &str, port: u16, protocol: &str, user: &str) -> Result<bool> {
		// Validate host
		if !self.validator.validate_host(host).await? {
			self.auditor.log_event(SecurityEvent::PermissionViolation {
				resource: format!("{}:{}", host, port),
				operation: protocol.to_string(),
				user: user.to_string(),
				timestamp: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				reason: "Invalid host".to_string(),
			}).await?;
			return Ok(false);
		}
		
		// Check network permissions
		if !self.permissions.can_access_network(host, port, protocol, user).await? {
			self.auditor.log_event(SecurityEvent::PermissionViolation {
				resource: format!("{}:{}", host, port),
				operation: protocol.to_string(),
				user: user.to_string(),
				timestamp: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				reason: "Network access denied".to_string(),
			}).await?;
			return Ok(false);
		}
		
		// Log network access
		self.auditor.log_event(SecurityEvent::NetworkAccess {
			host: host.to_string(),
			port,
			protocol: protocol.to_string(),
			user: user.to_string(),
			timestamp: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)?
				.as_secs(),
			success: true,
		}).await?;
		
		Ok(true)
	}
	
	/**
	 * Creates a sandboxed process
	 * 
	 * @param command - Command to execute
	 * @param user - User executing the command
	 * @return Result<u32> - Process ID or error
	 */
	pub async fn create_sandboxed_process(&self, command: &str, user: &str) -> Result<u32> {
		self.sandbox.create_process(command, user).await
	}
	
	/**
	 * Encrypts sensitive data
	 * 
	 * @param data - Data to encrypt
	 * @return Result<Vec<u8>> - Encrypted data or error
	 */
	pub async fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
		self.encryption.encrypt(data).await
	}
	
	/**
	 * Decrypts sensitive data
	 * 
	 * @param data - Data to decrypt
	 * @return Result<Vec<u8>> - Decrypted data or error
	 */
	pub async fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
		self.encryption.decrypt(data).await
	}
	
	/**
	 * Gets current security status
	 * 
	 * @return SecurityStatus - Current security status
	 */
	pub async fn get_security_status(&self) -> SecurityStatus {
		SecurityStatus {
			sandbox_active: self.sandbox.is_active().await,
			validation_active: self.validator.is_active().await,
			audit_active: self.auditor.is_active().await,
			permissions_active: self.permissions.is_active().await,
			encryption_active: self.encryption.is_active().await,
			isolation_active: self.isolation.is_active().await,
			monitoring_active: self.monitor.is_active().await,
			alerts: self.monitor.get_alerts().await,
		}
	}
	
	/**
	 * Updates security configuration
	 * 
	 * @param config - New security configuration
	 */
	pub async fn update_config(&self, config: SecurityConfig) {
		let mut config_guard = self.config.write().await;
		*config_guard = config;
	}
	
	/**
	 * Gets current configuration
	 * 
	 * @return SecurityConfig - Current security configuration
	 */
	pub async fn get_config(&self) -> SecurityConfig {
		self.config.read().await.clone()
	}
}

/**
 * Security status information
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
	/// Whether sandbox is active
	pub sandbox_active: bool,
	/// Whether validation is active
	pub validation_active: bool,
	/// Whether audit logging is active
	pub audit_active: bool,
	/// Whether permissions are active
	pub permissions_active: bool,
	/// Whether encryption is active
	pub encryption_active: bool,
	/// Whether isolation is active
	pub isolation_active: bool,
	/// Whether monitoring is active
	pub monitoring_active: bool,
	/// Current security alerts
	pub alerts: Vec<SecurityEvent>,
} 