/**
 * @file executor.rs
 * @brief Command execution functionality
 * 
 * This module handles execution of external commands using std::process::Command,
 * including input/output redirection and process management.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file executor.rs
 * @description Command executor that handles external command execution with
 * proper process management, I/O redirection, and error handling.
 */

use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};
use crate::shell::parser::ParsedCommand;

/**
 * Command executor that handles external command execution
 * 
 * Provides functionality to execute external commands with proper
 * process management, I/O redirection, and error handling.
 */
pub struct CommandExecutor {
    /// Default timeout for command execution (in seconds)
    timeout_seconds: u64,
}

impl CommandExecutor {
    /**
     * Creates a new command executor instance
     * 
     * @return CommandExecutor - New executor instance
     */
    pub fn new() -> Self {
        Self {
            timeout_seconds: 30,
        }
    }
    
    /**
     * Executes a parsed command
     * 
     * Creates a new process for the command, handles I/O redirection,
     * and captures the output for display in the TUI.
     * 
     * @param command - Parsed command to execute
     * @param working_dir - Working directory for the command
     * @return Result<String> - Command output or error
     */
    pub async fn execute(&self, command: &ParsedCommand, working_dir: &Path) -> Result<String> {
        let mut cmd = Command::new(&command.command);
        
        cmd.current_dir(working_dir);
        cmd.args(&command.args);
        
        if let Some(ref input_file) = command.input_redirect {
            let input_file = std::fs::File::open(input_file)?;
            cmd.stdin(Stdio::from(input_file));
        } else {
            cmd.stdin(Stdio::inherit());
        }
        
        if let Some(ref output_file) = command.output_redirect {
            let output_file = std::fs::File::create(output_file)?;
            let output_file_clone = output_file.try_clone()?;
            cmd.stdout(Stdio::from(output_file));
            cmd.stderr(Stdio::from(output_file_clone));
        } else {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }
        
        let output = if command.background {
            self.execute_background(cmd)?
        } else {
            self.execute_foreground(cmd).await?
        };
        
        Ok(output)
    }
    
    /**
     * Executes a command in the foreground
     * 
     * Waits for the command to complete and captures its output.
     * 
     * @param mut cmd - Command to execute
     * @return Result<String> - Command output or error
     */
    async fn execute_foreground(&self, mut cmd: Command) -> Result<String> {
        let output = cmd.output()?;
        
        let mut result = String::new();
        
        if !output.stdout.is_empty() {
            result.push_str(&String::from_utf8_lossy(&output.stdout));
        }
        
        if !output.stderr.is_empty() {
            result.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command failed with exit code: {}",
                output.status.code().unwrap_or(-1)
            ));
        }
        
        Ok(result)
    }
    
    /**
     * Executes a command in the background
     * 
     * Starts the command but doesn't wait for completion.
     * Returns immediately with a status message.
     * 
     * @param mut cmd - Command to execute
     * @return Result<String> - Status message or error
     */
    fn execute_background(&self, mut cmd: Command) -> Result<String> {
        let mut child = cmd.spawn()?;
        
        let pid = child.id();
        
        Ok(format!("[{}] Background process started with PID: {}", 
            std::process::id(), pid))
    }
    
    /**
     * Sets the timeout for command execution
     * 
     * @param seconds - Timeout in seconds
     */
    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout_seconds = seconds;
    }
    
    /**
     * Checks if a command exists in the system PATH
     * 
     * @param command - Command name to check
     * @return bool - True if command exists
     */
    pub fn command_exists(&self, command: &str) -> bool {
        if let Ok(path) = std::env::var("PATH") {
            for dir in path.split(':') {
                let command_path = std::path::Path::new(dir).join(command);
                if command_path.exists() && command_path.is_file() {
                    return true;
                }
            }
        }
        false
    }
} 