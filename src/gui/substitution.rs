
/**
 * Command substitution module for Sare terminal
 * 
 * This module provides command substitution functionality including
 * $(command) and `command` syntax parsing and execution.
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
		 * コマンド置換検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。ネストした括弧処理が
		 * 難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 複数の置換構文とネストした括弧の深さ追跡の複雑なロジックです (◕‿◕)
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
		 * コマンド実行の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なコマンド実行を行います。プロセス生成と
		 * 出力処理が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 外部プロセス実行とストリーム処理の複雑なロジックです
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
		 * コマンド置換処理の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な置換処理を行います。逆順処理による
		 * インデックス維持が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 逆順処理によるインデックス維持とエラーハンドリングの複雑なロジックです
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
	pub fn is_substitution_mode(&self) -> bool {
		self.substitution_mode
	}
	
	pub fn set_substitution_mode(&mut self, mode: bool) {
		self.substitution_mode = mode;
	}
	
	pub fn get_depth(&self) -> usize {
		self.substitution_depth
	}
	
	pub fn set_depth(&mut self, depth: usize) {
		self.substitution_depth = depth;
	}
	
	pub fn get_buffer(&self) -> String {
		self.substitution_buffer.clone()
	}
	
	pub fn set_buffer(&mut self, buffer: String) {
		self.substitution_buffer = buffer;
	}
	
	pub fn process_substitutions(&self, input: &str) -> Result<String> {
		SubstitutionProcessor::process_command_substitutions(input)
	}
} 