
/**
 * Command substitution processing module for Sare terminal
 * 
 * This module provides command substitution functionality including
 * $(command) and `command` syntax parsing, execution, and result
 * integration for dynamic command output embedding.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: substitution.rs
 * Description: Command substitution processing and execution
 */

use anyhow::Result;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SubstitutionState {
	/// Whether in substitution mode
	pub substitution_mode: bool,
	/// Current substitution depth
	pub substitution_depth: usize,
	/// Substitution buffer for nested commands
	pub substitution_buffer: String,
}

impl Default for SubstitutionState {
	fn default() -> Self {
		Self {
			substitution_mode: false,
			substitution_depth: 0,
			substitution_buffer: String::new(),
		}
	}
}

pub struct SubstitutionProcessor;

impl SubstitutionProcessor {
	pub fn detect_command_substitutions(input: &str) -> Vec<(usize, usize, String)> {
		/**
		 * コマンド置換パターンを検出する関数です
		 * 
		 * $(command) と `command` 形式のコマンド置換を見つけて、
		 * 開始位置、終了位置、コマンド内容を返します。
		 * 
		 * ネストした括弧も正しく処理して、深さを追跡しながら
		 * 各置換パターンの位置を正確に特定します
		 */
		
		let mut substitutions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
			if input[i..].starts_with("$(") {
				let start = i;
				let mut depth = 1;
				let mut j = i + 2;
				
				while j < input.len() && depth > 0 {
					match input.chars().nth(j) {
						Some('(') => depth += 1,
						Some(')') => depth -= 1,
						_ => {}
					}
					j += 1;
				}
				
				if depth == 0 {
					let command = input[i + 2..j - 1].to_string();
					substitutions.push((start, j, command));
				}
				
				i = j;
			}
			else if input[i..].starts_with('`') {
				let start = i;
				let mut j = i + 1;
				
				while j < input.len() {
					if input.chars().nth(j) == Some('`') {
						break;
					}
					j += 1;
				}
				
				if j < input.len() {
					let command = input[i + 1..j].to_string();
					substitutions.push((start, j + 1, command));
				}
				
				i = j + 1;
			} else {
				i += 1;
			}
		}
		
		substitutions
	}
	
	pub fn execute_substitution_command(command: &str) -> Result<String> {
		/**
		 * コマンドを実行して結果を返す関数です
		 * 
		 * 指定されたコマンドを外部プロセスとして実行し、
		 * 標準出力と標準エラー出力を取得して返します。
		 * 
		 * コマンドの引数を適切に分割して、プロセスを生成し
		 * 実行結果を文字列として返します
		 */
		
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(String::new());
		}
		
		let output = Command::new(parts[0])
			.args(&parts[1..])
			.output()?;
		
		let stdout = String::from_utf8(output.stdout)?;
		let stderr = String::from_utf8(output.stderr)?;
		
		let mut result = stdout;
		if !stderr.is_empty() {
			result.push_str(&stderr);
		}
		
		Ok(result.trim().to_string())
	}
	
	pub fn process_command_substitutions(input: &str) -> Result<String> {
		/**
		 * コマンド置換を処理する関数です
		 * 
		 * 入力文字列内の $(command) と `command` 形式の置換を検出して、
		 * 各コマンドを実行して結果に置き換えます。
		 * 
		 * 逆順で置換を処理することで文字列のインデックスを維持し、
		 * 最終的に置換された文字列を返します
		 */
		
		let mut result = input.to_string();
		let substitutions = Self::detect_command_substitutions(&result);
		
		for (start, end, command) in substitutions.iter().rev() {
			match Self::execute_substitution_command(command) {
				Ok(output) => {
					result.replace_range(*start..*end, &output);
				}
				Err(_) => {
					result.replace_range(*start..*end, "");
				}
			}
		}
		
		Ok(result)
	}
}

impl SubstitutionState {
	/**
	 * Checks if substitution mode is currently active
	 * 
	 * @return bool - True if substitution mode is enabled
	 */
	pub fn is_substitution_mode(&self) -> bool {
		self.substitution_mode
	}
	
	/**
	 * Sets the substitution mode state
	 * 
	 * @param mode - Whether to enable or disable substitution mode
	 */
	pub fn set_substitution_mode(&mut self, mode: bool) {
		self.substitution_mode = mode;
	}
	
	/**
	 * Gets the current substitution depth for nested commands
	 * 
	 * @return usize - Current substitution depth
	 */
	pub fn get_depth(&self) -> usize {
		self.substitution_depth
	}
	
	/**
	 * Sets the substitution depth for nested command processing
	 * 
	 * @param depth - New substitution depth value
	 */
	pub fn set_depth(&mut self, depth: usize) {
		self.substitution_depth = depth;
	}
	
	/**
	 * Gets the substitution buffer for nested commands
	 * 
	 * @return String - Current substitution buffer
	 */
	pub fn get_buffer(&self) -> String {
		self.substitution_buffer.clone()
	}
	
	/**
	 * Sets the substitution buffer for nested command processing
	 * 
	 * @param buffer - New substitution buffer
	 */
	pub fn set_buffer(&mut self, buffer: String) {
		self.substitution_buffer = buffer;
	}
	
	/**
	 * Processes command substitutions using current state
	 * 
	 * @param input - Input string to process
	 * @return Result<String> - Processed string or error
	 */
	pub fn process_substitutions(&self, input: &str) -> Result<String> {
		SubstitutionProcessor::process_command_substitutions(input)
	}
} 