
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
	pub fn is_multiline(&self) -> bool {
		self.multiline_mode
	}
	
	pub fn set_multiline(&mut self, mode: bool) {
		self.multiline_mode = mode;
	}
	
	pub fn set_continuation_char(&mut self, char: Option<char>) {
		self.continuation_char = char;
	}
	
	pub fn get_continuation_char(&self) -> Option<char> {
		self.continuation_char
	}
	
	pub fn set_prompt(&mut self, prompt: String) {
		self.multiline_prompt = prompt;
	}
	
	pub fn update(&mut self, input: &str) {
		*self = MultilineProcessor::update_multiline_state(self.clone(), input);
	}
}

pub struct MultilineProcessor;

impl MultilineProcessor {
	pub fn check_multiline_continuation(input: &str) -> (bool, Option<char>) {
		/**
		 * マルチライン継続検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。引用符と括弧の
		 * バランス検証が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 複数の構文要素とエスケープ処理の複雑なロジックです (◕‿◕)
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