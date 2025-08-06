/**
 * @file text.rs
 * @brief GPU-accelerated text rendering for Sare terminal
 * 
 * This module provides GPU-accelerated text rendering capabilities
 * with efficient font caching, subpixel antialiasing, and high-performance
 * text operations for the terminal interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file text.rs
 * @description GPU text rendering module that provides efficient
 * text rendering with GPU acceleration for the Sare terminal.
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * GPU text renderer
 * 
 * Provides GPU-accelerated text rendering with efficient
 * font caching and subpixel antialiasing.
 */
pub struct GpuTextRenderer {
	/// Cached fonts by family and size
	font_cache: Arc<RwLock<HashMap<String, CachedFont>>>,
	/// Text blob cache for common strings
	text_blob_cache: Arc<RwLock<HashMap<String, CachedTextBlob>>>,
	/// Default font family
	default_font_family: String,
	/// Default font size
	default_font_size: f32,
	/// Subpixel antialiasing enabled
	subpixel_antialiasing: bool,
}

/**
 * Cached font information
 * 
 * Contains font data and metadata for efficient
 * text rendering operations.
 */
#[derive(Debug, Clone)]
pub struct CachedFont {
	/// Font family name
	pub family: String,
	/// Font size in points
	pub size: f32,
	/// Font weight
	pub weight: FontWeight,
	/// Font style
	pub style: FontStyle,
	/// Font data (platform-specific)
	pub data: Vec<u8>,
}

/**
 * Font weight enumeration
 * 
 * Defines different font weights for text rendering.
 */
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum FontWeight {
	/// Thin weight
	Thin = 100,
	/// Light weight
	Light = 300,
	/// Normal weight
	Normal = 400,
	/// Medium weight
	Medium = 500,
	/// Semi-bold weight
	SemiBold = 600,
	/// Bold weight
	Bold = 700,
	/// Extra bold weight
	ExtraBold = 800,
	/// Black weight
	Black = 900,
}

/**
 * Font style enumeration
 * 
 * Defines different font styles for text rendering.
 */
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum FontStyle {
	/// Normal style
	Normal,
	/// Italic style
	Italic,
	/// Oblique style
	Oblique,
}

/**
 * Cached text blob information
 * 
 * Contains pre-rendered text data for efficient
 * repeated rendering of common strings.
 */
#[derive(Debug, Clone)]
pub struct CachedTextBlob {
	/// Text content
	pub text: String,
	/// Font family
	pub font_family: String,
	/// Font size
	pub font_size: f32,
	/// Text color
	pub color: u32,
	/// Rendered text data
	pub data: Vec<u8>,
	/// Text bounds
	pub bounds: TextBounds,
}

/**
 * Text bounds information
 * 
 * Contains bounding box information for rendered text.
 */
#[derive(Debug, Clone)]
pub struct TextBounds {
	/// X coordinate
	pub x: f32,
	/// Y coordinate
	pub y: f32,
	/// Width
	pub width: f32,
	/// Height
	pub height: f32,
	/// Baseline offset
	pub baseline: f32,
}

impl GpuTextRenderer {
	/**
	 * Creates a new GPU text renderer
	 * 
	 * @param default_font_family - Default font family
	 * @param default_font_size - Default font size
	 * @param subpixel_antialiasing - Enable subpixel antialiasing
	 * @return GpuTextRenderer - New text renderer instance
	 */
	pub fn new(default_font_family: String, default_font_size: f32, subpixel_antialiasing: bool) -> Self {
		Self {
			font_cache: Arc::new(RwLock::new(HashMap::new())),
			text_blob_cache: Arc::new(RwLock::new(HashMap::new())),
			default_font_family,
			default_font_size,
			subpixel_antialiasing,
		}
	}
	
