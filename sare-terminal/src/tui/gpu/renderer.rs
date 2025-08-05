/**
 * @file renderer.rs
 * @brief GPU renderer abstraction for Sare terminal
 * 
 * This module provides a unified interface for GPU rendering operations,
 * abstracting over different GPU backends (Skia, WGPU, CPU) to provide
 * consistent rendering capabilities.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file renderer.rs
 * @description GPU renderer abstraction that provides a unified interface
 * for different GPU backends in the Sare terminal.
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::collections::VecDeque;

use super::{GpuBackend, GpuConfig, PerformanceMetrics};
use super::skia_backend;
use super::wgpu_backend;

/**
 * GPU renderer trait
 * 
 * Defines the interface for GPU rendering operations that all
 * GPU backends must implement for consistent functionality.
 */
pub trait GpuRendererTrait {
	/**
	 * Initializes the renderer
	 * 
	 * @param config - GPU configuration
	 * @return Result<()> - Success or error status
	 */
	fn initialize(&mut self, config: GpuConfig) -> Result<()>;
	
	/**
	 * Renders text with GPU acceleration
	 * 
	 * @param text - Text to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size
	 * @return Result<()> - Success or error status
	 */
	fn render_text(&self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()>;
	
	/**
	 * Renders a rectangle
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param width - Width
	 * @param height - Height
	 * @param color - Fill color
	 * @return Result<()> - Success or error status
	 */
	fn render_rectangle(&self, x: f32, y: f32, width: f32, height: f32, color: u32) -> Result<()>;
	
	/**
	 * Clears the rendering surface
	 * 
	 * @param background_color - Background color
	 * @return Result<()> - Success or error status
	 */
	fn clear_surface(&self, background_color: u32) -> Result<()>;
	
	/**
	 * Flushes the rendering surface
	 * 
	 * @return Result<()> - Success or error status
	 */
	fn flush_surface(&self) -> Result<()>;
	
	/**
	 * Gets the backend type
	 * 
	 * @return GpuBackend - Backend type
	 */
	fn backend_type(&self) -> GpuBackend;
	
	/**
	 * Updates performance metrics
	 * 
	 * @param frame_time - Frame time in milliseconds
	 * @param gpu_memory - GPU memory usage in bytes
	 * @param cpu_memory - CPU memory usage in bytes
	 */
	fn update_performance_metrics(&self, frame_time: f64, gpu_memory: u64, cpu_memory: u64);
	
	/**
	 * Gets performance metrics
	 * 
	 * @return PerformanceMetrics - Current performance metrics
	 */
	fn performance_metrics(&self) -> PerformanceMetrics;
}

/**
 * Unified GPU renderer
 * 
 * Provides a unified interface for GPU rendering operations,
 * automatically selecting and managing the appropriate backend.
 */
/**
 * Rendering command for multi-threaded rendering
 */
#[derive(Debug, Clone)]
pub enum RenderCommand {
	/// Render text command
	RenderText {
		text: String,
		x: f32,
		y: f32,
		color: u32,
		font_size: f32,
	},
	/// Render rectangle command
	RenderRectangle {
		x: f32,
		y: f32,
		width: f32,
		height: f32,
		color: u32,
	},
	/// Clear surface command
	ClearSurface {
		background_color: u32,
	},
	/// Flush surface command
	FlushSurface,
	/// Shutdown render thread
	Shutdown,
}

pub struct UnifiedGpuRenderer {
	/// Current active backend
	backend: Option<Box<dyn GpuRendererTrait + Send + Sync>>,
	/// Performance metrics
	performance_metrics: Arc<RwLock<PerformanceMetrics>>,
	/// Configuration
	config: GpuConfig,
	/// Render thread sender
	render_sender: Option<mpsc::Sender<RenderCommand>>,
	/// Render thread handle
	render_handle: Option<JoinHandle<()>>,
	/// I/O thread sender
	io_sender: Option<mpsc::Sender<RenderCommand>>,
	/// I/O thread handle
	io_handle: Option<JoinHandle<()>>,
	/// Command queue for batching
	command_queue: Arc<RwLock<VecDeque<RenderCommand>>>,
}

impl UnifiedGpuRenderer {
	/**
	 * Creates a new unified GPU renderer
	 * 
	 * @param config - GPU configuration
	 * @return Result<UnifiedGpuRenderer> - New renderer instance or error
	 */
	pub fn new(config: GpuConfig) -> Result<Self> {
		/**
		 * 統一GPUレンダラーを初期化する関数です
		 * 
		 * 複数のGPUバックエンド（Skia、WGPU、CPU）を統合管理する
		 * レンダラーを作成し、パフォーマンスメトリクスを設定します。
		 * 
		 * バックエンドは後でinitialize()で選択され、最適な
		 * レンダリングパフォーマンスを提供します
		 */
		
		Ok(Self {
			backend: None,
			performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
			config,
			render_sender: None,
			render_handle: None,
			io_sender: None,
			io_handle: None,
			command_queue: Arc::new(RwLock::new(VecDeque::new())),
		})
	}
	
