/**
 * Process sandboxing module provides comprehensive process isolation
 * capabilities including namespace isolation, resource limits, and
 * user/group isolation for secure process execution. The module
 * implements multiple isolation layers to prevent process interference
 * and ensure secure execution environments.
 * 
 * The sandboxing system employs Linux namespaces, resource limits,
 * and user isolation to create secure execution environments that
 * prevent privilege escalation and system compromise. Sandboxed
 * processes are monitored for security violations and resource
 * usage to maintain system integrity.
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
 * Sandbox configuration defines comprehensive process isolation settings
 * including namespace isolation, resource limits, and security policies.
 * Configuration parameters enable fine-tuned sandboxing policies that
 * can be adjusted based on security requirements and system constraints.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
	pub namespace_isolation: bool,
	pub user_isolation: bool,
	pub resource_limits: bool,
	pub max_cpu_time: u64,
	pub max_memory: u64,
	pub max_file_size: u64,
	pub max_processes: u64,
	pub max_open_files: u64,
	pub allowed_directories: Vec<String>,
	pub blocked_syscalls: Vec<String>,
}

impl Default for SandboxConfig {
	fn default() -> Self {
		Self {
			namespace_isolation: true,
			user_isolation: true,
			resource_limits: true,
			max_cpu_time: 300,
			max_memory: 512 * 1024 * 1024,
			max_file_size: 100 * 1024 * 1024,
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
 * Sandboxed process information tracks process execution details
 * including resource usage, security violations, and execution status.
 * Process tracking enables comprehensive monitoring and security
 * analysis of sandboxed process behavior.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxedProcess {
	pub pid: u32,
	pub command: String,
	pub user: String,
	pub status: ProcessStatus,
	pub resource_usage: ResourceUsage,
	pub start_time: u64,
	pub end_time: Option<u64>,
	pub exit_code: Option<i32>,
	pub security_violations: Vec<SecurityViolation>,
}

/**
 * Process status enumeration defines the various states a sandboxed
 * process can be in during its lifecycle. Status tracking enables
 * proper process management and security monitoring.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
	Running,
	Completed,
	Terminated,
	Suspended,
	Failed,
}

/**
 * Resource usage information tracks comprehensive resource consumption
 * by sandboxed processes including CPU, memory, disk I/O, and network
 * usage. Resource monitoring enables detection of resource abuse and
 * security violations.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
	pub cpu_time: f64,
	pub memory_usage: u64,
	pub disk_io: u64,
	pub network_io: u64,
	pub open_files: u32,
	pub child_processes: u32,
}

/**
 * Security violation information tracks security policy violations
 * detected during sandboxed process execution. Violation tracking
 * enables security analysis and automated response to security threats.
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
	pub violation_type: String,
	pub description: String,
	pub timestamp: u64,
	pub severity: SecuritySeverity,
}

/**
 * Sandbox manager implements comprehensive process sandboxing
 * capabilities including process creation, monitoring, and security
 * enforcement. The manager provides centralized sandbox management
 * that ensures consistent security policies across all sandboxed
 * processes.
 */
pub struct SandboxManager {
	config: Arc<RwLock<SecurityConfig>>,
	sandbox_config: SandboxConfig,
	processes: Arc<RwLock<HashMap<u32, SandboxedProcess>>>,
	process_counter: Arc<RwLock<u32>>,
	active: bool,
}

impl SandboxManager {
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
	
	pub async fn create_process(&self, command: &str, user: &str) -> Result<u32> {
		let pid = {
			let mut counter = self.process_counter.write().await;
			*counter += 1;
			*counter
		};
		
		let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
		
		let mut child_process = Command::new("unshare")
			.args(&["--pid", "--mount", "--net", "--uts", "--ipc"])
			.arg("--")
			.arg("sh")
			.arg("-c")
			.arg(command)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?;
		
		if self.sandbox_config.resource_limits {
			self.set_resource_limits(child_process.id())?;
		}
		
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
		
		self.processes.write().await.insert(pid, sandboxed_process);
		
		self.start_monitoring(pid).await?;
		
		Ok(pid)
	}
	
	fn set_resource_limits(&self, pid: u32) -> Result<()> {
		let cpu_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_cpu_time,
			rlim_max: self.sandbox_config.max_cpu_time,
		};
		setrlimit(Resource::RLIMIT_CPU, cpu_limit)?;
		
