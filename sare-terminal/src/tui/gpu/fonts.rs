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
		 * フォントマネージャーを初期化する関数です
		 * 
		 * システムフォントディレクトリを検出し、プラットフォーム固有の
		 * フォントパスを設定してフォントキャッシュを初期化します。
		 * 
		 * Linux、macOS、Windowsの各プラットフォームで適切な
		 * フォントディレクトリを自動検出します
		 */
		
		let mut font_paths = Vec::new();
		
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
		 * システムフォントを読み込む関数です
		 * 
		 * 指定されたファミリー、サイズ、ウェイト、スタイルに基づいて
		 * システムフォントを検索し、フォントデータを読み込んでキャッシュします。
		 * 
		 * フォントファイルを動的に検索し、フォントデータを
		 * メモリに読み込んで効率的なアクセスを提供します
		 */
		
		let cache_key = format!("{}:{}:{}:{}", family, size, weight.clone() as u32, style.clone() as u32);
		let mut font_cache = self.font_cache.write().await;
		
		if let Some(cached_font) = font_cache.get(&cache_key) {
			return Ok(cached_font.clone());
		}
		
		let font_path = self.find_font_file(family, &weight, &style).await?;
		
		let font_data = std::fs::read(&font_path)?;
		
		let cached_font = CachedFont {
			family: family.to_string(),
			size,
			weight: weight.clone(),
			style: style.clone(),
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
	async fn find_font_file(&self, family: &str, weight: &FontWeight, style: &FontStyle) -> Result<PathBuf> {
		/**
		 * フォントファイルを検索する関数です
		 * 
		 * 指定されたファミリー、ウェイト、スタイルに一致する
		 * フォントファイルをシステムのフォントディレクトリから検索します。
		 * 
		 * 複数のフォントディレクトリを順次検索し、ファイル名の
		 * パターンマッチングを使用して適切なフォントファイルを特定します
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
	fn matches_font_criteria(&self, file_name: &str, family: &str, weight: &FontWeight, style: &FontStyle) -> bool {
		// Convert family name to lowercase for comparison
		let family_lower = family.to_lowercase();
		let file_name_lower = file_name.to_lowercase();
		
		// Check if file name contains family name
		if !file_name_lower.contains(&family_lower) {
			return false;
		}
		
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
			_ => {}
		}
		
		match style {
			FontStyle::Italic => {
				if !file_name_lower.contains("italic") && !file_name_lower.contains("oblique") {
					return false;
				}
			}
			_ => {} 
		}
		
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
		 * 利用可能なフォントファミリーを取得する関数です
		 * 
		 * システムで利用可能なすべてのフォントファミリーを
		 * 検索して一覧を返します。
		 * 
		 * フォントディレクトリを走査し、フォントファイルから
		 * ファミリー名を抽出して重複を除去したリストを作成します
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
		if !file_name.ends_with(".ttf") && !file_name.ends_with(".otf") && !file_name.ends_with(".ttc") {
			return None;
		}
		
		unsafe {
			use std::ffi::CString;
			
			let fontconfig_lib = match CString::new("libfontconfig.so.1") {
				Ok(s) => s,
				Err(_) => return self.simple_extract_family(file_name),
			};
			
			let handle = libc::dlopen(fontconfig_lib.as_ptr(), libc::RTLD_NOW);
			if !handle.is_null() {
	
				if let Ok(family) = self.extract_family_with_fontconfig(file_name) {
					libc::dlclose(handle);
					return Some(family);
				}
				libc::dlclose(handle);
			}
		}
		
		self.simple_extract_family(file_name)
	}
	
	/**
	 * Simple font family extraction as fallback
	 * 
	 * @param file_name - Font file name
	 * @return Option<String> - Extracted font family name
	 */
	fn simple_extract_family(&self, file_name: &str) -> Option<String> {
		// Remove file extensions
		let name = file_name
			.replace(".ttf", "")
			.replace(".otf", "")
			.replace(".ttc", "");
		
		if name.contains('-') {
			if let Some(family) = name.split('-').next() {
				return Some(family.to_string());
			}
		}
		
		if name.contains('_') {
			if let Some(family) = name.split('_').next() {
				return Some(family.to_string());
			}
		}
		
		if name.chars().any(|c| c.is_uppercase()) {
			let mut result = String::new();
			let mut chars = name.chars().peekable();
			
			while let Some(c) = chars.next() {
				if c.is_uppercase() && !result.is_empty() {
					break;
				}
				result.push(c);
			}
			
			if !result.is_empty() {
				return Some(result);
			}
		}
		
		Some(name)
	}
	
	/**
	 * Extract font family using fontconfig library
	 * 
	 * @param file_name - Font file name
	 * @return Result<String> - Font family name
	 */
	fn extract_family_with_fontconfig(&self, file_name: &str) -> Result<String> {

		Err(anyhow::anyhow!("Fontconfig not available"))
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