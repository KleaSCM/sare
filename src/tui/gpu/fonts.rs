/**
 * @file fonts.rs
 * @brief Font management and caching for Sare terminal
 * 
 * This module provides font management capabilities including font loading,
 * caching, and optimization for GPU-accelerated text rendering in the
 * Sare terminal interface.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file fonts.rs
 * @description Font management module that provides efficient font loading
 * and caching for GPU-accelerated text rendering in the Sare terminal.
 */

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::text::{FontWeight, FontStyle};

/**
 * Font manager for GPU rendering
 * 
 * Manages font loading, caching, and optimization for
 * GPU-accelerated text rendering operations.
 */
pub struct FontManager {
	/// Cached fonts by family and size
	font_cache: Arc<RwLock<HashMap<String, CachedFont>>>,
	/// Font search paths
	font_paths: Vec<PathBuf>,
	/// Default font family
	default_font_family: String,
	/// Default font size
	default_font_size: f32,
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
	/// Font file path
	pub file_path: Option<PathBuf>,
	/// Font data (platform-specific)
	pub data: Vec<u8>,
	/// Glyph cache
	pub glyph_cache: HashMap<char, CachedGlyph>,
}

/**
 * Cached glyph information
 * 
 * Contains individual glyph data for efficient
 * text rendering operations.
 */
#[derive(Debug, Clone)]
pub struct CachedGlyph {
	/// Character code
	pub character: char,
	/// Glyph width
	pub width: f32,
	/// Glyph height
	pub height: f32,
	/// Glyph advance
	pub advance: f32,
	/// Glyph bounds
	pub bounds: GlyphBounds,
	/// Glyph texture data
	pub texture_data: Vec<u8>,
}

/**
 * Glyph bounds information
 * 
 * Contains bounding box information for individual glyphs.
 */
#[derive(Debug, Clone)]
pub struct GlyphBounds {
	/// Left bound
	pub left: f32,
	/// Top bound
	pub top: f32,
	/// Right bound
	pub right: f32,
	/// Bottom bound
	pub bottom: f32,
}

