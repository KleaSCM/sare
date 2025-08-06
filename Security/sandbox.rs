/**
 * Process Sandboxing System for Sare Terminal
 * 
 * This module provides comprehensive process sandboxing capabilities,
 * including namespace isolation, resource limits, and security containment
 * to prevent malicious processes from affecting the system.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: sandbox.rs
 * Description: Process sandboxing and isolation system
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;
use nix::unistd::{setuid, setgid, Uid, Gid};
use nix::sched::{unshare, CloneFlags};
use nix::sys::resource::{setrlimit, Resource, Rlimit};

use super::{SecurityConfig, SecurityEvent, SecuritySeverity};

/**
 * Sandbox configuration
 * 
 * サンドボックス設定を管理する構造体です。
 * プロセス分離、リソース制限、セキュリティ
 * コンテナの設定を提供します。
 */
#[derive(Debug, Clone)]
pub struct SandboxConfig {
	/// Enable namespace isolation
	pub namespace_isolation: bool,
	/// Enable resource limits
	pub resource_limits: bool,
	/// Enable user/group isolation
	pub user_isolation: bool,
	/// Enable filesystem isolation
	pub filesystem_isolation: bool,
	/// Enable network isolation
	pub network_isolation: bool,
	/// Maximum CPU time (seconds)
	pub max_cpu_time: u64,
	/// Maximum memory usage (bytes)
	pub max_memory: u64,
	/// Maximum file size (bytes)
	pub max_file_size: u64,
	/// Maximum number of processes
	pub max_processes: u64,
	/// Maximum number of open files
	pub max_open_files: u64,
	/// Allowed directories
	pub allowed_directories: Vec<String>,
	/// Blocked directories
	pub blocked_directories: Vec<String>,
	/// Allowed system calls
	pub allowed_syscalls: Vec<String>,
	/// Blocked system calls
	pub blocked_syscalls: Vec<String>,
}

