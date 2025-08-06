/**
 * Unicode width handling for Sare terminal
 * 
 * This module provides comprehensive Unicode width handling including
 * CJK character support, emoji width handling, and proper cursor positioning.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: width_handler.rs
 * Description: Unicode width handling with CJK and emoji support
 */

use anyhow::Result;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/**
 * Unicode character width
 * 
 * Unicode文字の幅です。
 * 文字の表示幅を管理します。
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharWidth {
	/// Zero width (combining characters)
	Zero,
	/// Half width (ASCII, Latin)
	Half,
	/// Full width (CJK characters)
	Full,
	/// Double width (wide emoji)
	Double,
	/// Ambiguous width (depends on context)
	Ambiguous,
}

	/**
	 * Gets the display width of a character
	 * 
	 * @param ch - Character to check
	 * @return CharWidth - Character width
	 */
impl CharWidth {

	pub fn from_char(ch: char) -> Self {
		match ch {
			// Zero-width characters
			'\u{0300}'..='\u{036F}' | // Combining diacritical marks
			'\u{1AB0}'..='\u{1AFF}' | // Combining diacritical marks extended
			'\u{20D0}'..='\u{20FF}' | // Combining diacritical marks for symbols
			'\u{FE20}'..='\u{FE2F}' | // Combining half marks
			'\u{1F3FB}'..='\u{1F3FF}' | // Emoji modifiers
			'\u{200D}' | // Zero width joiner
			'\u{FE0F}' | // Variation selector-16
			'\u{FE0E}' => Self::Zero, // Variation selector-15
			
			// Full-width characters (CJK)
			'\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
			'\u{3400}'..='\u{4DBF}' | // CJK Unified Ideographs Extension A
			'\u{20000}'..='\u{2A6DF}' | // CJK Unified Ideographs Extension B
			'\u{2A700}'..='\u{2B73F}' | // CJK Unified Ideographs Extension C
			'\u{2B740}'..='\u{2B81F}' | // CJK Unified Ideographs Extension D
			'\u{2B820}'..='\u{2CEAF}' | // CJK Unified Ideographs Extension E
			'\u{F900}'..='\u{FAFF}' | // CJK Compatibility Ideographs
			'\u{2F800}'..='\u{2FA1F}' | // CJK Compatibility Ideographs Supplement
			'\u{3000}' | // Ideographic space
			'\u{FF01}'..='\u{FF60}' | // Fullwidth ASCII variants
			'\u{FFE0}'..='\u{FFE6}' => Self::Full, // Fullwidth symbol variants
			
			// Double-width emoji
			'\u{1F600}'..='\u{1F64F}' | // Emoticons
			'\u{1F300}'..='\u{1F5FF}' | // Miscellaneous symbols and pictographs
			'\u{1F680}'..='\u{1F6FF}' | // Transport and map symbols
			'\u{1F1E0}'..='\u{1F1FF}' | // Regional indicator symbols
			'\u{2600}'..='\u{26FF}' | // Miscellaneous symbols
			'\u{2700}'..='\u{27BF}' | // Dingbats
			'\u{1F900}'..='\u{1F9FF}' | // Supplemental symbols and pictographs
			'\u{1F018}'..='\u{1F270}' => Self::Double, // Various emoji ranges
			
			// Ambiguous width characters
			'\u{00A1}' | '\u{00A4}' | '\u{00A7}' | '\u{00A8}' |
			'\u{00AA}' | '\u{00AD}' | '\u{00AE}' | '\u{00B0}' |
			'\u{00B2}' | '\u{00B3}' | '\u{00B5}' | '\u{00B6}' |
			'\u{00B7}' | '\u{00B9}' | '\u{00BA}' | '\u{00BC}' |
			'\u{00BD}' | '\u{00BE}' | '\u{00C0}' | '\u{00C1}' |
			'\u{00C2}' | '\u{00C3}' | '\u{00C4}' | '\u{00C5}' |
			'\u{00C6}' | '\u{00C7}' | '\u{00C8}' | '\u{00C9}' |
			'\u{00CA}' | '\u{00CB}' | '\u{00CC}' | '\u{00CD}' |
			'\u{00CE}' | '\u{00CF}' | '\u{00D1}' | '\u{00D2}' |
			'\u{00D3}' | '\u{00D4}' | '\u{00D5}' | '\u{00D6}' |
			'\u{00D9}' | '\u{00DA}' | '\u{00DB}' | '\u{00DC}' |
			'\u{00DD}' | '\u{00E0}' | '\u{00E1}' | '\u{00E2}' |
			'\u{00E3}' | '\u{00E4}' | '\u{00E5}' | '\u{00E6}' |
			'\u{00E7}' | '\u{00E8}' | '\u{00E9}' | '\u{00EA}' |
			'\u{00EB}' | '\u{00EC}' | '\u{00ED}' | '\u{00EE}' |
			'\u{00EF}' | '\u{00F1}' | '\u{00F2}' | '\u{00F3}' |
			'\u{00F4}' | '\u{00F5}' | '\u{00F6}' | '\u{00F9}' |
			'\u{00FA}' | '\u{00FB}' | '\u{00FC}' | '\u{00FD}' |
			'\u{00FE}' | '\u{00FF}' => Self::Ambiguous,
			
			// Default to half-width
			_ => Self::Half,
		}
	}
	
