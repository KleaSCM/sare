/**
 * @file builtins.rs
 * @brief Built-in command implementations
 * 
 * This module contains implementations of built-in shell commands
 * such as cd, exit, clear, history, pwd, echo, help, jobs, and kill.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file builtins.rs
 * @description Built-in command implementations that provide core shell
 * functionality without requiring external processes.
 */

use anyhow::Result;
use crate::shell::{Shell, parser::ParsedCommand};

/**
 * Built-in command handler
 * 
 * Provides implementations for all built-in shell commands
 * and manages their execution within the shell environment.
 */
pub struct BuiltinCommands;

impl BuiltinCommands {
    /**
     * Creates a new built-in commands handler
     * 
     * @return BuiltinCommands - New built-in commands instance
     */
    pub fn new() -> Self {
        Self
    }
    
    /**
     * Executes a built-in command
     * 
     * @param command - Parsed command to execute
     * @param shell - Shell instance for context
     * @return Result<Option<String>> - Command output or None if not a built-in
     */
    pub async fn execute(&self, command: &ParsedCommand, shell: &mut Shell) -> Result<Option<String>> {
        match command.command.as_str() {
            "cd" => Self::cmd_cd(command, shell),
            "exit" => Self::cmd_exit(command, shell),
            "clear" => Self::cmd_clear(command, shell),
            "history" => Self::cmd_history(command, shell),
            "pwd" => Self::cmd_pwd(command, shell),
            "echo" => Self::cmd_echo(command, shell),
            "help" => Self::cmd_help(command, shell),
            "jobs" => Self::cmd_jobs(command, shell),
            "kill" => Self::cmd_kill(command, shell),
            _ => Ok(None),
        }
    }
    
    /**
     * Built-in cd command implementation
     * 
     * Changes the current working directory.
     * 
     * @param command - Parsed command with arguments
     * @param shell - Shell instance
     * @return Result<Option<String>> - Success message or error
     */
    fn cmd_cd(command: &ParsedCommand, shell: &mut Shell) -> Result<Option<String>> {
        		/**
		 * ディレクトリを変更する関数です
		 * 
		 * 指定されたパスにディレクトリを変更し、成功時は
		 * 新しいディレクトリパスを返します。
		 * 
		 * パスをトリムして潜在的な破損を除去し、シェルの
		 * change_directory()メソッドを使用してディレクトリを
		 * 変更します。エラー時は適切なエラーメッセージを返します。
		 * 
		 * 引数が指定されていない場合はホームディレクトリ（~）に
		 * 変更します。
		 */
        
        let path = command.args.first().unwrap_or(&"~".to_string()).clone();
        
        // Clean the path to remove any potential corruption
        let clean_path = path.trim();
        
        match shell.change_directory(clean_path) {
            Ok(_) => Ok(Some(format!("Changed directory to: {}", shell.current_path().display()))),
            Err(e) => {
                let error_msg = format!("cd: {}: {}", clean_path, e);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }
    
    /**
     * Built-in exit command implementation
     * 
     * Exits the shell with optional exit code.
     * 
     * @param command - Parsed command with arguments
     * @param _shell - Shell instance (unused)
     * @return Result<Option<String>> - Never returns (exits process)
     */
    fn cmd_exit(command: &ParsedCommand, _shell: &mut Shell) -> Result<Option<String>> {
        let exit_code = command.args
            .first()
            .and_then(|arg| arg.parse::<i32>().ok())
            .unwrap_or(0);
        
        std::process::exit(exit_code);
    }
    
    /**
     * Built-in clear command implementation
     * 
     * Clears the terminal screen.
     * 
     * @param _command - Parsed command (unused)
     * @param _shell - Shell instance (unused)
     * @return Result<Option<String>> - Empty string for clear effect
     */
    fn cmd_clear(_command: &ParsedCommand, _shell: &mut Shell) -> Result<Option<String>> {
        Ok(Some("".to_string()))
    }
    
    /**
     * Built-in history command implementation
     * 
     * Displays command history.
     * 
     * @param _command - Parsed command (unused)
     * @param _shell - Shell instance (unused)
     * @return Result<Option<String>> - History output
     */
    fn cmd_history(_command: &ParsedCommand, _shell: &mut Shell) -> Result<Option<String>> {
        Ok(Some("History command not yet implemented".to_string()))
    }
    
    /**
     * Built-in pwd command implementation
     * 
     * Prints the current working directory.
     * 
     * @param _command - Parsed command (unused)
     * @param shell - Shell instance
     * @return Result<Option<String>> - Current directory path
     */
    fn cmd_pwd(_command: &ParsedCommand, shell: &mut Shell) -> Result<Option<String>> {
        Ok(Some(shell.current_path().to_string_lossy().to_string()))
    }
    
    /**
     * Built-in echo command implementation
     * 
     * Prints arguments to stdout.
     * 
     * @param command - Parsed command with arguments
     * @param _shell - Shell instance (unused)
     * @return Result<Option<String>> - Echoed arguments
     */
    fn cmd_echo(command: &ParsedCommand, _shell: &mut Shell) -> Result<Option<String>> {
        let output = command.args.join(" ");
        Ok(Some(output))
    }
    
    /**
     * Built-in help command implementation
     * 
     * Displays help information for built-in commands.
     * 
     * @param _command - Parsed command (unused)
     * @param _shell - Shell instance (unused)
     * @return Result<Option<String>> - Help text
     */
    fn cmd_help(_command: &ParsedCommand, _shell: &mut Shell) -> Result<Option<String>> {
        let help_text = r#"
Sare Shell - Built-in Commands

cd [directory]     - Change directory
exit [code]        - Exit shell with optional code
clear              - Clear terminal screen
history            - Show command history
pwd                - Print working directory
echo [args...]     - Print arguments
help               - Show this help
jobs               - List background jobs
kill [job_id]      - Kill background job

External commands are also supported.
"#;
        Ok(Some(help_text.to_string()))
    }
    
    /**
     * Built-in jobs command implementation
     * 
     * Lists all background jobs.
     * 
     * @param _command - Parsed command (unused)
     * @param shell - Shell instance
     * @return Result<Option<String>> - Jobs list
     */
    fn cmd_jobs(_command: &ParsedCommand, shell: &mut Shell) -> Result<Option<String>> {
        let jobs = shell.job_manager.get_jobs();
        if jobs.is_empty() {
            Ok(Some("No background jobs".to_string()))
        } else {
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
            Ok(Some(output))
        }
    }
    
    /**
     * Built-in kill command implementation
     * 
     * Kills a background job by ID.
     * 
     * @param command - Parsed command with job ID
     * @param shell - Shell instance
     * @return Result<Option<String>> - Success or error message
     */
    fn cmd_kill(command: &ParsedCommand, shell: &mut Shell) -> Result<Option<String>> {
        if let Some(job_id_str) = command.args.first() {
            if let Ok(job_id) = job_id_str.parse::<u32>() {
                shell.job_manager.kill_job(job_id)?;
                Ok(Some(format!("Killed job {}", job_id)))
            } else {
                Err(anyhow::anyhow!("Invalid job ID: {}", job_id_str))
            }
        } else {
            Err(anyhow::anyhow!("Usage: kill <job_id>"))
        }
    }
} 