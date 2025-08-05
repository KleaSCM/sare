/**
 * Pipeline management for Sare terminal
 * 
 * This module provides pipeline creation, management, and I/O operations
 * with actual pipe system calls for real pipeline functionality.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: pipelines.rs
 * Description: Pipeline management with actual pipe system calls
 */

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/**
 * Pipeline information
 * 
 * Contains information about a pipeline connection
 * between multiple processes.
 */
#[derive(Debug, Clone)]
pub struct Pipeline {
	/// Pipeline ID
	pub pipeline_id: String,
	/// Process IDs in the pipeline
	pub process_ids: Vec<u32>,
	/// Pipeline state
	pub state: PipelineState,
	/// Pipeline buffer
	pub buffer: VecDeque<u8>,
	/// Pipe file descriptors
	pub pipe_fds: Vec<i32>,
}

/**
 * Pipeline state enumeration
 * 
 * Defines the different states a pipeline can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineState {
	/// Pipeline is active
	Active,
	/// Pipeline is complete
	Complete,
	/// Pipeline has an error
	Error(String),
}

/**
 * Pipeline manager for handling I/O pipelines
 * 
 * Provides functionality to create, manage, and operate on
 * I/O pipelines with actual pipe system calls.
 */
pub struct PipelineManager;

impl PipelineManager {
	/**
	 * Creates a new pipeline with actual pipe system calls
	 * 
	 * @param pipelines - Pipelines collection
	 * @param process_ids - Process IDs in the pipeline
	 * @return Result<String> - Pipeline ID or error
	 */
	pub async fn create_pipeline(
		pipelines: &mut Arc<RwLock<Vec<Pipeline>>>,
		process_ids: Vec<u32>,
	) -> Result<String> {
		/**
		 * パイプラインを作成する関数です
		 * 
		 * 複数のプロセス間でパイプを作成し、プロセス間通信を
		 * 設定してデータの流れを管理します。
		 * 
		 * 各プロセスペア間にパイプを作成し、ファイルディスクリプタを
		 * 適切に設定してパイプラインを初期化します
		 */
		
		let pipeline_id = uuid::Uuid::new_v4().to_string();
		let pipe_fds = Self::create_pipes(process_ids.len())?;
		
		let pipeline = Pipeline {
			pipeline_id: pipeline_id.clone(),
			process_ids,
			state: PipelineState::Active,
			buffer: VecDeque::new(),
			pipe_fds,
		};
		
		let mut pipelines_guard = pipelines.write().await;
		pipelines_guard.push(pipeline);
		
		Ok(pipeline_id)
	}
	
	/**
	 * Creates actual pipes for pipeline
	 * 
	 * @param num_processes - Number of processes in pipeline
	 * @return Result<Vec<i32>> - Pipe file descriptors or error
	 */
	fn create_pipes(num_processes: usize) -> Result<Vec<i32>> {
		let mut pipe_fds = Vec::new();
		
		// Create pipes for each process pair
		for _ in 0..num_processes - 1 {
			let mut pipe_array = [0; 2];
			unsafe {
				use libc::pipe;
				if pipe(pipe_array.as_mut_ptr()) != 0 {
					return Err(anyhow::anyhow!("Failed to create pipe"));
				}
			}
			pipe_fds.extend_from_slice(&pipe_array);
		}
		
		Ok(pipe_fds)
	}
	
	/**
	 * Writes data to a pipeline with actual pipe operations
	 * 
	 * @param pipelines - Pipelines collection
	 * @param pipeline_id - Pipeline ID
	 * @param data - Data to write
	 * @return Result<usize> - Number of bytes written or error
	 */
	pub async fn write_to_pipeline(
		pipelines: &Arc<RwLock<Vec<Pipeline>>>,
		pipeline_id: &str,
		data: &[u8],
	) -> Result<usize> {
		let mut pipelines_guard = pipelines.write().await;
		
		if let Some(pipeline) = pipelines_guard.iter_mut().find(|p| p.pipeline_id == pipeline_id) {
			match pipeline.state {
				PipelineState::Active => {
					// Write to first pipe in pipeline
					if let Some(&write_fd) = pipeline.pipe_fds.get(1) {
						let bytes_written = unsafe {
							use libc::{write, c_void};
							let ptr = data.as_ptr() as *const c_void;
							let len = data.len();
							write(write_fd, ptr, len)
						};
						
						if bytes_written < 0 {
							pipeline.state = PipelineState::Error("Pipeline write failed".to_string());
							return Err(anyhow::anyhow!("Failed to write to pipeline"));
						}
						
						// Also add to buffer for caching
						pipeline.buffer.extend(data.iter().cloned());
						
						Ok(bytes_written as usize)
					} else {
						Err(anyhow::anyhow!("No write pipe available"))
					}
				}
				PipelineState::Complete => {
					Err(anyhow::anyhow!("Pipeline is complete"))
				}
				PipelineState::Error(ref msg) => {
					Err(anyhow::anyhow!("Pipeline error: {}", msg))
				}
			}
		} else {
			Err(anyhow::anyhow!("Pipeline not found"))
		}
	}
	
