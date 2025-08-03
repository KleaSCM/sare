/**
 * @file mod.rs
 * @brief Core shell functionality module
 * 
 * This module contains the main shell logic including command parsing,
 * execution, job control, and built-in command implementations.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file mod.rs
 * @description Core shell module that handles command parsing, execution,
 * job management, and built-in command implementations for the Sare shell.
 */

pub mod parser;
pub mod executor;
pub mod job;
pub mod builtins;

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use job::JobManager;
use parser::CommandParser;
use executor::CommandExecutor;
use builtins::BuiltinCommands;

/**
 * Main shell structure that manages the shell state
 * 
 * Contains the current working directory, job manager,
 * command history, and execution state.
 */
pub struct Shell {
    /// Current working directory
    current_path: PathBuf,
    /// Job management system
    job_manager: JobManager,
    /// Command parser instance
    parser: CommandParser,
    /// Command executor instance
    executor: CommandExecutor,
    /// Built-in command handlers
    builtins: BuiltinCommands,
    /// Current command input buffer
    input_buffer: String,
    /// Command output history
    output_history: Vec<String>,
    /// Environment variables
    environment: HashMap<String, String>,
}

impl Shell {
    /**
     * Creates a new shell instance
     * 
     * Initializes the shell with default settings, loads environment
     * variables, and sets up the initial working directory.
     * 
     * @return Result<Shell> - New shell instance or error
     */
    pub fn new() -> Result<Self> {
        let current_path = std::env::current_dir()?;
        let mut environment = HashMap::new();
        
        for (key, value) in std::env::vars() {
            environment.insert(key, value);
        }
        
        Ok(Self {
            current_path,
            job_manager: JobManager::new(),
            parser: CommandParser::new(),
            executor: CommandExecutor::new(),
            builtins: BuiltinCommands::new(),
            input_buffer: String::new(),
            output_history: Vec::new(),
            environment,
        })
    }
    
    /**
     * Gets the current working directory
     * 
     * @return &PathBuf - Reference to current path
     */
    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }
    
    /**
     * Adds a character to the input buffer
     * 
     * @param c - Character to add
     */
    pub fn add_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }
    
    /**
     * Removes the last character from the input buffer
     */
    pub fn remove_char(&mut self) {
        self.input_buffer.pop();
    }
    
    /**
     * Gets the current input buffer content
     * 
     * @return &str - Current input text
     */
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }
    
    /**
     * Gets the output history for display
     * 
     * @return Vec<Line> - Formatted output lines
     */
    pub fn get_output(&self) -> Vec<ratatui::text::Line> {
        self.output_history
            .iter()
            .map(|line| ratatui::text::Line::from(line.clone()))
            .collect()
    }
    
    /**
     * Handles Ctrl+C signal
     * 
     * Interrupts the current foreground job and clears the input buffer.
     */
    pub fn handle_ctrl_c(&mut self) {
        self.job_manager.interrupt_current_job();
        self.input_buffer.clear();
    }
    
    /**
     * Executes the current command in the input buffer
     * 
     * Parses the command, determines if it's a built-in or external
     * command, and executes it accordingly.
     * 
     * @return Result<()> - Success or error status
     */
    pub async fn execute_command(&mut self) -> Result<()> {
        let command = self.input_buffer.trim();
        if command.is_empty() {
            self.input_buffer.clear();
            return Ok(());
        }
        
        let parsed = self.parser.parse(command)?;
        
        let result = self.execute_parsed_command(&parsed).await?;
        self.output_history.push(result);
        
        self.input_buffer.clear();
        
        Ok(())
    }
    
    /**
     * パースされたコマンドを実行する関数です (｡◕‿◕｡)
     * 
     * この関数は複雑な借用チェックの問題を解決するために、
     * ビルトインコマンドと外部コマンドを分けて処理します。
     * Rustのライフタイム管理が難しい部分なので、
     * 文字列のクローンを使用して借用の競合を避けています (◕‿◕)
     * 
     * @param parsed - 実行するパースされたコマンド
     * @return Result<String> - コマンドの出力またはエラー
     */
    async fn execute_parsed_command(&mut self, parsed: &crate::shell::parser::ParsedCommand) -> Result<String> {
        match parsed.command.as_str() {
            "pwd" => Ok(self.current_path().to_string_lossy().to_string()),
            "echo" => Ok(parsed.args.join(" ")),
            "help" => Ok(r#"
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
"#.to_string()),
            _ => {
                let parsed_clone = parsed.clone();
                let is_builtin = matches!(parsed.command.as_str(), "cd" | "exit" | "clear" | "history" | "jobs" | "kill");
                
                if is_builtin {
                    match parsed.command.as_str() {
                        "cd" => {
                            let path = parsed.args.first().unwrap_or(&"~".to_string()).clone();
                            self.change_directory(&path)?;
                            Ok(format!("Changed directory to: {}", self.current_path().display()))
                        }
                        "exit" => {
                            let exit_code = parsed.args.first().and_then(|arg| arg.parse::<i32>().ok()).unwrap_or(0);
                            std::process::exit(exit_code);
                        }
                        "clear" => Ok("".to_string()),
                        "history" => Ok("History command not yet implemented".to_string()),
                        "jobs" => {
                            let jobs = self.job_manager.get_jobs();
                            if jobs.is_empty() {
                                Ok("No background jobs".to_string())
                            } else {
                                let mut output = String::new();
                                for job in jobs {
                                    let status = match job.state {
                                        crate::shell::job::JobState::Running => "Running",
                                        crate::shell::job::JobState::Completed => "Completed",
                                        crate::shell::job::JobState::Terminated => "Terminated",
                                        crate::shell::job::JobState::Suspended => "Suspended",
                                    };
                                    output.push_str(&format!("[{}] {} {} {}\n", job.id, status, job.pid, job.command));
                                }
                                Ok(output)
                            }
                        }
                        "kill" => {
                            if let Some(job_id_str) = parsed.args.first() {
                                if let Ok(job_id) = job_id_str.parse::<u32>() {
                                    self.job_manager.kill_job(job_id)?;
                                    Ok(format!("Killed job {}", job_id))
                                } else {
                                    Err(anyhow::anyhow!("Invalid job ID: {}", job_id_str))
                                }
                            } else {
                                Err(anyhow::anyhow!("Usage: kill <job_id>"))
                            }
                        }
                        _ => Err(anyhow::anyhow!("Unknown built-in command"))
                    }
                } else {
                    self.executor.execute(&parsed_clone, &self.current_path).await
                }
            }
        }
    }
    
    /**
     * Changes the current working directory
     * 
     * @param path - New directory path
     * @return Result<()> - Success or error status
     */
    pub fn change_directory(&mut self, path: &str) -> Result<()> {
        let new_path = if path == "~" {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        } else {
            self.current_path.join(path)
        };
        
        if new_path.exists() && new_path.is_dir() {
            self.current_path = new_path.canonicalize()?;
            std::env::set_current_dir(&self.current_path)?;
        } else {
            return Err(anyhow::anyhow!("Directory not found: {}", path));
        }
        
        Ok(())
    }
} 