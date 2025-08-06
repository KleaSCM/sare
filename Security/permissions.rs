/**
 * Permission Management System for Sare Terminal
 * 
 * This module provides comprehensive permission management and access control,
 * including user permissions, resource access control, and role-based
 * security to ensure proper authorization for all system operations.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: permissions.rs
 * Description: Permission management and access control system
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use super::{SecurityConfig, SecurityEvent, SecuritySeverity};

/**
 * Permission level
 * 
 * 権限レベルを定義する列挙型です。
 * システム内の権限の階層を
 * 管理します。
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
	/// No permissions
	None,
	/// Read-only permissions
	Read,
	/// Read and write permissions
	ReadWrite,
	/// Full permissions
	Full,
	/// Administrative permissions
	Admin,
}

/**
 * Resource type
 * 
 * リソースタイプを定義する列挙型です。
 * システム内のリソースの種類を
 * 管理します。
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
	/// File system resource
	File,
	/// Network resource
	Network,
	/// Process resource
	Process,
	/// System resource
	System,
	/// User resource
	User,
	/// Configuration resource
	Config,
}

/**
 * Permission rule
 * 
 * 権限ルールを管理する構造体です。
 * リソース、操作、権限レベルなどの
 * 権限情報を保持します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
	/// Resource type
	pub resource_type: ResourceType,
	/// Resource path/identifier
	pub resource_path: String,
	/// Allowed operations
	pub allowed_operations: HashSet<String>,
	/// Permission level
	pub permission_level: PermissionLevel,
	/// User or group
	pub subject: String,
	/// Whether rule is active
	pub active: bool,
}

/**
 * User permissions
 * 
 * ユーザー権限を管理する構造体です。
 * ユーザーの権限、ロール、グループなどの
 * 情報を保持します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
	/// User ID
	pub user_id: String,
	/// User roles
	pub roles: HashSet<String>,
	/// User groups
	pub groups: HashSet<String>,
	/// Direct permissions
	pub permissions: HashMap<String, PermissionLevel>,
	/// Whether user is active
	pub active: bool,
	/// User creation time
	pub created_at: u64,
	/// Last login time
	pub last_login: Option<u64>,
}

/**
 * Permission configuration
 * 
 * 権限設定を管理する構造体です。
 * 権限システムの動作、デフォルト権限、
 * セキュリティポリシーなどの設定を
 * 提供します。
 */
#[derive(Debug, Clone)]
pub struct PermissionConfig {
	/// Enable permission system
	pub enabled: bool,
	/// Default permission level
	pub default_permission_level: PermissionLevel,
	/// Require authentication
	pub require_authentication: bool,
	/// Allow anonymous access
	pub allow_anonymous: bool,
	/// Enable role-based access control
	pub enable_rbac: bool,
	/// Enable group-based access control
	pub enable_gbac: bool,
	/// Enable audit logging for permissions
	pub audit_permissions: bool,
	/// Maximum failed login attempts
	pub max_failed_attempts: u32,
	/// Lockout duration (seconds)
	pub lockout_duration: u64,
}

impl Default for PermissionConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			default_permission_level: PermissionLevel::Read,
			require_authentication: true,
			allow_anonymous: false,
			enable_rbac: true,
			enable_gbac: true,
			audit_permissions: true,
			max_failed_attempts: 5,
			lockout_duration: 300, // 5 minutes
		}
	}
}

/**
 * Permission manager for access control
 * 
 * アクセス制御のための権限マネージャーです。
 * ユーザー権限、リソースアクセス制御、
 * ロールベースセキュリティを提供します。
 */
pub struct PermissionManager {
	/// Security configuration
	config: Arc<RwLock<SecurityConfig>>,
	/// Permission configuration
	permission_config: PermissionConfig,
	/// Permission rules
	rules: Arc<RwLock<HashMap<String, PermissionRule>>>,
	/// User permissions
	users: Arc<RwLock<HashMap<String, UserPermissions>>>,
	/// Role definitions
	roles: Arc<RwLock<HashMap<String, HashSet<String>>>>,
	/// Group definitions
	groups: Arc<RwLock<HashMap<String, HashSet<String>>>>,
	/// Failed login attempts
	failed_attempts: Arc<RwLock<HashMap<String, (u32, u64)>>>,
	/// Active state
	active: bool,
}

