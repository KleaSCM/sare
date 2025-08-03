/**
 * @file process.rs
 * @brief Process and job control commands
 * 
 * This module implements process-related built-in commands
 * for job control and process management.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file process.rs
 * @description Process commands including jobs, kill, bg, fg, wait
 * with proper job control and signal handling.
 */

use anyhow::Result;
use crate::shell::parser::ParsedCommand;
use crate::shell::Shell;
use crate::shell::commands::{CommandHandler, CommandResult};

/**
 * Jobs command
 * 
 * Implements the jobs command for listing background jobs.
 * Shows job status, PID, and command information.
 */
pub struct JobsCommand;

impl CommandHandler for JobsCommand {
    fn execute(&self, _command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let jobs = shell.get_jobs();
        
        if jobs.is_empty() {
            return Ok(CommandResult {
                output: "No background jobs".to_string(),
                exit_code: 0,
            });
        }
        
        let mut output = String::new();
        for job in jobs {
            let status = match job.state {
                crate::shell::job::JobState::Running => "Running",
                crate::shell::job::JobState::Completed => "Completed",
                crate::shell::job::JobState::Terminated => "Terminated",
                crate::shell::job::JobState::Suspended => "Suspended",
            };
            
            output.push_str(&format!("[{}] {} {} {}\n", 
                job.id, status, job.pid, job.command));
        }
        
        Ok(CommandResult {
            output,
            exit_code: 0,
        })
    }
    
    fn help(&self) -> &str {
        "jobs [options] - List background jobs\n\
         Shows all background jobs with their status and PID.\n\
         Options:\n\
         -l    Show process IDs\n\
         -p    Show only process IDs"
    }
    
    fn name(&self) -> &str {
        "jobs"
    }
}

/**
 * Kill command
 * 
 * Implements the kill command for terminating processes.
 * Supports job-based and PID-based killing.
 */
pub struct KillCommand;

impl CommandHandler for KillCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        if command.args.is_empty() {
            return Err(anyhow::anyhow!("Usage: kill <job_id|pid>"));
        }
        
        let target = &command.args[0];
        
        if target.starts_with('%') {
            let job_id = target[1..].parse::<u32>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID: {}", target))?;
            
            shell.kill_job(job_id)?;
            
            Ok(CommandResult {
                output: format!("Killed job {}", job_id),
                exit_code: 0,
            })
        } else {
            let pid = target.parse::<u32>()
                .map_err(|_| anyhow::anyhow!("Invalid PID: {}", target))?;
            
            unsafe {
                if libc::kill(pid as libc::pid_t, libc::SIGTERM) != 0 {
                    return Err(anyhow::anyhow!("Failed to kill process {}: {}", 
                        pid, std::io::Error::last_os_error()));
                }
            }
            
            Ok(CommandResult {
                output: format!("Killed process {}", pid),
                exit_code: 0,
            })
        }
    }
    
    fn help(&self) -> &str {
        "kill <job_id|pid> - Kill processes or jobs\n\
         Usage: kill %1 (kill job 1)\n\
         Usage: kill 1234 (kill process 1234)\n\
         Usage: kill -9 %1 (force kill job 1)"
    }
    
    fn name(&self) -> &str {
        "kill"
    }
}

/**
 * Background command
 * 
 * Implements the bg command for resuming suspended jobs in background.
 */
pub struct BgCommand;

impl CommandHandler for BgCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let job_id = if command.args.is_empty() {
            shell.get_current_job()
        } else {
            let job_spec = &command.args[0];
            if job_spec.starts_with('%') {
                job_spec[1..].parse::<u32>().ok()
            } else {
                job_spec.parse::<u32>().ok()
            }
        };
        
        if let Some(job_id) = job_id {
            shell.resume_job_background(job_id)?;
            
            Ok(CommandResult {
                output: format!("Resumed job {} in background", job_id),
                exit_code: 0,
            })
        } else {
            Err(anyhow::anyhow!("No current job or invalid job specification"))
        }
    }
    
    fn help(&self) -> &str {
        "bg [job_id] - Resume job in background\n\
         Usage: bg (resume current job)\n\
         Usage: bg %1 (resume job 1 in background)"
    }
    
    fn name(&self) -> &str {
        "bg"
    }
}

/**
 * Foreground command
 * 
 * Implements the fg command for resuming jobs in foreground.
 */
pub struct FgCommand;

impl CommandHandler for FgCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let job_id = if command.args.is_empty() {
            shell.get_current_job()
        } else {
            let job_spec = &command.args[0];
            if job_spec.starts_with('%') {
                job_spec[1..].parse::<u32>().ok()
            } else {
                job_spec.parse::<u32>().ok()
            }
        };
        
        if let Some(job_id) = job_id {
            shell.resume_job_foreground(job_id)?;
            
            Ok(CommandResult {
                output: format!("Resumed job {} in foreground", job_id),
                exit_code: 0,
            })
        } else {
            Err(anyhow::anyhow!("No current job or invalid job specification"))
        }
    }
    
    fn help(&self) -> &str {
        "fg [job_id] - Resume job in foreground\n\
         Usage: fg (resume current job)\n\
         Usage: fg %1 (resume job 1 in foreground)"
    }
    
    fn name(&self) -> &str {
        "fg"
    }
}

/**
 * Wait command
 * 
 * Implements the wait command for waiting for job completion.
 */
pub struct WaitCommand;

impl CommandHandler for WaitCommand {
    fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<CommandResult> {
        let job_id = if command.args.is_empty() {
            shell.get_current_job()
        } else {
            let job_spec = &command.args[0];
            if job_spec.starts_with('%') {
                job_spec[1..].parse::<u32>().ok()
            } else {
                job_spec.parse::<u32>().ok()
            }
        };
        
        if let Some(job_id) = job_id {

            // TODO: Implement actual job waiting
            Ok(CommandResult {
                output: format!("Waiting for job {} to complete...", job_id),
                exit_code: 0,
            })
        } else {
            Err(anyhow::anyhow!("No current job or invalid job specification"))
        }
    }
    
    fn help(&self) -> &str {
        "wait [job_id] - Wait for job completion\n\
         Usage: wait (wait for current job)\n\
         Usage: wait %1 (wait for job 1 to complete)"
    }
    
    fn name(&self) -> &str {
        "wait"
    }
} 