impl Default for SandboxConfig {
	fn default() -> Self {
		Self {
			namespace_isolation: true,
			resource_limits: true,
			user_isolation: true,
			filesystem_isolation: true,
			network_isolation: false,
			max_cpu_time: 300, // 5 minutes
			max_memory: 512 * 1024 * 1024, // 512MB
			max_file_size: 100 * 1024 * 1024, // 100MB
			max_processes: 10,
			max_open_files: 100,
			allowed_directories: vec![
				"/tmp".to_string(),
				"/home".to_string(),
				"/usr/bin".to_string(),
				"/usr/lib".to_string(),
			],
			blocked_directories: vec![
				"/etc".to_string(),
				"/var".to_string(),
				"/sys".to_string(),
				"/proc".to_string(),
				"/dev".to_string(),
			],
			allowed_syscalls: vec![
				"read".to_string(), "write".to_string(), "open".to_string(),
				"close".to_string(), "stat".to_string(), "lstat".to_string(),
				"fstat".to_string(), "access".to_string(), "chdir".to_string(),
				"getcwd".to_string(), "chmod".to_string(), "fchmod".to_string(),
				"umask".to_string(), "getuid".to_string(), "getgid".to_string(),
				"geteuid".to_string(), "getegid".to_string(), "setuid".to_string(),
				"setgid".to_string(), "getpid".to_string(), "getppid".to_string(),
				"getpgrp".to_string(), "setpgid".to_string(), "setsid".to_string(),
				"getsid".to_string(), "exit".to_string(), "exit_group".to_string(),
				"wait4".to_string(), "waitpid".to_string(), "clone".to_string(),
				"fork".to_string(), "vfork".to_string(), "execve".to_string(),
				"execveat".to_string(), "kill".to_string(), "sigaction".to_string(),
				"sigprocmask".to_string(), "sigreturn".to_string(), "rt_sigaction".to_string(),
				"rt_sigprocmask".to_string(), "rt_sigreturn".to_string(), "sigaltstack".to_string(),
				"nanosleep".to_string(), "clock_gettime".to_string(), "clock_getres".to_string(),
				"gettimeofday".to_string(), "settimeofday".to_string(), "adjtimex".to_string(),
				"getrlimit".to_string(), "setrlimit".to_string(), "getrusage".to_string(),
				"times".to_string(), "ptrace".to_string(), "getuid".to_string(),
				"syslog".to_string(), "getgid".to_string(), "setuid".to_string(),
				"setgid".to_string(), "geteuid".to_string(), "getegid".to_string(),
				"setpgid".to_string(), "getppid".to_string(), "getpgrp".to_string(),
				"setsid".to_string(), "setreuid".to_string(), "setregid".to_string(),
				"getgroups".to_string(), "setgroups".to_string(), "setresuid".to_string(),
				"getresuid".to_string(), "setresgid".to_string(), "getresgid".to_string(),
				"getpgid".to_string(), "setfsuid".to_string(), "setfsgid".to_string(),
				"getsid".to_string(), "capget".to_string(), "capset".to_string(),
				"rt_sigpending".to_string(), "rt_sigtimedwait".to_string(),
				"rt_sigqueueinfo".to_string(), "rt_sigsuspend".to_string(),
				"sigaltstack".to_string(), "utime".to_string(), "mknod".to_string(),
				"uselib".to_string(), "personality".to_string(), "ustat".to_string(),
				"statfs".to_string(), "fstatfs".to_string(), "sysfs".to_string(),
				"getpriority".to_string(), "setpriority".to_string(), "sched_setparam".to_string(),
				"sched_getparam".to_string(), "sched_setscheduler".to_string(),
				"sched_getscheduler".to_string(), "sched_get_priority_max".to_string(),
				"sched_get_priority_min".to_string(), "sched_rr_get_interval".to_string(),
				"mlock".to_string(), "munlock".to_string(), "mlockall".to_string(),
				"munlockall".to_string(), "vhangup".to_string(), "modify_ldt".to_string(),
				"pivot_root".to_string(), "_sysctl".to_string(), "prctl".to_string(),
				"arch_prctl".to_string(), "adjtimex".to_string(), "setrlimit".to_string(),
				"chroot".to_string(), "sync".to_string(), "acct".to_string(),
				"settimeofday".to_string(), "mount".to_string(), "umount2".to_string(),
				"swapon".to_string(), "swapoff".to_string(), "reboot".to_string(),
				"sethostname".to_string(), "setdomainname".to_string(),
				"iopl".to_string(), "ioperm".to_string(), "create_module".to_string(),
				"init_module".to_string(), "delete_module".to_string(),
				"get_kernel_syms".to_string(), "query_module".to_string(),
				"quotactl".to_string(), "nfsservctl".to_string(), "getpmsg".to_string(),
				"putpmsg".to_string(), "afs_syscall".to_string(), "tuxcall".to_string(),
				"security".to_string(), "gettid".to_string(), "readahead".to_string(),
				"setxattr".to_string(), "lsetxattr".to_string(), "fsetxattr".to_string(),
				"getxattr".to_string(), "lgetxattr".to_string(), "fgetxattr".to_string(),
				"listxattr".to_string(), "llistxattr".to_string(), "flistxattr".to_string(),
				"removexattr".to_string(), "lremovexattr".to_string(), "fremovexattr".to_string(),
				"tkill".to_string(), "time".to_string(), "futex".to_string(),
				"sched_setaffinity".to_string(), "sched_getaffinity".to_string(),
				"set_thread_area".to_string(), "io_setup".to_string(), "io_destroy".to_string(),
				"io_getevents".to_string(), "io_submit".to_string(), "io_cancel".to_string(),
				"get_thread_area".to_string(), "lookup_dcookie".to_string(),
				"epoll_create".to_string(), "epoll_ctl_old".to_string(), "epoll_wait_old".to_string(),
				"remap_file_pages".to_string(), "getdents64".to_string(),
				"set_tid_address".to_string(), "restart_syscall".to_string(),
				"semtimedop".to_string(), "fadvise64".to_string(), "timer_create".to_string(),
				"timer_settime".to_string(), "timer_gettime".to_string(), "timer_getoverrun".to_string(),
				"timer_delete".to_string(), "clock_settime".to_string(), "clock_gettime".to_string(),
				"clock_getres".to_string(), "clock_nanosleep".to_string(), "exit_group".to_string(),
				"epoll_wait".to_string(), "epoll_ctl".to_string(), "tgkill".to_string(),
				"utimes".to_string(), "vserver".to_string(), "mbind".to_string(),
				"set_mempolicy".to_string(), "get_mempolicy".to_string(), "mq_open".to_string(),
				"mq_unlink".to_string(), "mq_timedsend".to_string(), "mq_timedreceive".to_string(),
				"mq_notify".to_string(), "mq_getsetattr".to_string(), "kexec_load".to_string(),
				"waitid".to_string(), "add_key".to_string(), "request_key".to_string(),
				"keyctl".to_string(), "ioprio_set".to_string(), "ioprio_get".to_string(),
				"inotify_init".to_string(), "inotify_add_watch".to_string(),
				"inotify_rm_watch".to_string(), "migrate_pages".to_string(),
				"openat".to_string(), "mkdirat".to_string(), "mknodat".to_string(),
				"fchownat".to_string(), "futimesat".to_string(), "newfstatat".to_string(),
				"unlinkat".to_string(), "renameat".to_string(), "linkat".to_string(),
				"symlinkat".to_string(), "readlinkat".to_string(), "fchmodat".to_string(),
				"faccessat".to_string(), "pselect6".to_string(), "ppoll".to_string(),
				"unshare".to_string(), "set_robust_list".to_string(), "get_robust_list".to_string(),
				"splice".to_string(), "tee".to_string(), "sync_file_range".to_string(),
				"vmsplice".to_string(), "move_pages".to_string(), "utimensat".to_string(),
				"epoll_pwait".to_string(), "signalfd".to_string(), "timerfd_create".to_string(),
				"eventfd".to_string(), "fallocate".to_string(), "timerfd_settime".to_string(),
				"timerfd_gettime".to_string(), "accept4".to_string(), "signalfd4".to_string(),
				"eventfd2".to_string(), "epoll_create1".to_string(), "dup3".to_string(),
				"pipe2".to_string(), "inotify_init1".to_string(), "preadv".to_string(),
				"pwritev".to_string(), "rt_tgsigqueueinfo".to_string(), "perf_event_open".to_string(),
				"recvmmsg".to_string(), "fanotify_init".to_string(), "fanotify_mark".to_string(),
				"prlimit64".to_string(), "name_to_handle_at".to_string(), "open_by_handle_at".to_string(),
				"clock_adjtime".to_string(), "syncfs".to_string(), "sendmmsg".to_string(),
				"setns".to_string(), "getcpu".to_string(), "process_vm_readv".to_string(),
				"process_vm_writev".to_string(), "kcmp".to_string(), "finit_module".to_string(),
				"sched_setattr".to_string(), "sched_getattr".to_string(), "renameat2".to_string(),
				"seccomp".to_string(), "getrandom".to_string(), "memfd_create".to_string(),
				"kexec_file_load".to_string(), "bpf".to_string(), "execveat".to_string(),
				"userfaultfd".to_string(), "membarrier".to_string(), "mlock2".to_string(),
				"copy_file_range".to_string(), "preadv2".to_string(), "pwritev2".to_string(),
				"pkey_mprotect".to_string(), "pkey_alloc".to_string(), "pkey_free".to_string(),
				"statx".to_string(), "io_pgetevents".to_string(), "rseq".to_string(),
				"pidfd_send_signal".to_string(), "io_uring_setup".to_string(),
				"io_uring_enter".to_string(), "io_uring_register".to_string(),
				"open_tree".to_string(), "move_mount".to_string(), "fsopen".to_string(),
				"fsconfig".to_string(), "fsmount".to_string(), "fspick".to_string(),
				"pidfd_open".to_string(), "clone3".to_string(), "close_range".to_string(),
				"openat2".to_string(), "pidfd_getfd".to_string(), "faccessat2".to_string(),
				"process_madvise".to_string(), "epoll_pwait2".to_string(), "mount_setattr".to_string(),
				"quotactl_fd".to_string(), "landlock_create_ruleset".to_string(),
				"landlock_add_rule".to_string(), "landlock_restrict_self".to_string(),
				"memfd_secret".to_string(), "process_mrelease".to_string(),
				"futex_waitv".to_string(), "set_mempolicy_home_node".to_string(),
			],
			blocked_syscalls: vec![
				"ptrace".to_string(), "personality".to_string(), "modify_ldt".to_string(),
				"arch_prctl".to_string(), "set_tid_address".to_string(), "restart_syscall".to_string(),
				"exit_group".to_string(), "unshare".to_string(), "set_robust_list".to_string(),
				"get_robust_list".to_string(), "splice".to_string(), "tee".to_string(),
				"sync_file_range".to_string(), "vmsplice".to_string(), "move_pages".to_string(),
				"utimensat".to_string(), "epoll_pwait".to_string(), "signalfd".to_string(),
				"timerfd_create".to_string(), "eventfd".to_string(), "fallocate".to_string(),
				"timerfd_settime".to_string(), "timerfd_gettime".to_string(), "accept4".to_string(),
				"signalfd4".to_string(), "eventfd2".to_string(), "epoll_create1".to_string(),
				"dup3".to_string(), "pipe2".to_string(), "inotify_init1".to_string(),
				"preadv".to_string(), "pwritev".to_string(), "rt_tgsigqueueinfo".to_string(),
				"perf_event_open".to_string(), "recvmmsg".to_string(), "fanotify_init".to_string(),
				"fanotify_mark".to_string(), "prlimit64".to_string(), "name_to_handle_at".to_string(),
				"open_by_handle_at".to_string(), "clock_adjtime".to_string(), "syncfs".to_string(),
				"sendmmsg".to_string(), "setns".to_string(), "getcpu".to_string(),
				"process_vm_readv".to_string(), "process_vm_writev".to_string(), "kcmp".to_string(),
				"finit_module".to_string(), "sched_setattr".to_string(), "sched_getattr".to_string(),
				"renameat2".to_string(), "seccomp".to_string(), "getrandom".to_string(),
				"memfd_create".to_string(), "kexec_file_load".to_string(), "bpf".to_string(),
				"execveat".to_string(), "userfaultfd".to_string(), "membarrier".to_string(),
				"mlock2".to_string(), "copy_file_range".to_string(), "preadv2".to_string(),
				"pwritev2".to_string(), "pkey_mprotect".to_string(), "pkey_alloc".to_string(),
				"pkey_free".to_string(), "statx".to_string(), "io_pgetevents".to_string(),
				"rseq".to_string(), "pidfd_send_signal".to_string(), "io_uring_setup".to_string(),
				"io_uring_enter".to_string(), "io_uring_register".to_string(), "open_tree".to_string(),
				"move_mount".to_string(), "fsopen".to_string(), "fsconfig".to_string(),
				"fsmount".to_string(), "fspick".to_string(), "pidfd_open".to_string(),
				"clone3".to_string(), "close_range".to_string(), "openat2".to_string(),
				"pidfd_getfd".to_string(), "faccessat2".to_string(), "process_madvise".to_string(),
				"epoll_pwait2".to_string(), "mount_setattr".to_string(), "quotactl_fd".to_string(),
				"landlock_create_ruleset".to_string(), "landlock_add_rule".to_string(),
				"landlock_restrict_self".to_string(), "memfd_secret".to_string(),
				"process_mrelease".to_string(), "futex_waitv".to_string(),
				"set_mempolicy_home_node".to_string(),
			],
		}
	}
}