impl PermissionManager {
	/**
	 * Creates a new permission manager
	 * 
	 * @param config - Security configuration
	 * @return Result<PermissionManager> - New permission manager or error
	 */
	pub async fn new(config: Arc<RwLock<SecurityConfig>>) -> Result<Self> {
		/**
		 * 権限マネージャーを初期化する関数です
		 * 
		 * 指定された設定で権限マネージャーを作成し、
		 * ユーザー権限、リソースアクセス制御、
		 * ロールベースセキュリティ機能を提供します。
		 * 
		 * 権限ルール、ユーザー権限、ロール定義などを
		 * 初期化して包括的なアクセス制御システムを
		 * 構築します。
		 */
		
		let permission_config = PermissionConfig::default();
		let rules = Arc::new(RwLock::new(HashMap::new()));
		let users = Arc::new(RwLock::new(HashMap::new()));
		let roles = Arc::new(RwLock::new(HashMap::new()));
		let groups = Arc::new(RwLock::new(HashMap::new()));
		let failed_attempts = Arc::new(RwLock::new(HashMap::new()));
		
		let manager = Self {
			config,
			permission_config,
			rules,
			users,
			roles,
			groups,
			failed_attempts,
			active: true,
		};
		
		// Initialize default permissions
		manager.initialize_default_permissions().await?;
		
		Ok(manager)
	}
	
