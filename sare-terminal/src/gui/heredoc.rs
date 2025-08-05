
/**
 * Heredoc module for Sare terminal
 * 
 * This module provides heredoc functionality including
 * << delimiter detection, content collection, and variable expansion.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: heredoc.rs
 * Description: Heredoc processing and state management
 */

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct HeredocState {
	/// Whether in heredoc mode
	pub heredoc_mode: bool,
	/// Heredoc delimiter
	pub heredoc_delimiter: String,
	/// Heredoc content being collected
	pub heredoc_content: String,
	/// Whether heredoc content should expand variables
	pub heredoc_expand_vars: bool,
}

impl Default for HeredocState {
	fn default() -> Self {
		Self {
			heredoc_mode: false,
			heredoc_delimiter: String::new(),
			heredoc_content: String::new(),
			heredoc_expand_vars: false,
		}
	}
}

impl HeredocState {
	/**
	 * Checks if heredoc mode is currently active
	 * 
	 * @return bool - True if heredoc mode is enabled
	 */
	pub fn is_heredoc(&self) -> bool {
		self.heredoc_mode
	}
	
	/**
	 * Sets the heredoc mode state
	 * 
	 * @param mode - Whether to enable or disable heredoc mode
	 */
	pub fn set_heredoc(&mut self, mode: bool) {
		self.heredoc_mode = mode;
	}
	
	/**
	 * Sets the heredoc delimiter string
	 * 
	 * @param delimiter - New delimiter string
	 */
	pub fn set_delimiter(&mut self, delimiter: String) {
		self.heredoc_delimiter = delimiter;
	}
	
	/**
	 * Gets the current heredoc delimiter
	 * 
	 * @return String - Current delimiter string
	 */
	pub fn get_delimiter(&self) -> String {
		self.heredoc_delimiter.clone()
	}
	
	/**
	 * Sets the heredoc content buffer
	 * 
	 * @param content - New heredoc content
	 */
	pub fn set_heredoc_content(&mut self, content: String) {
		self.heredoc_content = content;
	}
	
	/**
	 * Gets the current heredoc content
	 * 
	 * @return String - Current heredoc content
	 */
	pub fn get_heredoc_content(&self) -> String {
		self.heredoc_content.clone()
	}
	
	/**
	 * Sets whether variables should be expanded in heredoc content
	 * 
	 * @param expand - Whether to enable variable expansion
	 */
	pub fn set_expand_vars(&mut self, expand: bool) {
		self.heredoc_expand_vars = expand;
	}
	
	/**
	 * Checks if variables should be expanded in heredoc content
	 * 
	 * @return bool - True if variable expansion is enabled
	 */
	pub fn should_expand_vars(&self) -> bool {
		self.heredoc_expand_vars
	}
	
	/**
	 * Adds content to the heredoc buffer with a newline
	 * 
	 * @param content - Content to add to heredoc buffer
	 */
	pub fn add_heredoc_content(&mut self, content: String) {
		self.heredoc_content.push_str(&content);
		self.heredoc_content.push('\n');
	}
	
	/**
	 * Detects heredoc patterns in input using current state
	 * 
	 * @param input - Input string to check
	 * @return Option<(String, bool)> - Delimiter and expansion flag if found
	 */
	pub fn detect_heredoc(&self, input: &str) -> Option<(String, bool)> {
		HeredocProcessor::detect_heredoc(input)
	}
}

pub struct HeredocProcessor;

impl HeredocProcessor {
	pub fn detect_heredoc(input: &str) -> Option<(String, bool)> {
		/**
		 * ヒアドキュメントパターンを検出する関数です
		 * 
		 * <<delimiter、<<'delimiter、<<"delimiter 形式のヒアドキュメントを検出して、
		 * デリミタ文字列と変数展開フラグを返します。
		 * 
		 * 引用符付きデリミタは変数展開を無効にし、引用符なしは変数展開を有効にします
		 */
		
		let words: Vec<&str> = input.split_whitespace().collect();
		
		for (i, word) in words.iter().enumerate() {
			if word.starts_with("<<") {
				if word.starts_with("<<'") || word.starts_with("<<\"") {
					let quote_char = word.chars().nth(2).unwrap();
					let delimiter = word[3..].to_string();
					return Some((delimiter, false));
				}
				
				if word.len() > 2 {
					let delimiter = word[2..].to_string();
					return Some((delimiter, true));
				}
			}
		}
		
		None
	}
	
	pub fn is_heredoc_delimiter(state: &HeredocState, line: &str) -> bool {
		if !state.heredoc_mode {
			return false;
		}
		
		let trimmed = line.trim();
		trimmed == state.heredoc_delimiter
	}
	
	pub fn expand_heredoc_variables(content: &str) -> String {
		/**
		 * ヒアドキュメント内の変数を展開する関数です
		 * 
		 * $VARIABLE 形式の環境変数を検出して、対応する環境変数の値に置き換えます。
		 * 
		 * 環境変数が見つからない場合は元の文字列を保持し、$記号のみの場合は
		 * そのまま出力します
		 */
		
		let mut result = String::new();
		let mut i = 0;
		
		while i < content.len() {
			if content[i..].starts_with('$') {
				let var_start = i + 1;
				let mut var_end = var_start;
				
				while var_end < content.len() {
					let ch = content.chars().nth(var_end).unwrap();
					if ch.is_alphanumeric() || ch == '_' {
						var_end += 1;
					} else {
						break;
					}
				}
				
				if var_end > var_start {
					let var_name = &content[var_start..var_end];
					
					if let Ok(var_value) = std::env::var(var_name) {
						result.push_str(&var_value);
					} else {
						result.push_str(&content[i..var_end]);
					}
					
					i = var_end;
				} else {
					result.push('$');
					i += 1;
				}
			} else {
				result.push(content.chars().nth(i).unwrap());
				i += 1;
			}
		}
		
		result
	}
	
	pub fn update_heredoc_state(mut state: HeredocState, input: &str) -> HeredocState {
		if let Some((delimiter, expand_vars)) = Self::detect_heredoc(input) {
			state.heredoc_mode = true;
			state.heredoc_delimiter = delimiter;
			state.heredoc_expand_vars = expand_vars;
		}
		
		state
	}
	
	pub fn add_heredoc_content(mut state: HeredocState, content: &str) -> HeredocState {
		if state.heredoc_mode {
			let processed_content = if state.heredoc_expand_vars {
				Self::expand_heredoc_variables(content)
			} else {
				content.to_string()
			};
			
			state.heredoc_content.push_str(&processed_content);
			state.heredoc_content.push('\n');
		}
		
		state
	}
	
	pub fn get_prompt(state: &HeredocState) -> String {
		if state.heredoc_mode {
			"heredoc> ".to_string()
		} else {
			"$ ".to_string()
		}
	}
} 