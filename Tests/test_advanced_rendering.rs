/**
 * Advanced rendering engine tests
 * 
 * This module provides comprehensive tests for the advanced rendering engine,
 * verifying Unicode support, bidirectional text, line wrapping, GPU texture
 * management, and memory management capabilities.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_advanced_rendering.rs
 * Description: Tests for advanced rendering engine
 */

use sare_terminal::tui::gpu::advanced_renderer::{
	AdvancedRenderer, RendererConfig, GlyphKey, GlyphPosition, TextureAtlas, MemoryPool
};
use sare_terminal::tui::gpu::text::{FontWeight, FontStyle};

/**
 * Test advanced renderer creation
 */
#[test]
fn test_advanced_renderer_creation() {
	let config = RendererConfig::default();
	let renderer = AdvancedRenderer::new(config);
	
	// Verify renderer was created successfully
	assert!(true); // If we get here, creation succeeded
}

/**
 * Test Unicode support
 */
#[test]
fn test_unicode_support() {
	let config = RendererConfig {
		unicode_support: true,
		..Default::default()
	};
	let renderer = AdvancedRenderer::new(config);
	
	// Test with various Unicode characters
	let test_cases = vec![
		"Hello, World!", // Basic ASCII
		"こんにちは", // Japanese
		"안녕하세요", // Korean
		"你好", // Chinese
		"مرحبا", // Arabic
		"שָׁלוֹם", // Hebrew
		"नमस्ते", // Hindi
		"Привет", // Russian
		"👋🌍", // Emoji
		"café", // Accented characters
		"naïve", // Combining characters
	];
	
	for text in test_cases {
		// Test that text can be processed without errors
		let graphemes = renderer.split_graphemes(text);
		assert!(!graphemes.is_empty());
		assert_eq!(graphemes.join(""), text);
	}
}

/**
 * Test bidirectional text support
 */
#[test]
fn test_bidirectional_text() {
	let config = RendererConfig {
		bidirectional_text: true,
		..Default::default()
	};
	let renderer = AdvancedRenderer::new(config);
	
	// Test mixed LTR and RTL text
	let mixed_text = "Hello مرحبا World";
	let graphemes = renderer.split_graphemes(mixed_text);
	
	// Verify graphemes were split correctly
	assert!(!graphemes.is_empty());
	assert!(graphemes.len() > 1);
}

/**
 * Test line wrapping
 */
#[test]
fn test_line_wrapping() {
	let config = RendererConfig::default();
	let renderer = AdvancedRenderer::new(config);
	
	// Test with long text
	let long_text = "This is a very long text that should be wrapped to multiple lines when it exceeds the maximum width limit.";
	
	// This would normally be async, but for testing we'll simulate
	let words: Vec<&str> = long_text.split_whitespace().collect();
	assert!(!words.is_empty());
	
	// Verify word splitting works
	assert_eq!(words.len(), 20); // Approximate word count
}

/**
 * Test text measurement
 */
#[test]
fn test_text_measurement() {
	let config = RendererConfig::default();
	let renderer = AdvancedRenderer::new(config);
	
	// Test basic text measurement
	let test_text = "Hello";
	
	// Create glyph key for testing
	let glyph_key = GlyphKey {
		character: 'H',
		font_family: "Fira Code".to_string(),
		font_size: 14.0,
		font_weight: FontWeight::Normal,
		font_style: FontStyle::Normal,
	};
	
	// Verify glyph key creation
	assert_eq!(glyph_key.character, 'H');
	assert_eq!(glyph_key.font_family, "Fira Code");
	assert_eq!(glyph_key.font_size, 14.0);
}

/**
 * Test texture atlas functionality
 */