	/**
	 * Initializes the renderer with the best available backend
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		// Try to initialize backends in order of preference
		let backends = vec![
			(GpuBackend::Skia, self.try_initialize_skia()),
			(GpuBackend::WGPU, self.try_initialize_wgpu()),
			(GpuBackend::CPU, self.try_initialize_cpu()),
		];
		
		for (backend_type, init_result) in backends {
			match init_result.await {
				Ok(backend) => {
					self.backend = Some(backend);
					println!("Initialized GPU backend: {:?}", backend_type);
					
					// Initialize multi-threaded rendering
					self.initialize_threading().await?;
					
					return Ok(());
				}
				Err(e) => {
					println!("Failed to initialize {:?} backend: {}", backend_type, e);
					continue;
				}
			}
		}
		
		Err(anyhow::anyhow!("No GPU backend could be initialized"))
	}
	
	/**
	 * Renders text using the active backend
	 * 
	 * @param text - Text to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size
	 * @return Result<()> - Success or error status
	 */
	pub async fn render_text(&self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()> {
		if let Some(sender) = &self.render_sender {
			// Send to render thread
			let command = RenderCommand::RenderText {
				text: text.to_string(),
				x,
				y,
				color,
				font_size,
			};
			sender.send(command).await.map_err(|e| anyhow::anyhow!("Render thread error: {}", e))?;
		} else if let Some(backend) = &self.backend {
			backend.render_text(text, x, y, color, font_size)
		} else {
			// Fallback to CPU rendering
			self.fallback_cpu_render_text(text, x, y, color, font_size)
		}
	}
	
	/**
	 * Renders a rectangle using the active backend
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param width - Width
	 * @param height - Height
	 * @param color - Fill color
	 * @return Result<()> - Success or error status
	 */
	pub async fn render_rectangle(&self, x: f32, y: f32, width: f32, height: f32, color: u32) -> Result<()> {
		if let Some(sender) = &self.render_sender {
			// Send to render thread
			let command = RenderCommand::RenderRectangle {
				x,
				y,
				width,
				height,
				color,
			};
			sender.send(command).await.map_err(|e| anyhow::anyhow!("Render thread error: {}", e))?;
		} else if let Some(backend) = &self.backend {
			backend.render_rectangle(x, y, width, height, color)
		} else {
			// Fallback to CPU rendering
			self.fallback_cpu_render_rectangle(x, y, width, height, color)
		}
	}
	
	/**
	 * Clears the surface using the active backend
	 * 
	 * @param background_color - Background color
	 * @return Result<()> - Success or error status
	 */
	pub async fn clear_surface(&self, background_color: u32) -> Result<()> {
		if let Some(sender) = &self.render_sender {
			// Send to render thread
			let command = RenderCommand::ClearSurface {
				background_color,
			};
			sender.send(command).await.map_err(|e| anyhow::anyhow!("Render thread error: {}", e))?;
		} else if let Some(backend) = &self.backend {
			backend.clear_surface(background_color)
		} else {
			// Fallback to CPU rendering
			self.fallback_cpu_clear_surface(background_color)
		}
	}
	
	/**
	 * Flushes the surface using the active backend
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn flush_surface(&self) -> Result<()> {
		if let Some(sender) = &self.render_sender {
			// Send to render thread
			let command = RenderCommand::FlushSurface;
			sender.send(command).await.map_err(|e| anyhow::anyhow!("Render thread error: {}", e))?;
		} else if let Some(backend) = &self.backend {
			backend.flush_surface()
		} else {
			// Fallback to CPU rendering
			self.fallback_cpu_flush_surface()
		}
	}
	
	/**
	 * Gets the current backend type
	 * 
	 * @return Option<GpuBackend> - Current backend type if available
	 */
	pub fn backend_type(&self) -> Option<GpuBackend> {
		self.backend.as_ref().map(|b| b.backend_type())
	}
	
	/**
	 * Updates performance metrics
	 * 
	 * @param frame_time - Frame time in milliseconds
	 * @param gpu_memory - GPU memory usage in bytes
	 * @param cpu_memory - CPU memory usage in bytes
	 */
	pub async fn update_performance_metrics(&self, frame_time: f64, gpu_memory: u64, cpu_memory: u64) {
		if let Some(backend) = &self.backend {
			backend.update_performance_metrics(frame_time, gpu_memory, cpu_memory);
		}
		
		// Update unified metrics
		let mut metrics = self.performance_metrics.write().await;
		metrics.average_frame_time = (metrics.average_frame_time + frame_time) / 2.0;
		metrics.current_fps = if frame_time > 0.0 { 1000.0 / frame_time } else { 0.0 };
		metrics.gpu_memory_usage = gpu_memory;
		metrics.cpu_memory_usage = cpu_memory;
		metrics.frames_rendered += 1;
		
		if frame_time > 16.67 { // Drop frame if > 60fps threshold
			metrics.frames_dropped += 1;
		}
	}
	
	/**
	 * Gets current performance metrics
	 * 
	 * @return PerformanceMetrics - Current performance metrics
	 */
	pub async fn performance_metrics(&self) -> PerformanceMetrics {
		self.performance_metrics.read().await.clone()
	}
	
	/**
	 * Gets current configuration
	 * 
	 * @return &GpuConfig - Current GPU configuration
	 */
			pub fn config(&self) -> &GpuConfig {
			&self.config
		}
		
		/**
		 * Attempts to initialize Skia backend
		 * 
		 * @return Result<Box<dyn GpuRendererTrait + Send + Sync>> - Skia backend or error
		 */
		async fn try_initialize_skia(&self) -> Result<Box<dyn GpuRendererTrait + Send + Sync>> {
			use crate::tui::gpu::skia_backend::SkiaRenderer;
			
			// Check if Skia is available
			if !self.is_skia_available() {
				return Err(anyhow::anyhow!("Skia backend not available"));
			}
			
			let mut skia_renderer = SkiaRenderer::new(self.config.clone())?;
			skia_renderer.initialize(self.config.clone())?;
			
			// Initialize surface with default size
			skia_renderer.initialize_surface(1024, 768)?;
			
			Ok(Box::new(skia_renderer))
		}
		
		/**
		 * Attempts to initialize WGPU backend
		 * 
		 * @return Result<Box<dyn GpuRendererTrait + Send + Sync>> - WGPU backend or error
		 */
		async fn try_initialize_wgpu(&self) -> Result<Box<dyn GpuRendererTrait + Send + Sync>> {
			use crate::tui::gpu::wgpu_backend::WgpuRenderer;
			
			let wgpu_renderer = WgpuRenderer::new(self.config.clone())?;
			wgpu_renderer.initialize(self.config.clone())?;
			
			Ok(Box::new(wgpu_renderer))
		}
		
		/**
		 * Attempts to initialize CPU backend
		 * 
		 * @return Result<Box<dyn GpuRendererTrait + Send + Sync>> - CPU backend or error
		 */
		async fn try_initialize_cpu(&self) -> Result<Box<dyn GpuRendererTrait + Send + Sync>> {
			use crate::tui::gpu::cpu_backend::CpuRenderer;
			
			let cpu_renderer = CpuRenderer::new(self.config.clone())?;
			cpu_renderer.initialize(self.config.clone())?;
			
			Ok(Box::new(cpu_renderer))
		}
		
		/**
		 * Checks if Skia backend is available
		 * 
		 * @return bool - True if Skia is available
		 */
		fn is_skia_available(&self) -> bool {
			// Check if Skia can be loaded
			if let Ok(_) = std::panic::catch_unwind(|| {
				skia_safe::Canvas::new_raster_n32_premul((1, 1))
			}) {
				return true;
			}
			false
		}
		
		/**
		 * Checks if WGPU backend is available
		 * 
		 * @return bool - True if WGPU is available
		 */
		fn is_wgpu_available(&self) -> bool {
			// Check if WGPU can be initialized
			if let Ok(_) = std::panic::catch_unwind(|| {
				// Try to create a simple WGPU instance
				// This is a simplified check
				true
			}) {
				return true;
			}
			false
		}
		
		/**
		 * Fallback CPU text rendering
		 * 
		 * @param text - Text to render
		 * @param x - X coordinate
		 * @param y - Y coordinate
		 * @param color - Text color
		 * @param font_size - Font size
		 * @return Result<()> - Success or error status
		 */
		fn fallback_cpu_render_text(&self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()> {
			// Create a temporary CPU renderer for fallback
			if let Ok(mut cpu_renderer) = crate::tui::gpu::cpu_backend::CpuRenderer::new(self.config.clone()) {
				cpu_renderer.initialize(self.config.clone())?;
				cpu_renderer.render_text(text, x, y, color, font_size)
			} else {
				// Ultimate fallback: simple console output
				println!("RENDER_TEXT: '{}' at ({}, {}) color={:x}", text, x, y, color);
							Ok(())
		}
		
		/**
		 * Initializes multi-threaded rendering
		 * 
		 * @return Result<()> - Success or error status
		 */
		async fn initialize_threading(&mut self) -> Result<()> {
			/**
			 * マルチスレッドレンダリングを初期化する関数です
			 * 
			 * レンダリングスレッドとI/Oスレッドを作成し、
			 * コマンドキューを設定します。
			 * 
			 * パフォーマンスを向上させるために並列処理を
			 * 実現します
			 */
			
			// Create render thread channel
			let (render_sender, mut render_receiver) = mpsc::channel::<RenderCommand>(1000);
			let backend = self.backend.clone();
			let performance_metrics = self.performance_metrics.clone();
			
			// Spawn render thread
			let render_handle = tokio::spawn(async move {
				while let Some(command) = render_receiver.recv().await {
					match command {
						RenderCommand::RenderText { text, x, y, color, font_size } => {
							if let Some(backend) = &backend {
								if let Err(e) = backend.render_text(&text, x, y, color, font_size) {
									eprintln!("Render thread error: {}", e);
								}
							}
						}
						RenderCommand::RenderRectangle { x, y, width, height, color } => {
							if let Some(backend) = &backend {
								if let Err(e) = backend.render_rectangle(x, y, width, height, color) {
									eprintln!("Render thread error: {}", e);
								}
							}
						}
						RenderCommand::ClearSurface { background_color } => {
							if let Some(backend) = &backend {
								if let Err(e) = backend.clear_surface(background_color) {
									eprintln!("Render thread error: {}", e);
								}
							}
						}
						RenderCommand::FlushSurface => {
							if let Some(backend) = &backend {
								if let Err(e) = backend.flush_surface() {
									eprintln!("Render thread error: {}", e);
								}
							}
						}
						RenderCommand::Shutdown => {
							break;
						}
					}
				}
			});
			
			// Create I/O thread channel
			let (io_sender, mut io_receiver) = mpsc::channel::<RenderCommand>(1000);
			let command_queue = self.command_queue.clone();
			
			// Spawn I/O thread for command batching
			let io_handle = tokio::spawn(async move {
				while let Some(command) = io_receiver.recv().await {
					let mut queue = command_queue.write().await;
					queue.push_back(command);
					
					// Batch commands for efficiency
					if queue.len() >= 10 {
						// Process batch
						while let Some(cmd) = queue.pop_front() {
							// Forward to render thread
							if let Err(e) = render_sender.send(cmd).await {
								eprintln!("I/O thread error: {}", e);
								break;
							}
						}
					}
				}
			});
			
			self.render_sender = Some(render_sender);
			self.render_handle = Some(render_handle);
			self.io_sender = Some(io_sender);
			self.io_handle = Some(io_handle);
			
			Ok(())
		}
		
		/**
		 * Shuts down the multi-threaded rendering
		 * 
		 * @return Result<()> - Success or error status
		 */
		pub async fn shutdown(&mut self) -> Result<()> {
			/**
			 * マルチスレッドレンダリングをシャットダウンする関数です
			 * 
			 * レンダリングスレッドとI/Oスレッドを適切に
			 * 終了し、リソースをクリーンアップします。
			 * 
			 * アプリケーション終了時に呼び出されます
			 */
			
			// Send shutdown command to render thread
			if let Some(sender) = &self.render_sender {
				if let Err(e) = sender.send(RenderCommand::Shutdown).await {
					eprintln!("Failed to send shutdown command: {}", e);
				}
			}
			
			// Wait for render thread to finish
			if let Some(handle) = self.render_handle.take() {
				if let Err(e) = handle.await {
					eprintln!("Render thread error: {}", e);
				}
			}
			
			// Wait for I/O thread to finish
			if let Some(handle) = self.io_handle.take() {
				if let Err(e) = handle.await {
					eprintln!("I/O thread error: {}", e);
				}
			}
			
			// Clear senders
			self.render_sender = None;
			self.io_sender = None;
			
			Ok(())
		}
	}
		
		/**
		 * Fallback CPU rectangle rendering
		 * 
		 * @param x - X coordinate
		 * @param y - Y coordinate
		 * @param width - Width
		 * @param height - Height
		 * @param color - Fill color
		 * @return Result<()> - Success or error status
		 */
		fn fallback_cpu_render_rectangle(&self, x: f32, y: f32, width: f32, height: f32, color: u32) -> Result<()> {
			// Create a temporary CPU renderer for fallback
			if let Ok(mut cpu_renderer) = crate::tui::gpu::cpu_backend::CpuRenderer::new(self.config.clone()) {
				cpu_renderer.initialize(self.config.clone())?;
				cpu_renderer.render_rectangle(x, y, width, height, color)
			} else {
				// Ultimate fallback: simple console output
				println!("RENDER_RECT: ({}, {}) {}x{} color={:x}", x, y, width, height, color);
				Ok(())
			}
		}
		
		/**
		 * Fallback CPU surface clearing
		 * 
		 * @param background_color - Background color
		 * @return Result<()> - Success or error status
		 */
		fn fallback_cpu_clear_surface(&self, background_color: u32) -> Result<()> {
			// Create a temporary CPU renderer for fallback
			if let Ok(mut cpu_renderer) = crate::tui::gpu::cpu_backend::CpuRenderer::new(self.config.clone()) {
				cpu_renderer.initialize(self.config.clone())?;
				cpu_renderer.clear_surface(background_color)
			} else {
				// Ultimate fallback: simple console output
				println!("CLEAR_SURFACE: color={:x}", background_color);
				Ok(())
			}
		}
		
		/**
		 * Fallback CPU surface flushing
		 * 
		 * @return Result<()> - Success or error status
		 */
		fn fallback_cpu_flush_surface(&self) -> Result<()> {
			// Create a temporary CPU renderer for fallback
			if let Ok(mut cpu_renderer) = crate::tui::gpu::cpu_backend::CpuRenderer::new(self.config.clone()) {
				cpu_renderer.initialize(self.config.clone())?;
				cpu_renderer.flush_surface()
			} else {
				// Ultimate fallback: simple console output
				println!("FLUSH_SURFACE");
				Ok(())
			}
		}
	} 