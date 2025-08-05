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
	 * パースされたコマンドを実行する関数です
	 * 
	 * 指定されたコマンドと引数を使用してプロセスを開始し、
	 * 入力・出力・エラーのリダイレクションを適切に設定します。
	 * 
	 * 入力リダイレクション（<）、出力リダイレクション（>）、
	 * 追記リダイレクション（>>）をサポートし、バックグラウンド
	 * 実行オプションにも対応します。
	 * 
	 * ファイルディスクリプタを適切に管理し、標準入出力の
	 * 継承またはパイプ設定を行ってコマンド実行環境を構築します。
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
	 * フォアグラウンドでコマンドを実行する関数です
	 * 
	 * 指定されたコマンドを同期的に実行し、標準出力と
	 * 標準エラーをキャプチャして結果を返します。
	 * 
	 * コマンドの終了を待機し、成功時は出力を文字列として
	 * 返し、失敗時はエラーコードと共にエラーを返します。
	 * 
	 * 標準出力と標準エラーの両方をキャプチャし、空でない
	 * 場合は結果に含めます。
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
	 * コマンドパイプラインを実行する関数です
	 * 
	 * 複数のコマンドをパイプ、AND、OR、シーケンシャル演算子で
	 * 接続して順次実行します。
	 * 
	 * 各演算子（|、&&、||、;）に応じて適切な処理を行い、
	 * 前のコマンドの終了コードに基づいて次のコマンドの実行を
	 * 決定します。パイプの場合はexecute_with_pipe()を使用し、
	 * その他の場合はexecute_with_realtime_output()を使用します。
	 * 
	 * 最終的な出力と終了コードを追跡して結果を返します。
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
	 * 実際のパイプライン処理を実行する関数です
	 * 
	 * パイプで接続された複数のコマンドを順次実行し、
	 * 前のコマンドの出力を次のコマンドの入力として使用します。
	 * 
	 * 現在は適切なパイプ実装まで順次実行を行い、
	 * 各コマンドの出力を連結して結果を返します。
	 * 
	 * 将来的にはプロセス間通信を使用した実際のパイプ処理を
	 * 実装する予定です。
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
	 * パイプを使用してコマンドを実行する関数です
	 * 
	 * 指定されたコマンドをパイプ設定で実行し、標準出力と
	 * 標準エラーをリアルタイムで読み取ります。
	 * 
	 * 次のコマンドがある場合は標準出力をパイプに設定し、
	 * 標準エラーもパイプに設定してエラー出力もキャプチャします。
	 * 
	 * BufReaderを使用して標準出力と標準エラーを順次読み取り、
	 * 各行を結果に追加します。プロセスの終了を待機して
	 * 終了コードと出力を返します。
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
	 * リアルタイム出力でコマンドを実行する関数です
	 * 
	 * 指定されたコマンドを実行し、標準出力と標準エラーを
	 * リアルタイムでストリーミングします。
	 * 
	 * 入力リダイレクション（<）、出力リダイレクション（>）、
	 * 追記リダイレクション（>>）を適切に設定し、標準入出力を
	 * 継承またはパイプに設定します。
	 * 
	 * プロセスを開始し、標準出力と標準エラーを同時に読み取り、
	 * 各行をリアルタイムで結果に追加します。プロセスの終了を
	 * 待機して終了コードと出力を返します。
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