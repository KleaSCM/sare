/**
 * Process sandboxing module
 * 
 * This module provides process sandboxing capabilities including namespace isolation,
 * resource limits, and user/group isolation for secure process execution.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: sandbox.rs
 * Description: Process sandboxing with namespace isolation and resource limits
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use nix::unistd::{unshare, setuid, setgid};
use nix::sched::CloneFlags;
use nix::sys::resource::{setrlimit, Resource, Rlimit};
use libc::{uid_t, gid_t};

use super::{SecurityConfig, SecurityEvent, SecuritySeverity};

/**
 * Sandbox configuration
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
	/// Enable namespace isolation
	pub namespace_isolation: bool,
	/// Enable user isolation
	pub user_isolation: bool,
	/// Enable resource limits
	pub resource_limits: bool,
	/// Maximum CPU time (seconds)
	pub max_cpu_time: u64,
	/// Maximum memory (bytes)
	pub max_memory: u64,
	/// Maximum file size (bytes)
	pub max_file_size: u64,
	/// Maximum number of processes
	pub max_processes: u64,
	/// Maximum number of open files
	pub max_open_files: u64,
	/// Allowed directories
	pub allowed_directories: Vec<String>,
	/// Blocked system calls
	pub blocked_syscalls: Vec<String>,
}

impl Default for SandboxConfig {
	fn default() -> Self {
		Self {
			namespace_isolation: true,
			user_isolation: true,
			resource_limits: true,
			max_cpu_time: 300, // 5 minutes
			max_memory: 512 * 1024 * 1024, // 512MB
			max_file_size: 100 * 1024 * 1024, // 100MB
			max_processes: 10,
			max_open_files: 100,
			allowed_directories: vec![
				"/tmp".to_string(),
				"/home".to_string(),
				"/var/tmp".to_string(),
			],
			blocked_syscalls: vec![
				"execve".to_string(),
				"fork".to_string(),
				"clone".to_string(),
				"kill".to_string(),
				"ptrace".to_string(),
			],
		}
	}
}

/**
 * Sandboxed process information
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxedProcess {
	/// Process ID
	pub pid: u32,
	/// Command being executed
	pub command: String,
	/// User running the process
	pub user: String,
	/// Process status
	pub status: ProcessStatus,
	/// Resource usage
	pub resource_usage: ResourceUsage,
	/// Start time
	pub start_time: u64,
	/// End time (if terminated)
	pub end_time: Option<u64>,
	/// Exit code (if terminated)
	pub exit_code: Option<i32>,
	/// Security violations
	pub security_violations: Vec<SecurityViolation>,
}

/**
 * Process status
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
	/// Process is running
	Running,
	/// Process has completed
	Completed,
	/// Process was terminated
	Terminated,
	/// Process was suspended
	Suspended,
	/// Process failed to start
	Failed,
}

/**
 * Resource usage information
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
	/// CPU time used (seconds)
	pub cpu_time: f64,
	/// Memory usage (bytes)
	pub memory_usage: u64,
	/// Disk I/O operations
	pub disk_io: u64,
	/// Network I/O operations
	pub network_io: u64,
	/// Number of open files
	pub open_files: u32,
	/// Number of child processes
	pub child_processes: u32,
}

/**
 * Security violation
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
	/// Violation type
	pub violation_type: String,
	/// Description
	pub description: String,
	/// Timestamp
	pub timestamp: u64,
	/// Severity
	pub severity: SecuritySeverity,
}

/**
 * Sandbox manager
 */
pub struct SandboxManager {
	/// Security configuration
	config: Arc<RwLock<SecurityConfig>>,
	/// Sandbox configuration
	sandbox_config: SandboxConfig,
	/// Active sandboxed processes
	processes: Arc<RwLock<HashMap<u32, SandboxedProcess>>>,
	/// Process counter
	process_counter: Arc<RwLock<u32>>,
	/// Active state
	active: bool,
}

impl SandboxManager {
	/**
	 * Creates a new sandbox manager
	 */
	pub async fn new(config: Arc<RwLock<SecurityConfig>>) -> Result<Self> {
		let sandbox_config = SandboxConfig::default();
		
		Ok(Self {
			config,
			sandbox_config,
			processes: Arc::new(RwLock::new(HashMap::new())),
			process_counter: Arc::new(RwLock::new(1)),
			active: true,
		})
	}
	
