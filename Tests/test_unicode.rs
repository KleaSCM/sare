/**
 * Unicode functionality tests
 * 
 * This module provides comprehensive tests for Unicode support including
 * CJK character handling, emoji width support, and bidirectional text.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_unicode.rs
 * Description: Tests for Unicode support functionality
 */

use sare_terminal::unicode::{UnicodeProcessor, ProcessedText};
use sare_terminal::unicode::width_handler::{UnicodeWidthHandler, CharWidth};
use sare_terminal::unicode::bidi_handler::{BidiHandler, TextDirection, BidiType};

/**
 * Test Unicode width handler creation
 */
#[test]
fn test_unicode_width_handler_creation() {
	let mut handler = UnicodeWidthHandler::new();
	assert_eq!(handler.get_string_width("Hello"), 5);
}

/**
 * Test CJK character width handling
 */
#[test]
fn test_cjk_width_handling() {
	let mut handler = UnicodeWidthHandler::new();
	
	// Test full-width CJK characters
	assert_eq!(handler.get_string_width("日本語"), 6); // 3 characters, 2 width each
	assert_eq!(handler.get_string_width("中文"), 4); // 2 characters, 2 width each
	assert_eq!(handler.get_string_width("한국어"), 6); // 3 characters, 2 width each
	
	// Test mixed CJK and ASCII
	assert_eq!(handler.get_string_width("Hello世界"), 9); // 5 + 4
	assert_eq!(handler.get_string_width("世界Hello"), 9); // 4 + 5
}

/**
 * Test emoji width handling
 */
#[test]
fn test_emoji_width_handling() {
	let mut handler = UnicodeWidthHandler::new();
	
	// Test double-width emoji
	assert_eq!(handler.get_string_width("😀"), 2); // Double-width emoji
	assert_eq!(handler.get_string_width("🚀"), 2); // Double-width emoji
	assert_eq!(handler.get_string_width("🇺🇸"), 2); // Flag emoji
	
	// Test mixed emoji and text
	assert_eq!(handler.get_string_width("Hello😀"), 7); // 5 + 2
	assert_eq!(handler.get_string_width("😀Hello"), 7); // 2 + 5
}

/**
 * Test zero-width character handling
 */
#[test]
fn test_zero_width_handling() {
	let mut handler = UnicodeWidthHandler::new();
	
	// Test combining characters
	assert_eq!(handler.get_string_width("e\u{0301}"), 1); // e with acute accent
	assert_eq!(handler.get_string_width("a\u{0308}"), 1); // a with umlaut
	
	// Test emoji modifiers
	assert_eq!(handler.get_string_width("👨\u{200D}👩\u{200D}👧"), 2); // Family emoji
}

/**
 * Test cursor positioning with Unicode
 */
#[test]
fn test_cursor_positioning() {
	let mut handler = UnicodeWidthHandler::new();
	
	// Test ASCII text
	assert_eq!(handler.get_cursor_position("Hello", 2), 2);
	assert_eq!(handler.get_cursor_position("Hello", 5), 5);
	
	// Test CJK text
	assert_eq!(handler.get_cursor_position("日本語", 3), 2); // After first character
	assert_eq!(handler.get_cursor_position("日本語", 6), 4); // After second character
	assert_eq!(handler.get_cursor_position("日本語", 9), 6); // After third character
	
	// Test mixed text
	assert_eq!(handler.get_cursor_position("Hello世界", 5), 5); // After "Hello"
	assert_eq!(handler.get_cursor_position("Hello世界", 8), 7); // After "Hello世"
}

/**
 * Test text splitting with Unicode
 */
