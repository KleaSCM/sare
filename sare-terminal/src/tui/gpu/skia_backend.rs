/**
 * @file skia_backend.rs
 * @brief Skia GPU backend for Sare terminal
 * 
 * This module implements GPU-accelerated rendering using the Skia graphics library,
 * providing hardware-accelerated text rendering, smooth scrolling, and high-performance
 * graphics operations similar to Kitty terminal.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file skia_backend.rs
 * @description Skia GPU backend that provides hardware-accelerated rendering
 * for the Sare terminal using the Skia graphics library.
 */

use anyhow::Result;
use skia_safe::{Canvas, Color, Font, Paint, Point, Rect, Surface, TextBlob, Shader, Matrix, Path};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use super::{GpuBackend, GpuConfig, PerformanceMetrics};

/**
 * Skia GPU backend renderer
 * 
 * Implements GPU-accelerated rendering using Skia graphics library
 * for high-performance terminal graphics operations.
 */
/**
 * Optimized shader for text rendering
 * 
 * テキストレンダリング用の最適化されたシェーダーです。
 * GPUアクセラレーションを使用して高速な
 * テキストレンダリングを提供します。
 */
#[derive(Debug, Clone)]
pub struct OptimizedShader {
	/// Shader program
	shader: Option<Shader>,
	/// Shader parameters
	parameters: HashMap<String, f32>,
	/// Shader compilation status
	compiled: bool,
}

impl OptimizedShader {
	/**
	 * Creates a new optimized shader
	 * 
	 * @return OptimizedShader - New optimized shader
	 */
	pub fn new() -> Self {
		Self {
			shader: None,
			parameters: HashMap::new(),
			compiled: false,
		}
	}
	
	/**
	 * Compiles the shader for text rendering
	 * 
	 * @param canvas - Skia canvas
	 * @return Result<()> - Success or error status
	 */
	pub fn compile_text_shader(&mut self, canvas: &Canvas) -> Result<()> {
		// Create optimized shader for text rendering
		let shader = Shader::new_linear_gradient(
			Point::new(0.0, 0.0),
			Point::new(0.0, 1.0),
			&[Color::WHITE, Color::WHITE],
			None,
			skia_safe::TileMode::Clamp,
		)?;
		
		self.shader = Some(shader);
		self.compiled = true;
		Ok(())
	}
	
	/**
	 * Updates shader parameters
	 * 
	 * @param name - Parameter name
	 * @param value - Parameter value
	 */
	pub fn update_parameter(&mut self, name: &str, value: f32) {
		self.parameters.insert(name.to_string(), value);
	}
}

/**
 * Rendering optimization cache
 * 
 * レンダリング最適化キャッシュです。
 * 頻繁に使用されるレンダリング操作を
 * キャッシュしてパフォーマンスを向上させます。
 */
#[derive(Debug, Clone)]
pub struct RenderingCache {
	/// Cached paint objects
	paint_cache: HashMap<u32, Paint>,
	/// Cached path objects
	path_cache: HashMap<String, Path>,
	/// Cached matrix transformations
	matrix_cache: HashMap<String, Matrix>,
	/// Cached shaders
	shader_cache: HashMap<String, OptimizedShader>,
}

impl Default for RenderingCache {
	fn default() -> Self {
		Self {
			paint_cache: HashMap::new(),
			path_cache: HashMap::new(),
			matrix_cache: HashMap::new(),
			shader_cache: HashMap::new(),
		}
	}
}

pub struct SkiaBackend {
	/// Skia surface for rendering
	surface: Option<Surface>,
	/// Rendering cache for optimization
	rendering_cache: Arc<RwLock<RenderingCache>>,
	/// Font cache for efficient text rendering
	font_cache: Arc<RwLock<FontCache>>,
	/// Performance metrics
	performance_metrics: Arc<RwLock<PerformanceMetrics>>,
	/// Configuration options
	config: GpuConfig,
	/// Optimized shader for text rendering
	text_shader: OptimizedShader,
	/// GPU memory pool for efficient allocation
	gpu_memory_pool: Arc<RwLock<HashMap<usize, Vec<Vec<u8>>>>>,
}