/**
 * Sandboxed process information
 * 
 * サンドボックス化されたプロセスの情報を管理する構造体です。
 * プロセスID、コマンド、ユーザー、開始時刻などの
 * 情報を保持します。
 */
#[derive(Debug, Clone)]
pub struct SandboxedProcess {
	/// Process ID
	pub pid: u32,
	/// Command being executed
	pub command: String,
	/// User executing the command
	pub user: String,
	/// Start time
	pub start_time: u64,
	/// Process status
	pub status: ProcessStatus,
	/// Resource usage
	pub resource_usage: ResourceUsage,
}

/**
 * Process status
 */
#[derive(Debug, Clone)]
pub enum ProcessStatus {
	/// Process is running
	Running,
	/// Process has completed
	Completed,
	/// Process was terminated
	Terminated,
	/// Process was killed
	Killed,
	/// Process failed to start
	Failed,
}

/**
 * Resource usage information
 * 
 * リソース使用量の情報を管理する構造体です。
 * CPU時間、メモリ使用量、ファイルサイズなどの
 * リソース使用量を追跡します。
 */
#[derive(Debug, Clone)]
pub struct ResourceUsage {
	/// CPU time used (seconds)
	pub cpu_time: f64,
	/// Memory usage (bytes)
	pub memory_usage: u64,
	/// File size (bytes)
	pub file_size: u64,
	/// Number of processes
	pub process_count: u32,
	/// Number of open files
	pub open_files: u32,
}

