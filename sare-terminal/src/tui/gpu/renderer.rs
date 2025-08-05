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
pub struct UnifiedGpuRenderer {
	/// Current active backend
	backend: Option<Box<dyn GpuRendererTrait + Send + Sync>>,
	/// Performance metrics
	performance_metrics: Arc<RwLock<PerformanceMetrics>>,
	/// Configuration
	config: GpuConfig,
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
	pub fn render_text(&self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()> {
		if let Some(backend) = &self.backend {
			backend.render_text(text, x, y, color, font_size)
		} else {
			// Fallback to no-op for now
			Ok(())
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
	pub fn render_rectangle(&self, x: f32, y: f32, width: f32, height: f32, color: u32) -> Result<()> {
		if let Some(backend) = &self.backend {
			backend.render_rectangle(x, y, width, height, color)
		} else {
			// Fallback to no-op for now
			Ok(())
		}
	}
	
	/**
	 * Clears the surface using the active backend
	 * 
	 * @param background_color - Background color
	 * @return Result<()> - Success or error status
	 */
	pub fn clear_surface(&self, background_color: u32) -> Result<()> {
		if let Some(backend) = &self.backend {
			backend.clear_surface(background_color)
		} else {
			// Fallback to no-op for now
			Ok(())
		}
	}
	
	/**
	 * Flushes the surface using the active backend
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub fn flush_surface(&self) -> Result<()> {
		if let Some(backend) = &self.backend {
			backend.flush_surface()
		} else {
			// Fallback to no-op for now
			Ok(())
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
	} 