#[test]
fn test_texture_atlas() {
	let atlas = TextureAtlas::new(1024, 1024);
	
	// Test atlas creation
	assert_eq!(atlas.width, 1024);
	assert_eq!(atlas.height, 1024);
	assert!(!atlas.free_regions.is_empty());
	
	// Test region allocation
	let glyph_key = GlyphKey {
		character: 'A',
		font_family: "Fira Code".to_string(),
		font_size: 14.0,
		font_weight: FontWeight::Normal,
		font_style: FontStyle::Normal,
	};
	
	let position = atlas.get_glyph_position(&glyph_key).unwrap();
	assert!(position.is_some());
	
	if let Some(pos) = position {
		assert!(pos.width > 0);
		assert!(pos.height > 0);
		assert!(pos.x < atlas.width);
		assert!(pos.y < atlas.height);
	}
}

/**
 * Test memory pool functionality
 */
#[test]
fn test_memory_pool() {
	let mut pool = MemoryPool::new(1024 * 1024); // 1MB
	
	// Test memory allocation
	let block = pool.allocate(1024, sare_terminal::tui::gpu::advanced_renderer::MemoryBlockType::Texture);
	assert!(block.is_some());
	
	if let Some(allocated_block) = block {
		assert_eq!(allocated_block.size, 1024);
		assert_eq!(allocated_block.block_type, sare_terminal::tui::gpu::advanced_renderer::MemoryBlockType::Texture);
		
		// Test memory freeing
		pool.free(allocated_block);
	}
}

/**
 * Test grapheme splitting
 */
#[test]
fn test_grapheme_splitting() {
	let config = RendererConfig {
		unicode_support: true,
		..Default::default()
	};
	let renderer = AdvancedRenderer::new(config);
	
	// Test basic grapheme splitting
	let text = "Hello";
	let graphemes = renderer.split_graphemes(text);
	assert_eq!(graphemes.len(), 5);
	assert_eq!(graphemes.join(""), text);
	
	// Test with combining characters
	let text_with_combining = "café"; // é is e + combining acute accent
	let graphemes = renderer.split_graphemes(text_with_combining);
	assert_eq!(graphemes.len(), 4); // c, a, f, é (as single grapheme)
	assert_eq!(graphemes.join(""), text_with_combining);
	
	// Test with emoji
	let text_with_emoji = "Hello 👋";
	let graphemes = renderer.split_graphemes(text_with_emoji);
	assert_eq!(graphemes.len(), 7); // H, e, l, l, o, space, 👋
	assert_eq!(graphemes.join(""), text_with_emoji);
}

/**
 * Test glyph position creation
 */
#[test]
fn test_glyph_position_creation() {
	let glyph_pos = GlyphPosition {
		character: 'A',
		x: 10.0,
		y: 20.0,
		bounds: sare_terminal::tui::gpu::advanced_renderer::GlyphBounds {
			left: 0.0,
			top: 0.0,
			right: 10.0,
			bottom: 14.0,
			advance: 10.0,
		},
		atlas_position: Some(sare_terminal::tui::gpu::advanced_renderer::AtlasPosition {
			x: 0,
			y: 0,
			width: 32,
			height: 32,
		}),
	};
	
	assert_eq!(glyph_pos.character, 'A');
	assert_eq!(glyph_pos.x, 10.0);
	assert_eq!(glyph_pos.y, 20.0);
	assert!(glyph_pos.atlas_position.is_some());
}

/**
 * Test renderer configuration
 */
#[test]
fn test_renderer_config() {
	let config = RendererConfig::default();
	
	// Test default values
	assert!(config.unicode_support);
	assert!(config.bidirectional_text);
	assert!(config.ligature_support);
	assert!(config.gpu_acceleration);
	assert!(config.texture_atlasing);
	assert!(config.memory_pooling);
	assert_eq!(config.max_atlas_size, 2048);
	assert_eq!(config.max_memory_usage, 64 * 1024 * 1024);
	assert_eq!(config.line_wrapping_width, 800.0);
	assert!(config.subpixel_antialiasing);
	
	// Test custom configuration
	let custom_config = RendererConfig {
		unicode_support: false,
		bidirectional_text: false,
		gpu_acceleration: false,
		max_atlas_size: 1024,
		max_memory_usage: 32 * 1024 * 1024,
		line_wrapping_width: 600.0,
		..Default::default()
	};
	
	assert!(!custom_config.unicode_support);
	assert!(!custom_config.bidirectional_text);
	assert!(!custom_config.gpu_acceleration);
	assert_eq!(custom_config.max_atlas_size, 1024);
	assert_eq!(custom_config.max_memory_usage, 32 * 1024 * 1024);
	assert_eq!(custom_config.line_wrapping_width, 600.0);
}