	/**
	 * Creates a sandboxed process
	 */
	pub async fn create_process(&self, command: &str, user: &str) -> Result<u32> {
		let pid = {
			let mut counter = self.process_counter.write().await;
			*counter += 1;
			*counter
		};
		
		let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
		
		// Create sandboxed process with isolation
		let mut child_process = Command::new("unshare")
			.args(&["--pid", "--mount", "--net", "--uts", "--ipc"])
			.arg("--")
			.arg("sh")
			.arg("-c")
			.arg(command)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?;
		
		// Set resource limits
		if self.sandbox_config.resource_limits {
			self.set_resource_limits(child_process.id())?;
		}
		
		// Create sandboxed process record
		let sandboxed_process = SandboxedProcess {
			pid: child_process.id(),
			command: command.to_string(),
			user: user.to_string(),
			status: ProcessStatus::Running,
			resource_usage: ResourceUsage {
				cpu_time: 0.0,
				memory_usage: 0,
				disk_io: 0,
				network_io: 0,
				open_files: 0,
				child_processes: 0,
			},
			start_time,
			end_time: None,
			exit_code: None,
			security_violations: Vec::new(),
		};
		
		// Store process
		self.processes.write().await.insert(pid, sandboxed_process);
		
		// Start monitoring thread
		self.start_monitoring(pid).await?;
		
		Ok(pid)
	}
	
	/**
	 * Sets resource limits for a process
	 */
	fn set_resource_limits(&self, pid: u32) -> Result<()> {
		// Set CPU time limit
		let cpu_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_cpu_time,
			rlim_max: self.sandbox_config.max_cpu_time,
		};
		setrlimit(Resource::RLIMIT_CPU, cpu_limit)?;
		