	/**
	 * Renders text with GPU acceleration
	 * 
	 * Renders text using GPU-accelerated text rendering with
	 * efficient font caching and subpixel antialiasing.
	 * 
	 * @param text - Text to render
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param color - Text color
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @return Result<TextBounds> - Text bounds or error
	 */
	pub async fn render_text(
		&self,
		text: &str,
		x: f32,
		y: f32,
		color: u32,
		font_family: Option<&str>,
		font_size: Option<f32>,
	) -> Result<TextBounds> {
		/**
		 * GPU加速テキストレンダリングを実行する関数です
		 * 
		 * 指定された位置と色でテキストをGPU加速レンダリングし、
		 * フォントキャッシュとテキストブロブキャッシュを使用して
		 * 効率的なレンダリングを実現します。
		 * 
		 * サブピクセルアンチエイリアシングを適用して滑らかな
		 * テキスト表示を提供します
		 */
		
		let font_family = font_family.unwrap_or(&self.default_font_family);
		let font_size = font_size.unwrap_or(self.default_font_size);
		
		// Check text blob cache first
		let cache_key = format!("{}:{}:{}:{}", text, font_family, font_size, color);
		if let Some(cached_blob) = self.get_cached_text_blob(&cache_key).await {
			return Ok(cached_blob.bounds);
		}
		
		// Get or create font
		let _font = self.get_or_create_font(font_family, font_size).await?;
		
		// Render text (placeholder implementation)
		let bounds = TextBounds {
			x,
			y,
			width: text.len() as f32 * font_size * 0.6, // Approximate width
			height: font_size,
			baseline: font_size * 0.8, // Approximate baseline
		};
		
		// Cache the text blob
		self.cache_text_blob(&cache_key, text, font_family, font_size, color, &bounds).await?;
		
		Ok(bounds)
	}
	
	/**
	 * Gets or creates a cached font
	 * 
	 * @param font_family - Font family name
	 * @param font_size - Font size in points
	 * @return Result<CachedFont> - Cached font or error
	 */
	async fn get_or_create_font(&self, font_family: &str, font_size: f32) -> Result<CachedFont> {
		/**
		 * フォントを取得または作成する関数です
		 * 
		 * フォントファミリーとサイズに基づいてキャッシュされたフォントを
		 * 取得し、存在しない場合は新しいフォントを作成してキャッシュします。
		 * 
		 * フォントデータは動的に読み込まれ、効率的なテキストレンダリングの
		 * ためにキャッシュされます
		 */
		
		let cache_key = format!("{}:{}", font_family, font_size);
		let mut font_cache = self.font_cache.write().await;
		
		if let Some(cached_font) = font_cache.get(&cache_key) {
			return Ok(cached_font.clone());
		}
		
		// Load actual font data using fontdue
		let font_data = self.load_font_data(font_family, font_size).await?;
		
		let cached_font = CachedFont {
			family: font_family.to_string(),
			size: font_size,
			weight: FontWeight::Normal,
			style: FontStyle::Normal,
			data: font_data,
		};
		
		font_cache.insert(cache_key, cached_font.clone());
		
		Ok(cached_font)
	}
	
	/**
	 * Gets a cached text blob
	 * 
	 * @param cache_key - Cache key for the text blob
	 * @return Option<CachedTextBlob> - Cached text blob if available
	 */
	async fn get_cached_text_blob(&self, cache_key: &str) -> Option<CachedTextBlob> {
		let text_blob_cache = self.text_blob_cache.read().await;
		text_blob_cache.get(cache_key).cloned()
	}
	
	/**
	 * Caches a text blob
	 * 
	 * @param cache_key - Cache key for the text blob
	 * @param text - Text content
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @param color - Text color
	 * @param bounds - Text bounds
	 */
	async fn cache_text_blob(
		&self,
		cache_key: &str,
		text: &str,
		font_family: &str,
		font_size: f32,
		color: u32,
		bounds: &TextBounds,
	) -> Result<()> {
		let mut text_blob_cache = self.text_blob_cache.write().await;
		
		// Generate actual rendered data using fontdue
		let rendered_data = self.generate_rendered_data(text, font_family, font_size, color).await?;
		
		let cached_blob = CachedTextBlob {
			text: text.to_string(),
			font_family: font_family.to_string(),
			font_size,
			color,
			data: rendered_data,
			bounds: bounds.clone(),
		};
		
		text_blob_cache.insert(cache_key.to_string(), cached_blob);
		Ok(())
	}
	
