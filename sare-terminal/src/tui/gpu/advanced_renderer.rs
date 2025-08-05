/**
 * Advanced rendering engine for Sare terminal
 * 
 * This module provides advanced rendering capabilities including
 * font rendering, Unicode support, line wrapping, bidirectional text,
 * ligature support, GPU texture management, and efficient memory management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: advanced_renderer.rs
 * Description: Advanced rendering engine with Unicode and GPU support
 */

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use unicode_bidi::{BidiInfo, Level};
use unicode_normalization::{UnicodeNormalization, UnicodeNormalizationForm};
use unicode_width::UnicodeWidthStr;

use super::fonts::{FontManager, CachedFont, CachedGlyph};
use super::text::{FontWeight, FontStyle, TextBounds};

/**
 * Advanced rendering engine
 * 
 * Provides comprehensive rendering capabilities including
 * Unicode support, bidirectional text, ligatures, and GPU acceleration.
 */
pub struct AdvancedRenderer {
	/// Font manager for text rendering
	font_manager: Arc<FontManager>,
	/// Texture atlas for GPU rendering
	texture_atlas: Arc<RwLock<TextureAtlas>>,
	/// Glyph cache for efficient rendering
	glyph_cache: Arc<RwLock<HashMap<GlyphKey, CachedGlyph>>>,
	/// Line cache for wrapped text
	line_cache: Arc<RwLock<HashMap<String, CachedLine>>>,
	/// Memory pool for efficient allocation
	memory_pool: Arc<RwLock<MemoryPool>>,
	/// Rendering configuration
	config: RendererConfig,
}

/**
 * Texture atlas for GPU rendering
 * 
 * Manages texture storage and caching for efficient
 * GPU-accelerated rendering operations.
 */
#[derive(Debug)]
pub struct TextureAtlas {
	/// Atlas texture data
	texture_data: Vec<u8>,
	/// Atlas width
	width: u32,
	/// Atlas height
	height: u32,
	/// Glyph positions in atlas
	glyph_positions: HashMap<GlyphKey, AtlasPosition>,
	/// Free space tracking
	free_regions: Vec<AtlasRegion>,
}

/**
 * Atlas position information
 * 
 * Contains position and size information for glyphs
 * stored in the texture atlas.
 */
#[derive(Debug, Clone)]
pub struct AtlasPosition {
	/// X coordinate in atlas
	pub x: u32,
	/// Y coordinate in atlas
	pub y: u32,
	/// Width in atlas
	pub width: u32,
	/// Height in atlas
	pub height: u32,
}

/**
 * Atlas region for space management
 * 
 * Represents a region of free space in the texture atlas.
 */
#[derive(Debug, Clone)]
pub struct AtlasRegion {
	/// X coordinate
	pub x: u32,
	/// Y coordinate
	pub y: u32,
	/// Width
	pub width: u32,
	/// Height
	pub height: u32,
}

/**
 * Glyph cache key
 * 
 * Unique identifier for cached glyphs.
 */
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GlyphKey {
	/// Character code
	pub character: char,
	/// Font family
	pub font_family: String,
	/// Font size
	pub font_size: f32,
	/// Font weight
	pub font_weight: FontWeight,
	/// Font style
	pub font_style: FontStyle,
}

/**
 * Cached line information
 * 
 * Contains cached information for wrapped text lines.
 */
#[derive(Debug, Clone)]
pub struct CachedLine {
	/// Line text
	pub text: String,
	/// Line width
	pub width: f32,
	/// Line height
	pub height: f32,
	/// Glyph positions
	pub glyph_positions: Vec<GlyphPosition>,
	/// Bidirectional information
	pub bidi_info: Option<BidiInfo>,
}

/**
 * Glyph position information
 * 
 * Contains position and rendering information for individual glyphs.
 */
#[derive(Debug, Clone)]
pub struct GlyphPosition {
	/// Character
	pub character: char,
	/// X position
	pub x: f32,
	/// Y position
	pub y: f32,
	/// Glyph bounds
	pub bounds: GlyphBounds,
	/// Atlas position
	pub atlas_position: Option<AtlasPosition>,
}