#[test]
fn test_text_splitting() {
	let mut handler = UnicodeWidthHandler::new();
	
	// Test ASCII text splitting
	let lines = handler.split_at_width("Hello World", 5);
	assert_eq!(lines.len(), 2);
	assert_eq!(lines[0], "Hello");
	assert_eq!(lines[1], "World");
	
	// Test CJK text splitting
	let lines = handler.split_at_width("日本語の文章", 4);
	assert_eq!(lines.len(), 2);
	assert_eq!(lines[0], "日本語");
	assert_eq!(lines[1], "の文章");
	
	// Test mixed text splitting
	let lines = handler.split_at_width("Hello世界", 6);
	assert_eq!(lines.len(), 2);
	assert_eq!(lines[0], "Hello");
	assert_eq!(lines[1], "世界");
}

/**
 * Test bidirectional text handling
 */
#[test]
fn test_bidi_handling() {
	let mut handler = BidiHandler::new();
	
	// Test LTR text
	assert_eq!(handler.get_base_direction("Hello"), TextDirection::LTR);
	
	// Test RTL text
	assert_eq!(handler.get_base_direction("مرحبا"), TextDirection::RTL);
	assert_eq!(handler.get_base_direction("שלום"), TextDirection::RTL);
	
	// Test mixed text
	let mixed_text = "Hello مرحبا";
	let reordered = handler.reorder_text(mixed_text);
	assert!(!reordered.is_empty());
}

/**
 * Test bidirectional text reordering
 */
#[test]
fn test_bidi_reordering() {
	let mut handler = BidiHandler::new();
	
	// Test simple RTL text
	let rtl_text = "مرحبا";
	let reordered = handler.reorder_text(rtl_text);
	assert_eq!(reordered, rtl_text); // Should remain the same
	
	// Test mixed LTR and RTL
	let mixed_text = "Hello مرحبا World";
	let reordered = handler.reorder_text(mixed_text);
	assert!(!reordered.is_empty());
	assert_ne!(reordered, mixed_text); // Should be reordered
}

/**
 * Test character mirroring for RTL
 */
#[test]
fn test_character_mirroring() {
	let mut handler = BidiHandler::new();
	
	// Test basic mirroring
	assert_eq!(handler.mirror_text("()"), ")(");
	assert_eq!(handler.mirror_text("[]"), "][");
	assert_eq!(handler.mirror_text("{}"), "}{");
	assert_eq!(handler.mirror_text("<>"), "><");
	
	// Test mixed text
	assert_eq!(handler.mirror_text("Hello(World)"), "Hello)World(");
}

/**
 * Test Unicode processor integration
 */
#[test]
fn test_unicode_processor() {
	let mut processor = UnicodeProcessor::new();
	
	// Test basic text processing
	let processed = processor.process_text("Hello World", 10);
	assert_eq!(processed.line_count(), 1);
	assert_eq!(processed.display_width, 11);
	assert!(!processed.is_rtl());
	
	// Test CJK text processing
	let processed = processor.process_text("日本語の文章", 10);
	assert_eq!(processed.line_count(), 1);
	assert_eq!(processed.display_width, 10);
	assert!(!processed.is_rtl());
	
	// Test RTL text processing
	let processed = processor.process_text("مرحبا بالعالم", 10);
	assert_eq!(processed.line_count(), 1);
	assert!(processed.is_rtl());
}

/**
 * Test text validation
 */
#[test]
fn test_text_validation() {
	let mut processor = UnicodeProcessor::new();
	
	// Test valid text
	assert!(processor.validate_text("Hello World").unwrap());
	assert!(processor.validate_text("日本語").unwrap());
	assert!(processor.validate_text("مرحبا").unwrap());
	
	// Test invalid text (with replacement characters)
	assert!(!processor.validate_text("Hello\u{FFFD}World").unwrap());
}

/**
 * Test text normalization
 */
#[test]
fn test_text_normalization() {
	let mut processor = UnicodeProcessor::new();
	
	// Test null character replacement
	let normalized = processor.normalize_text("Hello\u{0000}World");
	assert_eq!(normalized, "Hello\u{FFFD}World");
	
	// Test control character replacement
	let normalized = processor.normalize_text("Hello\u{0001}World");
	assert_eq!(normalized, "Hello\u{FFFD}World");
	
	// Test valid text unchanged
	let normalized = processor.normalize_text("Hello World");
	assert_eq!(normalized, "Hello World");
}