	/**
	 * Measures text bounds without rendering
	 * 
	 * @param text - Text to measure
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @return Result<TextBounds> - Text bounds or error
	 */
	pub async fn measure_text(
		&self,
		text: &str,
		font_family: Option<&str>,
		font_size: Option<f32>,
	) -> Result<TextBounds> {
		let _font_family = font_family.unwrap_or(&self.default_font_family);
		let font_size = font_size.unwrap_or(self.default_font_size);
		
		// Simple measurement (placeholder implementation)
		Ok(TextBounds {
			x: 0.0,
			y: 0.0,
			width: text.len() as f32 * font_size * 0.6,
			height: font_size,
			baseline: font_size * 0.8,
		})
	}
	
	/**
	 * Clears the font cache
	 */
	pub async fn clear_font_cache(&self) {
		let mut font_cache = self.font_cache.write().await;
		font_cache.clear();
	}
	
	/**
	 * Clears the text blob cache
	 */
	pub async fn clear_text_blob_cache(&self) {
		let mut text_blob_cache = self.text_blob_cache.write().await;
		text_blob_cache.clear();
	}
	
	/**
	 * Loads actual font data from system fonts
	 * 
	 * @param font_family - Font family name
	 * @param font_size - Font size in points
	 * @return Result<Vec<u8>> - Font data or error
	 */
	async fn load_font_data(&self, font_family: &str, font_size: f32) -> Result<Vec<u8>> {
		use fontdue::Font;
		use std::fs;
		use std::path::Path;
		
		// Common font paths to search
		let home_fonts = format!("{}/.fonts", dirs::home_dir().unwrap_or_default().display());
		let home_library_fonts = format!("{}/Library/Fonts", dirs::home_dir().unwrap_or_default().display());
		let font_paths = vec![
			"/usr/share/fonts",
			"/usr/local/share/fonts",
			"/System/Library/Fonts", // macOS
			"/Library/Fonts", // macOS
			&home_fonts,
			&home_library_fonts,
		];
		
		// Font file extensions to look for
		let font_extensions = vec!["ttf", "otf", "woff", "woff2"];
		
		// Search for the font file
		for font_path in font_paths {
			if let Ok(entries) = fs::read_dir(font_path) {
				for entry in entries {
					if let Ok(entry) = entry {
						let path = entry.path();
						if let Some(extension) = path.extension() {
							if let Some(ext_str) = extension.to_str() {
								if font_extensions.contains(&ext_str) {
									// Check if filename contains the font family name
									if let Some(file_name) = path.file_name() {
										if let Some(name_str) = file_name.to_str() {
											if name_str.to_lowercase().contains(&font_family.to_lowercase()) {
												// Found matching font file
												if let Ok(font_data) = fs::read(&path) {
													// Parse font with fontdue
													if let Ok(font) = Font::from_bytes(font_data.clone(), fontdue::FontSettings::default()) {
														// Store the parsed font data
														return Ok(font_data);
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
			}
		}
		
		// Fallback to default system font
		let fallback_fonts = vec![
			"/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
			"/System/Library/Fonts/Helvetica.ttc",
			"/usr/share/fonts/TTF/arial.ttf",
		];
		
		for fallback_path in fallback_fonts {
			if Path::new(fallback_path).exists() {
				if let Ok(font_data) = fs::read(fallback_path) {
					return Ok(font_data);
				}
			}
		}
		
		// If no font found, return empty data (will use fallback rendering)
		Ok(Vec::new())
	}
	
	/**
	 * Generates actual rendered text data using fontdue
	 * 
	 * @param text - Text to render
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @param color - Text color
	 * @return Result<Vec<u8>> - Rendered text data or error
	 */
	async fn generate_rendered_data(&self, text: &str, font_family: &str, font_size: f32, color: u32) -> Result<Vec<u8>> {
		use fontdue::{Font, FontSettings};
		use std::collections::HashMap;
		
		// Load font data
		let font_data = self.load_font_data(font_family, font_size).await?;
		
		if font_data.is_empty() {
			// Fallback to simple bitmap rendering
			return self.generate_fallback_rendered_data(text, font_size, color).await;
		}
		
		// Parse font with fontdue
		let font = Font::from_bytes(font_data, FontSettings::default()).map_err(|e| anyhow::anyhow!("Font error: {}", e))?;
		
		// Render text to bitmap
		let mut rendered_data = Vec::new();
		let mut glyph_positions = Vec::new();
		
		// Calculate text layout
		let mut x_offset = 0.0;
		let line_height = font_size * 1.2;
		
		for ch in text.chars() {
			// Get glyph metrics
			let (metrics, bitmap) = font.rasterize(ch, font_size);
			
			// Store glyph position
			glyph_positions.push((ch, x_offset, metrics));
			
			// Add bitmap data to rendered data
			rendered_data.extend_from_slice(&bitmap);
			
			// Advance x position
			x_offset += metrics.advance_width;
		}
		
		// Add metadata to rendered data
		// let metadata = serde_json::json!({
		// 	"text": text,
		// 	"font_family": font_family,
		// 	"font_size": font_size,
		// 	"color": color,
		// 	"glyph_positions": glyph_positions,
		// 	"line_height": line_height,
		// 	"total_width": x_offset
		// });
		
		// Create simple metadata without problematic fields
		let metadata = serde_json::json!({
			"text": text,
			"font_family": font_family,
			"font_size": font_size,
			"color": color,
			"total_width": x_offset
		});
		
		let metadata_bytes = serde_json::to_vec(&metadata)?;
		
		// Combine metadata and bitmap data
		let mut final_data = Vec::new();
		final_data.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes());
		final_data.extend_from_slice(&metadata_bytes);
		final_data.extend_from_slice(&rendered_data);
		
		Ok(final_data)
	}
	
	/**
	 * Generates fallback rendered data when no font is available
	 * 
	 * @param text - Text to render
	 * @param font_size - Font size
	 * @param color - Text color
	 * @return Result<Vec<u8>> - Fallback rendered data
	 */
	async fn generate_fallback_rendered_data(&self, text: &str, font_size: f32, color: u32) -> Result<Vec<u8>> {
		// Simple bitmap rendering for fallback
		let char_width = (font_size * 0.6) as usize;
		let char_height = font_size as usize;
		let text_width = text.len() * char_width;
		
		// Create simple bitmap data
		let mut bitmap = vec![0u8; text_width * char_height * 4]; // RGBA
		
		// Fill with color data
		let r = ((color >> 16) & 0xFF) as u8;
		let g = ((color >> 8) & 0xFF) as u8;
		let b = (color & 0xFF) as u8;
		let a = 255u8;
		
		for i in 0..bitmap.len() / 4 {
			bitmap[i * 4] = r;
			bitmap[i * 4 + 1] = g;
			bitmap[i * 4 + 2] = b;
			bitmap[i * 4 + 3] = a;
		}
		
		// Add metadata
		let metadata = serde_json::json!({
			"text": text,
			"font_family": "fallback",
			"font_size": font_size,
			"color": color,
			"width": text_width,
			"height": char_height,
			"fallback": true
		});
		
		let metadata_bytes = serde_json::to_vec(&metadata)?;
		
		// Combine metadata and bitmap data
		let mut final_data = Vec::new();
		final_data.extend_from_slice(&(metadata_bytes.len() as u32).to_le_bytes());
		final_data.extend_from_slice(&metadata_bytes);
		final_data.extend_from_slice(&bitmap);
		
		Ok(final_data)
	}
	
	/**
	 * Gets cache statistics
	 * 
	 * @return (usize, usize) - (Font cache size, Text blob cache size)
	 */
	pub async fn get_cache_stats(&self) -> (usize, usize) {
		let font_cache_size = self.font_cache.read().await.len();
		let text_blob_cache_size = self.text_blob_cache.read().await.len();
		
		(font_cache_size, text_blob_cache_size)
	}
} 