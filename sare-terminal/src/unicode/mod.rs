/**
 * Unicode support module for Sare terminal
 * 
 * This module provides comprehensive Unicode support including
 * CJK character handling, emoji width support, and bidirectional
 * text rendering for international text display.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Unicode support with CJK and emoji handling
 */

pub mod width_handler;
pub mod bidi_handler;

use anyhow::Result;
use std::collections::HashMap;
use width_handler::{UnicodeWidthHandler, CharWidth};
use bidi_handler::{BidiHandler, TextDirection, BidiType};

/**
 * Unicode text processor
 * 
 * Unicodeテキストプロセッサーです。
 * CJK文字と絵文字の幅処理、
 * 双方向テキスト表示を統合します。
 */
pub struct UnicodeProcessor {
	/// Width handler for character width calculation
	width_handler: UnicodeWidthHandler,
	/// Bidirectional text handler
	bidi_handler: BidiHandler,
	/// Character width cache
	width_cache: HashMap<char, CharWidth>,
	/// Text direction cache
	direction_cache: HashMap<String, TextDirection>,
}

impl UnicodeProcessor {
	/**
	 * Creates a new Unicode processor
	 * 
	 * @return UnicodeProcessor - New Unicode processor
	 */
	pub fn new() -> Self {
		Self {
			width_handler: UnicodeWidthHandler::new(),
			bidi_handler: BidiHandler::new(),
			width_cache: HashMap::new(),
			direction_cache: HashMap::new(),
		}
	}
	
	/**
	 * Sets ambiguous character context
	 * 
	 * @param full_width - Whether to use full-width for ambiguous characters
	 */
	pub fn set_ambiguous_context(&mut self, full_width: bool) {
		self.width_handler.set_ambiguous_context(full_width);
	}
	
	/**
	 * Sets default text direction
	 * 
	 * @param direction - Default text direction
	 */
	pub fn set_default_direction(&mut self, direction: TextDirection) {
		self.bidi_handler.set_default_direction(direction);
	}
	
	/**
	 * Gets the display width of a string
	 * 
	 * @param text - Text to measure
	 * @return u32 - Display width
	 */
	pub fn get_string_width(&mut self, text: &str) -> u32 {
		self.width_handler.get_string_width(text)
	}
	
	/**
	 * Gets cursor position for a string
	 * 
	 * @param text - Text to analyze
	 * @param byte_pos - Byte position in string
	 * @return u32 - Cursor column position
	 */
	pub fn get_cursor_position(&mut self, text: &str, byte_pos: usize) -> u32 {
		self.width_handler.get_cursor_position(text, byte_pos)
	}
	
	/**
	 * Gets byte position from cursor position
	 * 
	 * @param text - Text to analyze
	 * @param cursor_pos - Cursor column position
	 * @return usize - Byte position in string
	 */
	pub fn get_byte_position(&mut self, text: &str, cursor_pos: u32) -> usize {
		self.width_handler.get_byte_position(text, cursor_pos)
	}
	
	/**
	 * Splits text at width boundary with proper handling
	 * 
	 * @param text - Text to split
	 * @param max_width - Maximum width
	 * @return Vec<String> - Split text lines
	 */
	pub fn split_at_width(&mut self, text: &str, max_width: u32) -> Vec<String> {
		// First, reorder text for bidirectional display
		let reordered_text = self.bidi_handler.reorder_text(text);
		
		// Then split the reordered text
		self.width_handler.split_at_width(&reordered_text, max_width)
	}
	
	/**
	 * Truncates text to fit width with proper handling
	 * 
	 * @param text - Text to truncate
	 * @param max_width - Maximum width
	 * @return String - Truncated text
	 */
	pub fn truncate_to_width(&mut self, text: &str, max_width: u32) -> String {
		// First, reorder text for bidirectional display
		let reordered_text = self.bidi_handler.reorder_text(text);
		
		// Then truncate the reordered text
		self.width_handler.truncate_to_width(&reordered_text, max_width)
	}
	
	/**
	 * Pads text to specified width with proper handling
	 * 
	 * @param text - Text to pad
	 * @param target_width - Target width
	 * @param pad_char - Character to use for padding
	 * @return String - Padded text
	 */
	pub fn pad_to_width(&mut self, text: &str, target_width: u32, pad_char: char) -> String {
		// First, reorder text for bidirectional display
		let reordered_text = self.bidi_handler.reorder_text(text);
		
		// Then pad the reordered text
		self.width_handler.pad_to_width(&reordered_text, target_width, pad_char)
	}
	
	/**
	 * Gets the base direction of text
	 * 
	 * @param text - Text to analyze
	 * @return TextDirection - Base text direction
	 */
	pub fn get_base_direction(&mut self, text: &str) -> TextDirection {
		// Check cache first
		if let Some(&direction) = self.direction_cache.get(text) {
			return direction;
		}
		
		let direction = self.bidi_handler.get_base_direction(text);
		self.direction_cache.insert(text.to_string(), direction);
		direction
	}
	
