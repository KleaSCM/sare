/**
 * Bidirectional text handling for Sare terminal
 * 
 * This module provides bidirectional text rendering support including
 * right-to-left text handling and proper text direction management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: bidi_handler.rs
 * Description: Bidirectional text handling with RTL support
 */

use anyhow::Result;
use std::collections::HashMap;

/**
 * Text direction
 * 
 * テキスト方向です。
 * テキストの表示方向を
 * 管理します。
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
	/// Left-to-right text
	LTR,
	/// Right-to-left text
	RTL,
	/// Neutral text
	Neutral,
}

/**
 * Bidirectional character type
 * 
 * 双方向文字タイプです。
 * Unicode双方向アルゴリズムの
 * 文字タイプを管理します。
 */
	/**
	 * Gets bidirectional type for a character
	 * 
	 * @param ch - Character to check
	 * @return BidiType - Bidirectional type
	 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidiType {
	/// Left-to-right
	L,
	/// Right-to-left
	R,
	/// Arabic letter
	AL,
	/// European number
	EN,
	/// European separator
	ES,
	/// European terminator
	ET,
	/// Arabic number
	AN,
	/// Common separator
	CS,
	/// Paragraph separator
	B,
	/// Segment separator
	S,
	/// Whitespace
	WS,
	/// Other neutral
	ON,
}

impl BidiType {

	pub fn from_char(ch: char) -> Self {
		match ch {
			// Left-to-right characters
			'\u{0041}'..='\u{005A}' | // Latin capital letters
			'\u{0061}'..='\u{007A}' | // Latin small letters
			'\u{00C0}'..='\u{00D6}' | // Latin-1 supplement
			'\u{00D8}'..='\u{00F6}' | // Latin-1 supplement
			'\u{00F8}'..='\u{00FF}' | // Latin-1 supplement
			'\u{0100}'..='\u{017F}' | // Latin extended
			'\u{0180}'..='\u{024F}' | // Latin extended-B
			'\u{0250}'..='\u{02AF}' | // IPA extensions
			'\u{02B0}'..='\u{02FF}' | // Spacing modifier letters
			'\u{0300}'..='\u{036F}' | // Combining diacritical marks
			'\u{0370}'..='\u{03FF}' | // Greek and Coptic
			'\u{0400}'..='\u{04FF}' | // Cyrillic
			'\u{0500}'..='\u{052F}' | // Cyrillic supplement
			'\u{0530}'..='\u{058F}' | // Armenian
			'\u{0590}'..='\u{05FF}' | // Hebrew
			'\u{0600}'..='\u{06FF}' | // Arabic
			'\u{0700}'..='\u{074F}' | // Syriac
			'\u{0750}'..='\u{077F}' | // Arabic supplement
			'\u{0780}'..='\u{07BF}' | // Thaana
			'\u{07C0}'..='\u{07FF}' | // NKo
			'\u{0800}'..='\u{083F}' | // Samaritan
			'\u{0840}'..='\u{085F}' | // Mandaic
			'\u{0860}'..='\u{086F}' | // Syriac supplement
			'\u{0870}'..='\u{089F}' | // Arabic extended-B
			'\u{08A0}'..='\u{08FF}' | // Arabic extended-A
			'\u{0900}'..='\u{097F}' | // Devanagari
			'\u{0980}'..='\u{09FF}' | // Bengali
			'\u{0A00}'..='\u{0A7F}' | // Gurmukhi
			'\u{0A80}'..='\u{0AFF}' | // Gujarati
			'\u{0B00}'..='\u{0B7F}' | // Oriya
			'\u{0B80}'..='\u{0BFF}' | // Tamil
			'\u{0C00}'..='\u{0C7F}' | // Telugu
			'\u{0C80}'..='\u{0CFF}' | // Kannada
			'\u{0D00}'..='\u{0D7F}' | // Malayalam
			'\u{0D80}'..='\u{0DFF}' | // Sinhala
			'\u{0E00}'..='\u{0E7F}' | // Thai
			'\u{0E80}'..='\u{0EFF}' | // Lao
			'\u{0F00}'..='\u{0FFF}' | // Tibetan
			'\u{1000}'..='\u{109F}' | // Myanmar
			'\u{10A0}'..='\u{10FF}' | // Georgian
			'\u{1100}'..='\u{11FF}' | // Hangul Jamo
			'\u{1200}'..='\u{137F}' | // Ethiopic
			'\u{1380}'..='\u{139F}' | // Ethiopic supplement
			'\u{13A0}'..='\u{13FF}' | // Cherokee
			'\u{1400}'..='\u{167F}' | // Unified Canadian Aboriginal syllabics
			'\u{1680}'..='\u{169F}' | // Ogham
			'\u{16A0}'..='\u{16FF}' | // Runic
			'\u{1700}'..='\u{171F}' | // Tagalog
			'\u{1720}'..='\u{173F}' | // Hanunoo
			'\u{1740}'..='\u{175F}' | // Buhid
			'\u{1760}'..='\u{177F}' | // Tagbanwa
			'\u{1780}'..='\u{17FF}' | // Khmer
			'\u{1800}'..='\u{18AF}' | // Mongolian
			'\u{18B0}'..='\u{18FF}' | // Unified Canadian Aboriginal syllabics extended
			'\u{1900}'..='\u{194F}' | // Limbu
			'\u{1950}'..='\u{197F}' | // Tai Le
			'\u{1980}'..='\u{19DF}' | // New Tai Lue
			'\u{19E0}'..='\u{19FF}' | // Khmer symbols
			'\u{1A00}'..='\u{1A4F}' | // Buginese
			'\u{1A50}'..='\u{1A7F}' | // Tai Tham
			'\u{1A80}'..='\u{1A9F}' | // Combining diacritical marks extended
			'\u{1AA0}'..='\u{1AFF}' | // Tai Tham
			'\u{1B00}'..='\u{1B4F}' | // Balinese
			'\u{1B50}'..='\u{1B7F}' | // Sundanese
			'\u{1B80}'..='\u{1BBF}' | // Sundanese supplement
			'\u{1BC0}'..='\u{1BFF}' | // Batak
			'\u{1C00}'..='\u{1C4F}' | // Lepcha
			'\u{1C50}'..='\u{1C7F}' | // Ol Chiki
			'\u{1C80}'..='\u{1C8F}' | // Cyrillic extended-C
			'\u{1C90}'..='\u{1CBF}' | // Georgian extended
			'\u{1CC0}'..='\u{1CCF}' | // Sundanese supplement
			'\u{1CD0}'..='\u{1CFF}' | // Vedic extensions
			'\u{1D00}'..='\u{1D7F}' | // Phonetic extensions
			'\u{1D80}'..='\u{1DBF}' | // Phonetic extensions supplement
			'\u{1DC0}'..='\u{1DFF}' | // Combining diacritical marks supplement
			'\u{1E00}'..='\u{1EFF}' | // Latin extended additional
			'\u{1F00}'..='\u{1FFF}' | // Greek extended
			'\u{2000}'..='\u{206F}' | // General punctuation
			'\u{2070}'..='\u{209F}' | // Superscripts and subscripts
			'\u{20A0}'..='\u{20CF}' | // Currency symbols
			'\u{20D0}'..='\u{20FF}' | // Combining diacritical marks for symbols
			'\u{2100}'..='\u{214F}' | // Letterlike symbols
			'\u{2150}'..='\u{218F}' | // Number forms
			'\u{2190}'..='\u{21FF}' | // Arrows
			'\u{2200}'..='\u{22FF}' | // Mathematical operators
			'\u{2300}'..='\u{23FF}' | // Miscellaneous technical
			'\u{2400}'..='\u{243F}' | // Control pictures
			'\u{2440}'..='\u{245F}' | // Optical character recognition
			'\u{2460}'..='\u{24FF}' | // Enclosed alphanumerics
			'\u{2500}'..='\u{257F}' | // Box drawing
			'\u{2580}'..='\u{259F}' | // Block elements
			'\u{25A0}'..='\u{25FF}' | // Geometric shapes
			'\u{2600}'..='\u{26FF}' | // Miscellaneous symbols
			'\u{2700}'..='\u{27BF}' | // Dingbats
			'\u{27C0}'..='\u{27EF}' | // Miscellaneous mathematical symbols-A
			'\u{27F0}'..='\u{27FF}' | // Supplemental arrows-A
			'\u{2800}'..='\u{28FF}' | // Braille patterns
			'\u{2900}'..='\u{297F}' | // Supplemental arrows-B
			'\u{2980}'..='\u{29FF}' | // Miscellaneous mathematical symbols-B
			'\u{2A00}'..='\u{2AFF}' | // Supplemental mathematical operators
			'\u{2B00}'..='\u{2BFF}' | // Miscellaneous symbols and arrows
			'\u{2C00}'..='\u{2C5F}' | // Glagolitic
			'\u{2C60}'..='\u{2C7F}' | // Latin extended-C
			'\u{2C80}'..='\u{2CFF}' | // Coptic
			'\u{2D00}'..='\u{2D2F}' | // Georgian supplement
			'\u{2D30}'..='\u{2D7F}' | // Tifinagh
			'\u{2D80}'..='\u{2DDF}' | // Ethiopic extended
			'\u{2DE0}'..='\u{2DFF}' | // Cyrillic extended-A
			'\u{2E00}'..='\u{2E7F}' | // Supplemental punctuation
			'\u{2E80}'..='\u{2EFF}' | // CJK radicals supplement
			'\u{2F00}'..='\u{2FDF}' | // Kangxi radicals
			'\u{2FE0}'..='\u{2FEF}' | // Ideographic description characters
			'\u{2FF0}'..='\u{2FFF}' | // CJK symbols and punctuation
			'\u{3000}'..='\u{303F}' | // CJK symbols and punctuation
			'\u{3040}'..='\u{309F}' | // Hiragana
			'\u{30A0}'..='\u{30FF}' | // Katakana
			'\u{3100}'..='\u{312F}' | // Bopomofo
			'\u{3130}'..='\u{318F}' | // Hangul compatibility jamo
			'\u{3190}'..='\u{319F}' | // Kanbun
			'\u{31A0}'..='\u{31BF}' | // Bopomofo extended
			'\u{31C0}'..='\u{31EF}' | // CJK strokes
			'\u{31F0}'..='\u{31FF}' | // Katakana phonetic extensions
			'\u{3200}'..='\u{32FF}' | // Enclosed CJK letters and months
			'\u{3300}'..='\u{33FF}' | // CJK compatibility
			'\u{3400}'..='\u{4DBF}' | // CJK unified ideographs extension A
			'\u{4DC0}'..='\u{4DFF}' | // Yijing hexagram symbols
			'\u{4E00}'..='\u{9FFF}' | // CJK unified ideographs
			'\u{A000}'..='\u{A48F}' | // Yi syllables
			'\u{A490}'..='\u{A4CF}' | // Yi radicals
			'\u{A4D0}'..='\u{A4FF}' | // Lisu
			'\u{A500}'..='\u{A63F}' | // Vai
			'\u{A640}'..='\u{A69F}' | // Cyrillic extended-B
			'\u{A6A0}'..='\u{A6FF}' | // Bamum
			'\u{A700}'..='\u{A71F}' | // Modifier tone letters
			'\u{A720}'..='\u{A7FF}' | // Latin extended-D
			'\u{A800}'..='\u{A82F}' | // Syloti Nagri
			'\u{A830}'..='\u{A83F}' | // Common Indic number forms
			'\u{A840}'..='\u{A87F}' | // Phags-pa
			'\u{A880}'..='\u{A8DF}' | // Saurashtra
			'\u{A8E0}'..='\u{A8FF}' | // Devanagari extended
			'\u{A900}'..='\u{A92F}' | // Kayah Li
			'\u{A930}'..='\u{A95F}' | // Rejang
			'\u{A960}'..='\u{A97F}' | // Hangul jamo extended-A
			'\u{A980}'..='\u{A9DF}' | // Javanese
			'\u{A9E0}'..='\u{A9FF}' | // Myanmar extended-B
			'\u{AA00}'..='\u{AA5F}' | // Cham
			'\u{AA60}'..='\u{AA7F}' | // Myanmar extended-A
			'\u{AA80}'..='\u{AADF}' | // Tai Viet
			'\u{AAE0}'..='\u{AAFF}' | // Meetei mayek extensions
			'\u{AB00}'..='\u{AB2F}' | // Ethiopic extended-A
			'\u{AB30}'..='\u{AB6F}' | // Latin extended-E
			'\u{AB70}'..='\u{ABBF}' | // Cherokee supplement
			'\u{ABC0}'..='\u{ABFF}' | // Meetei mayek
			'\u{AC00}'..='\u{D7AF}' | // Hangul syllables
			'\u{D7B0}'..='\u{D7FF}' | // Hangul jamo extended-B
			'\u{E000}'..='\u{F8FF}' | // Private use area
			'\u{F900}'..='\u{FAFF}' | // CJK compatibility ideographs
			'\u{FB00}'..='\u{FB4F}' | // Alphabetic presentation forms
			'\u{FB50}'..='\u{FDFF}' | // Arabic presentation forms-A
			'\u{FE00}'..='\u{FE0F}' | // Variation selectors
			'\u{FE10}'..='\u{FE1F}' | // Vertical forms
			'\u{FE20}'..='\u{FE2F}' | // Combining half marks
			'\u{FE30}'..='\u{FE4F}' | // CJK compatibility forms
			'\u{FE50}'..='\u{FE6F}' | // Small form variants
			'\u{FE70}'..='\u{FEFF}' | // Arabic presentation forms-B
			'\u{FF00}'..='\u{FFEF}' | // Halfwidth and fullwidth forms
			'\u{FFF0}'..='\u{FFFF}' => Self::L, // Specials
			
			// Right-to-left characters (Arabic, Hebrew, etc.)
			'\u{0590}'..='\u{05FF}' | // Hebrew
			'\u{0600}'..='\u{06FF}' | // Arabic
			'\u{0750}'..='\u{077F}' | // Arabic supplement
			'\u{08A0}'..='\u{08FF}' | // Arabic extended-A
			'\u{FB1D}'..='\u{FB4F}' | // Hebrew presentation forms
			'\u{FB50}'..='\u{FDFF}' | // Arabic presentation forms-A
			'\u{FE70}'..='\u{FEFF}' => Self::R, // Arabic presentation forms-B
			
			// Arabic letters
			'\u{0600}'..='\u{06FF}' | // Arabic
			'\u{0750}'..='\u{077F}' | // Arabic supplement
			'\u{08A0}'..='\u{08FF}' | // Arabic extended-A
			'\u{FB50}'..='\u{FDFF}' | // Arabic presentation forms-A
			'\u{FE70}'..='\u{FEFF}' => Self::AL, // Arabic presentation forms-B
			
			// European numbers
			'\u{0030}'..='\u{0039}' => Self::EN, // ASCII digits
			
			// European separators
			'\u{002B}' | // Plus sign
			'\u{002D}' => Self::ES, // Hyphen-minus
			
			// European terminators
			'\u{0021}' | // Exclamation mark
			'\u{002C}' | // Comma
			'\u{002E}' | // Full stop
			'\u{003A}' | // Colon
			'\u{003B}' | // Semicolon
			'\u{003F}' => Self::ET, // Question mark
			
			// Arabic numbers
			'\u{0660}'..='\u{0669}' | // Arabic-indic digits
			'\u{06F0}'..='\u{06F9}' => Self::AN, // Extended Arabic-indic digits
			
			// Common separators
			'\u{002C}' | // Comma
			'\u{002E}' | // Full stop
			'\u{003A}' | // Colon
			'\u{003B}' | // Semicolon
			'\u{060C}' | // Arabic comma
			'\u{06D4}' => Self::CS, // Arabic full stop
			
			// Paragraph separators
			'\u{000A}' | // Line feed
			'\u{000D}' | // Carriage return
			'\u{2029}' => Self::B, // Paragraph separator
			
			// Segment separators
			'\u{0009}' | // Tab
			'\u{000B}' | // Vertical tab
			'\u{000C}' | // Form feed
			'\u{001C}' | // File separator
			'\u{001D}' | // Group separator
			'\u{001E}' | // Record separator
			'\u{001F}' | // Unit separator
			'\u{2028}' => Self::S, // Line separator
			
			// Whitespace
			'\u{0020}' | // Space
			'\u{00A0}' | // No-break space
			'\u{1680}' | // Ogham space mark
			'\u{2000}'..='\u{200A}' | // Various spaces
			'\u{202F}' | // Narrow no-break space
			'\u{205F}' | // Medium mathematical space
			'\u{3000}' => Self::WS, // Ideographic space
			
			// Default to other neutral
			_ => Self::ON,
		}
	}
}

/**
 * Bidirectional text handler
 * 
 * 双方向テキストハンドラーです。
 * 右から左へのテキスト表示を
 * 管理します。
 */