impl Default for ResourceUsage {
	fn default() -> Self {
		Self {
			cpu_time: 0.0,
			memory_usage: 0,
			file_size: 0,
			process_count: 0,
			open_files: 0,
		}
	}
}

/**
 * Sandbox manager for process isolation
 * 
 * プロセス分離のためのサンドボックスマネージャーです。
 * プロセスのサンドボックス化、リソース制限、
 * セキュリティ監視を提供します。
 */
pub struct SandboxManager {
	/// Sandbox configuration
	config: Arc<RwLock<SecurityConfig>>,
	/// Sandbox configuration
	sandbox_config: SandboxConfig,
	/// Active sandboxed processes
	processes: Arc<RwLock<HashMap<u32, SandboxedProcess>>>,
	/// Active state
	active: bool,
}

impl SandboxManager {
	/**
	 * Creates a new sandbox manager
	 * 
	 * @param config - Security configuration
	 * @return Result<SandboxManager> - New sandbox manager or error
	 */
	pub async fn new(config: Arc<RwLock<SecurityConfig>>) -> Result<Self> {
		/**
		 * サンドボックスマネージャーを初期化する関数です
		 * 
		 * 指定された設定でサンドボックスマネージャーを作成し、
		 * プロセス分離とリソース制限の機能を提供します。
		 * 
		 * 名前空間分離、ユーザー分離、ファイルシステム分離などの
		 * セキュリティ機能を統合して安全なプロセス実行環境を
		 * 構築します。
		 */
		
		Ok(Self {
			config,
			sandbox_config: SandboxConfig::default(),
			processes: Arc::new(RwLock::new(HashMap::new())),
			active: true,
		})
	}
	