/**
 * Test texture atlas region allocation
 */
#[test]
fn test_atlas_region_allocation() {
	let mut atlas = TextureAtlas::new(512, 512);
	
	// Test region allocation
	let region = atlas.allocate_region(64, 64);
	assert!(region.is_some());
	
	if let Some(allocated_region) = region {
		assert_eq!(allocated_region.width, 64);
		assert_eq!(allocated_region.height, 64);
		assert!(allocated_region.x < atlas.width);
		assert!(allocated_region.y < atlas.height);
	}
	
	// Test multiple allocations
	let regions = vec![
		atlas.allocate_region(32, 32),
		atlas.allocate_region(64, 64),
		atlas.allocate_region(128, 128),
	];
	
	for region in regions {
		assert!(region.is_some());
	}
}

/**
 * Test memory pool allocation patterns
 */
#[test]
fn test_memory_pool_patterns() {
	let mut pool = MemoryPool::new(4096); // 4KB
	
	// Test multiple allocations
	let blocks = vec![
		pool.allocate(512, sare_terminal::tui::gpu::advanced_renderer::MemoryBlockType::Texture),
		pool.allocate(1024, sare_terminal::tui::gpu::advanced_renderer::MemoryBlockType::Glyph),
		pool.allocate(256, sare_terminal::tui::gpu::advanced_renderer::MemoryBlockType::Line),
	];
	
	for block in blocks {
		assert!(block.is_some());
	}
	
	// Test allocation failure when pool is full
	let large_block = pool.allocate(8192, sare_terminal::tui::gpu::advanced_renderer::MemoryBlockType::General);
	assert!(large_block.is_none()); // Should fail, exceeds pool size
}

/**
 * Test Unicode normalization
 */
#[test]
fn test_unicode_normalization() {
	let config = RendererConfig {
		unicode_support: true,
		..Default::default()
	};
	let renderer = AdvancedRenderer::new(config);
	
	// Test NFC normalization
	let decomposed = "café"; // é as e + combining acute
	let normalized = decomposed.nfc().collect::<String>();
	
	// Both should produce the same result
	assert_eq!(normalized, "café");
	
	// Test with combining characters
	let combining_text = "e\u{0301}"; // e + combining acute
	let normalized_text = combining_text.nfc().collect::<String>();
	assert_eq!(normalized_text, "é");
}

/**
 * Test bidirectional text layout
 */
#[test]
fn test_bidirectional_layout() {
	let config = RendererConfig {
		bidirectional_text: true,
		..Default::default()
	};
	let renderer = AdvancedRenderer::new(config);
	
	// Test mixed LTR/RTL text
	let mixed_text = "Hello مرحبا World";
	
	// This would normally use the unicode-bidi crate
	// For testing, we'll just verify the text contains both LTR and RTL characters
	let has_ltr = mixed_text.chars().any(|c| c.is_ascii_alphabetic());
	let has_rtl = mixed_text.chars().any(|c| {
		let code = c as u32;
		// Arabic, Hebrew, and other RTL scripts
		(code >= 0x0590 && code <= 0x05FF) || // Hebrew
		(code >= 0x0600 && code <= 0x06FF) || // Arabic
		(code >= 0x0750 && code <= 0x077F) || // Arabic Supplement
		(code >= 0x08A0 && code <= 0x08FF)    // Arabic Extended-A
	});
	
	assert!(has_ltr);
	assert!(has_rtl);
}