pub struct BidiHandler {
	/// Character type cache
	type_cache: HashMap<char, BidiType>,
	/// Default text direction
	default_direction: TextDirection,
}
	/**
	 * Creates a new bidirectional text handler
	 * 
	 * @return BidiHandler - New bidirectional handler
	 */
impl BidiHandler {

	pub fn new() -> Self {
		Self {
			type_cache: HashMap::new(),
			default_direction: TextDirection::LTR,
		}
	}
	
	/**
	 * Sets the default text direction
	 * 
	 * @param direction - Default text direction
	 */
	pub fn set_default_direction(&mut self, direction: TextDirection) {
		self.default_direction = direction;
	}
	
	/**
	 * Gets the bidirectional type for a character
	 * 
	 * @param ch - Character to check
	 * @return BidiType - Bidirectional type
	 */
	pub fn get_bidi_type(&mut self, ch: char) -> BidiType {
		// Check cache first
		if let Some(&bidi_type) = self.type_cache.get(&ch) {
			return bidi_type;
		}
		
		let bidi_type = BidiType::from_char(ch);
		self.type_cache.insert(ch, bidi_type);
		bidi_type
	}
	
	/**
	 * Determines the base direction of text
	 * 
	 * @param text - Text to analyze
	 * @return TextDirection - Base text direction
	 */
	pub fn get_base_direction(&mut self, text: &str) -> TextDirection {
		for ch in text.chars() {
			let bidi_type = self.get_bidi_type(ch);
			match bidi_type {
				BidiType::L => return TextDirection::LTR,
				BidiType::R | BidiType::AL => return TextDirection::RTL,
				_ => continue,
			}
		}
		
		self.default_direction
	}
	