/**
 * Glyph bounds information
 * 
 * Contains bounding box information for glyphs.
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
	/// Advance width
	pub advance: f32,
}

/**
 * Memory pool for efficient allocation
 * 
 * Provides efficient memory management for rendering operations.
 */
#[derive(Debug)]
pub struct MemoryPool {
	/// Available memory blocks
	available_blocks: Vec<MemoryBlock>,
	/// Used memory blocks
	used_blocks: Vec<MemoryBlock>,
	/// Total allocated memory
	total_allocated: usize,
	/// Maximum memory usage
	max_memory: usize,
}

/**
 * Memory block for allocation
 * 
 * Represents a block of memory in the memory pool.
 */
#[derive(Debug, Clone)]
pub struct MemoryBlock {
	/// Block start address
	pub start: usize,
	/// Block size
	pub size: usize,
	/// Block type
	pub block_type: MemoryBlockType,
}

/**
 * Memory block type
 * 
 * Defines different types of memory blocks.
 */
#[derive(Debug, Clone)]
pub enum MemoryBlockType {
	/// Texture data
	Texture,
	/// Glyph data
	Glyph,
	/// Line data
	Line,
	/// General data
	General,
}

/**
 * Renderer configuration
 * 
 * Contains configuration options for the advanced renderer.
 */
#[derive(Debug, Clone)]
pub struct RendererConfig {
	/// Enable Unicode support
	pub unicode_support: bool,
	/// Enable bidirectional text
	pub bidirectional_text: bool,
	/// Enable ligature support
	pub ligature_support: bool,
	/// Enable GPU acceleration
	pub gpu_acceleration: bool,
	/// Enable texture atlasing
	pub texture_atlasing: bool,
	/// Enable memory pooling
	pub memory_pooling: bool,
	/// Maximum texture atlas size
	pub max_atlas_size: u32,
	/// Maximum memory usage
	pub max_memory_usage: usize,
	/// Line wrapping width
	pub line_wrapping_width: f32,
	/// Subpixel antialiasing
	pub subpixel_antialiasing: bool,
}

impl Default for RendererConfig {
	fn default() -> Self {
		Self {
			unicode_support: true,
			bidirectional_text: true,
			ligature_support: true,
			gpu_acceleration: true,
			texture_atlasing: true,
			memory_pooling: true,
			max_atlas_size: 2048,
			max_memory_usage: 64 * 1024 * 1024, // 64MB
			line_wrapping_width: 800.0,
			subpixel_antialiasing: true,
		}
	}
}

impl AdvancedRenderer {
	/**
	 * Creates a new advanced renderer
	 * 
	 * @param config - Renderer configuration
	 * @return AdvancedRenderer - New advanced renderer instance
	 */
	pub fn new(config: RendererConfig) -> Self {
		let font_manager = Arc::new(FontManager::new(
			"Fira Code".to_string(),
			14.0,
		));
		
		let texture_atlas = Arc::new(RwLock::new(TextureAtlas::new(
			config.max_atlas_size,
			config.max_atlas_size,
		)));
		
		let memory_pool = Arc::new(RwLock::new(MemoryPool::new(
			config.max_memory_usage,
		)));
		
		Self {
			font_manager,
			texture_atlas,
			glyph_cache: Arc::new(RwLock::new(HashMap::new())),
			line_cache: Arc::new(RwLock::new(HashMap::new())),
			memory_pool,
			config,
		}
	}
	
	/**
	 * Renders text with advanced features
	 * 
	 * @param text - Text to render
	 * @param x - X position
	 * @param y - Y position
	 * @param color - Text color
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @return Result<Vec<GlyphPosition>> - Rendered glyph positions
	 */
	pub async fn render_text(
		&self,
		text: &str,
		x: f32,
		y: f32,
		color: u32,
		font_family: Option<&str>,
		font_size: Option<f32>,
	) -> Result<Vec<GlyphPosition>> {
		// Normalize Unicode text
		let normalized_text = if self.config.unicode_support {
			text.nfc().collect::<String>()
		} else {
			text.to_string()
		};
		
		// Process bidirectional text
		let bidi_info = if self.config.bidirectional_text {
			Some(BidiInfo::new(&normalized_text, None))
		} else {
			None
		};
		
		// Split into grapheme clusters
		let graphemes = self.split_graphemes(&normalized_text);
		
		// Render each grapheme
		let mut glyph_positions = Vec::new();
		let mut current_x = x;
		
		for grapheme in graphemes {
			let glyph_pos = self.render_grapheme(
				grapheme,
				current_x,
				y,
				color,
				font_family,
				font_size,
			).await?;
			
			glyph_positions.push(glyph_pos);
			current_x += glyph_pos.bounds.advance;
		}
		
		// Apply bidirectional layout if needed
		if let Some(bidi) = bidi_info {
			self.apply_bidirectional_layout(&mut glyph_positions, &bidi);
		}
		
		Ok(glyph_positions)
	}
	