	/**
	 * Creates a sandboxed process
	 * 
	 * @param command - Command to execute
	 * @param user - User executing the command
	 * @return Result<u32> - Process ID or error
	 */
	pub async fn create_process(&self, command: &str, user: &str) -> Result<u32> {
		/**
		 * サンドボックス化されたプロセスを作成する関数です
		 * 
		 * 指定されたコマンドをサンドボックス環境で実行し、
		 * リソース制限とセキュリティ監視を適用します。
		 * 
		 * 名前空間分離、ユーザー分離、ファイルシステム分離を
		 * 使用して安全なプロセス実行環境を構築し、
		 * プロセスIDを返します。
		 */
		
		// Parse command
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Err(anyhow::anyhow!("Empty command"));
		}
		
		let program = parts[0];
		let args = &parts[1..];
		
		// Create command with sandboxing
		let mut cmd = Command::new(program);
		cmd.args(args);
		
		// Set up process isolation
		if self.sandbox_config.namespace_isolation {
			cmd.before_exec(|| {
				// Create new namespace
				unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS)?;
				Ok(())
			});
		}
		
		// Set up user isolation
		if self.sandbox_config.user_isolation {
			cmd.before_exec(|| {
				// Drop privileges to nobody user
				setuid(Uid::from_raw(65534))?;
				setgid(Gid::from_raw(65534))?;
				Ok(())
			});
		}
		
		// Set up resource limits
		if self.sandbox_config.resource_limits {
			cmd.before_exec(|| {
				// Set CPU time limit
				setrlimit(Resource::RLIMIT_CPU, Rlimit::new(self.sandbox_config.max_cpu_time, self.sandbox_config.max_cpu_time))?;
				
				// Set memory limit
				setrlimit(Resource::RLIMIT_AS, Rlimit::new(self.sandbox_config.max_memory, self.sandbox_config.max_memory))?;
				
				// Set file size limit
				setrlimit(Resource::RLIMIT_FSIZE, Rlimit::new(self.sandbox_config.max_file_size, self.sandbox_config.max_file_size))?;
				
				// Set process limit
				setrlimit(Resource::RLIMIT_NPROC, Rlimit::new(self.sandbox_config.max_processes, self.sandbox_config.max_processes))?;
				
				// Set open files limit
				setrlimit(Resource::RLIMIT_NOFILE, Rlimit::new(self.sandbox_config.max_open_files, self.sandbox_config.max_open_files))?;
				
				Ok(())
			});
		}
		
		// Set up I/O
		cmd.stdin(Stdio::piped());
		cmd.stdout(Stdio::piped());
		cmd.stderr(Stdio::piped());
		
		// Spawn process
		let mut child = cmd.spawn()?;
		let pid = child.id();
		
		// Create sandboxed process info
		let process = SandboxedProcess {
			pid,
			command: command.to_string(),
			user: user.to_string(),
			start_time: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)?
				.as_secs(),
			status: ProcessStatus::Running,
			resource_usage: ResourceUsage::default(),
		};
		
		// Store process info
		{
			let mut processes = self.processes.write().await;
			processes.insert(pid, process);
		}
		
		// Start monitoring thread
		self.start_monitoring(pid).await?;
		
		Ok(pid)
	}
	
	/**
	 * Starts monitoring a sandboxed process
	 * 
	 * @param pid - Process ID to monitor
	 * @return Result<()> - Success or error status
	 */
	async fn start_monitoring(&self, pid: u32) -> Result<()> {
		/**
		 * サンドボックス化されたプロセスを監視する関数です
		 * 
		 * 指定されたプロセスIDのプロセスを監視し、
		 * リソース使用量とセキュリティ違反を追跡します。
		 * 
		 * CPU時間、メモリ使用量、ファイルアクセスなどの
		 * リソース使用量を監視し、制限を超えた場合は
		 * プロセスを終了します。
		 */
		
		let processes = self.processes.clone();
		let config = self.config.clone();
		
		tokio::spawn(async move {
			// Monitor process resource usage
			loop {
				tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
				
				// Check if process still exists
				if let Ok(mut processes_guard) = processes.try_write() {
					if let Some(process) = processes_guard.get_mut(&pid) {
						// Update resource usage
						if let Ok(usage) = Self::get_process_usage(pid).await {
							process.resource_usage = usage;
						}
						
						// Check resource limits
						if process.resource_usage.cpu_time > 300.0 {
							// Kill process if CPU time exceeded
							let _ = std::process::Command::new("kill")
								.arg("-9")
								.arg(pid.to_string())
								.output();
							process.status = ProcessStatus::Killed;
						}
						
						if process.resource_usage.memory_usage > 512 * 1024 * 1024 {
							// Kill process if memory usage exceeded
							let _ = std::process::Command::new("kill")
								.arg("-9")
								.arg(pid.to_string())
								.output();
							process.status = ProcessStatus::Killed;
						}
					} else {
						// Process no longer exists
						break;
					}
				}
			}
		});
		
		Ok(())
	}
	
	/**
	 * Gets process resource usage
	 * 
	 * @param pid - Process ID
	 * @return Result<ResourceUsage> - Resource usage or error
	 */
	async fn get_process_usage(pid: u32) -> Result<ResourceUsage> {
		// For now, return default usage
		// TODO: Implement actual process resource monitoring
		Ok(ResourceUsage::default())
	}
	
	/**
	 * Terminates a sandboxed process
	 * 
	 * @param pid - Process ID to terminate
	 * @return Result<()> - Success or error status
	 */
	pub async fn terminate_process(&self, pid: u32) -> Result<()> {
		// Kill process
		let _ = std::process::Command::new("kill")
			.arg("-9")
			.arg(pid.to_string())
			.output();
		
		// Update process status
		if let Ok(mut processes) = self.processes.try_write() {
			if let Some(process) = processes.get_mut(&pid) {
				process.status = ProcessStatus::Terminated;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets all sandboxed processes
	 * 
	 * @return Vec<SandboxedProcess> - List of sandboxed processes
	 */
	pub async fn get_processes(&self) -> Vec<SandboxedProcess> {
		let processes = self.processes.read().await;
		processes.values().cloned().collect()
	}
	
	/**
	 * Checks if sandbox is active
	 * 
	 * @return bool - Whether sandbox is active
	 */
	pub async fn is_active(&self) -> bool {
		self.active
	}
	
	/**
	 * Updates sandbox configuration
	 * 
	 * @param config - New sandbox configuration
	 */
	pub fn update_config(&mut self, config: SandboxConfig) {
		self.sandbox_config = config;
	}
	
	/**
	 * Gets current sandbox configuration
	 * 
	 * @return SandboxConfig - Current sandbox configuration
	 */
	pub fn get_config(&self) -> SandboxConfig {
		self.sandbox_config.clone()
	}
} 