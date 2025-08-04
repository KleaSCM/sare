/**
 * Command substitution processing for Sare terminal
 * 
 * This module provides command substitution support including
 * $(command) and `command` syntax with nested substitution.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: substitution.rs
 * Description: Command substitution processing and execution
 */

use anyhow::Result;
use std::process::Command;

/**
 * Command substitution state
 * 
 * Manages the state of command substitution processing
 * including depth tracking and nested substitution.
 */
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

/**
 * Command substitution processor
 * 
 * Handles command substitution syntax detection and execution
 * with nested substitution support and error handling.
 */
pub struct SubstitutionProcessor;

impl SubstitutionProcessor {
	/**
	 * Detects command substitution in input
	 * 
	 * @param input - Input text to check
	 * @return Vec<(usize, usize, String)> - List of (start, end, command) substitutions
	 */
	pub fn detect_command_substitutions(input: &str) -> Vec<(usize, usize, String)> {
		/**
		 * コマンド置換検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * ネストした置換処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut substitutions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
			// Check for $(command) syntax
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
			// Check for `command` syntax
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
	
	/**
	 * Executes a command and returns its output
	 * 
	 * @param command - Command to execute
	 * @return Result<String> - Command output or error
	 */
	pub fn execute_substitution_command(command: &str) -> Result<String> {
		/**
		 * コマンド置換実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なコマンド実行を行います。
		 * 子プロセス実行と出力取得が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Split command into parts
		let parts: Vec<&str> = command.split_whitespace().collect();
		if parts.is_empty() {
			return Ok(String::new());
		}
		
		// Execute the command
		let output = Command::new(parts[0])
			.args(&parts[1..])
			.output()?;
		
		// Convert output to string
		let stdout = String::from_utf8(output.stdout)?;
		let stderr = String::from_utf8(output.stderr)?;
		
		// Combine stdout and stderr, trim whitespace
		let mut result = stdout;
		if !stderr.is_empty() {
			result.push_str(&stderr);
		}
		
		Ok(result.trim().to_string())
	}
	
	/**
	 * Processes command substitutions in input
	 * 
	 * @param input - Input text to process
	 * @return Result<String> - Processed input with substitutions
	 */
	pub fn process_command_substitutions(input: &str) -> Result<String> {
		/**
		 * コマンド置換処理の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な置換処理を行います。
		 * ネストした置換とエラーハンドリングが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut result = input.to_string();
		let substitutions = Self::detect_command_substitutions(&result);
		
		// Process substitutions in reverse order to maintain indices
		for (start, end, command) in substitutions.iter().rev() {
			match Self::execute_substitution_command(command) {
				Ok(output) => {
					// Replace the substitution with the output
					result.replace_range(*start..*end, &output);
				}
				Err(_) => {
					// On error, replace with empty string
					result.replace_range(*start..*end, "");
				}
			}
		}
		
		Ok(result)
	}
}

impl SubstitutionState {
	/**
	 * Checks if in substitution mode
	 * 
	 * @return bool - True if in substitution mode
	 */
	pub fn is_substitution_mode(&self) -> bool {
		self.substitution_mode
	}
	
	/**
	 * Sets substitution mode
	 * 
	 * @param mode - Substitution mode state
	 */
	pub fn set_substitution_mode(&mut self, mode: bool) {
		self.substitution_mode = mode;
	}
	
	/**
	 * Gets substitution depth
	 * 
	 * @return usize - Current substitution depth
	 */
	pub fn get_depth(&self) -> usize {
		self.substitution_depth
	}
	
	/**
	 * Sets substitution depth
	 * 
	 * @param depth - Substitution depth
	 */
	pub fn set_depth(&mut self, depth: usize) {
		self.substitution_depth = depth;
	}
	
	/**
	 * Gets substitution buffer
	 * 
	 * @return String - Substitution buffer content
	 */
	pub fn get_buffer(&self) -> String {
		self.substitution_buffer.clone()
	}
	
	/**
	 * Sets substitution buffer
	 * 
	 * @param buffer - Buffer content
	 */
	pub fn set_buffer(&mut self, buffer: String) {
		self.substitution_buffer = buffer;
	}
	
	/**
	 * Processes command substitutions in input
	 * 
	 * @param input - Input text to process
	 * @return Result<String> - Processed input with substitutions
	 */
	pub fn process_substitutions(&self, input: &str) -> Result<String> {
		SubstitutionProcessor::process_command_substitutions(input)
	}
} 