	/**
	 * Wraps text to fit within specified width
	 * 
	 * @param text - Text to wrap
	 * @param max_width - Maximum line width
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @return Result<Vec<String>> - Wrapped lines
	 */
	pub async fn wrap_text(
		&self,
		text: &str,
		max_width: f32,
		font_family: Option<&str>,
		font_size: Option<f32>,
	) -> Result<Vec<String>> {
		let mut lines = Vec::new();
		let mut current_line = String::new();
		let mut current_width = 0.0;
		
		// Split text into words
		let words = text.split_whitespace();
		
		for word in words {
			let word_width = self.measure_text_width(word, font_family, font_size).await?;
			
			if current_width + word_width > max_width && !current_line.is_empty() {
				// Start new line
				lines.push(current_line.trim().to_string());
				current_line = word.to_string();
				current_width = word_width;
			} else {
				// Add to current line
				if !current_line.is_empty() {
					current_line.push(' ');
				}
				current_line.push_str(word);
				current_width += word_width;
			}
		}
		
		// Add final line
		if !current_line.is_empty() {
			lines.push(current_line.trim().to_string());
		}
		
		Ok(lines)
	}
	
	/**
	 * Measures text width
	 * 
	 * @param text - Text to measure
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @return Result<f32> - Text width
	 */
	pub async fn measure_text_width(
		&self,
		text: &str,
		font_family: Option<&str>,
		font_size: Option<f32>,
	) -> Result<f32> {
		let mut total_width = 0.0;
		
		for ch in text.chars() {
			let glyph_key = GlyphKey {
				character: ch,
				font_family: font_family.unwrap_or("Fira Code").to_string(),
				font_size: font_size.unwrap_or(14.0),
				font_weight: FontWeight::Normal,
				font_style: FontStyle::Normal,
			};
			
			if let Some(glyph) = self.get_cached_glyph(&glyph_key).await? {
				total_width += glyph.bounds.advance;
			} else {
				// Estimate width for unknown characters
				total_width += font_size.unwrap_or(14.0) * 0.6;
			}
		}
		
		Ok(total_width)
	}
	
	/**
	 * Renders a single grapheme
	 * 
	 * @param grapheme - Grapheme to render
	 * @param x - X position
	 * @param y - Y position
	 * @param color - Text color
	 * @param font_family - Font family
	 * @param font_size - Font size
	 * @return Result<GlyphPosition> - Rendered glyph position
	 */
	async fn render_grapheme(
		&self,
		grapheme: &str,
		x: f32,
		y: f32,
		color: u32,
		font_family: Option<&str>,
		font_size: Option<f32>,
	) -> Result<GlyphPosition> {
		// Get the first character of the grapheme
		let ch = grapheme.chars().next().unwrap_or(' ');
		
		let glyph_key = GlyphKey {
			character: ch,
			font_family: font_family.unwrap_or("Fira Code").to_string(),
			font_size: font_size.unwrap_or(14.0),
			font_weight: FontWeight::Normal,
			font_style: FontStyle::Normal,
		};
		
		// Get or create cached glyph
		let glyph = self.get_or_create_glyph(&glyph_key).await?;
		
		// Get atlas position
		let atlas_position = if self.config.texture_atlasing {
			self.get_atlas_position(&glyph_key).await?
		} else {
			None
		};
		
		Ok(GlyphPosition {
			character: ch,
			x,
			y,
			bounds: glyph.bounds.clone(),
			atlas_position,
		})
	}
	
