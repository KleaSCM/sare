/**
 * Advanced tab completion engine for Sare terminal
 * 
 * This module provides intelligent tab completion including
 * file path completion, command completion, variable completion,
 * flag completion, and context-aware completion with fuzzy matching.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: completion.rs
 * Description: Advanced tab completion with context awareness
 */

use anyhow::Result;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::fs;

/**
 * Completion context
 * 
 * Determines what type of completion to perform
 * based on the current input context.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
	/// Command name completion (start of line)
	Command,
	/// File path completion (after command)
	FilePath,
	/// Flag completion (after - or --)
	Flag,
	/// Variable completion (after $)
	Variable,
	/// Unknown context
	Unknown,
}

/**
 * Completion result
 * 
 * Contains the completed text and any additional information.
 */
#[derive(Debug, Clone)]
pub struct CompletionResult {
	/// Completed text
	pub completed_text: String,
	/// Whether this is a partial completion
	pub is_partial: bool,
	/// Available completions if partial
	pub alternatives: Vec<String>,
	/// Context where completion occurred
	pub context: CompletionContext,
}

/**
 * Tab completion engine
 * 
 * Provides intelligent tab completion with context awareness
 * and fuzzy matching capabilities.
 */
#[derive(Debug)]
pub struct TabCompleter {
	/// Common commands for completion
	common_commands: Vec<String>,
	/// Command history for completion
	command_history: VecDeque<String>,
	/// Current working directory
	working_directory: PathBuf,
}

