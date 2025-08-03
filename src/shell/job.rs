/**
 * @file job.rs
 * @brief Job control and management functionality
 * 
 * This module handles background job management, including job tracking,
 * process control, and job state management.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file job.rs
 * @description Job management system that handles background processes,
 * job tracking, and process control for the Sare shell.
 */

use anyhow::Result;
use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};
use libc::{pid_t, SIGINT, SIGTERM, SIGSTOP, SIGCONT};

/**
 * Represents the state of a job
 */
#[derive(Debug, Clone, PartialEq)]
pub enum JobState {
    /// Job is currently running
    Running,
    /// Job has completed successfully
    Completed,
    /// Job has been terminated
    Terminated,
    /// Job has been suspended
    Suspended,
}

/**
 * Represents a job in the shell
 * 
 * Contains information about a background process including
 * its PID, state, and command information.
 */
#[derive(Debug, Clone)]
pub struct Job {
    /// Unique job ID
    pub id: u32,
    /// Process ID
    pub pid: u32,
    /// Command that was executed
    pub command: String,
    /// Current state of the job
    pub state: JobState,
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
}

/**
 * Job manager that handles background job tracking
 * 
 * Provides functionality to track, control, and manage
 * background processes in the shell.
 */
pub struct JobManager {
    /// Map of job ID to job information
    jobs: HashMap<u32, Job>,
    /// Next available job ID
    next_job_id: u32,
    /// Currently active foreground job
    current_foreground: Option<u32>,
}

impl JobManager {
    /**
     * Creates a new job manager instance
     * 
     * @return JobManager - New job manager instance
     */
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            next_job_id: 1,
            current_foreground: None,
        }
    }
    
    /**
     * ジョブ追加の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なジョブ管理を行います。
     * プロセスID追跡と状態管理が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param pid - プロセスID
     * @param command - 実行されたコマンド
     * @return u32 - ジョブID
     */
    pub fn add_job(&mut self, pid: u32, command: String) -> u32 {
        let job_id = self.next_job_id;
        self.next_job_id += 1;
        
        let job = Job {
            id: job_id,
            pid,
            command,
            state: JobState::Running,
            exit_code: None,
        };
        
        self.jobs.insert(job_id, job);
        job_id
    }
    
    /**
     * Sets the current foreground job
     * 
     * @param job_id - ID of the foreground job
     */
    pub fn set_foreground_job(&mut self, job_id: u32) {
        self.current_foreground = Some(job_id);
    }
    
    /**
     * Clears the current foreground job
     */
    pub fn clear_foreground_job(&mut self) {
        self.current_foreground = None;
    }
    
    /**
     * Gets the current foreground job ID
     * 
     * @return Option<u32> - Current foreground job ID
     */
    pub fn get_foreground_job(&self) -> Option<u32> {
        self.current_foreground
    }
    
    /**
     * 現在のフォアグラウンドジョブを中断する関数です (◕‿◕)
     * 
     * この関数は複雑なプロセス制御を行います。
     * libcを使用してシグナル送信を行う難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     */
    pub fn interrupt_current_job(&mut self) {
        if let Some(job_id) = self.current_foreground {
            if let Some(job) = self.jobs.get_mut(&job_id) {
                if job.state == JobState::Running {
                    unsafe {
                        if libc::kill(job.pid as pid_t, SIGINT) != 0 {
                            eprintln!("Failed to interrupt job {}: {}", job_id, std::io::Error::last_os_error());
                        }
                    }
                }
            }
        }
    }
    
    /**
     * シグナル処理の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なシグナル処理を行います。
     * リアルタイムシグナル配信が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param signal - 送信するシグナル番号
     * @param pid - 対象プロセスID
     * @return Result<()> - 成功またはエラー
     */
    pub fn send_signal_to_job(&mut self, job_id: u32, signal: i32) -> Result<()> {
        if let Some(job) = self.jobs.get(&job_id) {
            if job.state == JobState::Running {
                unsafe {
                    if libc::kill(job.pid as pid_t, signal) != 0 {
                        return Err(anyhow::anyhow!("Failed to send signal {} to job {}: {}", 
                            signal, job_id, std::io::Error::last_os_error()));
                    }
                }
            }
        }
        Ok(())
    }
    
    /**
     * リアルタイムシグナル処理の複雑な処理です (◕‿◕)
     * 
     * この関数は複雑なリアルタイムシグナル処理を行います。
     * プロセス状態監視が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     * 
     * @param pid - 監視するプロセスID
     * @return Result<()> - 成功またはエラー
     */
    pub fn monitor_job_status(&mut self, job_id: u32) -> Result<()> {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            unsafe {
                let mut status = 0;
                let result = libc::waitpid(job.pid as pid_t, &mut status, libc::WNOHANG);
                
                if result > 0 {
                    if libc::WIFEXITED(status) {
                        job.state = JobState::Completed;
                        job.exit_code = Some(libc::WEXITSTATUS(status));
                    } else if libc::WIFSIGNALED(status) {
                        job.state = JobState::Terminated;
                        job.exit_code = Some(libc::WTERMSIG(status));
                    }
                }
            }
        }
        Ok(())
    }
    
    /**
     * Gets all jobs
     * 
     * @return Vec<&Job> - List of all jobs
     */
    pub fn get_jobs(&self) -> Vec<&Job> {
        self.jobs.values().collect()
    }
    
    /**
     * Gets a specific job by ID
     * 
     * @param job_id - Job ID to retrieve
     * @return Option<&Job> - Job if found
     */
    pub fn get_job(&self, job_id: u32) -> Option<&Job> {
        self.jobs.get(&job_id)
    }
    
    /**
     * Removes a completed job
     * 
     * @param job_id - Job ID to remove
     */
    pub fn remove_job(&mut self, job_id: u32) {
        self.jobs.remove(&job_id);
    }
    
    /**
     * Updates the state of a job
     * 
     * @param job_id - Job ID to update
     * @param state - New state
     * @param exit_code - Exit code (if completed)
     */
    pub fn update_job_state(&mut self, job_id: u32, state: JobState, exit_code: Option<i32>) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.state = state;
            job.exit_code = exit_code;
        }
    }
    
    /**
     * Kills a job
     * 
     * @param job_id - Job ID to kill
     * @return Result<()> - Success or error
     */
    pub fn kill_job(&mut self, job_id: u32) -> Result<()> {
        if let Some(job) = self.jobs.get(&job_id) {
            if job.state == JobState::Running {
                unsafe {
                    if libc::kill(job.pid as pid_t, SIGTERM) != 0 {
                        return Err(anyhow::anyhow!("Failed to kill job {}: {}", job_id, std::io::Error::last_os_error()));
                    }
                }
                self.update_job_state(job_id, JobState::Terminated, None);
            }
        }
        Ok(())
    }
    
    /**
     * Suspends a job
     * 
     * @param job_id - Job ID to suspend
     * @return Result<()> - Success or error
     */
    pub fn suspend_job(&mut self, job_id: u32) -> Result<()> {
        if let Some(job) = self.jobs.get(&job_id) {
            if job.state == JobState::Running {
                unsafe {
                    if libc::kill(job.pid as pid_t, SIGSTOP) != 0 {
                        return Err(anyhow::anyhow!("Failed to suspend job {}: {}", job_id, std::io::Error::last_os_error()));
                    }
                }
                self.update_job_state(job_id, JobState::Suspended, None);
            }
        }
        Ok(())
    }
    
    /**
     * Resumes a suspended job
     * 
     * @param job_id - Job ID to resume
     * @return Result<()> - Success or error
     */
    pub fn resume_job(&mut self, job_id: u32) -> Result<()> {
        if let Some(job) = self.jobs.get(&job_id) {
            if job.state == JobState::Suspended {
                unsafe {
                    if libc::kill(job.pid as pid_t, SIGCONT) != 0 {
                        return Err(anyhow::anyhow!("Failed to resume job {}: {}", job_id, std::io::Error::last_os_error()));
                    }
                }
                self.update_job_state(job_id, JobState::Running, None);
            }
        }
        Ok(())
    }
} 

