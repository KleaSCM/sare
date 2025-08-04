/**
 * Multiline input handling for Sare terminal
 * 
 * This module provides multiline input support including
 * continuation lines, visual indicators, and state management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: multiline.rs
 * Description: Multiline input processing and state management
 */

use anyhow::Result;

/**
 * Multiline input state
 * 
 * Manages the state of multiline input including
 * continuation mode and visual indicators.
 */
#[derive(Debug, Clone)]
pub struct MultilineState {
	/// Whether in multiline mode
	pub multiline_mode: bool,
	/// Continuation character that triggered multiline
	pub continuation_char: Option<char>,
	/// Multiline prompt prefix
	pub multiline_prompt: String,
}

impl Default for MultilineState {
	fn default() -> Self {
		Self {
			multiline_mode: false,
			continuation_char: None,
			multiline_prompt: String::new(),
		}
	}
}

impl MultilineState {
	/**
	 * Checks if in multiline mode
	 * 
	 * @return bool - True if in multiline mode
	 */
	pub fn is_multiline(&self) -> bool {
		self.multiline_mode
	}
	
	/**
	 * Sets multiline mode
	 * 
	 * @param mode - Multiline mode state
	 */
	pub fn set_multiline(&mut self, mode: bool) {
		self.multiline_mode = mode;
	}
	
	/**
	 * Sets continuation character
	 * 
	 * @param char - Continuation character
	 */
	pub fn set_continuation_char(&mut self, char: Option<char>) {
		self.continuation_char = char;
	}
	
	/**
	 * Gets continuation character
	 * 
	 * @return Option<char> - Continuation character
	 */
	pub fn get_continuation_char(&self) -> Option<char> {
		self.continuation_char
	}
	
	/**
	 * Sets multiline prompt
	 * 
	 * @param prompt - Prompt string
	 */
	pub fn set_prompt(&mut self, prompt: String) {
		self.multiline_prompt = prompt;
	}
	
	/**
	 * Updates the state based on input
	 * 
	 * @param input - Input text to check
	 */
	pub fn update(&mut self, input: &str) {
		*self = MultilineProcessor::update_multiline_state(self.clone(), input);
	}
}

/**
 * Multiline input processor
 * 
 * Handles multiline input parsing and state management
 * for continuation lines and visual feedback.
 */
pub struct MultilineProcessor;

impl MultilineProcessor {
	/**
	 * Checks if input needs multiline continuation
	 * 
	 * @param input - Input text to check
	 * @return (bool, Option<char>) - (Needs continuation, continuation character)
	 */
	pub fn check_multiline_continuation(input: &str) -> (bool, Option<char>) {
		/**
		 * マルチライン継続チェックの複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * 継続文字とクォート処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let trimmed = input.trim();
		
		// Check for backslash continuation
		if trimmed.ends_with('\\') {
			return (true, Some('\\'));
		}
		
		// Check for pipe continuation
		if trimmed.ends_with('|') {
			return (true, Some('|'));
		}
		
		// Check for unclosed quotes
		let mut in_single_quotes = false;
		let mut in_double_quotes = false;
		let mut escaped = false;
		
		for ch in input.chars() {
			if escaped {
				escaped = false;
				continue;
			}
			
			match ch {
				'\\' => escaped = true,
				'\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
				'"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
				_ => {}
			}
		}
		
		if in_single_quotes {
			return (true, Some('\''));
		}
		
		if in_double_quotes {
			return (true, Some('"'));
		}
		
		// Check for unclosed parentheses
		let mut paren_count = 0;
		let mut brace_count = 0;
		let mut bracket_count = 0;
		
		for ch in input.chars() {
			match ch {
				'(' => paren_count += 1,
				')' => paren_count -= 1,
				'{' => brace_count += 1,
				'}' => brace_count -= 1,
				'[' => bracket_count += 1,
				']' => bracket_count -= 1,
				_ => {}
			}
		}
		
		if paren_count > 0 {
			return (true, Some('('));
		}
		
		if brace_count > 0 {
			return (true, Some('{'));
		}
		
		if bracket_count > 0 {
			return (true, Some('['));
		}
		
		// No continuation needed
		(false, None)
	}
	
	/**
	 * Updates multiline state based on input
	 * 
	 * @param state - Current multiline state
	 * @param input - Input text to check
	 * @return MultilineState - Updated state
	 */
	pub fn update_multiline_state(mut state: MultilineState, input: &str) -> MultilineState {
		let (needs_continuation, continuation_char) = Self::check_multiline_continuation(input);
		
		state.multiline_mode = needs_continuation;
		state.continuation_char = continuation_char;
		
		// Set appropriate prompt
		if needs_continuation {
			match continuation_char {
				Some('\\') => state.multiline_prompt = "> ".to_string(),
				Some('|') => state.multiline_prompt = "| ".to_string(),
				Some('\'') => state.multiline_prompt = "'> ".to_string(),
				Some('"') => state.multiline_prompt = "\"> ".to_string(),
				Some('(') => state.multiline_prompt = "(> ".to_string(),
				Some('{') => state.multiline_prompt = "{> ".to_string(),
				Some('[') => state.multiline_prompt = "[> ".to_string(),
				_ => state.multiline_prompt = "> ".to_string(),
			}
		} else {
			state.multiline_prompt.clear();
		}
		
		state
	}
	
	/**
	 * Gets the appropriate prompt for multiline state
	 * 
	 * @param state - Multiline state
	 * @return String - Prompt string
	 */
	pub fn get_prompt(state: &MultilineState) -> String {
		if state.multiline_mode {
			state.multiline_prompt.clone()
		} else {
			"$ ".to_string()
		}
	}
} 