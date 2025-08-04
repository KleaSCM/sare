/**
 * I/O utilities for Sare terminal
 * 
 * This module provides utility functions for I/O operations
 * including file descriptor management and I/O helpers.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: utils.rs
 * Description: I/O utilities and helper functions
 */

use anyhow::Result;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

/**
 * I/O utilities for common I/O operations
 * 
 * Provides helper functions for file operations,
 * file descriptor management, and I/O utilities.
 */
pub struct IoUtils;

impl IoUtils {
	/**
	 * Sets up file redirection with actual file operations
	 * 
	 * @param options - Redirection options
	 * @return Result<()> - Success or error
	 */
	pub fn setup_file_redirection(options: &super::redirection::IoRedirectOptions) -> Result<()> {
		super::redirection::RedirectionManager::setup_file_redirection(options)
	}
	
	/**
	 * Sets up pipeline redirection with actual pipe operations
	 * 
	 * @param commands - Commands in pipeline
	 * @return Result<()> - Success or error
	 */
	pub fn setup_pipeline_redirection(commands: &[String]) -> Result<()> {
		super::redirection::RedirectionManager::setup_pipeline_redirection(commands)
	}
	
	/**
	 * Creates a temporary file for redirection
	 * 
	 * @return Result<String> - Temporary file path or error
	 */
	pub fn create_temp_file() -> Result<String> {
		super::redirection::RedirectionManager::create_temp_file()
	}
	
	/**
	 * Opens a file with appropriate flags
	 * 
	 * @param path - File path
	 * @param read - Whether to open for reading
	 * @param write - Whether to open for writing
	 * @param create - Whether to create if not exists
	 * @param append - Whether to append
	 * @return Result<i32> - File descriptor or error
	 */
	pub fn open_file(
		path: &str,
		read: bool,
		write: bool,
		create: bool,
		append: bool,
	) -> Result<i32> {
		let mut open_options = OpenOptions::new();
		
		if read {
			open_options.read(true);
		}
		if write {
			open_options.write(true);
		}
		if create {
			open_options.create(true);
		}
		if append {
			open_options.append(true);
		}
		
		let file = open_options.open(path)?;
		Ok(file.as_raw_fd())
	}
	
	/**
	 * Duplicates a file descriptor
	 * 
	 * @param old_fd - Old file descriptor
	 * @param new_fd - New file descriptor
	 * @return Result<()> - Success or error
	 */
	pub fn duplicate_fd(old_fd: i32, new_fd: i32) -> Result<()> {
		unsafe {
			use libc::dup2;
			if dup2(old_fd, new_fd) < 0 {
				return Err(anyhow::anyhow!("Failed to duplicate file descriptor"));
			}
		}
		Ok(())
	}
	
	/**
	 * Closes a file descriptor
	 * 
	 * @param fd - File descriptor to close
	 * @return Result<()> - Success or error
	 */
	pub fn close_fd(fd: i32) -> Result<()> {
		unsafe {
			use libc::close;
			if close(fd) < 0 {
				return Err(anyhow::anyhow!("Failed to close file descriptor"));
			}
		}
		Ok(())
	}
	
	/**
	 * Creates a pipe
	 * 
	 * @return Result<(i32, i32)> - Read and write file descriptors or error
	 */
	pub fn create_pipe() -> Result<(i32, i32)> {
		let mut pipe_array = [0; 2];
		unsafe {
			use libc::pipe;
			if pipe(pipe_array.as_mut_ptr()) != 0 {
				return Err(anyhow::anyhow!("Failed to create pipe"));
			}
		}
		Ok((pipe_array[0], pipe_array[1]))
	}
} 