	/**
	 * Reorders text for bidirectional display
	 * 
	 * @param text - Text to reorder
	 * @return String - Reordered text
	 */
	pub fn reorder_text(&mut self, text: &str) -> String {
		let base_direction = self.get_base_direction(text);
		
		// Simple bidirectional algorithm implementation
		let mut result = String::new();
		let mut rtl_segments = Vec::new();
		let mut ltr_segments = Vec::new();
		let mut current_segment = String::new();
		let mut current_direction = base_direction;
		
		for ch in text.chars() {
			let bidi_type = self.get_bidi_type(ch);
			let char_direction = match bidi_type {
				BidiType::L => TextDirection::LTR,
				BidiType::R | BidiType::AL => TextDirection::RTL,
				_ => current_direction,
			};
			
			if char_direction != current_direction {
				// Store current segment
				if !current_segment.is_empty() {
					match current_direction {
						TextDirection::LTR => ltr_segments.push(current_segment.clone()),
						TextDirection::RTL => rtl_segments.push(current_segment.clone()),
						TextDirection::Neutral => ltr_segments.push(current_segment.clone()),
					}
				}
				
				current_segment.clear();
				current_direction = char_direction;
			}
			
			current_segment.push(ch);
		}
		
		// Store final segment
		if !current_segment.is_empty() {
			match current_direction {
				TextDirection::LTR => ltr_segments.push(current_segment.clone()),
				TextDirection::RTL => rtl_segments.push(current_segment.clone()),
				TextDirection::Neutral => ltr_segments.push(current_segment.clone()),
			}
		}
		
		// Build result based on base direction
		match base_direction {
			TextDirection::LTR => {
				// LTR segments first, then RTL segments reversed
				for segment in ltr_segments {
					result.push_str(&segment);
				}
				for segment in rtl_segments.iter().rev() {
					result.push_str(&segment);
				}
			}
			TextDirection::RTL => {
				// RTL segments first, then LTR segments
				for segment in rtl_segments.iter().rev() {
					result.push_str(&segment);
				}
				for segment in ltr_segments {
					result.push_str(&segment);
				}
			}
			TextDirection::Neutral => {
				// Default to LTR ordering
				for segment in ltr_segments {
					result.push_str(&segment);
				}
				for segment in rtl_segments.iter().rev() {
					result.push_str(&segment);
				}
			}
		}
		
		result
	}
	