	/**
	 * Gets the numeric width value
	 * 
	 * @return u32 - Width value
	 */
	pub fn to_u32(&self) -> u32 {
		match self {
			CharWidth::Zero => 0,
			CharWidth::Half => 1,
			CharWidth::Full => 2,
			CharWidth::Double => 2,
			CharWidth::Ambiguous => 1, // Default to half-width
		}
	}
}

/**
 * Unicode string width handler
 * 
 * Unicode文字列の幅ハンドラーです。
 * 文字列の表示幅を計算し、
 * カーソル位置を管理します。
 */
pub struct UnicodeWidthHandler {
	/// Character width cache
	width_cache: HashMap<char, CharWidth>,
	/// Ambiguous character context
	ambiguous_context: bool, // true = full-width, false = half-width
}

impl UnicodeWidthHandler {
	/**
	 * Creates a new Unicode width handler
	 * 
	 * @return UnicodeWidthHandler - New width handler
	 */
	pub fn new() -> Self {
		Self {
			width_cache: HashMap::new(),
			ambiguous_context: false, // Default to half-width
		}
	}
	
	/**
	 * Gets the width of a character
	 * 
	 * @param ch - Character to check
	 * @return CharWidth - Character width
	 */
	pub fn get_char_width(&mut self, ch: char) -> CharWidth {
		// Check cache first
		if let Some(&width) = self.width_cache.get(&ch) {
			return width;
		}
		
		let width = CharWidth::from_char(ch);
		
		// Handle ambiguous characters based on context
		let final_width = match width {
			CharWidth::Ambiguous => {
				if self.ambiguous_context {
					CharWidth::Full
				} else {
					CharWidth::Half
				}
			}
			_ => width,
		};
		
		// Cache the result
		self.width_cache.insert(ch, final_width);
		
		final_width
	}
	
	/**
	 * Gets the display width of a string
	 * 
	 * @param text - Text to measure
	 * @return u32 - Display width
	 */
	pub fn get_string_width(&mut self, text: &str) -> u32 {
		let mut width = 0;
		for ch in text.chars() {
			width += self.get_char_width(ch).to_u32();
		}
		width
	}
	
	/**
	 * Sets ambiguous character context
	 * 
	 * @param full_width - Whether to use full-width for ambiguous characters
	 */
	pub fn set_ambiguous_context(&mut self, full_width: bool) {
		self.ambiguous_context = full_width;
		// Clear cache when context changes
		self.width_cache.clear();
	}
	
