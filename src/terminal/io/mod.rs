/**
 * I/O redirection and piping module for Sare terminal
 * 
 * This module provides modular I/O redirection capabilities for the Sare terminal,
 * including stdin/stdout/stderr redirection, pipeline support, and
 * background process I/O handling for developer workflows.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Main I/O module that orchestrates streams, pipelines, and background I/O
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod streams;
pub mod pipelines;
pub mod background;
pub mod redirection;
pub mod utils;

use streams::{IoStream, StreamType, StreamState};
use pipelines::{Pipeline, PipelineState};
use background::{BackgroundIo, BackgroundIoState};
use redirection::IoRedirectOptions;

/**
 * I/O redirection manager
 * 
 * Manages I/O redirection, piping, and background process
 * I/O handling for terminal sessions.
 */
pub struct IoManager {
	/// Active I/O streams
	streams: Arc<RwLock<std::collections::HashMap<String, IoStream>>>,
	/// Pipeline connections
	pipelines: Arc<RwLock<Vec<Pipeline>>>,
	/// Background process I/O
	background_io: Arc<RwLock<std::collections::HashMap<u32, BackgroundIo>>>,
}

impl IoManager {
	/**
	 * Creates a new I/O manager
	 * 
	 * @return IoManager - New I/O manager instance
	 */
	pub fn new() -> Self {
		Self {
			streams: Arc::new(RwLock::new(std::collections::HashMap::new())),
			pipelines: Arc::new(RwLock::new(Vec::new())),
			background_io: Arc::new(RwLock::new(std::collections::HashMap::new())),
		}
	}
	
	/**
	 * Creates a new I/O stream
	 * 
	 * Creates a new I/O stream with the specified type
	 * and configuration options.
	 * 
	 * @param stream_type - Type of stream to create
	 * @param options - Stream creation options
	 * @return Result<String> - Stream ID or error
	 */
	pub async fn create_stream(&mut self, stream_type: StreamType, options: IoRedirectOptions) -> Result<String> {
		streams::StreamManager::create_stream(&mut self.streams, stream_type, options).await
	}
	
	/**
	 * Writes data to a stream
	 * 
	 * @param stream_id - Stream ID
	 * @param data - Data to write
	 * @return Result<usize> - Number of bytes written or error
	 */
	pub async fn write_to_stream(&self, stream_id: &str, data: &[u8]) -> Result<usize> {
		streams::StreamManager::write_to_stream(&self.streams, stream_id, data).await
	}
	
	/**
	 * Reads data from a stream
	 * 
	 * @param stream_id - Stream ID
	 * @param buffer - Buffer to read into
	 * @return Result<usize> - Number of bytes read or error
	 */
	pub async fn read_from_stream(&self, stream_id: &str, buffer: &mut [u8]) -> Result<usize> {
		streams::StreamManager::read_from_stream(&self.streams, stream_id, buffer).await
	}
	
	/**
	 * Creates a new pipeline
	 * 
	 * @param process_ids - Process IDs in the pipeline
	 * @return Result<String> - Pipeline ID or error
	 */
	pub async fn create_pipeline(&mut self, process_ids: Vec<u32>) -> Result<String> {
		pipelines::PipelineManager::create_pipeline(&mut self.pipelines, process_ids).await
	}
	
	/**
	 * Writes data to a pipeline
	 * 
	 * @param pipeline_id - Pipeline ID
	 * @param data - Data to write
	 * @return Result<usize> - Number of bytes written or error
	 */
	pub async fn write_to_pipeline(&self, pipeline_id: &str, data: &[u8]) -> Result<usize> {
		pipelines::PipelineManager::write_to_pipeline(&self.pipelines, pipeline_id, data).await
	}
	
	/**
	 * Reads data from a pipeline
	 * 
	 * @param pipeline_id - Pipeline ID
	 * @param buffer - Buffer to read into
	 * @return Result<usize> - Number of bytes read or error
	 */
	pub async fn read_from_pipeline(&self, pipeline_id: &str, buffer: &mut [u8]) -> Result<usize> {
		pipelines::PipelineManager::read_from_pipeline(&self.pipelines, pipeline_id, buffer).await
	}
	
	/**
	 * Sets up background I/O for a process
	 * 
	 * @param pid - Process ID
	 * @return Result<()> - Success or error
	 */
	pub async fn setup_background_io(&mut self, pid: u32) -> Result<()> {
		background::BackgroundIoManager::setup_background_io(&mut self.background_io, pid).await
	}
	
	/**
	 * Writes background output
	 * 
	 * @param pid - Process ID
	 * @param output - Output string
	 * @return Result<()> - Success or error
	 */
	pub async fn write_background_output(&self, pid: u32, output: String) -> Result<()> {
		background::BackgroundIoManager::write_background_output(&self.background_io, pid, output).await
	}
	
	/**
	 * Gets background output for a process
	 * 
	 * @param pid - Process ID
	 * @return Vec<String> - Background output
	 */
	pub async fn get_background_output(&self, pid: u32) -> Vec<String> {
		background::BackgroundIoManager::get_background_output(&self.background_io, pid).await
	}
	
	/**
	 * Closes a stream
	 * 
	 * @param stream_id - Stream ID
	 * @return Result<()> - Success or error
	 */
	pub async fn close_stream(&mut self, stream_id: &str) -> Result<()> {
		streams::StreamManager::close_stream(&mut self.streams, stream_id).await
	}
	
	/**
	 * Closes a pipeline
	 * 
	 * @param pipeline_id - Pipeline ID
	 * @return Result<()> - Success or error
	 */
	pub async fn close_pipeline(&mut self, pipeline_id: &str) -> Result<()> {
		pipelines::PipelineManager::close_pipeline(&mut self.pipelines, pipeline_id).await
	}
	
	/**
	 * Gets a stream by ID
	 * 
	 * @param stream_id - Stream ID
	 * @return Option<IoStream> - Stream or None
	 */
	pub async fn get_stream(&self, stream_id: &str) -> Option<IoStream> {
		streams::StreamManager::get_stream(&self.streams, stream_id).await
	}
	
	/**
	 * Lists all streams
	 * 
	 * @return Vec<IoStream> - List of streams
	 */
	pub async fn list_streams(&self) -> Vec<IoStream> {
		streams::StreamManager::list_streams(&self.streams).await
	}
	
	/**
	 * Lists all pipelines
	 * 
	 * @return Vec<Pipeline> - List of pipelines
	 */
	pub async fn list_pipelines(&self) -> Vec<Pipeline> {
		pipelines::PipelineManager::list_pipelines(&self.pipelines).await
	}
} 