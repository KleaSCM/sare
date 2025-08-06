/**
 * Permission management module
 * 
 * This module provides comprehensive permission management and access control,
 * including user permissions, resource access control, and role-based security.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: permissions.rs
 * Description: Permission management with role-based access control
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{SecurityConfig, SecurityEvent, SecuritySeverity};

/**
 * Permission levels
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionLevel {
	/// No access
	Deny,
	/// Read-only access
	Read,
	/// Read and write access
	Write,
	/// Full access
	Full,
	/// Administrative access
	Admin,
}

/**
 * Resource types
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
	/// File system resource
	File,
	/// Network resource
	Network,
	/// Command resource
	Command,
	/// System resource
	System,
	/// User resource
	User,
}

/**
 * Permission rule
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
	/// Rule ID
	pub id: String,
	/// Resource type
	pub resource_type: ResourceType,
	/// Resource path
	pub resource_path: String,
	/// Permission level
	pub permission_level: PermissionLevel,
	/// Allowed users
	pub allowed_users: HashSet<String>,
	/// Allowed roles
	pub allowed_roles: HashSet<String>,
	/// Allowed groups
	pub allowed_groups: HashSet<String>,
	/// Denied users
	pub denied_users: HashSet<String>,
	/// Denied roles
	pub denied_roles: HashSet<String>,
	/// Denied groups
	pub denied_groups: HashSet<String>,
	/// Time restrictions
	pub time_restrictions: Option<TimeRestrictions>,
	/// IP restrictions
	pub ip_restrictions: Option<IpRestrictions>,
	/// Active state
	pub active: bool,
}

/**
 * Time restrictions
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
	/// Allowed days of week (0=Sunday, 6=Saturday)
	pub allowed_days: HashSet<u8>,
	/// Allowed hours (0-23)
	pub allowed_hours: HashSet<u8>,
	/// Allowed time range (start hour, end hour)
	pub time_range: Option<(u8, u8)>,
}

/**
 * IP restrictions
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRestrictions {
	/// Allowed IP addresses
	pub allowed_ips: HashSet<String>,
	/// Allowed IP ranges
	pub allowed_ranges: HashSet<String>,
	/// Denied IP addresses
	pub denied_ips: HashSet<String>,
	/// Denied IP ranges
	pub denied_ranges: HashSet<String>,
}

/**
 * User permissions
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
	/// User ID
	pub user_id: String,
	/// User roles
	pub roles: HashSet<String>,
	/// User groups
	pub groups: HashSet<String>,
	/// User permissions
	pub permissions: HashMap<String, PermissionLevel>,
	/// Failed login attempts
	pub failed_attempts: u32,
	/// Last failed attempt time
	pub last_failed_attempt: Option<u64>,
	/// Account locked until
	pub locked_until: Option<u64>,
	/// Account active
	pub active: bool,
}

/**
 * Permission configuration
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
	/// Enable permission system
	pub enabled: bool,
	/// Default permission level
	pub default_permission_level: PermissionLevel,
	/// Maximum failed attempts before lockout
	pub max_failed_attempts: u32,
	/// Lockout duration (seconds)
	pub lockout_duration: u64,
	/// Session timeout (seconds)
	pub session_timeout: u64,
	/// Enable role-based access control
	pub rbac_enabled: bool,
	/// Enable group-based access control
	pub gbac_enabled: bool,
	/// Enable time-based restrictions
	pub time_restrictions_enabled: bool,
	/// Enable IP-based restrictions
	pub ip_restrictions_enabled: bool,
}

impl Default for PermissionConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			default_permission_level: PermissionLevel::Read,
			max_failed_attempts: 5,
			lockout_duration: 300, // 5 minutes
			session_timeout: 3600, // 1 hour
			rbac_enabled: true,
			gbac_enabled: true,
			time_restrictions_enabled: true,
			ip_restrictions_enabled: true,
		}
	}
}

/**
 * Permission manager
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
	/// Failed attempts tracking
	failed_attempts: Arc<RwLock<HashMap<String, (u32, u64)>>>,
	/// Active state
	active: bool,
}

impl PermissionManager {
	/**
	 * Creates a new permission manager
	 */
	pub async fn new(config: Arc<RwLock<SecurityConfig>>) -> Result<Self> {
		let permission_config = PermissionConfig::default();
		
		let manager = Self {
			config,
			permission_config,
			rules: Arc::new(RwLock::new(HashMap::new())),
			users: Arc::new(RwLock::new(HashMap::new())),
			roles: Arc::new(RwLock::new(HashMap::new())),
			groups: Arc::new(RwLock::new(HashMap::new())),
			failed_attempts: Arc::new(RwLock::new(HashMap::new())),
			active: true,
		};
		
		// Initialize default permissions
		manager.initialize_default_permissions().await?;
		
		Ok(manager)
	}
	
	/**
	 * Checks if user can execute a command
	 */
	pub async fn can_execute_command(&self, command: &str, user: &str) -> Result<bool> {
		if !self.active || !self.permission_config.enabled {
			return Ok(true);
		}
		
		// Check if user is locked out
		if self.is_user_locked_out(user).await? {
			return Ok(false);
		}
		
		// Get user permissions
		let user_permissions = self.get_user_permissions(user).await?;
		
		// Check for explicit command rule
		if let Some(rule) = self.find_command_rule(command, user).await? {
			return Ok(self.check_rule_access(&rule, user, &user_permissions).await?);
		}
		
		// Check user's command permissions
		if let Some(level) = user_permissions.permissions.get("command") {
			match level {
				PermissionLevel::Deny => Ok(false),
				PermissionLevel::Read => Ok(false), // Read-only doesn't allow execution
				PermissionLevel::Write => Ok(true),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		} else {
			// Use default permission level
			Ok(self.permission_config.default_permission_level != PermissionLevel::Deny)
		}
	}
	
	/**
	 * Checks if user can access a file
	 */
	pub async fn can_access_file(&self, path: &str, operation: &str, user: &str) -> Result<bool> {
		if !self.active || !self.permission_config.enabled {
			return Ok(true);
		}
		
		// Check if user is locked out
		if self.is_user_locked_out(user).await? {
			return Ok(false);
		}
		
		// Get user permissions
		let user_permissions = self.get_user_permissions(user).await?;
		
		// Check for explicit file rule
		if let Some(rule) = self.find_file_rule(path, operation, user).await? {
			return Ok(self.check_rule_access(&rule, user, &user_permissions).await?);
		}
		
		// Check user's file permissions
		if let Some(level) = user_permissions.permissions.get("file") {
			match level {
				PermissionLevel::Deny => Ok(false),
				PermissionLevel::Read => Ok(operation == "read"),
				PermissionLevel::Write => Ok(operation == "read" || operation == "write"),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		} else {
			// Use default permission level
			Ok(self.permission_config.default_permission_level != PermissionLevel::Deny)
		}
	}
	
	/**
	 * Checks if user can access network
	 */
	pub async fn can_access_network(&self, host: &str, port: u16, protocol: &str, user: &str) -> Result<bool> {
		if !self.active || !self.permission_config.enabled {
			return Ok(true);
		}
		
		// Check if user is locked out
		if self.is_user_locked_out(user).await? {
			return Ok(false);
		}
		
		// Get user permissions
		let user_permissions = self.get_user_permissions(user).await?;
		
		// Check for explicit network rule
		let resource_path = format!("{}:{}", host, port);
		if let Some(rule) = self.find_network_rule(&resource_path, protocol, user).await? {
			return Ok(self.check_rule_access(&rule, user, &user_permissions).await?);
		}
		
		// Check user's network permissions
		if let Some(level) = user_permissions.permissions.get("network") {
			match level {
				PermissionLevel::Deny => Ok(false),
				PermissionLevel::Read => Ok(protocol == "http" || protocol == "https"),
				PermissionLevel::Write => Ok(true),
				PermissionLevel::Full => Ok(true),
				PermissionLevel::Admin => Ok(true),
			}
		} else {
			// Use default permission level
			Ok(self.permission_config.default_permission_level != PermissionLevel::Deny)
		}
	}
	
	/**
	 * Adds a permission rule
	 */
	pub async fn add_rule(&self, rule: PermissionRule) -> Result<()> {
		self.rules.write().await.insert(rule.id.clone(), rule);
		Ok(())
	}
	
	/**
	 * Removes a permission rule
	 */
	pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
		self.rules.write().await.remove(rule_id);
		Ok(())
	}
	
	/**
	 * Gets user permissions
	 */
	async fn get_user_permissions(&self, user: &str) -> Result<UserPermissions> {
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
				failed_attempts: 0,
				last_failed_attempt: None,
				locked_until: None,
				active: true,
			})
		}
	}
	
	/**
	 * Finds command rule
	 */
	async fn find_command_rule(&self, command: &str, user: &str) -> Result<Option<PermissionRule>> {
		let rules = self.rules.read().await;
		
		for rule in rules.values() {
			if rule.resource_type == ResourceType::Command && rule.active {
				// Check if command matches rule pattern
				if self.matches_pattern(command, &rule.resource_path) {
					// Check if user is allowed
					if rule.allowed_users.contains(user) {
						return Ok(Some(rule.clone()));
					}
					
					// Check if user is denied
					if rule.denied_users.contains(user) {
						return Ok(None);
					}
					
					// Check roles and groups
					let user_permissions = self.get_user_permissions(user).await?;
					if self.check_rule_access(rule, user, &user_permissions).await? {
						return Ok(Some(rule.clone()));
					}
				}
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Finds file rule
	 */
	async fn find_file_rule(&self, path: &str, operation: &str, user: &str) -> Result<Option<PermissionRule>> {
		let rules = self.rules.read().await;
		
		for rule in rules.values() {
			if rule.resource_type == ResourceType::File && rule.active {
				// Check if path matches rule pattern
				if self.matches_pattern(path, &rule.resource_path) {
					// Check if user is allowed
					if rule.allowed_users.contains(user) {
						return Ok(Some(rule.clone()));
					}
					
					// Check if user is denied
					if rule.denied_users.contains(user) {
						return Ok(None);
					}
					
					// Check roles and groups
					let user_permissions = self.get_user_permissions(user).await?;
					if self.check_rule_access(rule, user, &user_permissions).await? {
						return Ok(Some(rule.clone()));
					}
				}
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Finds network rule
	 */
	async fn find_network_rule(&self, resource_path: &str, protocol: &str, user: &str) -> Result<Option<PermissionRule>> {
		let rules = self.rules.read().await;
		
		for rule in rules.values() {
			if rule.resource_type == ResourceType::Network && rule.active {
				// Check if resource matches rule pattern
				if self.matches_pattern(resource_path, &rule.resource_path) {
					// Check if user is allowed
					if rule.allowed_users.contains(user) {
						return Ok(Some(rule.clone()));
					}
					
					// Check if user is denied
					if rule.denied_users.contains(user) {
						return Ok(None);
					}
					
					// Check roles and groups
					let user_permissions = self.get_user_permissions(user).await?;
					if self.check_rule_access(rule, user, &user_permissions).await? {
						return Ok(Some(rule.clone()));
					}
				}
			}
		}
		
		Ok(None)
	}
	
	/**
	 * Checks if user is locked out
	 */
	async fn is_user_locked_out(&self, user: &str) -> Result<bool> {
		let failed_attempts = self.failed_attempts.read().await;
		
		if let Some((attempts, last_attempt)) = failed_attempts.get(user) {
			let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
			let lockout_duration = self.permission_config.lockout_duration;
			
			if *attempts >= self.permission_config.max_failed_attempts {
				if now - last_attempt < lockout_duration {
					return Ok(true);
				}
			}
		}
		
		Ok(false)
	}
	
	/**
	 * Records a failed attempt
	 */
	pub async fn record_failed_attempt(&self, user: &str) {
		let mut failed_attempts = self.failed_attempts.write().await;
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
		
		let entry = failed_attempts.entry(user.to_string()).or_insert((0, now));
		entry.0 += 1;
		entry.1 = now;
	}
	
	/**
	 * Clears failed attempts for user
	 */
	pub async fn clear_failed_attempts(&self, user: &str) {
		self.failed_attempts.write().await.remove(user);
	}
	
	/**
	 * Checks rule access
	 */
	async fn check_rule_access(&self, rule: &PermissionRule, user: &str, user_permissions: &UserPermissions) -> Result<bool> {
		// Check time restrictions
		if let Some(time_restrictions) = &rule.time_restrictions {
			if !self.check_time_restrictions(time_restrictions).await? {
				return Ok(false);
			}
		}
		
		// Check IP restrictions
		if let Some(ip_restrictions) = &rule.ip_restrictions {
			if !self.check_ip_restrictions(ip_restrictions).await? {
				return Ok(false);
			}
		}
		
		// Check roles
		if !rule.allowed_roles.is_empty() {
			let has_role = user_permissions.roles.iter().any(|role| rule.allowed_roles.contains(role));
			if !has_role {
				return Ok(false);
			}
		}
		
		// Check groups
		if !rule.allowed_groups.is_empty() {
			let has_group = user_permissions.groups.iter().any(|group| rule.allowed_groups.contains(group));
			if !has_group {
				return Ok(false);
			}
		}
		
		// Check denied roles
		if !rule.denied_roles.is_empty() {
			let has_denied_role = user_permissions.roles.iter().any(|role| rule.denied_roles.contains(role));
			if has_denied_role {
				return Ok(false);
			}
		}
		
		// Check denied groups
		if !rule.denied_groups.is_empty() {
			let has_denied_group = user_permissions.groups.iter().any(|group| rule.denied_groups.contains(group));
			if has_denied_group {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Checks time restrictions
	 */
	async fn check_time_restrictions(&self, restrictions: &TimeRestrictions) -> Result<bool> {
		if !self.permission_config.time_restrictions_enabled {
			return Ok(true);
		}
		
		let now = chrono::Utc::now();
		let weekday = now.weekday().num_days_from_sunday();
		let hour = now.hour() as u8;
		
		// Check day restrictions
		if !restrictions.allowed_days.is_empty() && !restrictions.allowed_days.contains(&weekday) {
			return Ok(false);
		}
		
		// Check hour restrictions
		if !restrictions.allowed_hours.is_empty() && !restrictions.allowed_hours.contains(&hour) {
			return Ok(false);
		}
		
		// Check time range
		if let Some((start_hour, end_hour)) = restrictions.time_range {
			if hour < start_hour || hour > end_hour {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Checks IP restrictions
	 */
	async fn check_ip_restrictions(&self, restrictions: &IpRestrictions) -> Result<bool> {
		if !self.permission_config.ip_restrictions_enabled {
			return Ok(true);
		}
		
		// For now, use a default IP (in real implementation, get actual IP)
		let client_ip = "127.0.0.1";
		
		// Check denied IPs first
		if restrictions.denied_ips.contains(client_ip) {
			return Ok(false);
		}
		
		// Check denied ranges
		for range in &restrictions.denied_ranges {
			if self.ip_in_range(client_ip, range) {
				return Ok(false);
			}
		}
		
		// Check allowed IPs
		if !restrictions.allowed_ips.is_empty() && !restrictions.allowed_ips.contains(client_ip) {
			return Ok(false);
		}
		
		// Check allowed ranges
		if !restrictions.allowed_ranges.is_empty() {
			let in_allowed_range = restrictions.allowed_ranges.iter().any(|range| self.ip_in_range(client_ip, range));
			if !in_allowed_range {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Checks if IP is in range
	 */
	fn ip_in_range(&self, ip: &str, range: &str) -> bool {
		// Simple IP range check (in real implementation, use proper IP parsing)
		ip == range || range.contains(ip)
	}
	
	/**
	 * Checks if string matches pattern
	 */
	fn matches_pattern(&self, input: &str, pattern: &str) -> bool {
		// Simple pattern matching (in real implementation, use regex or glob)
		input.contains(pattern) || pattern.contains(input) || input == pattern
	}
	
	/**
	 * Initializes default permissions
	 */
	async fn initialize_default_permissions(&self) -> Result<()> {
		// Add default admin user
		let admin_user = UserPermissions {
			user_id: "admin".to_string(),
			roles: HashSet::from(["admin".to_string()]),
			groups: HashSet::from(["admin".to_string()]),
			permissions: HashMap::from([
				("command".to_string(), PermissionLevel::Admin),
				("file".to_string(), PermissionLevel::Admin),
				("network".to_string(), PermissionLevel::Admin),
				("system".to_string(), PermissionLevel::Admin),
			]),
			failed_attempts: 0,
			last_failed_attempt: None,
			locked_until: None,
			active: true,
		};
		
		self.users.write().await.insert("admin".to_string(), admin_user);
		
		// Add default roles
		let mut roles = self.roles.write().await;
		roles.insert("admin".to_string(), HashSet::from([
			"command".to_string(),
			"file".to_string(),
			"network".to_string(),
			"system".to_string(),
		]));
		roles.insert("user".to_string(), HashSet::from([
			"file".to_string(),
			"network".to_string(),
		]));
		roles.insert("guest".to_string(), HashSet::from([
			"file".to_string(),
		]));
		
		// Add default groups
		let mut groups = self.groups.write().await;
		groups.insert("admin".to_string(), HashSet::from(["admin".to_string()]));
		groups.insert("users".to_string(), HashSet::from(["user".to_string()]));
		groups.insert("guests".to_string(), HashSet::from(["guest".to_string()]));
		
		Ok(())
	}
	
	/**
	 * Checks if permission manager is active
	 */
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	/**
	 * Updates permission configuration
	 */
	pub fn update_config(&mut self, config: PermissionConfig) {
		self.permission_config = config;
	}
	
	/**
	 * Gets current configuration
	 */
	pub fn get_config(&self) -> PermissionConfig {
		self.permission_config.clone()
	}
} 