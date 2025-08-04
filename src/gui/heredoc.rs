/**
 * Heredoc processing for Sare terminal
 * 
 * This module provides heredoc support including
 * syntax detection, content collection, and variable expansion.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: heredoc.rs
 * Description: Heredoc processing and content management
 */

use anyhow::Result;

/**
 * Heredoc state
 * 
 * Manages the state of heredoc processing including
 * delimiter, content collection, and variable expansion.
 */
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
	 * Checks if in heredoc mode
	 * 
	 * @return bool - True if in heredoc mode
	 */
	pub fn is_heredoc(&self) -> bool {
		self.heredoc_mode
	}
	
	/**
	 * Sets heredoc mode
	 * 
	 * @param mode - Heredoc mode state
	 */
	pub fn set_heredoc(&mut self, mode: bool) {
		self.heredoc_mode = mode;
	}
	
	/**
	 * Sets heredoc delimiter
	 * 
	 * @param delimiter - Delimiter string
	 */
	pub fn set_delimiter(&mut self, delimiter: String) {
		self.heredoc_delimiter = delimiter;
	}
	
	/**
	 * Gets heredoc delimiter
	 * 
	 * @return String - Delimiter string
	 */
	pub fn get_delimiter(&self) -> String {
		self.heredoc_delimiter.clone()
	}
	
	/**
	 * Sets heredoc content
	 * 
	 * @param content - Content string
	 */
	pub fn set_heredoc_content(&mut self, content: String) {
		self.heredoc_content = content;
	}
	
	/**
	 * Gets heredoc content
	 * 
	 * @return String - Content string
	 */
	pub fn get_heredoc_content(&self) -> String {
		self.heredoc_content.clone()
	}
	
	/**
	 * Sets variable expansion flag
	 * 
	 * @param expand - Whether to expand variables
	 */
	pub fn set_expand_vars(&mut self, expand: bool) {
		self.heredoc_expand_vars = expand;
	}
	
	/**
	 * Checks if variables should be expanded
	 * 
	 * @return bool - True if variables should be expanded
	 */
	pub fn should_expand_vars(&self) -> bool {
		self.heredoc_expand_vars
	}
	
	/**
	 * Adds content to heredoc
	 * 
	 * @param content - Content to add
	 */
	pub fn add_heredoc_content(&mut self, content: String) {
		self.heredoc_content.push_str(&content);
		self.heredoc_content.push('\n');
	}
	
	/**
	 * Detects heredoc syntax in input
	 * 
	 * @param input - Input text to check
	 * @return Option<(String, bool)> - (Delimiter, expand variables) if heredoc found
	 */
	pub fn detect_heredoc(&self, input: &str) -> Option<(String, bool)> {
		HeredocProcessor::detect_heredoc(input)
	}
}

/**
 * Heredoc processor
 * 
 * Handles heredoc syntax detection and content processing
 * with variable expansion and delimiter handling.
 */
pub struct HeredocProcessor;

impl HeredocProcessor {
	/**
	 * Detects heredoc syntax in input
	 * 
	 * @param input - Input text to check
	 * @return Option<(String, bool)> - (Delimiter, expand variables) if heredoc found
	 */
	pub fn detect_heredoc(input: &str) -> Option<(String, bool)> {
		/**
		 * ヒアドキュメント検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * ヒアドキュメント構文の検出が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let words: Vec<&str> = input.split_whitespace().collect();
		
		for (i, word) in words.iter().enumerate() {
			if word.starts_with("<<") {
				// Check for quoted delimiter (no variable expansion)
				if word.starts_with("<<'") || word.starts_with("<<\"") {
					let quote_char = word.chars().nth(2).unwrap();
					let delimiter = word[3..].to_string();
					return Some((delimiter, false));
				}
				
				// Regular heredoc (with variable expansion)
				if word.len() > 2 {
					let delimiter = word[2..].to_string();
					return Some((delimiter, true));
				}
			}
		}
		
		None
	}
	
	/**
	 * Checks if current line matches heredoc delimiter
	 * 
	 * @param state - Current heredoc state
	 * @param line - Current line to check
	 * @return bool - True if line matches delimiter
	 */
	pub fn is_heredoc_delimiter(state: &HeredocState, line: &str) -> bool {
		if !state.heredoc_mode {
			return false;
		}
		
		let trimmed = line.trim();
		trimmed == state.heredoc_delimiter
	}
	
	/**
	 * Expands variables in heredoc content
	 * 
	 * @param content - Content to expand
	 * @return String - Content with variables expanded
	 */
	pub fn expand_heredoc_variables(content: &str) -> String {
		/**
		 * ヒアドキュメント変数展開の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な変数展開を行います。
		 * 環境変数の置換が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut result = String::new();
		let mut i = 0;
		
		while i < content.len() {
			if content[i..].starts_with('$') {
				// Found variable reference
				let var_start = i + 1;
				let mut var_end = var_start;
				
				// Find variable name
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
					
					// Get environment variable
					if let Ok(var_value) = std::env::var(var_name) {
						result.push_str(&var_value);
					} else {
						// Variable not found, keep original
						result.push_str(&content[i..var_end]);
					}
					
					i = var_end;
				} else {
					// Just a $, keep it
					result.push('$');
					i += 1;
				}
			} else {
				// Regular character
				result.push(content.chars().nth(i).unwrap());
				i += 1;
			}
		}
		
		result
	}
	
	/**
	 * Updates heredoc state based on input
	 * 
	 * @param state - Current heredoc state
	 * @param input - Input text to check
	 * @return HeredocState - Updated state
	 */
	pub fn update_heredoc_state(mut state: HeredocState, input: &str) -> HeredocState {
		// Check for heredoc syntax
		if let Some((delimiter, expand_vars)) = Self::detect_heredoc(input) {
			state.heredoc_mode = true;
			state.heredoc_delimiter = delimiter;
			state.heredoc_expand_vars = expand_vars;
		}
		
		state
	}
	
	/**
	 * Adds content to heredoc with variable expansion
	 * 
	 * @param state - Current heredoc state
	 * @param content - Content to add
	 * @return HeredocState - Updated state
	 */
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
	
	/**
	 * Gets the heredoc prompt for display
	 * 
	 * @param state - Heredoc state
	 * @return String - Prompt string
	 */
	pub fn get_prompt(state: &HeredocState) -> String {
		if state.heredoc_mode {
			"heredoc> ".to_string()
		} else {
			"$ ".to_string()
		}
	}
} 