/**
 * Signal handler for the shell
 * 
 * Handles various signals like SIGINT, SIGTERM, SIGSTOP
 * and manages signal delivery to child processes.
 */
pub struct SignalHandler {
    /// Currently registered signal handlers
    handlers: std::collections::HashMap<i32, Box<dyn Fn() + Send + Sync>>,
}

impl SignalHandler {
    /**
     * Creates a new signal handler
     * 
     * @return SignalHandler - New signal handler instance
     */
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    /**
     * シグナル処理の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なシグナル制御を行います。
     * libcを使用したシグナルハンドラ登録が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param signal - 処理するシグナル番号
     * @param handler - シグナルハンドラ関数
     */
    pub fn register_handler(&mut self, signal: i32, handler: Box<dyn Fn() + Send + Sync>) {
        self.handlers.insert(signal, handler);
        
        unsafe {
            let action = libc::sigaction {
                sa_sigaction: signal_handler_wrapper as libc::sighandler_t,
                sa_mask: std::mem::zeroed(),
                sa_flags: libc::SA_SIGINFO,
                sa_restorer: None,
            };
            
            let mut old_action = std::mem::zeroed();
            if libc::sigaction(signal, &action, &mut old_action) != 0 {
                eprintln!("Failed to register signal handler for signal {}", signal);
            }
        }
    }
    
    /**
     * Sends a signal to a process
     * 
     * @param pid - Process ID to send signal to
     * @param signal - Signal to send
     * @return Result<()> - Success or error
     */
    pub fn send_signal(&self, pid: u32, signal: i32) -> Result<()> {
        unsafe {
            if libc::kill(pid as libc::pid_t, signal) != 0 {
                return Err(anyhow::anyhow!("Failed to send signal {} to process {}: {}", 
                    signal, pid, std::io::Error::last_os_error()));
            }
        }
        Ok(())
    }
    
    /**
     * Handles SIGINT (Ctrl+C)
     * 
     * @param job_manager - Job manager to interrupt current job
     */
    pub fn handle_sigint(&self, job_manager: &mut JobManager) {
        job_manager.interrupt_current_job();
    }
    
    /**
     * Handles SIGTSTP (Ctrl+Z)
     * 
     * @param job_manager - Job manager to suspend current job
     */
    pub fn handle_sigtstp(&self, job_manager: &mut JobManager) {
        if let Some(job_id) = job_manager.get_foreground_job() {
            if let Err(e) = job_manager.suspend_job(job_id) {
                eprintln!("Failed to suspend job {}: {}", job_id, e);
            }
        }
    }
}

/**
 * Global signal handler wrapper
 * 
 * This function is called by the system when a signal is received.
 * It delegates to the appropriate handler in the SignalHandler.
 */
extern "C" fn signal_handler_wrapper(signal: i32) {
    // This is a simplified wrapper - in a real implementation,
    // you'd need to store the handlers in a global context
    match signal {
        libc::SIGINT => {
            eprintln!("Received SIGINT (Ctrl+C)");
        }
        libc::SIGTSTP => {
            eprintln!("Received SIGTSTP (Ctrl+Z)");
        }
        _ => {
            eprintln!("Received signal {}", signal);
        }
    }
} 