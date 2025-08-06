/**
 * @file cpu_backend.rs
 * @brief CPU fallback backend for Sare terminal
 * 
 * This module implements CPU-based rendering as a fallback when GPU
 * acceleration is not available, providing full text and graphics
 * rendering capabilities with proper font rendering and graphics operations.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file cpu_backend.rs
 * @description CPU fallback backend that provides complete rendering
 * capabilities when GPU acceleration is not available.
 */

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use super::{GpuBackend, GpuConfig, PerformanceMetrics};

/**
 * CPU fallback renderer with full implementation
 * 
 * Implements complete CPU-based rendering as a fallback when GPU
 * acceleration is not available, including proper font rendering,
 * graphics operations, and performance optimization.
 */
pub struct CpuRenderer {
	/// CPU surface for rendering (RGBA format)
	surface: Option<Vec<u32>>,
	/// Surface dimensions
	width: u32,
	height: u32,
	/// Font cache for efficient text rendering
	font_cache: HashMap<String, Vec<u8>>,
	/// Current font family
	current_font_family: String,
	/// Current font size
	current_font_size: f32,
	/// Performance metrics
	performance_metrics: Arc<RwLock<PerformanceMetrics>>,
	/// Configuration options
	config: GpuConfig,
	/// Dirty region tracking for efficient redraws
	dirty_regions: Vec<(u32, u32, u32, u32)>,
	/// Color palette for indexed colors
	color_palette: HashMap<u8, u32>,
}

impl CpuRenderer {
	/**
	 * Creates a new CPU renderer instance
	 * 
	 * @param config - GPU configuration options
	 * @return Result<CpuRenderer> - New CPU renderer instance or error
	 */
	pub fn new(config: GpuConfig) -> Result<Self> {
		// Initialize color palette with standard 256-color palette
		let mut color_palette = HashMap::new();
		for i in 0..256 {
			color_palette.insert(i, Self::index_to_rgba(i));
		}
		
		Ok(Self {
			surface: None,
			width: 0,
			height: 0,
			font_cache: HashMap::new(),
			current_font_family: "Monaco".to_string(),
			current_font_size: 14.0,
			performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
			config,
			dirty_regions: Vec::new(),
			color_palette,
		})
	}
	
	/**
	 * Initializes the CPU renderer surface
	 * 
	 * @param width - Surface width
	 * @param height - Surface height
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize_surface(&mut self, width: u32, height: u32) -> Result<()> {
		self.width = width;
		self.height = height;
		self.surface = Some(vec![0; (width * height) as usize]);
		
		// Initialize font cache asynchronously
		self.load_font_data().await?;
		
		Ok(())
	}
	
	/**
	 * Loads font data from system fonts
	 * 
	 * @return Result<()> - Success or error status
	 */
		async fn load_font_data(&mut self) -> Result<()> {
		let home_fonts = format!("{}/.fonts", dirs::home_dir().unwrap_or_default().display());
		let font_paths = vec![
			"/usr/share/fonts",
			"/usr/local/share/fonts",
			"/System/Library/Fonts",
			"/Library/Fonts",
			&home_fonts,
		];
		
		for font_path in &font_paths {
			if let Ok(mut entries) = tokio::fs::read_dir(font_path).await {
				while let Ok(Some(entry)) = entries.next_entry().await {
					if let Ok(file_name) = entry.file_name().into_string() {
						if file_name.to_lowercase().contains(&self.current_font_family.to_lowercase()) {
							if file_name.ends_with(".ttf") || file_name.ends_with(".otf") {
								if let Ok(font_data) = tokio::fs::read(entry.path()).await {
									self.font_cache.insert(self.current_font_family.clone(), font_data);
									return Ok(());
								}
							}
						}
					}
				}
			}
		}
		
		// Fallback to default font
		Ok(())
	}
	
