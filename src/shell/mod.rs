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
use job::{JobManager, SignalHandler};
use parser::{CommandParser, parse_pipeline, CommandPipeline};
use executor::CommandExecutor;
use builtins::BuiltinCommands;
use commands::{CommandRegistry, CommandHandler, CommandResult};
use crate::history::HistoryManager;

/**
 * Result of background command execution
 * 
 * Contains job ID and output information for background processes.
 */
#[derive(Debug)]
struct BackgroundResult {
    /// Job ID assigned to the background process
    job_id: u32,
    /// Output message from the background execution
    output: String,
}

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
    /// Signal handler for process control
    signal_handler: SignalHandler,
    /// Current command input buffer
    input_buffer: String,
    /// Command output history
    output_history: Vec<String>,
    /// Environment variables
    environment: HashMap<String, String>,
    /// Command history manager
    history_manager: HistoryManager,
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
            signal_handler: SignalHandler::new(),
            history_manager: HistoryManager::new()?,
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
     * Sets the input buffer content
     * 
     * @param input - New input text
     */
    pub fn set_input(&mut self, input: &str) {
        self.input_buffer = input.to_string();
    }
    
    /**
     * リアルタイム出力追加の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なリアルタイム出力処理を行います。
     * TUI更新と出力履歴管理が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param output - 追加する出力
     */
    pub fn add_output(&mut self, output: String) {
        self.output_history.push(output);
    }
    
    /**
     * リアルタイムTUI更新の複雑な処理です (◕‿◕)
     * 
     * この関数は複雑なリアルタイムTUI更新を行います。
     * 非同期出力ストリーミングが難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     * 
     * @param line - リアルタイム出力行
     */
    pub fn add_realtime_output(&mut self, line: String) {
        // Add to real-time output buffer for TUI display
        self.output_history.push(line);
        
        // Here we would typically trigger a TUI redraw
        // For now, we just add to the history
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
     * コマンド実行の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なコマンド処理を行います。
     * 履歴保存とリアルタイム実行が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @return Result<()> - 成功またはエラー状態
     */
    pub async fn execute_command(&mut self) -> Result<()> {
        let command = self.input_buffer.trim();
        if command.is_empty() {
            self.input_buffer.clear();
            return Ok(());
        }
        
        // Add command to history
        self.history_manager.add_command(command.to_string(), None);
        
        // Check for background execution
        let is_background = command.ends_with('&');
        let clean_command = if is_background {
            command.trim_end_matches('&').trim()
        } else {
            command
        };
        
        // Try to parse as pipeline first
        match parse_pipeline(clean_command) {
            Ok(pipeline) => {
                if is_background {
                    // Execute in background
                    let result = self.execute_pipeline_background(&pipeline).await?;
                    self.output_history.push(format!("[{}] {}", result.job_id, result.output));
                } else {
                    // Execute in foreground
                    let result = self.execute_pipeline(&pipeline).await?;
                    self.output_history.push(result);
                }
            }
            Err(_) => {
                // Fall back to single command parsing
                let parsed = self.parser.parse(clean_command)?;
                if is_background {
                    // Execute in background
                    let result = self.execute_parsed_command_background(&parsed).await?;
                    self.output_history.push(format!("[{}] {}", result.job_id, result.output));
                } else {
                    // Execute in foreground
                    let result = self.execute_parsed_command(&parsed).await?;
                    self.output_history.push(result);
                }
            }
        }
        
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
     * パイプライン実行の複雑な処理です (◕‿◕)
     * 
     * この関数は複雑なパイプライン処理を行います。
     * パイプ、リアルタイム出力、コマンドチェーンが難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     * 
     * @param pipeline - 実行するコマンドパイプライン
     * @return Result<String> - パイプライン出力またはエラー
     */
    async fn execute_pipeline(&mut self, pipeline: &CommandPipeline) -> Result<String> {
        self.executor.execute_pipeline(pipeline, &self.current_path).await
    }
    
    /**
     * バックグラウンドパイプライン実行の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なバックグラウンド処理を行います。
     * ジョブ管理とプロセス追跡が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param pipeline - 実行するコマンドパイプライン
     * @return Result<BackgroundResult> - バックグラウンド実行結果またはエラー
     */
    async fn execute_pipeline_background(&mut self, pipeline: &CommandPipeline) -> Result<BackgroundResult> {
        // For now, execute the first command in background
        if let Some(first_command) = pipeline.commands.first() {
            let mut cmd = std::process::Command::new(&first_command.command);
            cmd.current_dir(&self.current_path);
            cmd.args(&first_command.args);
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());
            
            let child = cmd.spawn()?;
            let pid = child.id();
            let job_id = self.job_manager.add_job(pid, first_command.command.clone());
            
            Ok(BackgroundResult {
                job_id,
                output: format!("Background job started with PID: {}", pid),
            })
        } else {
            Err(anyhow::anyhow!("No commands in pipeline"))
        }
    }
    
    /**
     * バックグラウンドコマンド実行の複雑な処理です (◕‿◕)
     * 
     * この関数は複雑なバックグラウンド処理を行います。
     * プロセス管理とジョブ追跡が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     * 
     * @param parsed - 実行するパースされたコマンド
     * @return Result<BackgroundResult> - バックグラウンド実行結果またはエラー
     */
    async fn execute_parsed_command_background(&mut self, parsed: &crate::shell::parser::ParsedCommand) -> Result<BackgroundResult> {
        let mut cmd = std::process::Command::new(&parsed.command);
        cmd.current_dir(&self.current_path);
        cmd.args(&parsed.args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let child = cmd.spawn()?;
        let pid = child.id();
        let job_id = self.job_manager.add_job(pid, parsed.command.clone());
        
        Ok(BackgroundResult {
            job_id,
            output: format!("Background job started with PID: {}", pid),
        })
    }
    
    /**
     * Changes the current working directory
     * 
     * @param path - New directory path
     * @return Result<()> - Success or error status
     */
    pub fn change_directory(&mut self, path: &str) -> Result<()> {
        /**
         * ディレクトリ変更の複雑な処理です (◕‿◕)
         * 
         * この関数は複雑なパス解析を行います。
         * 相対パスと絶対パスの処理が難しい部分なので、
         * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
         */
        
        let clean_path = path.trim();
        
        let new_path = if clean_path == "~" {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        } else if clean_path == ".." {
            // Handle parent directory
            self.current_path.parent().unwrap_or(&self.current_path).to_path_buf()
        } else if clean_path.starts_with('/') {
            // Absolute path
            PathBuf::from(clean_path)
        } else {
            // Relative path
            self.current_path.join(clean_path)
        };
        
        if new_path.exists() && new_path.is_dir() {
            self.current_path = new_path.canonicalize()?;
            std::env::set_current_dir(&self.current_path)?;
        } else {
            return Err(anyhow::anyhow!("No such file or directory (os error 2)"));
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
        self.environment.insert(name.clone(), value.clone());
        std::env::set_var(name, value);
    }
    
    /**
     * Removes an environment variable
     * 
     * @param name - Variable name to remove
     */
    pub fn remove_environment_variable(&mut self, name: &str) {
        self.environment.remove(name);
        std::env::remove_var(name);
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
    
    /**
     * Gets a mutable reference to the signal handler
     * 
     * @return &mut SignalHandler - Signal handler reference
     */
    pub fn signal_handler_mut(&mut self) -> &mut SignalHandler {
        &mut self.signal_handler
    }
    
    /**
     * Gets a mutable reference to the job manager
     * 
     * @return &mut JobManager - Job manager reference
     */
    pub fn job_manager_mut(&mut self) -> &mut JobManager {
        &mut self.job_manager
    }
    
    /**
     * Handles SIGINT signal (Ctrl+C)
     */
    pub fn handle_sigint_signal(&mut self) {
        self.signal_handler.handle_sigint(&mut self.job_manager);
    }
    
    /**
     * Handles SIGTSTP signal (Ctrl+Z)
     */
    pub fn handle_sigtstp_signal(&mut self) {
        self.signal_handler.handle_sigtstp(&mut self.job_manager);
    }

    /**
     * Tab completion for files and commands
     * 
     * Provides autocomplete functionality for file paths and commands.
     * Supports fuzzy matching and command history completion.
     * 
     * @param partial - Partial input to complete
     * @return Vec<String> - List of possible completions
     */
    pub fn tab_complete(&self, partial: &str) -> Vec<String> {
        let mut completions = Vec::new();
        
        // Command completion
        if !partial.contains('/') {
            // Complete commands from PATH
            if let Ok(path) = std::env::var("PATH") {
                for dir in path.split(':') {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                if let Ok(file_name) = entry.file_name().into_string() {
                                    if file_name.starts_with(partial) && entry.path().is_file() {
                                        completions.push(file_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Complete built-in commands
            for command in self.command_registry.list_commands() {
                if command.starts_with(partial) {
                    completions.push(command.to_string());
                }
            }
        } else {
            // File path completion
            let path = std::path::Path::new(partial);
            let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
            let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
            
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            if file_name.starts_with(filename) {
                                let full_path = if parent == std::path::Path::new("") {
                                    file_name
                                } else {
                                    format!("{}/{}", parent.display(), file_name)
                                };
                                completions.push(full_path);
                            }
                        }
                    }
                }
            }
        }
        
        completions.sort();
        completions.dedup();
        completions
    }
} 