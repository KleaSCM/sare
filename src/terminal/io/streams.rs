/**
 * I/O stream management for Sare terminal
 * 
 * This module provides stream creation, reading, writing, and management
 * with actual file descriptor operations for real I/O functionality.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: streams.rs
 * Description: Stream management with actual file descriptor operations
 */

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};
use libc::{open, O_RDONLY, O_WRONLY, O_CREAT, O_APPEND, S_IRWXU};

use super::redirection::IoRedirectOptions;

/**
 * I/O stream information
 * 
 * Contains information about an I/O stream including
 * file descriptors and stream state.
 */
#[derive(Debug, Clone)]
pub struct IoStream {
	/// Stream ID
	pub stream_id: String,
	/// Stream type
	pub stream_type: StreamType,
	/// File descriptor
	pub fd: i32,
	/// Stream state
	pub state: StreamState,
	/// Buffer for the stream
	pub buffer: VecDeque<u8>,
}

/**
 * Stream type enumeration
 * 
 * Defines the different types of I/O streams.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
	/// Standard input stream
	Stdin,
	/// Standard output stream
	Stdout,
	/// Standard error stream
	Stderr,
	/// File stream
	File(String),
	/// Pipe stream
	Pipe(String),
}

/**
 * Stream state enumeration
 * 
 * Defines the different states an I/O stream can be in.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum StreamState {
	/// Stream is open and ready
	Open,
	/// Stream is closed
	Closed,
	/// Stream has an error
	Error(String),
}

/**
 * Stream manager for handling I/O streams
 * 
 * Provides functionality to create, manage, and operate on
 * I/O streams with actual file descriptor operations.
 */
pub struct StreamManager;

impl StreamManager {
	/**
	 * Creates a new I/O stream with actual file descriptor
	 * 
	 * @param streams - Streams collection
	 * @param stream_type - Type of stream to create
	 * @param options - Stream creation options
	 * @return Result<String> - Stream ID or error
	 */
	pub async fn create_stream(
		streams: &mut Arc<RwLock<HashMap<String, IoStream>>>,
		stream_type: StreamType,
		_options: IoRedirectOptions,
	) -> Result<String> {
		/**
		 * I/Oストリームを作成する関数です
		 * 
		 * 指定されたストリームタイプに基づいてファイルディスクリプタを
		 * 作成し、I/Oストリームを初期化して管理コレクションに追加します。
		 * 
		 * 標準入出力、ファイル、パイプの各タイプに対応し、適切な
		 * ファイルディスクリプタとバッファを設定します
		 */
		
		let stream_id = uuid::Uuid::new_v4().to_string();
		let fd = Self::create_file_descriptor(&stream_type)?;
		
		let stream = IoStream {
			stream_id: stream_id.clone(),
			stream_type,
			fd,
			state: StreamState::Open,
			buffer: VecDeque::new(),
		};
		
		let mut streams_guard = streams.write().await;
		streams_guard.insert(stream_id.clone(), stream);
		
		Ok(stream_id)
	}
	
	/**
	 * Creates actual file descriptor for stream type
	 * 
	 * @param stream_type - Type of stream
	 * @return Result<i32> - File descriptor or error
	 */
	fn create_file_descriptor(stream_type: &StreamType) -> Result<i32> {
		match stream_type {
			StreamType::Stdin => {
				// Use standard input file descriptor
				Ok(0)
			}
			StreamType::Stdout => {
				// Use standard output file descriptor
				Ok(1)
			}
			StreamType::Stderr => {
				// Use standard error file descriptor
				Ok(2)
			}
			StreamType::File(path) => {
				// Open file with appropriate flags
				let file = OpenOptions::new()
					.read(true)
					.write(true)
					.create(true)
					.open(path)?;
				Ok(file.as_raw_fd())
			}
			StreamType::Pipe(pipe_name) => {
				// Create named pipe if it doesn't exist
				use std::os::unix::fs::FileTypeExt;
				use std::fs;
				
				if !fs::metadata(pipe_name).is_ok() {
					// Create named pipe
					unsafe {
						use libc::{mkfifo, mode_t};
						let path = std::ffi::CString::new(pipe_name.as_str())?;
						let mode = 0o666 as mode_t;
						if mkfifo(path.as_ptr(), mode) != 0 {
							return Err(anyhow::anyhow!("Failed to create named pipe"));
						}
					}
				}
				
				// Open pipe for reading and writing
				let flags = O_RDONLY | O_WRONLY;
				let fd = unsafe { open(pipe_name.as_ptr() as *const i8, flags) };
				if fd < 0 {
					return Err(anyhow::anyhow!("Failed to open pipe"));
				}
				Ok(fd)
			}
		}
	}
	