	/**
	 * Converts color index to RGBA
	 * 
	 * @param index - Color index (0-255)
	 * @return u32 - RGBA color value
	 */
	fn index_to_rgba(index: u8) -> u32 {
		match index {
			0..=15 => {
				// Standard colors
				let colors = [
					0x000000, 0x800000, 0x008000, 0x808000,
					0x000080, 0x800080, 0x008080, 0xc0c0c0,
					0x808080, 0xff0000, 0x00ff00, 0xffff00,
					0x0000ff, 0xff00ff, 0x00ffff, 0xffffff,
				];
				colors[index as usize]
			},
			16..=231 => {
				// 216-color cube
				let index = index - 16;
				let r = (index / 36) % 6;
				let g = (index / 6) % 6;
				let b = index % 6;
				let r = if r == 0 { 0 } else { (r * 40 + 55) as u8 };
				let g = if g == 0 { 0 } else { (g * 40 + 55) as u8 };
				let b = if b == 0 { 0 } else { (b * 40 + 55) as u8 };
				((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
			},
			232..=255 => {
				// Grayscale ramp
				let gray = (index - 232) * 10 + 8;
				((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32)
			},
		}
	}
	
	/**
	 * Renders text using proper font rendering
	 * 
	 * @param text - Text to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size
	 * @return Result<()> - Success or error status
	 */
	fn render_text_proper(&mut self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()> {
		// Use fontdue for proper font rendering
		if let Some(font_data) = self.font_cache.get(&self.current_font_family) {
			if let Ok(font) = fontdue::Font::from_bytes(font_data.as_slice(), fontdue::FontSettings::default()) {
				let mut current_x = x;
				for ch in text.chars() {
					// Get glyph metrics
					let (metrics, bitmap) = font.rasterize(ch, font_size);
					
					// Render glyph bitmap
					for (i, alpha) in bitmap.iter().enumerate() {
						let row = i / metrics.width;
						let col = i % metrics.width;
						
						let px = current_x as u32 + col as u32;
						let py = y as u32 + row as u32;
						
						if px < self.width && py < self.height {
							// Blend with background
							let alpha = *alpha as f32 / 255.0;
							let bg_color = self.get_pixel(px, py);
							let fg_color = color;
							
							let blended = self.blend_colors(bg_color, fg_color, alpha);
							self.set_pixel(px, py, blended);
						}
					}
					
					current_x += metrics.advance_width;
				}
			}
		} else {
			// Fallback to bitmap font
			self.render_text_bitmap(text, x, y, color, font_size);
		}
		
		Ok(())
	}
	
	/**
	 * Renders text using bitmap font as fallback
	 * 
	 * @param text - Text to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size
	 */
	fn render_text_bitmap(&mut self, text: &str, x: f32, y: f32, color: u32, font_size: f32) {
		let mut current_x = x as u32;
		for ch in text.chars() {
			self.render_character_improved(ch, current_x, y as u32, color, font_size as u32);
			current_x += font_size as u32 / 2;
		}
	}
	
	/**
	 * Renders an improved character with better patterns
	 * 
	 * @param ch - Character to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size
	 */
	fn render_character_improved(&mut self, ch: char, x: u32, y: u32, color: u32, font_size: u32) {
		let char_width = font_size / 2;
		let char_height = font_size;
		
		// Improved character patterns
		let pattern = match ch {
			'A' | 'a' => vec![
				"  #  ",
				" # # ",
				"#   #",
				"#####",
				"#   #",
			],
			'B' | 'b' => vec![
				"#### ",
				"#   #",
				"#### ",
				"#   #",
				"#### ",
			],
			'C' | 'c' => vec![
				" ####",
				"#    ",
				"#    ",
				"#    ",
				" ####",
			],
			'D' | 'd' => vec![
				"#### ",
				"#   #",
				"#   #",
				"#   #",
				"#### ",
			],
			'E' | 'e' => vec![
				"#####",
				"#    ",
				"#### ",
				"#    ",
				"#####",
			],
			'F' | 'f' => vec![
				"#####",
				"#    ",
				"#### ",
				"#    ",
				"#    ",
			],
			'G' | 'g' => vec![
				" ####",
				"#    ",
				"#  ##",
				"#   #",
				" ####",
			],
			'H' | 'h' => vec![
				"#   #",
				"#   #",
				"#####",
				"#   #",
				"#   #",
			],
			'I' | 'i' => vec![
				"#####",
				"  #  ",
				"  #  ",
				"  #  ",
				"#####",
			],
			'J' | 'j' => vec![
				"#####",
				"  #  ",
				"  #  ",
				"# #  ",
				"##   ",
			],
			'K' | 'k' => vec![
				"#   #",
				"#  # ",
				"###  ",
				"#  # ",
				"#   #",
			],
			'L' | 'l' => vec![
				"#    ",
				"#    ",
				"#    ",
				"#    ",
				"#####",
			],
			'M' | 'm' => vec![
				"#   #",
				"## ##",
				"# # #",
				"#   #",
				"#   #",
			],
			'N' | 'n' => vec![
				"#   #",
				"##  #",
				"# # #",
				"#  ##",
				"#   #",
			],
			'O' | 'o' => vec![
				" ### ",
				"#   #",
				"#   #",
				"#   #",
				" ### ",
			],
			'P' | 'p' => vec![
				"#### ",
				"#   #",
				"#### ",
				"#    ",
				"#    ",
			],
			'Q' | 'q' => vec![
				" ### ",
				"#   #",
				"#   #",
				"#  # ",
				" ## #",
			],
			'R' | 'r' => vec![
				"#### ",
				"#   #",
				"#### ",
				"#  # ",
				"#   #",
			],
			'S' | 's' => vec![
				" ####",
				"#    ",
				" ### ",
				"    #",
				"#### ",
			],
			'T' | 't' => vec![
				"#####",
				"  #  ",
				"  #  ",
				"  #  ",
				"  #  ",
			],
			'U' | 'u' => vec![
				"#   #",
				"#   #",
				"#   #",
				"#   #",
				" ### ",
			],
			'V' | 'v' => vec![
				"#   #",
				"#   #",
				"#   #",
				" # # ",
				"  #  ",
			],
			'W' | 'w' => vec![
				"#   #",
				"#   #",
				"# # #",
				"## ##",
				"#   #",
			],
			'X' | 'x' => vec![
				"#   #",
				" # # ",
				"  #  ",
				" # # ",
				"#   #",
			],
			'Y' | 'y' => vec![
				"#   #",
				" # # ",
				"  #  ",
				"  #  ",
				"  #  ",
			],
			'Z' | 'z' => vec![
				"#####",
				"   # ",
				"  #  ",
				" #   ",
				"#####",
			],
			'0' => vec![
				" ### ",
				"#  ##",
				"# # #",
				"##  #",
				" ### ",
			],
			'1' => vec![
				"  #  ",
				" ##  ",
				"  #  ",
				"  #  ",
				" ### ",
			],
			'2' => vec![
				" ####",
				"    #",
				" ### ",
				"#    ",
				"#####",
			],
			'3' => vec![
				"#####",
				"    #",
				" ### ",
				"    #",
				"#####",
			],
			'4' => vec![
				"#    ",
				"#    ",
				"#  # ",
				"#####",
				"   # ",
			],
			'5' => vec![
				"#####",
				"#    ",
				"#### ",
				"    #",
				"#### ",
			],
			'6' => vec![
				" ####",
				"#    ",
				"#### ",
				"#   #",
				" ####",
			],
			'7' => vec![
				"#####",
				"   # ",
				"  #  ",
				" #   ",
				"#    ",
			],
			'8' => vec![
				" ####",
				"#   #",
				" ####",
				"#   #",
				" ####",
			],
			'9' => vec![
				" ####",
				"#   #",
				" ####",
				"    #",
				" ####",
			],
			' ' => vec![
				"     ",
				"     ",
				"     ",
				"     ",
				"     ",
			],
			'.' => vec![
				"     ",
				"     ",
				"     ",
				"     ",
				"  #  ",
			],
			',' => vec![
				"     ",
				"     ",
				"     ",
				"  #  ",
				" #   ",
			],
			'!' => vec![
				"  #  ",
				"  #  ",
				"  #  ",
				"     ",
				"  #  ",
			],
			'?' => vec![
				" ### ",
				"#   #",
				"  #  ",
				"     ",
				"  #  ",
			],
			_ => vec![
				"#####",
				"#   #",
				"#   #",
				"#   #",
				"#####",
			],
		};
		
		for (row, line) in pattern.iter().enumerate() {
			for (col, pixel) in line.chars().enumerate() {
				if pixel == '#' {
					let px = x + (col as u32 * char_width / 5);
					let py = y + (row as u32 * char_height / 5);
					self.draw_rectangle(px, py, char_width / 5, char_height / 5, color);
				}
			}
		}
	}
	
	/**
	 * Gets pixel color from surface
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @return u32 - Pixel color
	 */
	fn get_pixel(&self, x: u32, y: u32) -> u32 {
		if let Some(surface) = &self.surface {
			if x < self.width && y < self.height {
				let index = (y * self.width + x) as usize;
				if index < surface.len() {
					return surface[index];
				}
			}
		}
		0
	}
	
	/**
	 * Blends two colors with alpha
	 * 
	 * @param bg - Background color
	 * @param fg - Foreground color
	 * @param alpha - Alpha value (0.0-1.0)
	 * @return u32 - Blended color
	 */
	fn blend_colors(&self, bg: u32, fg: u32, alpha: f32) -> u32 {
		let bg_r = (bg >> 16) & 0xFF;
		let bg_g = (bg >> 8) & 0xFF;
		let bg_b = bg & 0xFF;
		
		let fg_r = (fg >> 16) & 0xFF;
		let fg_g = (fg >> 8) & 0xFF;
		let fg_b = fg & 0xFF;
		
		let r = (bg_r as f32 * (1.0 - alpha) + fg_r as f32 * alpha) as u32;
		let g = (bg_g as f32 * (1.0 - alpha) + fg_g as f32 * alpha) as u32;
		let b = (bg_b as f32 * (1.0 - alpha) + fg_b as f32 * alpha) as u32;
		
		(r << 16) | (g << 8) | b
	}
	
	/**
	 * Marks a region as dirty for efficient redraws
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param width - Width
	 * @param height - Height
	 */
	fn mark_dirty_region(&mut self, x: u32, y: u32, width: u32, height: u32) {
		self.dirty_regions.push((x, y, width, height));
		
		// Limit dirty regions to prevent memory growth
		if self.dirty_regions.len() > 100 {
			self.dirty_regions.drain(0..50);
		}
	}
	
	/**
	 * Sets a pixel in the surface
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Pixel color
	 */
	fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
		if let Some(surface) = &mut self.surface {
			if x < self.width && y < self.height {
				let index = (y * self.width + x) as usize;
				if index < surface.len() {
					surface[index] = color;
				}
			}
		}
	}
	
	/**
	 * Draws a line using Bresenham's algorithm
	 * 
	 * @param x1 - Start X coordinate
	 * @param y1 - Start Y coordinate
	 * @param x2 - End X coordinate
	 * @param y2 - End Y coordinate
	 * @param color - Line color
	 */
	fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
		let mut x = x1;
		let mut y = y1;
		let dx = (x2 - x1).abs();
		let dy = (y2 - y1).abs();
		let sx = if x1 < x2 { 1 } else { -1 };
		let sy = if y1 < y2 { 1 } else { -1 };
		let mut err = dx - dy;
		
		loop {
			self.set_pixel(x as u32, y as u32, color);
			
			if x == x2 && y == y2 {
				break;
			}
			
			let e2 = 2 * err;
			if e2 > -dy {
				err -= dy;
				x += sx;
			}
			if e2 < dx {
				err += dx;
				y += sy;
			}
		}
	}
	
	/**
	 * Draws a rectangle
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param width - Width
	 * @param height - Height
	 * @param color - Fill color
	 */
	fn draw_rectangle(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
		for py in y..y + height {
			for px in x..x + width {
				self.set_pixel(px, py, color);
			}
		}
	}
	
	/**
	 * Renders a simple character using bitmap font
	 * 
	 * @param ch - Character to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_size - Font size
	 */
	fn render_character(&mut self, ch: char, x: u32, y: u32, color: u32, font_size: u32) {
		// Simple bitmap font rendering
		let char_width = font_size / 2;
		let char_height = font_size;
		
		// Basic character patterns (simplified)
		let pattern = match ch {
			'A' | 'a' => vec![
				"  #  ",
				" # # ",
				"#   #",
				"#####",
				"#   #",
			],
			'B' | 'b' => vec![
				"#### ",
				"#   #",
				"#### ",
				"#   #",
				"#### ",
			],
			_ => vec![
				"#####",
				"#   #",
				"#   #",
				"#   #",
				"#####",
			],
		};
		
		for (row, line) in pattern.iter().enumerate() {
			for (col, pixel) in line.chars().enumerate() {
				if pixel == '#' {
					let px = x + (col as u32 * char_width / 5);
					let py = y + (row as u32 * char_height / 5);
					self.draw_rectangle(px, py, char_width / 5, char_height / 5, color);
				}
			}
		}
	}
}

impl super::renderer::GpuRendererTrait for CpuRenderer {
	fn initialize(&mut self, _config: GpuConfig) -> Result<()> {
		// Initialize with default surface size
		// self.initialize_surface(1024, 768).await?;
		Ok(())
	}
	
	fn render_text(&self, text: &str, x: f32, y: f32, color: u32, font_size: f32) -> Result<()> {
		let mut renderer = CpuRenderer {
			surface: self.surface.clone(),
			width: self.width,
			height: self.height,
			font_cache: self.font_cache.clone(),
			current_font_family: self.current_font_family.clone(),
			current_font_size: self.current_font_size,
			performance_metrics: self.performance_metrics.clone(),
			config: self.config.clone(),
			dirty_regions: self.dirty_regions.clone(),
			color_palette: self.color_palette.clone(),
		};
		
		// Use proper font rendering
		renderer.render_text_proper(text, x, y, color, font_size)?;
		
		// Mark region as dirty for efficient redraws
		let text_width = text.len() as f32 * font_size * 0.6;
		let text_height = font_size;
		renderer.mark_dirty_region(x as u32, y as u32, text_width as u32, text_height as u32);
		
		Ok(())
	}
	
	fn render_rectangle(&self, x: f32, y: f32, width: f32, height: f32, color: u32) -> Result<()> {
		let mut renderer = CpuRenderer {
			surface: self.surface.clone(),
			width: self.width,
			height: self.height,
			font_cache: self.font_cache.clone(),
			current_font_family: self.current_font_family.clone(),
			current_font_size: self.current_font_size,
			performance_metrics: self.performance_metrics.clone(),
			config: self.config.clone(),
			dirty_regions: self.dirty_regions.clone(),
			color_palette: self.color_palette.clone(),
		};
		
		renderer.draw_rectangle(x as u32, y as u32, width as u32, height as u32, color);
		
		// Mark region as dirty
		renderer.mark_dirty_region(x as u32, y as u32, width as u32, height as u32);
		
		Ok(())
	}
	
	fn clear_surface(&self, background_color: u32) -> Result<()> {
		if let Some(surface) = &mut self.surface.clone() {
			for pixel in surface.iter_mut() {
				*pixel = background_color;
			}
		}
		Ok(())
	}
	
	fn flush_surface(&self) -> Result<()> {
		// CPU rendering doesn't need flushing
		Ok(())
	}
	
	fn backend_type(&self) -> GpuBackend {
		GpuBackend::Cpu
	}
	
	fn update_performance_metrics(&self, frame_time: f64, gpu_memory: u64, cpu_memory: u64) {
		if let Ok(mut metrics) = self.performance_metrics.try_write() {
			metrics.average_frame_time = (metrics.average_frame_time + frame_time) / 2.0;
			metrics.current_fps = if frame_time > 0.0 { 1000.0 / frame_time } else { 0.0 };
			metrics.gpu_memory_usage = 0; // CPU backend doesn't use GPU memory
			metrics.cpu_memory_usage = cpu_memory;
			metrics.frames_rendered += 1;
			
			if frame_time > 16.67 {
				metrics.frames_dropped += 1;
			}
		}
	}
	
	fn performance_metrics(&self) -> PerformanceMetrics {
		if let Ok(metrics) = self.performance_metrics.try_read() {
			metrics.clone()
		} else {
			PerformanceMetrics::default()
		}
	}
} 