	/**
	 * Gets the display order of characters
	 * 
	 * @param text - Text to analyze
	 * @return Vec<usize> - Character indices in display order
	 */
	pub fn get_display_order(&mut self, text: &str) -> Vec<usize> {
		let reordered = self.reorder_text(text);
		let mut order = Vec::new();
		
		// Map characters back to original positions
		let mut original_chars: Vec<char> = text.chars().collect();
		let mut reordered_chars: Vec<char> = reordered.chars().collect();
		
		for (i, reordered_char) in reordered_chars.iter().enumerate() {
			if let Some(original_pos) = original_chars.iter().position(|&c| c == *reordered_char) {
				order.push(original_pos);
				original_chars[original_pos] = '\0'; // Mark as used
			}
		}
		
		order
	}
	
	/**
	 * Mirrors characters for RTL display
	 * 
	 * @param text - Text to mirror
	 * @return String - Mirrored text
	 */
	pub fn mirror_text(&mut self, text: &str) -> String {
		let mut result = String::new();
		
		for ch in text.chars() {
			let mirrored = match ch {
				'(' => ')',
				')' => '(',
				'[' => ']',
				']' => '[',
				'{' => '}',
				'}' => '{',
				'<' => '>',
				'>' => '<',
				_ => ch,
			};
			result.push(mirrored);
		}
		
		result
	}
} 