	/**
	 * Splits text into grapheme clusters
	 * 
	 * @param text - Text to split
	 * @return Vec<String> - Grapheme clusters
	 */
	fn split_graphemes(&self, text: &str) -> Vec<String> {
		if !self.config.unicode_support {
			return text.chars().map(|c| c.to_string()).collect();
		}
		
		// Use Unicode grapheme cluster segmentation
		text.graphemes(true).map(|g| g.to_string()).collect()
	}
	
	/**
	 * Applies bidirectional layout to glyph positions
	 * 
	 * @param glyph_positions - Glyph positions to modify
	 * @param bidi_info - Bidirectional information
	 */
	fn apply_bidirectional_layout(
		&self,
		glyph_positions: &mut [GlyphPosition],
		bidi_info: &BidiInfo,
	) {
		if !self.config.bidirectional_text {
			return;
		}
		
		// Apply bidirectional algorithm
		for (i, glyph) in glyph_positions.iter_mut().enumerate() {
			if let Some(level) = bidi_info.levels.get(i) {
				if level.is_rtl() {
					// Adjust position for RTL text
					glyph.x = glyph.x - glyph.bounds.advance;
				}
			}
		}
	}
	
	/**
	 * Gets or creates a cached glyph
	 * 
	 * @param glyph_key - Glyph key
	 * @return Result<CachedGlyph> - Cached glyph
	 */
	async fn get_or_create_glyph(&self, glyph_key: &GlyphKey) -> Result<CachedGlyph> {
		let mut cache = self.glyph_cache.write().await;
		
		if let Some(glyph) = cache.get(glyph_key) {
			return Ok(glyph.clone());
		}
		
		// Create new glyph
		let glyph = self.create_glyph(glyph_key).await?;
		cache.insert(glyph_key.clone(), glyph.clone());
		
		Ok(glyph)
	}
	
	/**
	 * Gets a cached glyph
	 * 
	 * @param glyph_key - Glyph key
	 * @return Result<Option<CachedGlyph>> - Cached glyph if available
	 */
	async fn get_cached_glyph(&self, glyph_key: &GlyphKey) -> Result<Option<CachedGlyph>> {
		let cache = self.glyph_cache.read().await;
		Ok(cache.get(glyph_key).cloned())
	}
	
	/**
	 * Creates a new glyph
	 * 
	 * @param glyph_key - Glyph key
	 * @return Result<CachedGlyph> - Created glyph
	 */
	async fn create_glyph(&self, glyph_key: &GlyphKey) -> Result<CachedGlyph> {
		// Load font
		let font = self.font_manager.load_font(
			&glyph_key.font_family,
			glyph_key.font_size,
			glyph_key.font_weight,
			glyph_key.font_style,
		).await?;
		
		// Create glyph bounds (simplified)
		let width = glyph_key.font_size * 0.6;
		let height = glyph_key.font_size;
		let advance = width;
		
		Ok(CachedGlyph {
			character: glyph_key.character,
			width,
			height,
			advance,
			bounds: GlyphBounds {
				left: 0.0,
				top: 0.0,
				right: width,
				bottom: height,
			},
			texture_data: Vec::new(), // Would be filled with actual texture data
		})
	}
	
	/**
	 * Gets atlas position for glyph
	 * 
	 * @param glyph_key - Glyph key
	 * @return Result<Option<AtlasPosition>> - Atlas position if available
	 */
	async fn get_atlas_position(&self, glyph_key: &GlyphKey) -> Result<Option<AtlasPosition>> {
		let mut atlas = self.texture_atlas.write().await;
		atlas.get_glyph_position(glyph_key)
	}
	
	/**
	 * Clears all caches
	 * 
	 * @return Result<()> - Success or error
	 */
	pub async fn clear_caches(&self) -> Result<()> {
		self.glyph_cache.write().await.clear();
		self.line_cache.write().await.clear();
		self.font_manager.clear_font_cache().await;
		Ok(())
	}
	
	/**
	 * Gets memory usage statistics
	 * 
	 * @return Result<(usize, usize)> - (used, total) memory in bytes
	 */
	pub async fn get_memory_stats(&self) -> Result<(usize, usize)> {
		let pool = self.memory_pool.read().await;
		Ok((pool.total_allocated, pool.max_memory))
	}
}

