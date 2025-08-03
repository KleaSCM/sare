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
pub mod commands;

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use job::JobManager;
use parser::CommandParser;
use executor::CommandExecutor;
use builtins::BuiltinCommands;
use commands::{CommandRegistry, CommandHandler, CommandResult};

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
    /// Command registry for all built-in commands
    command_registry: CommandRegistry,
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
            command_registry: CommandRegistry::new(),
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
        // Try built-in command first, fall back to external
        let command_name = parsed.command.clone();
        
        // Try to execute as built-in command, fall back to external
        match self.command_registry.execute_safe(parsed) {
            Ok(result) => Ok(result.output),
            Err(_) => {
                let parsed_clone = parsed.clone();
                self.executor.execute(&parsed_clone, &self.current_path).await
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
    
    /**
     * Gets a mutable reference to the current path
     * 
     * @return &mut PathBuf - Mutable reference to current path
     */
    pub fn current_path_mut(&mut self) -> &mut PathBuf {
        &mut self.current_path
    }
    
    /**
     * Gets all jobs from the job manager
     * 
     * @return Vec<&Job> - List of all jobs
     */
    pub fn get_jobs(&self) -> Vec<&crate::shell::job::Job> {
        self.job_manager.get_jobs()
    }
    
    /**
     * Kills a job by ID
     * 
     * @param job_id - Job ID to kill
     * @return Result<()> - Success or error
     */
    pub fn kill_job(&mut self, job_id: u32) -> Result<()> {
        self.job_manager.kill_job(job_id)
    }
    
    /**
     * Gets the current job ID
     * 
     * @return Option<u32> - Current job ID if any
     */
    pub fn get_current_job(&self) -> Option<u32> {
        self.job_manager.get_foreground_job()
    }
    
    /**
     * Resumes a job in background
     * 
     * @param job_id - Job ID to resume
     * @return Result<()> - Success or error
     */
    pub fn resume_job_background(&mut self, job_id: u32) -> Result<()> {
        self.job_manager.resume_job(job_id)
    }
    
    /**
     * Resumes a job in foreground
     * 
     * @param job_id - Job ID to resume
     * @return Result<()> - Success or error
     */
    pub fn resume_job_foreground(&mut self, job_id: u32) -> Result<()> {
        self.job_manager.resume_job(job_id)
    }
    
    /**
     * Waits for a job to complete
     * 
     * @param job_id - Job ID to wait for
     * @return Result<()> - Success or error
     */
    pub async fn wait_for_job(&mut self, job_id: u32) -> Result<()> {
        // Simulated wait
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }
    
    /**
     * Gets command help from registry
     * 
     * @param command_name - Command name
     * @return Option<&str> - Help text if available
     */
    pub fn get_command_help(&self, command_name: &str) -> Option<&str> {
        self.command_registry.get_help(command_name)
    }
    
    /**
     * Gets project name from current directory
     * 
     * @return Option<String> - Project name if available
     */
    pub fn get_project_name(&self) -> Option<String> {
        self.current_path.file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
    
    /**
     * Gets aliases (placeholder)
     * 
     * @return Vec<(String, String)> - List of aliases
     */
    pub fn get_aliases(&self) -> Vec<(String, String)> {
        vec![("ll".to_string(), "ls -la".to_string())]
    }
    
    /**
     * Sets an alias
     * 
     * @param name - Alias name
     * @param value - Alias value
     */
    pub fn set_alias(&mut self, name: String, value: String) {
        // Placeholder implementation
    }
    
    /**
     * Removes an alias
     * 
     * @param name - Alias name to remove
     */
    pub fn remove_alias(&mut self, name: &str) {
        // Placeholder implementation
    }
    
    /**
     * Sets an environment variable
     * 
     * @param name - Variable name
     * @param value - Variable value
     */
    pub fn set_environment_variable(&mut self, name: String, value: String) {
        self.environment.insert(name, value);
    }
    
    /**
     * Removes an environment variable
     * 
     * @param name - Variable name to remove
     */
    pub fn remove_environment_variable(&mut self, name: &str) {
        self.environment.remove(name);
    }
    
    /**
     * Gets environment variables
     * 
     * @return Vec<(String, String)> - List of environment variables
     */
    pub fn get_environment(&self) -> Vec<(String, String)> {
        self.environment.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    
    /**
     * Clears command history
     */
    pub fn clear_history(&mut self) {
        // Placeholder implementation
    }
    
    /**
     * Gets command history
     * 
     * @return Vec<HistoryEntry> - Command history
     */
    pub fn get_history(&self) -> Vec<crate::history::HistoryEntry> {
        vec![crate::history::HistoryEntry {
            command: "ls".to_string(),
            timestamp: chrono::Utc::now(),
            exit_code: Some(0),
        }]
    }
    
    /**
     * Parses a command string
     * 
     * @param command - Command string to parse
     * @return Result<ParsedCommand> - Parsed command
     */
    pub fn parse_command(&self, command: &str) -> Result<crate::shell::parser::ParsedCommand> {
        self.parser.parse(command)
    }
} 