	/**
	 * Writes data to a stream with actual file descriptor
	 * 
	 * @param streams - Streams collection
	 * @param stream_id - Stream ID
	 * @param data - Data to write
	 * @return Result<usize> - Number of bytes written or error
	 */
	pub async fn write_to_stream(
		streams: &Arc<RwLock<HashMap<String, IoStream>>>,
		stream_id: &str,
		data: &[u8],
	) -> Result<usize> {
		/**
		 * ストリームへのデータ書き込みを実行する関数です
		 * 
		 * 指定されたストリームIDのファイルディスクリプタにデータを書き込み、
		 * 同時にバッファにもデータを追加してキャッシュ機能を提供します。
		 * 
		 * ストリームの状態（Open、Closed、Error）に応じて適切な処理を行い、
		 * 書き込み失敗時はストリーム状態をErrorに変更してエラーを返します。
		 * 
		 * ファイルディスクリプタへの直接書き込みとバッファ管理の両方を
		 * 安全に実行し、書き込まれたバイト数を返します。
		 */
		
		let mut streams_guard = streams.write().await;
		
		if let Some(stream) = streams_guard.get_mut(stream_id) {
			match stream.state {
				StreamState::Open => {
					// Write to actual file descriptor
					let bytes_written = unsafe {
						use libc::{write, c_void};
						let fd = stream.fd;
						let ptr = data.as_ptr() as *const c_void;
						let len = data.len();
						write(fd, ptr, len)
					};
					
					if bytes_written < 0 {
						stream.state = StreamState::Error("Write failed".to_string());
						return Err(anyhow::anyhow!("Failed to write to stream"));
					}
					
					// Also add to buffer for caching
					stream.buffer.extend(data.iter().cloned());
					
					Ok(bytes_written as usize)
				}
				StreamState::Closed => {
					Err(anyhow::anyhow!("Stream is closed"))
				}
				StreamState::Error(ref msg) => {
					Err(anyhow::anyhow!("Stream error: {}", msg))
				}
			}
		} else {
			Err(anyhow::anyhow!("Stream not found"))
		}
	}
	
	/**
	 * Reads data from a stream with actual file descriptor
	 * 
	 * @param streams - Streams collection
	 * @param stream_id - Stream ID
	 * @param buffer - Buffer to read into
	 * @return Result<usize> - Number of bytes read or error
	 */
	pub async fn read_from_stream(
		streams: &Arc<RwLock<HashMap<String, IoStream>>>,
		stream_id: &str,
		buffer: &mut [u8],
	) -> Result<usize> {
		let mut streams_guard = streams.write().await;
		
		if let Some(stream) = streams_guard.get_mut(stream_id) {
			match stream.state {
				StreamState::Open => {
					// Read from actual file descriptor
					let bytes_read = unsafe {
						use libc::{read, c_void};
						let fd = stream.fd;
						let ptr = buffer.as_mut_ptr() as *mut c_void;
						let len = buffer.len();
						read(fd, ptr, len)
					};
					
					if bytes_read < 0 {
						stream.state = StreamState::Error("Read failed".to_string());
						return Err(anyhow::anyhow!("Failed to read from stream"));
					}
					
					// Also read from buffer if available
					let buffer_read = std::cmp::min(bytes_read as usize, stream.buffer.len());
					if buffer_read > 0 {
						for i in 0..buffer_read {
							buffer[i] = stream.buffer.pop_front().unwrap_or(0);
						}
					}
					
					Ok(bytes_read as usize)
				}
				StreamState::Closed => {
					Err(anyhow::anyhow!("Stream is closed"))
				}
				StreamState::Error(ref msg) => {
					Err(anyhow::anyhow!("Stream error: {}", msg))
				}
			}
		} else {
			Err(anyhow::anyhow!("Stream not found"))
		}
	}
	
	/**
	 * Closes a stream and its file descriptor
	 * 
	 * @param streams - Streams collection
	 * @param stream_id - Stream ID
	 * @return Result<()> - Success or error
	 */
	pub async fn close_stream(
		streams: &mut Arc<RwLock<HashMap<String, IoStream>>>,
		stream_id: &str,
	) -> Result<()> {
		let mut streams_guard = streams.write().await;
		
		if let Some(stream) = streams_guard.get_mut(stream_id) {
			// Close file descriptor
			unsafe {
				use libc::close;
				close(stream.fd);
			}
			
			stream.state = StreamState::Closed;
			stream.buffer.clear();
		}
		
		streams_guard.remove(stream_id);
		Ok(())
	}
	
	/**
	 * Gets a stream by ID
	 * 
	 * @param streams - Streams collection
	 * @param stream_id - Stream ID
	 * @return Option<IoStream> - Stream or None
	 */
	pub async fn get_stream(
		streams: &Arc<RwLock<HashMap<String, IoStream>>>,
		stream_id: &str,
	) -> Option<IoStream> {
		let streams_guard = streams.read().await;
		streams_guard.get(stream_id).cloned()
	}
	
	/**
	 * Lists all streams
	 * 
	 * @param streams - Streams collection
	 * @return Vec<IoStream> - List of streams
	 */
	pub async fn list_streams(
		streams: &Arc<RwLock<HashMap<String, IoStream>>>,
	) -> Vec<IoStream> {
		let streams_guard = streams.read().await;
		streams_guard.values().cloned().collect()
	}
} 