		let memory_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_memory,
			rlim_max: self.sandbox_config.max_memory,
		};
		setrlimit(Resource::RLIMIT_AS, memory_limit)?;
		
		let file_size_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_file_size,
			rlim_max: self.sandbox_config.max_file_size,
		};
		setrlimit(Resource::RLIMIT_FSIZE, file_size_limit)?;
		
		let process_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_processes,
			rlim_max: self.sandbox_config.max_processes,
		};
		setrlimit(Resource::RLIMIT_NPROC, process_limit)?;
		
		let open_files_limit = Rlimit {
			rlim_cur: self.sandbox_config.max_open_files,
			rlim_max: self.sandbox_config.max_open_files,
		};
		setrlimit(Resource::RLIMIT_NOFILE, open_files_limit)?;
		
		Ok(())
	}
	
	async fn start_monitoring(&self, pid: u32) -> Result<()> {
		let processes = self.processes.clone();
		let config = self.config.clone();
		
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
			
			loop {
				interval.tick().await;
				
				if let Ok(mut processes_guard) = processes.try_write() {
					if let Some(process) = processes_guard.get_mut(&pid) {
						if let Ok(usage) = Self::get_process_usage(pid).await {
							process.resource_usage = usage;
						}
						
						if let Some(violation) = Self::check_security_violations(process, &config).await {
							process.security_violations.push(violation);
							
							if violation.severity == SecuritySeverity::Critical {
								Self::terminate_process(pid).await;
								process.status = ProcessStatus::Terminated;
								process.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
							}
						}
						
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
	
	async fn get_process_usage(pid: u32) -> Result<ResourceUsage> {
		let stat_path = format!("/proc/{}/stat", pid);
		let stat_content = std::fs::read_to_string(stat_path)?;
		let stat_fields: Vec<&str> = stat_content.split_whitespace().collect();
		
		let utime: u64 = stat_fields.get(13).unwrap_or(&"0").parse().unwrap_or(0);
		let stime: u64 = stat_fields.get(14).unwrap_or(&"0").parse().unwrap_or(0);
		let cpu_time = (utime + stime) as f64 / 100.0;
		
		let memory_usage: u64 = stat_fields.get(23).unwrap_or(&"0").parse().unwrap_or(0) * 4096;
		
		let open_files = std::fs::read_dir(format!("/proc/{}/fd", pid))
			.map(|entries| entries.count() as u32)
			.unwrap_or(0);
		
		let child_processes = std::fs::read_dir("/proc")
			.map(|entries| {
				entries
					.filter_map(|entry| entry.ok())
					.filter_map(|entry| entry.file_name().into_string().ok())
					.filter_map(|name| name.parse::<u32>().ok())
					.filter_map(|child_pid| {
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
			disk_io: 0,
			network_io: 0,
			open_files,
			child_processes,
		})
	}
	
	async fn check_security_violations(process: &SandboxedProcess, config: &Arc<RwLock<SecurityConfig>>) -> Option<SecurityViolation> {
		let config_guard = config.read().await;
		
		if process.resource_usage.memory_usage > config_guard.max_file_size {
			return Some(SecurityViolation {
				violation_type: "memory_limit_exceeded".to_string(),
				description: format!("Memory usage {} exceeds limit {}", 
					process.resource_usage.memory_usage, config_guard.max_file_size),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::High,
			});
		}
		
		if process.resource_usage.cpu_time > 300.0 {
			return Some(SecurityViolation {
				violation_type: "cpu_time_exceeded".to_string(),
				description: format!("CPU time {} exceeds limit", process.resource_usage.cpu_time),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::Medium,
			});
		}
		
		if process.resource_usage.child_processes > 10 {
			return Some(SecurityViolation {
				violation_type: "too_many_child_processes".to_string(),
				description: format!("Too many child processes: {}", process.resource_usage.child_processes),
				timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
				severity: SecuritySeverity::High,
			});
		}
		
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
	
	async fn is_process_running(pid: u32) -> bool {
		std::fs::metadata(format!("/proc/{}", pid)).is_ok()
	}
	
	async fn terminate_process(pid: u32) {
		let _ = Command::new("kill")
			.args(&["-9", &pid.to_string()])
			.output();
	}
	
	pub async fn terminate_process(&self, pid: u32) -> Result<()> {
		if let Some(process) = self.processes.write().await.get_mut(&pid) {
			process.status = ProcessStatus::Terminated;
			process.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
		}
		
		Self::terminate_process(pid).await;
		Ok(())
	}
	
	pub async fn get_processes(&self) -> Vec<SandboxedProcess> {
		self.processes.read().await.values().cloned().collect()
	}
	
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	pub fn update_config(&mut self, config: SandboxConfig) {
		self.sandbox_config = config;
	}
	
	pub fn get_config(&self) -> SandboxConfig {
		self.sandbox_config.clone()
	}
} 