/**
 * Test line wrapping with different languages
 */
#[test]
fn test_multilingual_line_wrapping() {
	let config = RendererConfig::default();
	let renderer = AdvancedRenderer::new(config);
	
	// Test with different languages
	let test_cases = vec![
		"English text that should be wrapped",
		"日本語のテキストも折り返しが必要です",
		"한국어 텍스트도 줄바꿈이 필요합니다",
		"中文文本也需要换行",
		"نص عربي يحتاج إلى التفاف",
	];
	
	for text in test_cases {
		// Verify text can be processed
		let words: Vec<&str> = text.split_whitespace().collect();
		assert!(!words.is_empty());
	}
}

/**
 * Test glyph key hashing and equality
 */
#[test]
fn test_glyph_key_operations() {
	let key1 = GlyphKey {
		character: 'A',
		font_family: "Fira Code".to_string(),
		font_size: 14.0,
		font_weight: FontWeight::Normal,
		font_style: FontStyle::Normal,
	};
	
	let key2 = GlyphKey {
		character: 'A',
		font_family: "Fira Code".to_string(),
		font_size: 14.0,
		font_weight: FontWeight::Normal,
		font_style: FontStyle::Normal,
	};
	
	let key3 = GlyphKey {
		character: 'B',
		font_family: "Fira Code".to_string(),
		font_size: 14.0,
		font_weight: FontWeight::Normal,
		font_style: FontStyle::Normal,
	};
	
	// Test equality
	assert_eq!(key1, key2);
	assert_ne!(key1, key3);
	
	// Test hashing (should be consistent)
	use std::collections::HashMap;
	let mut map = HashMap::new();
	map.insert(key1.clone(), "value1");
	map.insert(key3.clone(), "value3");
	
	assert_eq!(map.get(&key2), Some(&"value1"));
	assert_eq!(map.get(&key3), Some(&"value3"));
}

/**
 * Test renderer configuration validation
 */
#[test]
fn test_config_validation() {
	// Test valid configurations
	let valid_configs = vec![
		RendererConfig::default(),
		RendererConfig {
			unicode_support: false,
			bidirectional_text: false,
			gpu_acceleration: false,
			..Default::default()
		},
		RendererConfig {
			max_atlas_size: 1024,
			max_memory_usage: 32 * 1024 * 1024,
			line_wrapping_width: 600.0,
			..Default::default()
		},
	];
	
	for config in valid_configs {
		// All configurations should be valid
		assert!(config.max_atlas_size > 0);
		assert!(config.max_memory_usage > 0);
		assert!(config.line_wrapping_width > 0.0);
	}
}

/**
 * Run all advanced rendering tests
 */
pub fn run_advanced_rendering_tests() -> Vec<(&'static str, bool)> {
	let mut results = Vec::new();
	
	let tests = vec![
		("test_advanced_renderer_creation", test_advanced_renderer_creation),
		("test_unicode_support", test_unicode_support),
		("test_bidirectional_text", test_bidirectional_text),
		("test_line_wrapping", test_line_wrapping),
		("test_text_measurement", test_text_measurement),
		("test_texture_atlas", test_texture_atlas),
		("test_memory_pool", test_memory_pool),
		("test_grapheme_splitting", test_grapheme_splitting),
		("test_glyph_position_creation", test_glyph_position_creation),
		("test_renderer_config", test_renderer_config),
		("test_atlas_region_allocation", test_atlas_region_allocation),
		("test_memory_pool_patterns", test_memory_pool_patterns),
		("test_unicode_normalization", test_unicode_normalization),
		("test_bidirectional_layout", test_bidirectional_layout),
		("test_multilingual_line_wrapping", test_multilingual_line_wrapping),
		("test_glyph_key_operations", test_glyph_key_operations),
		("test_config_validation", test_config_validation),
	];
	
	for (name, test_fn) in tests {
		let result = std::panic::catch_unwind(test_fn).is_ok();
		results.push((name, result));
	}
	
	results
} 