		// Set memory limit
		let memory_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_memory,
			rlim_max: self.sandbox_config.max_memory,
		};
		setrlimit(Resource::RLIMIT_AS, memory_limit)?;
		
		// Set file size limit
		let file_size_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_file_size,
			rlim_max: self.sandbox_config.max_file_size,
		};
		setrlimit(Resource::RLIMIT_FSIZE, file_size_limit)?;
		
		// Set process limit
		let process_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_processes,
			rlim_max: self.sandbox_config.max_processes,
		};
		setrlimit(Resource::RLIMIT_NPROC, process_limit)?;
		
		// Set open files limit
		let open_files_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_open_files,
			rlim_max: self.sandbox_config.max_open_files,
		};
		setrlimit(Resource::RLIMIT_NOFILE, open_files_limit)?;
		
		Ok(())
	}
	
	/**
	 * Starts monitoring for a process
	 */
	async fn start_monitoring(&self, pid: u32) -> Result<()> {
		let processes = self.processes.clone();
		let config = self.config.clone();
		
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
			
			loop {
				interval.tick().await;
				
				if let Ok(mut processes_guard) = processes.try_write() {
					if let Some(process) = processes_guard.get_mut(&pid) {
						// Update resource usage
						if let Ok(usage) = Self::get_process_usage(pid).await {
							process.resource_usage = usage;
						}
						
						// Check for violations
						if let Some(violation) = Self::check_security_violations(process, &config).await {
							process.security_violations.push(violation);
							
							// Terminate process if critical violation
							if violation.severity == SecuritySeverity::Critical {
								Self::terminate_process(pid).await;
								process.status = ProcessStatus::Terminated;
								process.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
							}
						}
						
						// Check if process is still running
						if !Self::is_process_running(pid).await {
							process.status = ProcessStatus::Completed;
							process.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
							break;
						}
					} else {
						break;
					}
				}
			}
		});
		
		Ok(())
	}
	
	/**
	 * Gets resource usage for a process
	 */
	async fn get_process_usage(pid: u32) -> Result<ResourceUsage> {
		// Read /proc/{pid}/stat for CPU and memory info
		let stat_path = format!("/proc/{}/stat", pid);
		let stat_content = std::fs::read_to_string(stat_path)?;
		let stat_fields: Vec<&str> = stat_content.split_whitespace().collect();
		
		// Parse CPU time (fields 13 and 14)
		let utime: u64 = stat_fields.get(13).unwrap_or(&"0").parse().unwrap_or(0);
		let stime: u64 = stat_fields.get(14).unwrap_or(&"0").parse().unwrap_or(0);
		let cpu_time = (utime + stime) as f64 / 100.0; // Convert to seconds
		
		// Parse memory usage (field 23)
		let memory_usage: u64 = stat_fields.get(23).unwrap_or(&"0").parse().unwrap_or(0) * 4096; // Convert pages to bytes
		
		// Count open files
		let open_files = std::fs::read_dir(format!("/proc/{}/fd", pid))
			.map(|entries| entries.count() as u32)
			.unwrap_or(0);
		
		// Count child processes
		let child_processes = std::fs::read_dir("/proc")
			.map(|entries| {
				entries
					.filter_map(|entry| entry.ok())
					.filter_map(|entry| entry.file_name().into_string().ok())
					.filter_map(|name| name.parse::<u32>().ok())
					.filter_map(|child_pid| {
						// Check if this is a child of our process
						if let Ok(stat_content) = std::fs::read_to_string(format!("/proc/{}/stat", child_pid)) {
							let fields: Vec<&str> = stat_content.split_whitespace().collect();
							if let Some(ppid_str) = fields.get(3) {
								if let Ok(ppid) = ppid_str.parse::<u32>() {
									return ppid == pid;
								}
							}
						}
						None
					})
					.count() as u32
			})
			.unwrap_or(0);
		
		Ok(ResourceUsage {
			cpu_time,
			memory_usage,
			disk_io: 0, // Would need to read /proc/{pid}/io
			network_io: 0, // Would need to read /proc/net/tcp
			open_files,
			child_processes,
		})
	}
	
	/**
	 * Checks for security violations
	 */
	async fn check_security_violations(process: &SandboxedProcess, config: &Arc<RwLock<SecurityConfig>>) -> Option<SecurityViolation> {
		let config_guard = config.read().await;
		
		// Check memory usage
		if process.resource_usage.memory_usage > config_guard.max_file_size {
			return Some(SecurityViolation {
				violation_type: "memory_limit_exceeded".to_string(),
				description: format!("Memory usage {} exceeds limit {}", 
					process.resource_usage.memory_usage, config_guard.max_file_size),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::High,
			});
		}
		
		// Check CPU time
		if process.resource_usage.cpu_time > 300.0 { // 5 minutes
			return Some(SecurityViolation {
				violation_type: "cpu_time_exceeded".to_string(),
				description: format!("CPU time {} exceeds limit", process.resource_usage.cpu_time),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::Medium,
			});
		}
		
		// Check child processes
		if process.resource_usage.child_processes > 10 {
			return Some(SecurityViolation {
				violation_type: "too_many_child_processes".to_string(),
				description: format!("Too many child processes: {}", process.resource_usage.child_processes),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::High,
			});
		}
		
		// Check open files
		if process.resource_usage.open_files > 100 {
			return Some(SecurityViolation {
				violation_type: "too_many_open_files".to_string(),
				description: format!("Too many open files: {}", process.resource_usage.open_files),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::Medium,
			});
		}
		
		None
	}
	
	/**
	 * Checks if a process is still running
	 */
	async fn is_process_running(pid: u32) -> bool {
		std::fs::metadata(format!("/proc/{}", pid)).is_ok()
	}
	
	/**
	 * Terminates a process
	 */
	async fn terminate_process(pid: u32) {
		let _ = Command::new("kill")
			.args(&["-9", &pid.to_string()])
			.output();
	}
	
	/**
	 * Terminates a sandboxed process
	 */
	pub async fn terminate_process(&self, pid: u32) -> Result<()> {
		if let Some(process) = self.processes.write().await.get_mut(&pid) {
			process.status = ProcessStatus::Terminated;
			process.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
		}
		
		Self::terminate_process(pid).await;
		Ok(())
	}
	
	/**
	 * Gets all sandboxed processes
	 */
	pub async fn get_processes(&self) -> Vec<SandboxedProcess> {
		self.processes.read().await.values().cloned().collect()
	}
	
	/**
	 * Checks if sandbox is active
	 */
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	/**
	 * Updates sandbox configuration
	 */
	pub fn update_config(&mut self, config: SandboxConfig) {
		self.sandbox_config = config;
	}
	
	/**
	 * Gets current configuration
	 */
	pub fn get_config(&self) -> SandboxConfig {
		self.sandbox_config.clone()
	}
} 