impl FontManager {
	/**
	 * Creates a new font manager
	 * 
	 * @param default_font_family - Default font family
	 * @param default_font_size - Default font size
	 * @return FontManager - New font manager instance
	 */
	pub fn new(default_font_family: String, default_font_size: f32) -> Self {
		/**
		 * フォントマネージャー初期化の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なフォントパス検索を行います。
		 * システムフォントディレクトリの検出が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut font_paths = Vec::new();
		
		// Add system font paths
		if cfg!(target_os = "linux") {
			font_paths.push(PathBuf::from("/usr/share/fonts"));
			font_paths.push(PathBuf::from("/usr/local/share/fonts"));
			font_paths.push(PathBuf::from(format!("{}/.fonts", std::env::var("HOME").unwrap_or_default())));
		} else if cfg!(target_os = "macos") {
			font_paths.push(PathBuf::from("/System/Library/Fonts"));
			font_paths.push(PathBuf::from("/Library/Fonts"));
			font_paths.push(PathBuf::from(format!("{}/Library/Fonts", std::env::var("HOME").unwrap_or_default())));
		} else if cfg!(target_os = "windows") {
			font_paths.push(PathBuf::from("C:\\Windows\\Fonts"));
		}
		
		Self {
			font_cache: Arc::new(RwLock::new(HashMap::new())),
			font_paths,
			default_font_family,
			default_font_size,
		}
	}
	
	/**
	 * Loads a font from the system
	 * 
	 * Searches for and loads a font from the system font directories
	 * with the specified family name and characteristics.
	 * 
	 * @param family - Font family name
	 * @param size - Font size in points
	 * @param weight - Font weight
	 * @param style - Font style
	 * @return Result<CachedFont> - Loaded font or error
	 */
	pub async fn load_font(
		&self,
		family: &str,
		size: f32,
		weight: FontWeight,
		style: FontStyle,
	) -> Result<CachedFont> {
		/**
		 * フォント読み込みの複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なフォント検索を行います。
		 * システムフォントの動的読み込みが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let cache_key = format!("{}:{}:{}:{}", family, size, weight as u32, style as u32);
		let mut font_cache = self.font_cache.write().await;
		
		// Check cache first
		if let Some(cached_font) = font_cache.get(&cache_key) {
			return Ok(cached_font.clone());
		}
		
		// Search for font file
		let font_path = self.find_font_file(family, weight, style).await?;
		
		// Load font data
		let font_data = std::fs::read(&font_path)?;
		
		// Create cached font
		let cached_font = CachedFont {
			family: family.to_string(),
			size,
			weight,
			style,
			file_path: Some(font_path),
			data: font_data,
			glyph_cache: HashMap::new(),
		};
		
		font_cache.insert(cache_key, cached_font.clone());
		
		Ok(cached_font)
	}
	
	/**
	 * Finds a font file in the system
	 * 
	 * Searches through font directories to find a font file
	 * matching the specified family, weight, and style.
	 * 
	 * @param family - Font family name
	 * @param weight - Font weight
	 * @param style - Font style
	 * @return Result<PathBuf> - Font file path or error
	 */
	async fn find_font_file(&self, family: &str, weight: FontWeight, style: FontStyle) -> Result<PathBuf> {
		/**
		 * フォントファイル検索の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なファイルシステム検索を行います。
		 * 複数のフォントディレクトリの検索が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		for font_path in &self.font_paths {
			if let Ok(entries) = std::fs::read_dir(font_path) {
				for entry in entries {
					if let Ok(entry) = entry {
						let path = entry.path();
						if let Some(file_name) = path.file_name() {
							if let Some(file_name_str) = file_name.to_str() {
								if self.matches_font_criteria(file_name_str, family, weight, style) {
									return Ok(path);
								}
							}
						}
					}
				}
			}
		}
		
		Err(anyhow::anyhow!("Font not found: {} {:?} {:?}", family, weight, style))
	}
	
	/**
	 * Checks if a font file matches the specified criteria
	 * 
	 * @param file_name - Font file name
	 * @param family - Font family name
	 * @param weight - Font weight
	 * @param style - Font style
	 * @return bool - True if the font file matches the criteria
	 */
	fn matches_font_criteria(&self, file_name: &str, family: &str, weight: FontWeight, style: FontStyle) -> bool {
		// Convert family name to lowercase for comparison
		let family_lower = family.to_lowercase();
		let file_name_lower = file_name.to_lowercase();
		
		// Check if file name contains family name
		if !file_name_lower.contains(&family_lower) {
			return false;
		}
		
		// Check weight (simplified)
		match weight {
			FontWeight::Bold => {
				if !file_name_lower.contains("bold") && !file_name_lower.contains("heavy") {
					return false;
				}
			}
			FontWeight::Light => {
				if !file_name_lower.contains("light") && !file_name_lower.contains("thin") {
					return false;
				}
			}
			_ => {} // Normal weight is default
		}
		
		// Check style (simplified)
		match style {
			FontStyle::Italic => {
				if !file_name_lower.contains("italic") && !file_name_lower.contains("oblique") {
					return false;
				}
			}
			_ => {} // Normal style is default
		}
		
		// Check file extension
		file_name_lower.ends_with(".ttf") || 
		file_name_lower.ends_with(".otf") || 
		file_name_lower.ends_with(".woff") || 
		file_name_lower.ends_with(".woff2")
	}
	
	/**
	 * Gets a cached font
	 * 
	 * @param family - Font family name
	 * @param size - Font size in points
	 * @param weight - Font weight
	 * @param style - Font style
	 * @return Option<CachedFont> - Cached font if available
	 */
	pub async fn get_cached_font(
		&self,
		family: &str,
		size: f32,
		weight: FontWeight,
		style: FontStyle,
	) -> Option<CachedFont> {
		let cache_key = format!("{}:{}:{}:{}", family, size, weight as u32, style as u32);
		let font_cache = self.font_cache.read().await;
		font_cache.get(&cache_key).cloned()
	}
	
	/**
	 * Clears the font cache
	 */
	pub async fn clear_font_cache(&self) {
		let mut font_cache = self.font_cache.write().await;
		font_cache.clear();
	}
	
	/**
	 * Gets font cache statistics
	 * 
	 * @return usize - Number of cached fonts
	 */
	pub async fn get_font_cache_size(&self) -> usize {
		self.font_cache.read().await.len()
	}
	
	/**
	 * Gets available font families
	 * 
	 * @return Result<Vec<String>> - List of available font families
	 */
	pub async fn get_available_font_families(&self) -> Result<Vec<String>> {
		/**
		 * 利用可能フォント検索の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なフォント検索を行います。
		 * システムフォントの列挙が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut families = std::collections::HashSet::new();
		
		for font_path in &self.font_paths {
			if let Ok(entries) = std::fs::read_dir(font_path) {
				for entry in entries {
					if let Ok(entry) = entry {
						let path = entry.path();
						if let Some(file_name) = path.file_name() {
							if let Some(file_name_str) = file_name.to_str() {
								if let Some(family) = self.extract_font_family(file_name_str) {
									families.insert(family);
								}
							}
						}
					}
				}
			}
		}
		
		Ok(families.into_iter().collect())
	}
	
	/**
	 * Extracts font family name from file name
	 * 
	 * @param file_name - Font file name
	 * @return Option<String> - Extracted font family name
	 */
	fn extract_font_family(&self, file_name: &str) -> Option<String> {
		// Simple extraction (placeholder implementation)
		// TODO: Implement proper font family extraction
		if file_name.ends_with(".ttf") || file_name.ends_with(".otf") {
			let name = file_name.replace(".ttf", "").replace(".otf", "");
			Some(name)
		} else {
			None
		}
	}
	
	/**
	 * Gets default font family
	 * 
	 * @return &str - Default font family
	 */
	pub fn default_font_family(&self) -> &str {
		&self.default_font_family
	}
	
	/**
	 * Gets default font size
	 * 
	 * @return f32 - Default font size
	 */
	pub fn default_font_size(&self) -> f32 {
		self.default_font_size
	}
} 