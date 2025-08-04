/**
 * Background I/O management for Sare terminal
 * 
 * This module provides background process I/O handling
 * with actual process management and output buffering.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: background.rs
 * Description: Background I/O management with process handling
 */

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;

/**
 * Background I/O information
 * 
 * Contains information about background process I/O
 * including output buffering and state management.
 */
#[derive(Debug, Clone)]
pub struct BackgroundIo {
	/// Process ID
	pub pid: u32,
	/// Output buffer
	pub output_buffer: VecDeque<String>,
	/// Error buffer
	pub error_buffer: VecDeque<String>,
	/// I/O state
	pub io_state: BackgroundIoState,
	/// Output channel
	pub output_tx: Option<mpsc::Sender<String>>,
}

/**
 * Background I/O state enumeration
 * 
 * Defines the different states background I/O can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundIoState {
	/// I/O is active
	Active,
	/// I/O is suspended
	Suspended,
	/// I/O has completed
	Completed,
	/// I/O has an error
	Error(String),
}

/**
 * Background I/O manager for handling background process I/O
 * 
 * Provides functionality to manage background process I/O
 * with actual process monitoring and output collection.
 */
pub struct BackgroundIoManager;

impl BackgroundIoManager {
	/**
	 * Sets up background I/O for a process
	 * 
	 * @param background_io - Background I/O collection
	 * @param pid - Process ID
	 * @return Result<()> - Success or error
	 */
	pub async fn setup_background_io(
		background_io: &mut Arc<RwLock<HashMap<u32, BackgroundIo>>>,
		pid: u32,
	) -> Result<()> {
		let (tx, _rx) = mpsc::channel(100);
		
		let bg_io = BackgroundIo {
			pid,
			output_buffer: VecDeque::new(),
			error_buffer: VecDeque::new(),
			io_state: BackgroundIoState::Active,
			output_tx: Some(tx),
		};
		
		let mut bg_io_guard = background_io.write().await;
		bg_io_guard.insert(pid, bg_io);
		
		Ok(())
	}
	
	/**
	 * Writes background output
	 * 
	 * @param background_io - Background I/O collection
	 * @param pid - Process ID
	 * @param output - Output string
	 * @return Result<()> - Success or error
	 */
	pub async fn write_background_output(
		background_io: &Arc<RwLock<HashMap<u32, BackgroundIo>>>,
		pid: u32,
		output: String,
	) -> Result<()> {
		let mut bg_io_guard = background_io.write().await;
		
		if let Some(bg_io) = bg_io_guard.get_mut(&pid) {
			bg_io.output_buffer.push_back(output.clone());
			
			// Send to channel if available
			if let Some(ref tx) = bg_io.output_tx {
				let _ = tx.send(output).await;
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets background output for a process
	 * 
	 * @param background_io - Background I/O collection
	 * @param pid - Process ID
	 * @return Vec<String> - Background output
	 */
	pub async fn get_background_output(
		background_io: &Arc<RwLock<HashMap<u32, BackgroundIo>>>,
		pid: u32,
	) -> Vec<String> {
		let bg_io_guard = background_io.read().await;
		
		if let Some(bg_io) = bg_io_guard.get(&pid) {
			bg_io.output_buffer.iter().cloned().collect()
		} else {
			Vec::new()
		}
	}
} 