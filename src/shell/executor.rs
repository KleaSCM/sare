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
use std::io::{BufRead, BufReader};
use crate::shell::parser::{ParsedCommand, CommandPipeline, ChainOperator};
use crate::shell::commands::CommandResult;

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
     * リアルコマンド実行の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なI/Oリダイレクションを行います。
     * ファイルディスクリプタ管理が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param command - 実行するパースされたコマンド
     * @param working_dir - 作業ディレクトリ
     * @return Result<String> - コマンド出力またはエラー
     */
    pub async fn execute(&self, command: &ParsedCommand, working_dir: &Path) -> Result<String> {
        let mut cmd = Command::new(&command.command);
        
        cmd.current_dir(working_dir);
        cmd.args(&command.args);
        
        // Handle input redirection
        if let Some(ref input_file) = command.input_redirect {
            let input_file = std::fs::File::open(input_file)?;
            cmd.stdin(Stdio::from(input_file));
        } else {
            cmd.stdin(Stdio::inherit());
        }
        
        // Handle output redirection
        if let Some(ref output_file) = command.output_redirect {
            let output_file = std::fs::File::create(output_file)?;
            let output_file_clone = output_file.try_clone()?;
            cmd.stdout(Stdio::from(output_file));
            cmd.stderr(Stdio::from(output_file_clone));
        } else {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }
        
        // Handle append redirection
        if let Some(ref append_file) = command.append_redirect {
            let append_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(append_file)?;
            let append_file_clone = append_file.try_clone()?;
            cmd.stdout(Stdio::from(append_file));
            cmd.stderr(Stdio::from(append_file_clone));
        }
        
        let output = if command.background {
            self.execute_background(cmd)?
        } else {
            self.execute_foreground(cmd).await?
        };
        
        Ok(output)
    }
    
    /**
     * リアルコマンド実行の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なプロセス制御を行います。
     * リアルタイム出力キャプチャが難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param mut cmd - 実行するコマンド
     * @return Result<String> - コマンド出力またはエラー
     */
    async fn execute_foreground(&self, mut cmd: Command) -> Result<String> {
        // Use synchronous execution for now
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
    
    /**
     * パイプライン実行の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なパイプライン処理を行います。
     * パイプ、リアルタイム出力、コマンドチェーンが難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param pipeline - 実行するコマンドパイプライン
     * @param working_dir - 作業ディレクトリ
     * @return Result<String> - パイプライン出力またはエラー
     */
    pub async fn execute_pipeline(&self, pipeline: &CommandPipeline, working_dir: &Path) -> Result<String> {
        let mut output = String::new();
        let mut last_exit_code = 0;
        
        for (i, command) in pipeline.commands.iter().enumerate() {
            let operator = if i > 0 { 
                Some(&pipeline.operators[i - 1]) 
            } else { 
                None 
            };
            
            // Check if we should continue based on previous result
            if let Some(op) = operator {
                match op {
                    ChainOperator::And => {
                        if last_exit_code != 0 {
                            // Previous command failed, skip this one
                            continue;
                        }
                    }
                    ChainOperator::Or => {
                        if last_exit_code == 0 {
                            // Previous command succeeded, skip this one
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            
            // Handle pipes
            if let Some(op) = operator {
                match op {
                    ChainOperator::Pipe => {
                        // Execute with pipe to next command
                        let result = self.execute_with_pipe(command, working_dir, i < pipeline.commands.len() - 1).await?;
                        last_exit_code = result.exit_code;
                        output.push_str(&result.output);
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Execute command with real-time output
            let result = self.execute_with_realtime_output(command, working_dir).await?;
            last_exit_code = result.exit_code;
            output.push_str(&result.output);
            
            // Add separator for sequential commands
            if let Some(op) = operator {
                match op {
                    ChainOperator::Sequential => {
                        output.push('\n');
                    }
                    _ => {}
                }
            }
        }
        
        Ok(output)
    }
    
    /**
     * リアルパイプ実行の複雑な処理です (｡◕‿◕｡)
     * 
     * この関数は複雑なプロセス間通信を行います。
     * パイプ処理とリアルタイムデータ転送が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (◕‿◕)
     * 
     * @param commands - パイプで接続されたコマンドのリスト
     * @param working_dir - 作業ディレクトリ
     * @return Result<String> - パイプライン出力またはエラー
     */
    async fn execute_real_pipeline(&self, commands: &[ParsedCommand], working_dir: &Path) -> Result<String> {
        if commands.is_empty() {
            return Ok(String::new());
        }
        
        // For now, execute commands sequentially until we implement proper pipes
        let mut output = String::new();
        
        for command in commands {
            let result = self.execute_with_realtime_output(command, working_dir).await?;
            output.push_str(&result.output);
        }
        
        Ok(output)
    }
    
    /**
     * パイプ実行の複雑な処理です (◕‿◕)
     * 
     * この関数は複雑なパイプ処理を行います。
     * プロセス間通信が難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     * 
     * @param command - 実行するコマンド
     * @param working_dir - 作業ディレクトリ
     * @param has_next - 次のコマンドがあるかどうか
     * @return Result<CommandResult> - コマンド結果またはエラー
     */
    async fn execute_with_pipe(&self, command: &ParsedCommand, working_dir: &Path, has_next: bool) -> Result<CommandResult> {
        let mut cmd = Command::new(&command.command);
        
        cmd.current_dir(working_dir);
        cmd.args(&command.args);
        
        // Set up pipes
        if has_next {
            cmd.stdout(Stdio::piped());
        } else {
            cmd.stdout(Stdio::piped());
        }
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::inherit());
        
        let mut child = cmd.spawn()?;
        let mut output = String::new();
        
        // Read stdout in real-time
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line?;
                output.push_str(&line);
                output.push('\n');
            }
        }
        
        // Read stderr in real-time
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = line?;
                output.push_str(&line);
                output.push('\n');
            }
        }
        
        let status = child.wait()?;
        
        Ok(CommandResult {
            output,
            exit_code: status.code().unwrap_or(-1),
        })
    }
    
    /**
     * リアルタイム出力の複雑な処理です (◕‿◕)
     * 
     * この関数は複雑なリアルタイム出力処理を行います。
     * プロセス出力のストリーミングが難しい部分なので、
     * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
     * 
     * @param command - 実行するパースされたコマンド
     * @param working_dir - 作業ディレクトリ
     * @return Result<CommandResult> - コマンド結果またはエラー
     */
    async fn execute_with_realtime_output(&self, command: &ParsedCommand, working_dir: &Path) -> Result<CommandResult> {
        let mut cmd = Command::new(&command.command);
        
        cmd.current_dir(working_dir);
        cmd.args(&command.args);
        
        // Handle input redirection
        if let Some(ref input_file) = command.input_redirect {
            let input_file = std::fs::File::open(input_file)?;
            cmd.stdin(Stdio::from(input_file));
        } else {
            cmd.stdin(Stdio::inherit());
        }
        
        // Handle output redirection
        if let Some(ref output_file) = command.output_redirect {
            let output_file = std::fs::File::create(output_file)?;
            let output_file_clone = output_file.try_clone()?;
            cmd.stdout(Stdio::from(output_file));
            cmd.stderr(Stdio::from(output_file_clone));
        } else {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }
        
        // Handle append redirection
        if let Some(ref append_file) = command.append_redirect {
            let append_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(append_file)?;
            let append_file_clone = append_file.try_clone()?;
            cmd.stdout(Stdio::from(append_file));
            cmd.stderr(Stdio::from(append_file_clone));
        }
        
        let mut child = cmd.spawn()?;
        let mut output = String::new();
        
        // Read stdout in real-time
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line?;
                output.push_str(&line);
                output.push('\n');
                // Here you would typically send this to the TUI for real-time display
            }
        }
        
        // Read stderr in real-time
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = line?;
                output.push_str(&line);
                output.push('\n');
                // Here you would typically send this to the TUI for real-time display
            }
        }
        
        let status = child.wait()?;
        
        let exit_code = status.code().unwrap_or(-1);
        
        Ok(CommandResult {
            output,
            exit_code,
        })
    }
} 