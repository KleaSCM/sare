/**
 * I/O redirection management for Sare terminal
 * 
 * This module provides file redirection and pipeline redirection
 * with actual file operations and temporary file creation.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: redirection.rs
 * Description: I/O redirection with actual file operations
 */

use anyhow::Result;
use std::fs::{OpenOptions, File};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/**
 * I/O redirection options
 * 
 * Contains configuration options for I/O redirection
 * including file paths and append modes.
 */
#[derive(Debug, Clone)]
pub struct IoRedirectOptions {
	/// Input redirection
	pub stdin_redirect: Option<String>,
	/// Output redirection
	pub stdout_redirect: Option<String>,
	/// Error redirection
	pub stderr_redirect: Option<String>,
	/// Append mode for output
	pub append_output: bool,
	/// Append mode for error
	pub append_error: bool,
	/// Pipeline input
	pub pipeline_input: Option<String>,
	/// Pipeline output
	pub pipeline_output: Option<String>,
}

impl Default for IoRedirectOptions {
	fn default() -> Self {
		Self {
			stdin_redirect: None,
			stdout_redirect: None,
			stderr_redirect: None,
			append_output: false,
			append_error: false,
			pipeline_input: None,
			pipeline_output: None,
		}
	}
}

/**
 * Redirection manager for handling I/O redirection
 * 
 * Provides functionality to set up file redirection
 * and pipeline redirection with actual file operations.
 */
pub struct RedirectionManager;

impl RedirectionManager {
	/**
	 * Sets up file redirection with actual file operations
	 * 
	 * @param options - Redirection options
	 * @return Result<()> - Success or error
	 */
	pub fn setup_file_redirection(options: &IoRedirectOptions) -> Result<()> {
		/**
		 * ファイルリダイレクションを設定する関数です
		 * 
		 * 標準入力、標準出力、標準エラーのリダイレクションを設定し、
		 * ファイルディスクリプタを適切に管理します。
		 * 
		 * ファイルのオープン、アペンドモードの設定、ファイルディスクリプタの
		 * 複製を行ってリダイレクションを実現します
		 */
		
		// Set up stdin redirection
		if let Some(ref path) = options.stdin_redirect {
			let file = OpenOptions::new()
				.read(true)
				.open(path)?;
			unsafe {
				use libc::dup2;
				dup2(file.as_raw_fd(), 0);
			}
		}
		
		// Set up stdout redirection
		if let Some(ref path) = options.stdout_redirect {
			let mut open_options = OpenOptions::new();
			open_options.write(true);
			if options.append_output {
				open_options.append(true);
			} else {
				open_options.create(true).truncate(true);
			}
			let file = open_options.open(path)?;
			unsafe {
				use libc::dup2;
				dup2(file.as_raw_fd(), 1);
			}
		}
		
		// Set up stderr redirection
		if let Some(ref path) = options.stderr_redirect {
			let mut open_options = OpenOptions::new();
			open_options.write(true);
			if options.append_error {
				open_options.append(true);
			} else {
				open_options.create(true).truncate(true);
			}
			let file = open_options.open(path)?;
			unsafe {
				use libc::dup2;
				dup2(file.as_raw_fd(), 2);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Sets up pipeline redirection with actual pipe operations
	 * 
	 * @param commands - Commands in pipeline
	 * @return Result<()> - Success or error
	 */
	pub fn setup_pipeline_redirection(commands: &[String]) -> Result<()> {
		/**
		 * パイプラインリダイレクションを設定する関数です
		 * 
		 * 複数のコマンド間でパイプを作成し、プロセス間通信を
		 * 設定してデータの流れを管理します。
		 * 
		 * 各コマンドペア間にパイプを作成し、ファイルディスクリプタを
		 * 適切に設定してデータの流れを制御します
		 */
		
		if commands.len() < 2 {
			return Ok(());
		}
		
		// Create pipes for each command pair
		for i in 0..commands.len() - 1 {
			let mut pipe_array = [0; 2];
			unsafe {
				use libc::pipe;
				if pipe(pipe_array.as_mut_ptr()) != 0 {
					return Err(anyhow::anyhow!("Failed to create pipe"));
				}
			}
			
			// Set up file descriptors for current command
			unsafe {
				use libc::dup2;
				if i == 0 {
					// First command: write to pipe
					dup2(pipe_array[1], 1);
				} else {
					// Middle commands: read from previous pipe, write to current pipe
					dup2(pipe_array[0], 0);
					dup2(pipe_array[1], 1);
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Creates a temporary file for redirection
	 * 
	 * @return Result<String> - Temporary file path or error
	 */
	pub fn create_temp_file() -> Result<String> {
		use std::env;
		use std::fs;
		
		let temp_dir = env::temp_dir();
		let temp_file = temp_dir.join(format!("sare_redir_{}", uuid::Uuid::new_v4()));
		
		// Create empty file
		File::create(&temp_file)?;
		
		Ok(temp_file.to_string_lossy().to_string())
	}
} 