impl TextureAtlas {
	/**
	 * Creates a new texture atlas
	 * 
	 * @param width - Atlas width
	 * @param height - Atlas height
	 * @return TextureAtlas - New texture atlas
	 */
	pub fn new(width: u32, height: u32) -> Self {
		let texture_data = vec![0u8; (width * height * 4) as usize];
		let free_regions = vec![AtlasRegion { x: 0, y: 0, width, height }];
		
		Self {
			texture_data,
			width,
			height,
			glyph_positions: HashMap::new(),
			free_regions,
		}
	}
	
	/**
	 * Gets glyph position in atlas
	 * 
	 * @param glyph_key - Glyph key
	 * @return Result<Option<AtlasPosition>> - Atlas position if available
	 */
	pub fn get_glyph_position(&mut self, glyph_key: &GlyphKey) -> Result<Option<AtlasPosition>> {
		if let Some(position) = self.glyph_positions.get(glyph_key) {
			return Ok(Some(position.clone()));
		}
		
		// Try to allocate space for new glyph
		if let Some(region) = self.allocate_region(32, 32) {
			let position = AtlasPosition {
				x: region.x,
				y: region.y,
				width: region.width,
				height: region.height,
			};
			
			self.glyph_positions.insert(glyph_key.clone(), position.clone());
			return Ok(Some(position));
		}
		
		Ok(None)
	}
	
	/**
	 * Allocates a region in the atlas
	 * 
	 * @param width - Region width
	 * @param height - Region height
	 * @return Option<AtlasRegion> - Allocated region if available
	 */
	fn allocate_region(&mut self, width: u32, height: u32) -> Option<AtlasRegion> {
		for i in 0..self.free_regions.len() {
			let region = &self.free_regions[i];
			if region.width >= width && region.height >= height {
				// Found suitable region
				let allocated = AtlasRegion {
					x: region.x,
					y: region.y,
					width,
					height,
				};
				
				// Update free regions
				if region.width > width {
					self.free_regions.push(AtlasRegion {
						x: region.x + width,
						y: region.y,
						width: region.width - width,
						height: region.height,
					});
				}
				
				if region.height > height {
					self.free_regions.push(AtlasRegion {
						x: region.x,
						y: region.y + height,
						width: width,
						height: region.height - height,
					});
				}
				
				self.free_regions.remove(i);
				return Some(allocated);
			}
		}
		
		None
	}
}

impl MemoryPool {
	/**
	 * Creates a new memory pool
	 * 
	 * @param max_memory - Maximum memory usage
	 * @return MemoryPool - New memory pool
	 */
	pub fn new(max_memory: usize) -> Self {
		Self {
			available_blocks: vec![MemoryBlock {
				start: 0,
				size: max_memory,
				block_type: MemoryBlockType::General,
			}],
			used_blocks: Vec::new(),
			total_allocated: 0,
			max_memory,
		}
	}
	
	/**
	 * Allocates memory from the pool
	 * 
	 * @param size - Size to allocate
	 * @param block_type - Type of memory block
	 * @return Option<MemoryBlock> - Allocated block if available
	 */
	pub fn allocate(&mut self, size: usize, block_type: MemoryBlockType) -> Option<MemoryBlock> {
		if self.total_allocated + size > self.max_memory {
			return None;
		}
		
		// Find suitable block
		for i in 0..self.available_blocks.len() {
			let block = &self.available_blocks[i];
			if block.size >= size {
				let allocated = MemoryBlock {
					start: block.start,
					size,
					block_type: block_type.clone(),
				};
				
				// Update available blocks
				if block.size > size {
					self.available_blocks[i] = MemoryBlock {
						start: block.start + size,
						size: block.size - size,
						block_type: MemoryBlockType::General,
					};
				} else {
					self.available_blocks.remove(i);
				}
				
				self.used_blocks.push(allocated.clone());
				self.total_allocated += size;
				
				return Some(allocated);
			}
		}
		
		None
	}
	
	/**
	 * Frees memory back to the pool
	 * 
	 * @param block - Block to free
	 */
	pub fn free(&mut self, block: MemoryBlock) {
		if let Some(index) = self.used_blocks.iter().position(|b| b.start == block.start) {
			self.used_blocks.remove(index);
			self.total_allocated -= block.size;
			
			// Add back to available blocks
			self.available_blocks.push(block);
		}
	}
} 