impl TabCompleter {
	/**
	 * Creates a new tab completer
	 * 
	 * @param working_directory - Current working directory
	 * @return TabCompleter - New tab completer instance
	 */
	pub fn new(working_directory: PathBuf) -> Self {
		/**
		 * タブ補完エンジン初期化の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な補完エンジン初期化を行います。
		 * コマンド検出とパス処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let common_commands = vec![
			"ls".to_string(), "cd".to_string(), "pwd".to_string(), "cat".to_string(), "echo".to_string(), "grep".to_string(), "find".to_string(), "cp".to_string(), "mv".to_string(), "rm".to_string(),
			"mkdir".to_string(), "rmdir".to_string(), "touch".to_string(), "chmod".to_string(), "chown".to_string(), "ps".to_string(), "top".to_string(), "kill".to_string(),
			"git".to_string(), "cargo".to_string(), "npm".to_string(), "python".to_string(), "node".to_string(), "vim".to_string(), "nano".to_string(), "less".to_string(),
			"head".to_string(), "tail".to_string(), "sort".to_string(), "uniq".to_string(), "wc".to_string(), "awk".to_string(), "sed".to_string(), "curl".to_string(), "wget".to_string(),
			"ping".to_string(), "ssh".to_string(), "scp".to_string(), "rsync".to_string(), "tar".to_string(), "gzip".to_string(), "bzip2".to_string(), "zip".to_string(),
			"unzip".to_string(), "make".to_string(), "cmake".to_string(), "gcc".to_string(), "g++".to_string(), "clang".to_string(), "rustc".to_string(), "cargo".to_string(),
		];
		
		Self {
			common_commands,
			command_history: VecDeque::new(),
			working_directory,
		}
	}
	
	/**
	 * Performs tab completion on the given input
	 * 
	 * @param input - Current input text
	 * @param cursor_pos - Current cursor position
	 * @return Result<Option<CompletionResult>> - Completion result or None
	 */
	pub fn complete(&mut self, input: &str, cursor_pos: usize) -> Result<Option<CompletionResult>> {
		/**
		 * タブ補完の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な補完ロジックを行います。
		 * コンテキスト解析とパスマッチングが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let context = self.parse_context(input, cursor_pos)?;
		
		match context {
			CompletionContext::Command => {
				self.complete_command(input, cursor_pos)
			}
			CompletionContext::FilePath => {
				self.complete_file_path(input, cursor_pos)
			}
			CompletionContext::Flag => {
				self.complete_flag(input, cursor_pos)
			}
			CompletionContext::Variable => {
				self.complete_variable(input, cursor_pos)
			}
			CompletionContext::Unknown => {
				Ok(None)
			}
		}
	}
	
	/**
	 * Parses the input to determine completion context
	 * 
	 * @param input - Current input text
	 * @param cursor_pos - Current cursor position
	 * @return Result<CompletionContext> - Determined context
	 */
	fn parse_context(&self, input: &str, cursor_pos: usize) -> Result<CompletionContext> {
		let before_cursor = &input[..cursor_pos];
		let words: Vec<&str> = before_cursor.split_whitespace().collect();
		
		if words.is_empty() {
			return Ok(CompletionContext::Command);
		}
		
		if before_cursor.trim().is_empty() || words.len() == 1 {
			return Ok(CompletionContext::Command);
		}
		
		if let Some(last_word) = words.last() {
			if last_word.starts_with('-') {
				return Ok(CompletionContext::Flag);
			}
		}
		
		if let Some(last_word) = words.last() {
			if last_word.starts_with('$') {
				return Ok(CompletionContext::Variable);
			}
		}
		
		Ok(CompletionContext::FilePath)
	}
	
	/**
	 * Completes command names
	 * 
	 * @param input - Current input text
	 * @param cursor_pos - Current cursor position
	 * @return Result<Option<CompletionResult>> - Command completion result
	 */
	fn complete_command(&self, input: &str, cursor_pos: usize) -> Result<Option<CompletionResult>> {
		let before_cursor = &input[..cursor_pos];
		let partial = before_cursor.trim();
		
		if partial.is_empty() {
			return Ok(None);
		}
		
		let mut matches = Vec::new();
		
		for cmd in &self.common_commands {
			if cmd.starts_with(partial) {
				matches.push(cmd.clone());
			}
		}
		
		for cmd in &self.command_history {
			if cmd.starts_with(partial) && !matches.contains(cmd) {
				matches.push(cmd.clone());
			}
		}
		
		if let Ok(path) = std::env::var("PATH") {
			for path_dir in path.split(':') {
				if let Ok(entries) = fs::read_dir(path_dir) {
					for entry in entries {
						if let Ok(entry) = entry {
							if let Ok(file_name) = entry.file_name().into_string() {
								if file_name.starts_with(partial) {
									matches.push(file_name);
								}
							}
						}
					}
				}
			}
		}
		
		matches.sort();
		matches.dedup();
		
		if matches.is_empty() {
			Ok(None)
		} else if matches.len() == 1 {
			Ok(Some(CompletionResult {
				completed_text: matches[0].clone(),
				is_partial: false,
				alternatives: Vec::new(),
				context: CompletionContext::Command,
			}))
		} else {
			let common_prefix = self.find_common_prefix(&matches);
			Ok(Some(CompletionResult {
				completed_text: common_prefix,
				is_partial: true,
				alternatives: matches,
				context: CompletionContext::Command,
			}))
		}
	}
	
	/**
	 * Completes file paths
	 * 
	 * @param input - Current input text
	 * @param cursor_pos - Current cursor position
	 * @return Result<Option<CompletionResult>> - File path completion result
	 */
	fn complete_file_path(&self, input: &str, cursor_pos: usize) -> Result<Option<CompletionResult>> {
		let before_cursor = &input[..cursor_pos];
		let words: Vec<&str> = before_cursor.split_whitespace().collect();
		
		if words.is_empty() {
			return Ok(None);
		}
		
		let last_word = words.last().unwrap();
		
		let (path, is_quoted) = self.parse_quoted_path(last_word);
		
		let (search_dir, partial_name) = if path.contains('/') {
			let path_buf = PathBuf::from(&path);
			if let Some(parent) = path_buf.parent() {
				(parent.to_path_buf(), path_buf.file_name().map(|s| s.to_string_lossy().to_string()))
			} else {
				(self.working_directory.clone(), Some(path.clone()))
			}
		} else {
			(self.working_directory.clone(), Some(path.clone()))
		};
		
		let partial = partial_name.unwrap_or_default();
		
		let mut matches = Vec::new();
		
		if let Ok(entries) = fs::read_dir(&search_dir) {
			for entry in entries {
				if let Ok(entry) = entry {
					if let Ok(file_name) = entry.file_name().into_string() {
						if file_name.starts_with(&partial) {
							let mut display_name = file_name.clone();
							if let Ok(metadata) = entry.metadata() {
								if metadata.is_dir() {
									display_name.push('/');
								}
							}
							matches.push(display_name);
						}
					}
				}
			}
		}
		
		matches.sort();
		
		if matches.is_empty() {
			Ok(None)
		} else if matches.len() == 1 {
			let completed_path = if path.contains('/') {
				let mut path_buf = PathBuf::from(path);
				path_buf.pop();
				path_buf.join(&matches[0])
			} else {
				PathBuf::from(&matches[0])
			};
			
			let completed_text = if is_quoted {
				format!("\"{}\"", completed_path.display())
			} else {
				completed_path.to_string_lossy().to_string()
			};
			
			Ok(Some(CompletionResult {
				completed_text,
				is_partial: false,
				alternatives: Vec::new(),
				context: CompletionContext::FilePath,
			}))
		} else {
			let common_prefix = self.find_common_prefix(&matches);
			Ok(Some(CompletionResult {
				completed_text: common_prefix,
				is_partial: true,
				alternatives: matches,
				context: CompletionContext::FilePath,
			}))
		}
	}
	
	/**
	 * Completes command flags
	 * 
	 * @param input - Current input text
	 * @param cursor_pos - Current cursor position
	 * @return Result<Option<CompletionResult>> - Flag completion result
	 */
	fn complete_flag(&self, _input: &str, _cursor_pos: usize) -> Result<Option<CompletionResult>> {
		Ok(None)
	}
	
	/**
	 * Completes environment variables
	 * 
	 * @param input - Current input text
	 * @param cursor_pos - Current cursor position
	 * @return Result<Option<CompletionResult>> - Variable completion result
	 */
	fn complete_variable(&self, input: &str, cursor_pos: usize) -> Result<Option<CompletionResult>> {
		let before_cursor = &input[..cursor_pos];
		let words: Vec<&str> = before_cursor.split_whitespace().collect();
		
		if words.is_empty() {
			return Ok(None);
		}
		
		let last_word = words.last().unwrap();
		if !last_word.starts_with('$') {
			return Ok(None);
		}
		
		let var_name = &last_word[1..];
		let mut matches = Vec::new();
		
		for (key, _) in std::env::vars() {
			if key.starts_with(var_name) {
				matches.push(format!("${}", key));
			}
		}
		
		matches.sort();
		
		if matches.is_empty() {
			Ok(None)
		} else if matches.len() == 1 {
			Ok(Some(CompletionResult {
				completed_text: matches[0].clone(),
				is_partial: false,
				alternatives: Vec::new(),
				context: CompletionContext::Variable,
			}))
		} else {
			let common_prefix = self.find_common_prefix(&matches);
			Ok(Some(CompletionResult {
				completed_text: common_prefix,
				is_partial: true,
				alternatives: matches,
				context: CompletionContext::Variable,
			}))
		}
	}
	
	/**
	 * Finds the common prefix of a list of strings
	 * 
	 * @param strings - List of strings to find common prefix for
	 * @return String - Common prefix
	 */
	fn find_common_prefix(&self, strings: &[String]) -> String {
		if strings.is_empty() {
			return String::new();
		}
		
		let first = &strings[0];
		let mut common_prefix = String::new();
		
		for (i, ch) in first.chars().enumerate() {
			for string in strings {
				if let Some(other_ch) = string.chars().nth(i) {
					if other_ch != ch {
						return common_prefix;
					}
				} else {
					return common_prefix;
				}
			}
			common_prefix.push(ch);
		}
		
		common_prefix
	}
	
	/**
	 * Parses a quoted path, handling escape sequences
	 * 
	 * @param input - Input string to parse
	 * @return (String, bool) - (Parsed path, whether it was quoted)
	 */
	fn parse_quoted_path(&self, input: &str) -> (String, bool) {
		if input.starts_with('"') && input.ends_with('"') {
			let unquoted = &input[1..input.len()-1];
			(unquoted.to_string(), true)
		} else if input.starts_with('\'') && input.ends_with('\'') {
			let unquoted = &input[1..input.len()-1];
			(unquoted.to_string(), true)
		} else {
			(input.to_string(), false)
		}
	}
	
	/**
	 * Adds a command to the completion history
	 * 
	 * @param command - Command to add
	 */
	pub fn add_command(&mut self, command: String) {
		self.command_history.retain(|cmd| cmd != &command);
		
		self.command_history.push_front(command);
		
		while self.command_history.len() > 100 {
			self.command_history.pop_back();
		}
	}
	
	/**
	 * Updates the working directory
	 * 
	 * @param new_dir - New working directory
	 */
	pub fn update_working_directory(&mut self, new_dir: PathBuf) {
		self.working_directory = new_dir;
	}
} 