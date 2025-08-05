
/**
 * Multiline input processing module for Sare terminal
 * 
 * This module provides multiline functionality including
 * continuation character detection, quote handling, bracket matching,
 * and multiline state management for complex command input.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: multiline.rs
 * Description: Multiline input processing and state management
 */

use anyhow::Result;

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
	 * Checks if multiline mode is currently active
	 * 
	 * @return bool - True if multiline mode is enabled
	 */
	pub fn is_multiline(&self) -> bool {
		self.multiline_mode
	}
	
	/**
	 * Sets the multiline mode state
	 * 
	 * @param mode - Whether to enable or disable multiline mode
	 */
	pub fn set_multiline(&mut self, mode: bool) {
		self.multiline_mode = mode;
	}
	
	/**
	 * Sets the continuation character that triggered multiline mode
	 * 
	 * @param char - Continuation character (\, |, ', ", (, {, [)
	 */
	pub fn set_continuation_char(&mut self, char: Option<char>) {
		self.continuation_char = char;
	}
	
	/**
	 * Gets the current continuation character
	 * 
	 * @return Option<char> - Current continuation character
	 */
	pub fn get_continuation_char(&self) -> Option<char> {
		self.continuation_char
	}
	
	/**
	 * Sets the multiline prompt prefix
	 * 
	 * @param prompt - New multiline prompt string
	 */
	pub fn set_prompt(&mut self, prompt: String) {
		self.multiline_prompt = prompt;
	}
	
	/**
	 * Updates the multiline state based on input
	 * 
	 * @param input - Input string to analyze for continuation
	 */
	pub fn update(&mut self, input: &str) {
		*self = MultilineProcessor::update_multiline_state(self.clone(), input);
	}
}

pub struct MultilineProcessor;

impl MultilineProcessor {
	pub fn check_multiline_continuation(input: &str) -> (bool, Option<char>) {
		/**
		 * マルチライン継続を検出する関数です
		 * 
		 * バックスラッシュ (\)、パイプ (|)、引用符、括弧のバランスを
		 * 解析して、コマンドが継続が必要かどうかを判定します。
		 * 
		 * 引用符内の文字は無視し、エスケープ文字を適切に処理して、
		 * 括弧の開閉バランスを追跡します
		 */
		
		let trimmed = input.trim();
		
		if trimmed.ends_with('\\') {
			return (true, Some('\\'));
		}
		
		if trimmed.ends_with('|') {
			return (true, Some('|'));
		}
		
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
		
		(false, None)
	}
	
	pub fn update_multiline_state(mut state: MultilineState, input: &str) -> MultilineState {
		let (needs_continuation, continuation_char) = Self::check_multiline_continuation(input);
		
		state.multiline_mode = needs_continuation;
		state.continuation_char = continuation_char;
		
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
	
	pub fn get_prompt(state: &MultilineState) -> String {
		if state.multiline_mode {
			state.multiline_prompt.clone()
		} else {
			"$ ".to_string()
		}
	}
} 