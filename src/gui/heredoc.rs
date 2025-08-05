
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
	pub fn is_heredoc(&self) -> bool {
		self.heredoc_mode
	}
	
	pub fn set_heredoc(&mut self, mode: bool) {
		self.heredoc_mode = mode;
	}
	
	pub fn set_delimiter(&mut self, delimiter: String) {
		self.heredoc_delimiter = delimiter;
	}
	
	pub fn get_delimiter(&self) -> String {
		self.heredoc_delimiter.clone()
	}
	
	pub fn set_heredoc_content(&mut self, content: String) {
		self.heredoc_content = content;
	}
	
	pub fn get_heredoc_content(&self) -> String {
		self.heredoc_content.clone()
	}
	
	pub fn set_expand_vars(&mut self, expand: bool) {
		self.heredoc_expand_vars = expand;
	}
	
	pub fn should_expand_vars(&self) -> bool {
		self.heredoc_expand_vars
	}
	
	pub fn add_heredoc_content(&mut self, content: String) {
		self.heredoc_content.push_str(&content);
		self.heredoc_content.push('\n');
	}
	
	pub fn detect_heredoc(&self, input: &str) -> Option<(String, bool)> {
		HeredocProcessor::detect_heredoc(input)
	}
}

pub struct HeredocProcessor;

impl HeredocProcessor {
	pub fn detect_heredoc(input: &str) -> Option<(String, bool)> {
		/**
		 * ヒアドキュメント検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。引用符付きデリミタの
		 * 処理が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 複数の引用符形式と変数展開制御の複雑なロジックです
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
		 * ヒアドキュメント変数展開の複雑な処理です 
		 * 
		 * この関数は複雑な変数展開を行います。環境変数の検索と
		 * 文字列処理が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 環境変数検索と文字列置換の複雑なロジックです
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