/**
 * Test ambiguous character context
 */
#[test]
fn test_ambiguous_context() {
	let mut processor = UnicodeProcessor::new();
	
	// Test with half-width context
	processor.set_ambiguous_context(false);
	assert_eq!(processor.get_string_width("│"), 1);
	
	// Test with full-width context
	processor.set_ambiguous_context(true);
	assert_eq!(processor.get_string_width("│"), 2);
}

/**
 * Test text direction setting
 */
#[test]
fn test_text_direction_setting() {
	let mut processor = UnicodeProcessor::new();
	
	// Test default LTR
	assert_eq!(processor.get_base_direction("Hello"), TextDirection::LTR);
	
	// Test setting RTL as default
	processor.set_default_direction(TextDirection::RTL);
	assert_eq!(processor.get_base_direction("Hello"), TextDirection::LTR); // Still LTR due to content
	
	// Test neutral text with RTL default
	assert_eq!(processor.get_base_direction("123"), TextDirection::RTL);
}

/**
 * Test complex Unicode scenarios
 */
#[test]
fn test_complex_unicode_scenarios() {
	let mut processor = UnicodeProcessor::new();
	
	// Test CJK with emoji
	let text = "日本語😀English";
	let processed = processor.process_text(text, 20);
	assert_eq!(processed.display_width, 15); // 6 + 2 + 7
	
	// Test RTL with numbers
	let text = "مرحبا 123 World";
	let processed = processor.process_text(text, 20);
	assert!(processed.is_rtl());
	
	// Test mixed bidirectional text
	let text = "Hello مرحبا World שלום";
	let processed = processor.process_text(text, 30);
	assert_eq!(processed.line_count(), 1);
}

/**
 * Test edge cases
 */
#[test]
fn test_unicode_edge_cases() {
	let mut processor = UnicodeProcessor::new();
	
	// Test empty string
	let processed = processor.process_text("", 10);
	assert_eq!(processed.line_count(), 1);
	assert_eq!(processed.display_width, 0);
	
	// Test single character
	let processed = processor.process_text("A", 10);
	assert_eq!(processed.line_count(), 1);
	assert_eq!(processed.display_width, 1);
	
	// Test single CJK character
	let processed = processor.process_text("日", 10);
	assert_eq!(processed.line_count(), 1);
	assert_eq!(processed.display_width, 2);
	
	// Test single emoji
	let processed = processor.process_text("😀", 10);
	assert_eq!(processed.line_count(), 1);
	assert_eq!(processed.display_width, 2);
}

/**
 * Run all Unicode tests
 */
pub fn run_unicode_tests() -> Vec<(&'static str, bool)> {
	let mut results = Vec::new();
	
	let tests = vec![
		("test_unicode_width_handler_creation", test_unicode_width_handler_creation),
		("test_cjk_width_handling", test_cjk_width_handling),
		("test_emoji_width_handling", test_emoji_width_handling),
		("test_zero_width_handling", test_zero_width_handling),
		("test_cursor_positioning", test_cursor_positioning),
		("test_text_splitting", test_text_splitting),
		("test_bidi_handling", test_bidi_handling),
		("test_bidi_reordering", test_bidi_reordering),
		("test_character_mirroring", test_character_mirroring),
		("test_unicode_processor", test_unicode_processor),
		("test_text_validation", test_text_validation),
		("test_text_normalization", test_text_normalization),
		("test_ambiguous_context", test_ambiguous_context),
		("test_text_direction_setting", test_text_direction_setting),
		("test_complex_unicode_scenarios", test_complex_unicode_scenarios),
		("test_unicode_edge_cases", test_unicode_edge_cases),
	];
	
	for (name, test_fn) in tests {
		let result = std::panic::catch_unwind(test_fn).is_ok();
		results.push((name, result));
	}
	
	results
} 