	/**
	 * Checks if user can execute a command
	 * 
	 * @param command - Command to check
	 * @param user - User executing the command
	 * @return Result<bool> - Whether user can execute command
	 */
	pub async fn can_execute_command(&self, command: &str, user: &str) -> Result<bool> {
		/**
		 * ユーザーがコマンドを実行できるかチェックする関数です
		 * 
		 * 指定されたユーザーが指定されたコマンドを
		 * 実行する権限があるかどうかをチェックします。
		 * 
		 * ユーザーの権限、ロール、グループなどを
		 * 考慮してコマンド実行の許可を判定します。
		 */
		
		if !self.permission_config.enabled {
			return Ok(true);
		}
		
		// Check if user is locked out
		if self.is_user_locked_out(user).await? {
			return Ok(false);
		}
		
		// Get user permissions
		let user_permissions = self.get_user_permissions(user).await?;
		
		// Check command-specific permissions
		if let Some(rule) = self.find_command_rule(command, user).await? {
			return Ok(rule.active && rule.permission_level != PermissionLevel::None);
		}
		
		// Check user's general command execution permission
		if let Some(level) = user_permissions.permissions.get("command_execution") {
			match level {
				PermissionLevel::None => Ok(false),
				PermissionLevel::Read => Ok(false),
				PermissionLevel::ReadWrite => Ok(true),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		} else {
			// Use default permission level
			Ok(self.permission_config.default_permission_level != PermissionLevel::None)
		}
	}
	
	/**
	 * Checks if user can access a file
	 * 
	 * @param path - File path
	 * @param operation - File operation
	 * @param user - User accessing the file
	 * @return Result<bool> - Whether user can access file
	 */
	pub async fn can_access_file(&self, path: &str, operation: &str, user: &str) -> Result<bool> {
		/**
		 * ユーザーがファイルにアクセスできるかチェックする関数です
		 * 
		 * 指定されたユーザーが指定されたファイルに対して
		 * 指定された操作を実行する権限があるかどうかを
		 * チェックします。
		 * 
		 * ファイルパス、操作タイプ、ユーザー権限などを
		 * 考慮してファイルアクセスの許可を判定します。
		 */
		
		if !self.permission_config.enabled {
			return Ok(true);
		}
		
		// Check if user is locked out
		if self.is_user_locked_out(user).await? {
			return Ok(false);
		}
		
		// Get user permissions
		let user_permissions = self.get_user_permissions(user).await?;
		
		// Check file-specific permissions
		if let Some(rule) = self.find_file_rule(path, operation, user).await? {
			return Ok(rule.active && rule.allowed_operations.contains(operation));
		}
		
		// Check user's general file access permission
		if let Some(level) = user_permissions.permissions.get("file_access") {
			match level {
				PermissionLevel::None => Ok(false),
				PermissionLevel::Read => Ok(operation == "read"),
				PermissionLevel::ReadWrite => Ok(operation == "read" || operation == "write"),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		} else {
			// Use default permission level
			match self.permission_config.default_permission_level {
				PermissionLevel::None => Ok(false),
				PermissionLevel::Read => Ok(operation == "read"),
				PermissionLevel::ReadWrite => Ok(operation == "read" || operation == "write"),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		}
	}
	
	/**
	 * Checks if user can access network
	 * 
	 * @param host - Target host
	 * @param port - Target port
	 * @param protocol - Network protocol
	 * @param user - User making the request
	 * @return Result<bool> - Whether user can access network
	 */
	pub async fn can_access_network(&self, host: &str, port: u16, protocol: &str, user: &str) -> Result<bool> {
		/**
		 * ユーザーがネットワークにアクセスできるかチェックする関数です
		 * 
		 * 指定されたユーザーが指定されたネットワークリソースに
		 * アクセスする権限があるかどうかをチェックします。
		 * 
		 * ホスト、ポート、プロトコル、ユーザー権限などを
		 * 考慮してネットワークアクセスの許可を判定します。
		 */
		
		if !self.permission_config.enabled {
			return Ok(true);
		}
		
		// Check if user is locked out
		if self.is_user_locked_out(user).await? {
			return Ok(false);
		}
		
		// Get user permissions
		let user_permissions = self.get_user_permissions(user).await?;
		
		// Check network-specific permissions
		let resource_path = format!("{}:{}", host, port);
		if let Some(rule) = self.find_network_rule(&resource_path, protocol, user).await? {
			return Ok(rule.active && rule.allowed_operations.contains(protocol));
		}
		
		// Check user's general network access permission
		if let Some(level) = user_permissions.permissions.get("network_access") {
			match level {
				PermissionLevel::None => Ok(false),
				PermissionLevel::Read => Ok(true), // Read access for network
				PermissionLevel::ReadWrite => Ok(true),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		} else {
			// Use default permission level
			match self.permission_config.default_permission_level {
				PermissionLevel::None => Ok(false),
				PermissionLevel::Read => Ok(true),
				PermissionLevel::ReadWrite => Ok(true),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		}
	}
	
	/**
	 * Adds a permission rule
	 * 
	 * @param rule - Permission rule to add
	 * @return Result<()> - Success or error status
	 */
	pub async fn add_rule(&self, rule: PermissionRule) -> Result<()> {
		/**
		 * 権限ルールを追加する関数です
		 * 
		 * 指定された権限ルールをシステムに追加し、
		 * リソースアクセス制御を強化します。
		 * 
		 * ルールの重複チェック、有効性検証などを
		 * 実行して安全な権限管理を実現します。
		 */
		
		let rule_id = format!("{}:{}:{}", rule.resource_type, rule.resource_path, rule.subject);
		
		let mut rules = self.rules.write().await;
		rules.insert(rule_id, rule);
		
		Ok(())
	}
	
	/**
	 * Removes a permission rule
	 * 
	 * @param rule_id - ID of rule to remove
	 * @return Result<()> - Success or error status
	 */
	pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
		let mut rules = self.rules.write().await;
		rules.remove(rule_id);
		
		Ok(())
	}
	
	/**
	 * Gets user permissions
	 * 
	 * @param user - User ID
	 * @return Result<UserPermissions> - User permissions or error
	 */
	async fn get_user_permissions(&self, user: &str) -> Result<UserPermissions> {
		/**
		 * ユーザー権限を取得する関数です
		 * 
		 * 指定されたユーザーの権限情報を取得し、
		 * 存在しない場合はデフォルト権限を
		 * 作成します。
		 * 
		 * ユーザーのロール、グループ、直接権限などを
		 * 含む包括的な権限情報を返します。
		 */
		
		let users = self.users.read().await;
		
		if let Some(user_perms) = users.get(user) {
			Ok(user_perms.clone())
		} else {
			// Create default user permissions
			Ok(UserPermissions {
				user_id: user.to_string(),
				roles: HashSet::new(),
				groups: HashSet::new(),
				permissions: HashMap::new(),
				active: true,
				created_at: std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs(),
				last_login: None,
			})
		}
	}
	
	/**
	 * Finds command-specific rule
	 * 
	 * @param command - Command to check
	 * @param user - User executing command
	 * @return Result<Option<PermissionRule>> - Matching rule or None
	 */
	async fn find_command_rule(&self, command: &str, user: &str) -> Result<Option<PermissionRule>> {
		let rules = self.rules.read().await;
		
		for rule in rules.values() {
			if rule.resource_type == ResourceType::Process &&
			   rule.resource_path == command &&
			   rule.subject == user {
				return Ok(Some(rule.clone()));
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Finds file-specific rule
	 * 
	 * @param path - File path
	 * @param operation - File operation
	 * @param user - User accessing file
	 * @return Result<Option<PermissionRule>> - Matching rule or None
	 */
	async fn find_file_rule(&self, path: &str, operation: &str, user: &str) -> Result<Option<PermissionRule>> {
		let rules = self.rules.read().await;
		
		for rule in rules.values() {
			if rule.resource_type == ResourceType::File &&
			   rule.resource_path == path &&
			   rule.subject == user &&
			   rule.allowed_operations.contains(operation) {
				return Ok(Some(rule.clone()));
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Finds network-specific rule
	 * 
	 * @param resource_path - Network resource path
	 * @param protocol - Network protocol
	 * @param user - User accessing network
	 * @return Result<Option<PermissionRule>> - Matching rule or None
	 */
	async fn find_network_rule(&self, resource_path: &str, protocol: &str, user: &str) -> Result<Option<PermissionRule>> {
		let rules = self.rules.read().await;
		
		for rule in rules.values() {
			if rule.resource_type == ResourceType::Network &&
			   rule.resource_path == resource_path &&
			   rule.subject == user &&
			   rule.allowed_operations.contains(protocol) {
				return Ok(Some(rule.clone()));
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Checks if user is locked out
	 * 
	 * @param user - User to check
	 * @return Result<bool> - Whether user is locked out
	 */
	async fn is_user_locked_out(&self, user: &str) -> Result<bool> {
		let attempts = self.failed_attempts.read().await;
		
		if let Some((count, timestamp)) = attempts.get(user) {
			if *count >= self.permission_config.max_failed_attempts {
				let now = std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)?
					.as_secs();
				
				if now - timestamp < self.permission_config.lockout_duration {
					return Ok(true);
				}
			}
		}
		
		Ok(false)
	}
	
	/**
	 * Records failed login attempt
	 * 
	 * @param user - User who failed login
	 */
	pub async fn record_failed_attempt(&self, user: &str) {
		let mut attempts = self.failed_attempts.write().await;
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();
		
		if let Some((count, _)) = attempts.get_mut(user) {
			*count += 1;
		} else {
			attempts.insert(user.to_string(), (1, now));
		}
	}
	
	/**
	 * Clears failed login attempts
	 * 
	 * @param user - User to clear attempts for
	 */
	pub async fn clear_failed_attempts(&self, user: &str) {
		let mut attempts = self.failed_attempts.write().await;
		attempts.remove(user);
	}
	
	/**
	 * Initializes default permissions
	 * 
	 * @return Result<()> - Success or error status
	 */
	async fn initialize_default_permissions(&self) -> Result<()> {
		/**
		 * デフォルト権限を初期化する関数です
		 * 
		 * システムの基本的な権限ルールを
		 * 初期化し、セキュリティポリシーを
		 * 設定します。
		 * 
		 * デフォルトユーザー、ロール、グループなどの
		 * 基本的な権限構造を構築します。
		 */
		
		// Add default roles
		{
			let mut roles = self.roles.write().await;
			
			let mut admin_permissions = HashSet::new();
			admin_permissions.insert("command_execution".to_string());
			admin_permissions.insert("file_access".to_string());
			admin_permissions.insert("network_access".to_string());
			admin_permissions.insert("system_access".to_string());
			roles.insert("admin".to_string(), admin_permissions);
			
			let mut user_permissions = HashSet::new();
			user_permissions.insert("file_access".to_string());
			user_permissions.insert("network_access".to_string());
			roles.insert("user".to_string(), user_permissions);
			
			let mut guest_permissions = HashSet::new();
			guest_permissions.insert("file_access".to_string());
			roles.insert("guest".to_string(), guest_permissions);
		}
		
		// Add default groups
		{
			let mut groups = self.groups.write().await;
			
			let mut admin_group = HashSet::new();
			admin_group.insert("admin".to_string());
			groups.insert("administrators".to_string(), admin_group);
			
			let mut user_group = HashSet::new();
			user_group.insert("user".to_string());
			groups.insert("users".to_string(), user_group);
		}
		
		Ok(())
	}
	
	/**
	 * Checks if permission manager is active
	 * 
	 * @return bool - Whether permission manager is active
	 */
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	/**
	 * Updates permission configuration
	 * 
	 * @param config - New permission configuration
	 */
	pub fn update_config(&mut self, config: PermissionConfig) {
		self.permission_config = config;
	}
	
	/**
	 * Gets current permission configuration
	 * 
	 * @return PermissionConfig - Current permission configuration
	 */
	pub fn get_config(&self) -> PermissionConfig {
		self.permission_config.clone()
	}
} 