	/**
	 * Reads data from a pipeline with actual pipe operations
	 * 
	 * @param pipelines - Pipelines collection
	 * @param pipeline_id - Pipeline ID
	 * @param buffer - Buffer to read into
	 * @return Result<usize> - Number of bytes read or error
	 */
	pub async fn read_from_pipeline(
		pipelines: &Arc<RwLock<Vec<Pipeline>>>,
		pipeline_id: &str,
		buffer: &mut [u8],
	) -> Result<usize> {
		let mut pipelines_guard = pipelines.write().await;
		
		if let Some(pipeline) = pipelines_guard.iter_mut().find(|p| p.pipeline_id == pipeline_id) {
			match pipeline.state {
				PipelineState::Active => {
					// Read from last pipe in pipeline
					if let Some(&read_fd) = pipeline.pipe_fds.get(pipeline.pipe_fds.len() - 2) {
						let bytes_read = unsafe {
							use libc::{read, c_void};
							let ptr = buffer.as_mut_ptr() as *mut c_void;
							let len = buffer.len();
							read(read_fd, ptr, len)
						};
						
						if bytes_read < 0 {
							pipeline.state = PipelineState::Error("Pipeline read failed".to_string());
							return Err(anyhow::anyhow!("Failed to read from pipeline"));
						}
						
						// Also read from buffer if available
						let buffer_read = std::cmp::min(bytes_read as usize, pipeline.buffer.len());
						if buffer_read > 0 {
							for i in 0..buffer_read {
								buffer[i] = pipeline.buffer.pop_front().unwrap_or(0);
							}
						}
						
						Ok(bytes_read as usize)
					} else {
						Err(anyhow::anyhow!("No read pipe available"))
					}
				}
				PipelineState::Complete => {
					Err(anyhow::anyhow!("Pipeline is complete"))
				}
				PipelineState::Error(ref msg) => {
					Err(anyhow::anyhow!("Pipeline error: {}", msg))
				}
			}
		} else {
			Err(anyhow::anyhow!("Pipeline not found"))
		}
	}
	
	/**
	 * Closes a pipeline and its file descriptors
	 * 
	 * @param pipelines - Pipelines collection
	 * @param pipeline_id - Pipeline ID
	 * @return Result<()> - Success or error
	 */
	pub async fn close_pipeline(
		pipelines: &mut Arc<RwLock<Vec<Pipeline>>>,
		pipeline_id: &str,
	) -> Result<()> {
		let mut pipelines_guard = pipelines.write().await;
		
		if let Some(pipeline) = pipelines_guard.iter_mut().find(|p| p.pipeline_id == pipeline_id) {
			// Close all pipe file descriptors
			for &fd in &pipeline.pipe_fds {
				unsafe {
					use libc::close;
					close(fd);
				}
			}
			
			pipeline.state = PipelineState::Complete;
			pipeline.buffer.clear();
		}
		
		// Remove pipeline from collection
		pipelines_guard.retain(|p| p.pipeline_id != pipeline_id);
		Ok(())
	}
	
	/**
	 * Lists all pipelines
	 * 
	 * @param pipelines - Pipelines collection
	 * @return Vec<Pipeline> - List of pipelines
	 */
	pub async fn list_pipelines(
		pipelines: &Arc<RwLock<Vec<Pipeline>>>,
	) -> Vec<Pipeline> {
		let pipelines_guard = pipelines.read().await;
		pipelines_guard.clone()
	}
} 