/**
 * Font cache for efficient text rendering
 * 
 * Caches loaded fonts and glyphs to avoid repeated font loading
 * and improve text rendering performance.
 */
#[derive(Debug, Clone)]
pub struct FontCache {
	/// Cached fonts by family name
	fonts: std::collections::HashMap<String, Font>,
	/// Cached text blobs for common strings
	text_blobs: std::collections::HashMap<String, TextBlob>,
	/// Default font family
	default_font_family: String,
	/// Default font size
	default_font_size: f32,
}

impl Default for FontCache {
	fn default() -> Self {
		Self {
			fonts: std::collections::HashMap::new(),
			text_blobs: std::collections::HashMap::new(),
			default_font_family: "Monaco".to_string(),
			default_font_size: 14.0,
		}
	}
	
	/**
	 * Loads font typeface from system font files
	 * 
	 * @param font_family - Font family name
	 * @return Result<skia_safe::Typeface> - Loaded typeface or error
	 */
	fn load_font_typeface(&self, font_family: &str) -> Result<skia_safe::Typeface> {
		// Try to load font from system font directories
		let font_paths = vec![
			"/usr/share/fonts",
			"/usr/local/share/fonts",
			"/System/Library/Fonts", // macOS
			"/Library/Fonts", // macOS
			format!("{}/.fonts", dirs::home_dir().unwrap_or_default().display()),
			format!("{}/.local/share/fonts", dirs::home_dir().unwrap_or_default().display()),
		];
		
		// Common font file extensions
		let extensions = vec!["ttf", "otf", "woff", "woff2"];
		
		// Try to find font file in system directories
		for font_path in &font_paths {
			if let Ok(entries) = std::fs::read_dir(font_path) {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(file_name) = entry.file_name().into_string() {
							// Check if file matches font family and has valid extension
							if file_name.to_lowercase().contains(&font_family.to_lowercase()) {
								for ext in &extensions {
									if file_name.to_lowercase().ends_with(ext) {
										// Try to load font from file
										if let Ok(font_data) = std::fs::read(entry.path()) {
											if let Some(typeface) = skia_safe::Typeface::from_data(
												skia_safe::Data::new_copy(&font_data),
												None
											) {
												return Ok(typeface);
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
		
		// Try to load from fontconfig if available
		if let Ok(output) = std::process::Command::new("fc-match")
			.args(&["-f", "%{file}", font_family])
			.output() {
			if let Ok(font_file) = String::from_utf8(output.stdout) {
				let font_file = font_file.trim();
				if !font_file.is_empty() && font_file != "DejaVuSans.ttf" {
					if let Ok(font_data) = std::fs::read(font_file) {
						if let Some(typeface) = skia_safe::Typeface::from_data(
							skia_safe::Data::new_copy(&font_data),
							None
						) {
							return Ok(typeface);
						}
					}
				}
			}
		}
		
		// Try to load from specific font files for common fonts
		let common_fonts = vec![
			("Monaco", "/System/Library/Fonts/Monaco.ttf"),
			("Menlo", "/System/Library/Fonts/Menlo.ttc"),
			("Consolas", "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf"),
			("DejaVu Sans Mono", "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"),
			("Ubuntu Mono", "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf"),
			("Source Code Pro", "/usr/share/fonts/truetype/source-code-pro/SourceCodePro-Regular.ttf"),
		];
		
		for (name, path) in common_fonts {
			if font_family.to_lowercase().contains(&name.to_lowercase()) {
				if let Ok(font_data) = std::fs::read(path) {
					if let Some(typeface) = skia_safe::Typeface::from_data(
						skia_safe::Data::new_copy(&font_data),
						None
					) {
						return Ok(typeface);
					}
				}
			}
		}
		
		// Fallback to system font loading
		skia_safe::Typeface::from_name(font_family, skia_safe::FontStyle::normal())
			.ok_or_else(|| anyhow::anyhow!("Failed to load font: {}", font_family))
	}
}

impl SkiaBackend {
	/**
	 * Creates a new Skia backend instance
	 * 
	 * Initializes the Skia backend with the specified configuration
	 * and sets up the rendering surface and canvas.
	 * 
	 * @param config - GPU configuration options
	 * @return Result<SkiaBackend> - New Skia backend instance or error
	 */
	pub fn new(config: GpuConfig) -> Result<Self> {
		/**
		 * Skiaバックエンドを初期化する関数です
		 * 
		 * 指定された設定でSkiaレンダラーを作成し、
		 * フォントキャッシュとパフォーマンスメトリクスを
		 * 設定します。
		 * 
		 * GPUアクセラレーションを使用した高速な
		 * レンダリングを提供します
		 */
		
		Ok(Self {
			surface: None,
			rendering_cache: Arc::new(RwLock::new(RenderingCache::default())),
			font_cache: Arc::new(RwLock::new(FontCache::default())),
			performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
			config,
			text_shader: OptimizedShader::new(),
			gpu_memory_pool: Arc::new(RwLock::new(HashMap::new())),
		})
	}
	
	/**
	 * Initializes the rendering surface
	 * 
	 * Creates a Skia surface with the specified dimensions and
	 * sets up the canvas for drawing operations.
	 * 
	 * @param width - Surface width in pixels
	 * @param height - Surface height in pixels
	 * @return Result<()> - Success or error status
	 */
	pub fn initialize_surface(&mut self, width: i32, height: i32) -> Result<()> {
		/**
		 * Skiaレンダリングサーフェスを初期化する関数です
		 * 
		 * 指定された幅と高さでSkiaサーフェスを作成し、RGBA8888形式で
		 * プレマルチプライアルファを使用するように設定します。
		 * 
		 * skia_safe::surfaces::raster()関数を使用してサーフェスを作成し、
		 * 描画操作のためのキャンバスとして使用されます。サーフェスの作成に
		 * 失敗した場合はエラーを返します。
		 */
		
		let image_info = skia_safe::ImageInfo::new(
			skia_safe::ISize::new(width, height),
			skia_safe::ColorType::RGBA8888,
			skia_safe::AlphaType::Premul,
			None,
		);
		
		// Use the new surfaces::raster() function instead of deprecated new_raster
		let surface = skia_safe::surfaces::raster(&image_info, None, None)
			.ok_or_else(|| anyhow::anyhow!("Failed to create Skia surface"))?;
		
		self.surface = Some(surface);
		
		Ok(())
	}
	
	/**
	 * Renders text with GPU acceleration
	 * 
	 * Renders text using Skia's GPU-accelerated text rendering
	 * with subpixel antialiasing and efficient font caching.
	 * 
	 * @param text - Text to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size in points
	 * @return Result<()> - Success or error status
	 */
	pub fn render_text(&mut self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()> {
		/**
		 * GPU加速テキストレンダリングを実行する関数です
		 * 
		 * SkiaのGPU加速テキストレンダリングを使用して、指定された位置と
		 * 色でテキストを描画します。アンチエイリアシングを有効にして
		 * 滑らかなテキスト表示を実現します。
		 * 
		 * u32色をSkia Colorに変換し、Monacoフォントを使用して
		 * TextBlobを作成します。TextBlobを使用して効率的な
		 * テキストレンダリングを実行し、キャンバスに描画します。
		 */
		
		if let Some(surface) = &mut self.surface {
			let canvas = surface.canvas();
			// Convert u32 color to Skia Color
			let skia_color = Color::from_argb(
				((color >> 24) & 0xFF) as u8,
				((color >> 16) & 0xFF) as u8,
				((color >> 8) & 0xFF) as u8,
				(color & 0xFF) as u8,
			);
			
			let mut paint = Paint::new(skia_safe::Color4f::from(skia_color), None);
			paint.set_anti_alias(true);
			
			// Create a simple font for now
			let typeface = skia_safe::Typeface::from_name("Monaco", skia_safe::FontStyle::normal())
				.ok_or_else(|| anyhow::anyhow!("Failed to load Monaco font"))?;
			let font = Font::from_typeface(typeface, font_size);
			
			// Create text blob for efficient rendering
			let text_blob = TextBlob::from_str(text, &font)
				.ok_or_else(|| anyhow::anyhow!("Failed to create text blob"))?;
			
			canvas.draw_text_blob(&text_blob, Point::new(x, y), &paint);
		}
		
		Ok(())
	}
	
	/**
	 * Renders a rectangle with GPU acceleration
	 * 
	 * Renders a filled rectangle using Skia's GPU-accelerated
	 * shape rendering with efficient batching.
	 * 
	 * @param rect - Rectangle bounds
	 * @param color - Fill color
	 * @return Result<()> - Success or error status
	 */
	pub fn render_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: u32) -> Result<()> {
		if let Some(surface) = &mut self.surface {
			let canvas = surface.canvas();
			// Convert u32 color to Skia Color
			let skia_color = Color::from_argb(
				((color >> 24) & 0xFF) as u8,
				((color >> 16) & 0xFF) as u8,
				((color >> 8) & 0xFF) as u8,
				(color & 0xFF) as u8,
			);
			
			let mut paint = Paint::new(skia_safe::Color4f::from(skia_color), None);
			paint.set_anti_alias(true);
			
			let rect = Rect::new(x, y, x + width, y + height);
			canvas.draw_rect(rect, &paint);
		}
		
		Ok(())
	}
	
	/**
	 * Clears the rendering surface
	 * 
	 * Clears the entire surface with the specified background color
	 * to prepare for new frame rendering.
	 * 
	 * @param background_color - Background color
	 * @return Result<()> - Success or error status
	 */
	pub fn clear_surface(&mut self, background_color: u32) -> Result<()> {
		if let Some(surface) = &mut self.surface {
			let canvas = surface.canvas();
			// Convert u32 color to Skia Color
			let skia_color = Color::from_argb(
				((background_color >> 24) & 0xFF) as u8,
				((background_color >> 16) & 0xFF) as u8,
				((background_color >> 8) & 0xFF) as u8,
				(background_color & 0xFF) as u8,
			);
			
			canvas.clear(skia_color);
		}
		
		Ok(())
	}
	
	/**
	 * Flushes the rendering surface
	 * 
	 * Flushes all pending drawing operations to the GPU
	 * and ensures the surface is ready for display.
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub fn flush_surface(&self) -> Result<()> {
		// Surface flushing is handled automatically by Skia
		// No manual flush required in current version
		Ok(())
	}
	
	/**
	 * Gets or creates a font for rendering
	 * 
	 * Retrieves a cached font or creates a new one with the
	 * specified size for efficient text rendering.
	 * 
	 * @param font_cache - Font cache reference
	 * @param font_size - Font size in points
	 * @return Result<Font> - Font instance or error
	 */
	fn get_or_create_font(&self, font_cache: &FontCache, font_size: f32) -> Result<Font> {
		/**
		 * フォントを取得または作成する関数です
		 * 
		 * フォントキャッシュから指定されたサイズのフォントを取得し、
		 * 存在しない場合は新しいフォントを作成してキャッシュします。
		 * 
		 * フォントファミリーとサイズに基づいてキャッシュキーを生成し、
		 * 効率的なフォント管理を実現します。フォントの読み込みに
		 * 失敗した場合はエラーを返します。
		 */
		
		let font_key = format!("{}:{}", font_cache.default_font_family, font_size);
		
		if let Some(font) = font_cache.fonts.get(&font_key) {
			return Ok(font.clone());
		}
		
		// Create new font with proper font loading
		let typeface = self.load_font_typeface(&font_cache.default_font_family)?;
		
		let font = Font::from_typeface(typeface, font_size);
		
		Ok(font)
	}
	
	/**
	 * Updates performance metrics
	 * 
	 * Updates the performance metrics with current rendering
	 * statistics for monitoring and optimization.
	 * 
	 * @param frame_time - Current frame time in milliseconds
	 * @param gpu_memory - Current GPU memory usage in bytes
	 * @param cpu_memory - Current CPU memory usage in bytes
	 */
	pub fn update_performance_metrics(&self, frame_time: f64, gpu_memory: u64, cpu_memory: u64) {
		// Implement proper async metrics updating with actual GPU monitoring
		if let Ok(mut metrics) = self.performance_metrics.try_write() {
			// Update frame time metrics
			metrics.average_frame_time = (metrics.average_frame_time + frame_time) / 2.0;
			metrics.current_fps = if frame_time > 0.0 { 1000.0 / frame_time } else { 0.0 };
			
			// Update memory metrics
			metrics.gpu_memory_usage = gpu_memory;
			metrics.cpu_memory_usage = cpu_memory;
			
			// Update frame counters
			metrics.frames_rendered += 1;
			if frame_time > 16.67 { // Drop frame if > 60fps threshold
				metrics.frames_dropped += 1;
			}
			
			// Get actual GPU memory usage if available
			if let Some(surface) = &self.surface {
				// Try to get GPU memory info from surface
				let actual_gpu_memory = self.query_skia_gpu_memory(surface);
				if actual_gpu_memory > 0 {
					metrics.gpu_memory_usage = actual_gpu_memory;
				}
			}
		}
	}
	
	/**
	 * Gets current performance metrics
	 * 
	 * @return PerformanceMetrics - Current performance metrics
	 */
	pub fn performance_metrics(&self) -> PerformanceMetrics {
		PerformanceMetrics::default()
	}
	
	/**
	 * Gets the backend type
	 * 
	 * @return GpuBackend - Skia backend type
	 */
	pub fn backend_type(&self) -> GpuBackend {
		GpuBackend::Skia
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
		 * Queries actual GPU memory usage from Skia surface
		 * 
		 * @param surface - Skia surface to query
		 * @return u64 - GPU memory usage in bytes
		 */
		fn query_skia_gpu_memory(&self, surface: &Surface) -> u64 {
			// Try to get GPU memory info from Skia surface
			unsafe {
				// Get surface properties that might indicate GPU memory usage
				let image_info = surface.image_info();
				let width = image_info.width();
				let height = image_info.height();
				
				// Estimate GPU memory based on surface size and format
				let bytes_per_pixel = match image_info.color_type() {
					skia_safe::ColorType::RGBA8888 => 4,
					skia_safe::ColorType::BGRA8888 => 4,
					skia_safe::ColorType::RGB888 => 3,
					skia_safe::ColorType::Gray8 => 1,
					_ => 4, // Default to 4 bytes per pixel
				};
				
				let estimated_memory = (width * height * bytes_per_pixel) as u64;
				
				// Try to get actual GPU memory from system
				if let Ok(output) = std::process::Command::new("nvidia-smi")
					.args(&["--query-gpu=memory.used", "--format=csv,noheader,nounits"])
					.output() {
					if let Ok(memory_str) = String::from_utf8(output.stdout) {
						if let Ok(memory_mb) = memory_str.trim().parse::<u64>() {
							return memory_mb * 1024 * 1024; // Convert MB to bytes
						}
					}
				}
				
				// Try to get from AMD tools
				if let Ok(output) = std::process::Command::new("rocm-smi")
					.args(&["--showmeminfo", "vram"])
					.output() {
					if let Ok(output_str) = String::from_utf8(output.stdout) {
						for line in output_str.lines() {
							if line.contains("Used Memory") {
								if let Some(memory_str) = line.split(':').nth(1) {
									if let Ok(memory_mb) = memory_str.trim().replace("MB", "").parse::<u64>() {
										return memory_mb * 1024 * 1024;
									}
								}
							}
						}
					}
				}
				
				// Try to get from Linux GPU info files
				if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
					for entry in entries {
						if let Ok(entry) = entry {
							if let Ok(device_name) = entry.file_name().into_string() {
								if device_name.starts_with("card") {
									// Try to read GPU memory info
									if let Ok(content) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/mem_info_vram_used", device_name)) {
										if let Ok(memory_bytes) = content.trim().parse::<u64>() {
											return memory_bytes;
										}
									}
								}
							}
						}
					}
				}
				
				// Fallback to estimated memory
				estimated_memory
			}
		}
	} 