	/**
	 * Gets cursor position for a string
	 * 
	 * @param text - Text to analyze
	 * @param byte_pos - Byte position in string
	 * @return u32 - Cursor column position
	 */
	pub fn get_cursor_position(&mut self, text: &str, byte_pos: usize) -> u32 {
		let mut cursor_pos = 0;
		let mut current_byte = 0;
		
		for ch in text.chars() {
			let char_bytes = ch.len_utf8();
			if current_byte + char_bytes > byte_pos {
				break;
			}
			cursor_pos += self.get_char_width(ch).to_u32();
			current_byte += char_bytes;
		}
		
		cursor_pos
	}
	
	/**
	 * Gets byte position from cursor position
	 * 
	 * @param text - Text to analyze
	 * @param cursor_pos - Cursor column position
	 * @return usize - Byte position in string
	 */
	pub fn get_byte_position(&mut self, text: &str, cursor_pos: u32) -> usize {
		let mut current_cursor = 0;
		let mut current_byte = 0;
		
		for ch in text.chars() {
			let char_width = self.get_char_width(ch).to_u32();
			if current_cursor + char_width > cursor_pos {
				break;
			}
			current_cursor += char_width;
			current_byte += ch.len_utf8();
		}
		
		current_byte
	}
	
	/**
	 * Splits text at width boundary
	 * 
	 * @param text - Text to split
	 * @param max_width - Maximum width
	 * @return Vec<String> - Split text lines
	 */
	pub fn split_at_width(&mut self, text: &str, max_width: u32) -> Vec<String> {
		let mut lines = Vec::new();
		let mut current_line = String::new();
		let mut current_width = 0;
		
		for ch in text.chars() {
			let char_width = self.get_char_width(ch).to_u32();
			
			// Check if adding this character would exceed the width
			if current_width + char_width > max_width {
				// Handle double-width characters that would be split
				if char_width == 2 && current_width == max_width - 1 {
					// Don't split double-width characters
					lines.push(current_line.clone());
					current_line.clear();
					current_width = 0;
				}
				
				// Add character to current line if it fits
				if char_width <= max_width {
					current_line.push(ch);
					current_width = char_width;
				} else {
					// Character is too wide, start new line
					if !current_line.is_empty() {
						lines.push(current_line.clone());
						current_line.clear();
						current_width = 0;
					}
					current_line.push(ch);
					current_width = char_width;
				}
			} else {
				// Character fits on current line
				current_line.push(ch);
				current_width += char_width;
			}
		}
		
		// Add remaining line
		if !current_line.is_empty() {
			lines.push(current_line);
		}
		
		lines
	}
	
	/**
	 * Truncates text to fit width
	 * 
	 * @param text - Text to truncate
	 * @param max_width - Maximum width
	 * @return String - Truncated text
	 */
	pub fn truncate_to_width(&mut self, text: &str, max_width: u32) -> String {
		let mut result = String::new();
		let mut current_width = 0;
		
		for ch in text.chars() {
			let char_width = self.get_char_width(ch).to_u32();
			
			if current_width + char_width <= max_width {
				result.push(ch);
				current_width += char_width;
			} else {
				break;
			}
		}
		
		result
	}
	
	/**
	 * Pads text to specified width
	 * 
	 * @param text - Text to pad
	 * @param target_width - Target width
	 * @param pad_char - Character to use for padding
	 * @return String - Padded text
	 */
	pub fn pad_to_width(&mut self, text: &str, target_width: u32, pad_char: char) -> String {
		let text_width = self.get_string_width(text);
		if text_width >= target_width {
			return text.to_string();
		}
		
		let pad_width = target_width - text_width;
		let pad_char_width = self.get_char_width(pad_char).to_u32();
		let pad_count = pad_width / pad_char_width;
		
		let mut result = text.to_string();
		for _ in 0..pad_count {
			result.push(pad_char);
		}
		
		result
	}
} 