	/**
	 * Reorders text for bidirectional display
	 * 
	 * @param text - Text to reorder
	 * @return String - Reordered text
	 */
	pub fn reorder_text(&mut self, text: &str) -> String {
		self.bidi_handler.reorder_text(text)
	}
	
	/**
	 * Gets the display order of characters
	 * 
	 * @param text - Text to analyze
	 * @return Vec<usize> - Character indices in display order
	 */
	pub fn get_display_order(&mut self, text: &str) -> Vec<usize> {
		self.bidi_handler.get_display_order(text)
	}
	
	/**
	 * Mirrors characters for RTL display
	 * 
	 * @param text - Text to mirror
	 * @return String - Mirrored text
	 */
	pub fn mirror_text(&mut self, text: &str) -> String {
		self.bidi_handler.mirror_text(text)
	}
	
	/**
	 * Processes text for display with full Unicode support
	 * 
	 * @param text - Text to process
	 * @param max_width - Maximum display width
	 * @return ProcessedText - Processed text information
	 */
	pub fn process_text(&mut self, text: &str, max_width: u32) -> TextProcessingResult {
		let base_direction = self.get_base_direction(text);
		let reordered_text = self.reorder_text(text);
		let display_width = self.get_string_width(&reordered_text);
		let display_order = self.get_display_order(text);
		
		let lines = if display_width > max_width {
			self.split_at_width(text, max_width)
		} else {
			vec![reordered_text.to_string()]
		};
		
		TextProcessingResult {
			lines,
			display_width,
			display_order: display_order.into_iter().map(|i| i.to_string()).collect(),
			base_direction,
		}
	}
	
	/**
	 * Validates Unicode text for proper display
	 * 
	 * @param text - Text to validate
	 * @return Result<bool> - Whether text is valid
	 */
	pub fn validate_text(&mut self, text: &str) -> Result<bool> {
		// Check for invalid Unicode sequences
		for ch in text.chars() {
			if ch == '\u{FFFD}' {
				// Replacement character indicates invalid Unicode
				return Ok(false);
			}
		}
		
		// Check for proper bidirectional text
		let base_direction = self.get_base_direction(text);
		if base_direction == TextDirection::RTL {
			// Validate RTL text structure
			let reordered = self.reorder_text(text);
			if reordered.is_empty() {
				return Ok(false);
			}
		}
		
		Ok(true)
	}
	
	/**
	 * Normalizes Unicode text
	 * 
	 * @param text - Text to normalize
	 * @return String - Normalized text
	 */
	pub fn normalize_text(&mut self, text: &str) -> String {
		// Simple Unicode normalization
		let mut normalized = String::new();
		
		for ch in text.chars() {
			match ch {
				// Replace common problematic characters
				'\u{0000}' => normalized.push('\u{FFFD}'), // Null character
				'\u{0001}'..='\u{001F}' => normalized.push('\u{FFFD}'), // Control characters
				'\u{007F}'..='\u{009F}' => normalized.push('\u{FFFD}'), // Control characters
				_ => normalized.push(ch),
			}
		}
		
		normalized
	}
}

/**
 * Processed text information
 * 
 * 処理されたテキスト情報です。
 * Unicode処理の結果を
 * 格納します。
 */
#[derive(Debug, Clone)]
pub struct ProcessedText {
	/// Original text
	pub original_text: String,
	/// Reordered text for display
	pub reordered_text: String,
	/// Text split into lines
	pub lines: Vec<String>,
	/// Base text direction
	pub base_direction: TextDirection,
	/// Display width in columns
	pub display_width: u32,
	/// Character indices in display order
	pub display_order: Vec<usize>,
}

impl ProcessedText {
	/**
	 * Gets the number of lines
	 * 
	 * @return usize - Number of lines
	 */
	pub fn line_count(&self) -> usize {
		self.lines.len()
	}
	
	/**
	 * Gets a specific line
	 * 
	 * @param index - Line index
	 * @return Option<&str> - Line text or None
	 */
	pub fn get_line(&self, index: usize) -> Option<&str> {
		self.lines.get(index).map(|s| s.as_str())
	}
	
	/**
	 * Gets the maximum line width
	 * 
	 * @return u32 - Maximum line width
	 */
	pub fn max_line_width(&self) -> u32 {
		self.lines.iter()
			.map(|line| line.chars().count() as u32)
			.max()
			.unwrap_or(0)
	}
	
	/**
	 * Checks if text is right-to-left
	 * 
	 * @return bool - Whether text is RTL
	 */
	pub fn is_rtl(&self) -> bool {
		self.base_direction == TextDirection::RTL
	}
	
	/**
	 * Gets the display text for rendering
	 * 
	 * @return &str - Display text
	 */
	pub fn display_text(&self) -> &str {
		&self.reordered_text
	}
} 

#[derive(Debug, Clone)]
pub struct TextProcessingResult {
	/// Processed text lines
	pub lines: Vec<String>,
	/// Display width
	pub display_width: u32,
	/// Display order
	pub display_order: Vec<String>,
	/// Base direction
	pub base_direction: TextDirection,
} 