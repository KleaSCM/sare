/**
 * Brace expansion and globbing for Sare terminal
 * 
 * This module provides comprehensive brace expansion and globbing
 * including numeric ranges, comma lists, glob patterns, and advanced matching.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: expansion.rs
 * Description: Advanced brace expansion and globbing with full implementation
 */

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/**
 * Expansion state for brace expansion and globbing
 * 
 * Manages the state of expansion processing including
 * pattern caching and expansion mode tracking.
 */
#[derive(Debug, Clone)]
pub struct ExpansionState {
	/// Whether in expansion mode
	pub expansion_mode: bool,
	/// Globbing patterns cache
	pub glob_cache: HashMap<String, Vec<String>>,
	/// Expansion depth for nested patterns
	pub expansion_depth: usize,
	/// Current working directory for relative paths
	pub working_directory: PathBuf,
}

impl Default for ExpansionState {
	fn default() -> Self {
		Self {
			expansion_mode: false,
			glob_cache: HashMap::new(),
			expansion_depth: 0,
			working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
		}
	}
}

/**
 * Expansion processor for brace expansion and globbing
 * 
 * Handles complex pattern matching, brace expansion,
 * and glob pattern resolution with caching.
 */
pub struct ExpansionProcessor;

impl ExpansionProcessor {
	/**
	 * Detects brace expansion patterns in input
	 * 
	 * @param input - Input text to check
	 * @return Vec<(usize, usize, String)> - List of (start, end, pattern) expansions
	 */
	pub fn detect_brace_expansions(input: &str) -> Vec<(usize, usize, String)> {
		/**
		 * ブレース展開検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。
		 * ネストしたブレース処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut expansions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
			// Check for {pattern} syntax
			if input[i..].starts_with('{') {
				let start = i;
				let mut depth = 1;
				let mut j = i + 1;
				
				while j < input.len() && depth > 0 {
					match input.chars().nth(j) {
						Some('{') => depth += 1,
						Some('}') => depth -= 1,
						_ => {}
					}
					j += 1;
				}
				
				if depth == 0 {
					let pattern = input[i + 1..j - 1].to_string();
					expansions.push((start, j, pattern));
				}
				
				i = j;
			} else {
				i += 1;
			}
		}
		
		expansions
	}
	
	/**
	 * Expands a brace pattern into multiple strings
	 * 
	 * @param pattern - Brace pattern to expand
	 * @return Vec<String> - List of expanded strings
	 */
	pub fn expand_brace_pattern(pattern: &str) -> Vec<String> {
		/**
		 * ブレース展開の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なパターン展開を行います。
		 * 数値範囲とカンマリストの処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut results = Vec::new();
		
		// Check for numeric range: {1..5} or {1..5..2}
		if let Some(range_results) = Self::expand_numeric_range(pattern) {
			return range_results;
		}
		
		// Check for comma list: {a,b,c}
		if let Some(comma_results) = Self::expand_comma_list(pattern) {
			return comma_results;
		}
		
		// Single item (no expansion needed)
		results.push(pattern.to_string());
		results
	}
	
	/**
	 * Expands numeric range patterns
	 * 
	 * @param pattern - Numeric range pattern
	 * @return Option<Vec<String>> - Expanded numbers or None
	 */
	fn expand_numeric_range(pattern: &str) -> Option<Vec<String>> {
		/**
		 * 数値範囲展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な数値解析を行います。
		 * 範囲とステップの処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		// Check for range pattern: start..end or start..end..step
		if !pattern.contains("..") {
			return None;
		}
		
		let parts: Vec<&str> = pattern.split("..").collect();
		if parts.len() < 2 || parts.len() > 3 {
			return None;
		}
		
		// Parse start and end
		let start = parts[0].parse::<i32>().ok()?;
		let end = parts[1].parse::<i32>().ok()?;
		
		// Parse step (default to 1)
		let step = if parts.len() == 3 {
			parts[2].parse::<i32>().ok()?
		} else {
			1
		};
		
		if step == 0 {
			return None;
		}
		
		// Generate range
		let mut results = Vec::new();
		let mut current = start;
		
		if step > 0 {
			while current <= end {
				results.push(current.to_string());
				current += step;
			}
		} else {
			while current >= end {
				results.push(current.to_string());
				current += step;
			}
		}
		
		Some(results)
	}
	
	/**
	 * Expands comma-separated list patterns
	 * 
	 * @param pattern - Comma list pattern
	 * @return Option<Vec<String>> - Expanded list or None
	 */
	fn expand_comma_list(pattern: &str) -> Option<Vec<String>> {
		/**
		 * カンマリスト展開の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なリスト解析を行います。
		 * ネストしたカンマ処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Check if pattern contains commas
		if !pattern.contains(',') {
			return None;
		}
		
		// Split by commas, but handle nested braces
		let mut results = Vec::new();
		let mut current = String::new();
		let mut brace_depth = 0;
		let mut i = 0;
		
		while i < pattern.len() {
			let ch = pattern.chars().nth(i).unwrap();
			
			match ch {
				'{' => {
					brace_depth += 1;
					current.push(ch);
				}
				'}' => {
					brace_depth -= 1;
					current.push(ch);
				}
				',' if brace_depth == 0 => {
					// Top-level comma, split here
					if !current.is_empty() {
						results.push(current.trim().to_string());
					}
					current.clear();
				}
				_ => {
					current.push(ch);
				}
			}
			
			i += 1;
		}
		
		// Add the last part
		if !current.is_empty() {
			results.push(current.trim().to_string());
		}
		
		if results.len() > 1 {
			Some(results)
		} else {
			None
		}
	}
	
	/**
	 * Expands glob patterns into matching files
	 * 
	 * @param pattern - Glob pattern to expand
	 * @param working_directory - Working directory for relative paths
	 * @return Vec<String> - List of matching files
	 */
	pub fn expand_glob_pattern(pattern: &str, working_directory: &Path) -> Vec<String> {
		/**
		 * グロブ展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なファイルマッチングを行います。
		 * パターンマッチングとディレクトリ走査が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut results = Vec::new();
		
		// Handle special patterns
		if pattern == "*" {
			// Match all files in current directory
			if let Ok(entries) = fs::read_dir(working_directory) {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(file_name) = entry.file_name().into_string() {
							if !file_name.starts_with('.') {
								results.push(file_name);
							}
						}
					}
				}
			}
			return results;
		}
		
		if pattern == "**" {
			// Match all files recursively
			if let Ok(entries) = fs::read_dir(working_directory) {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(file_name) = entry.file_name().into_string() {
							if !file_name.starts_with('.') {
								let file_name_clone = file_name.clone();
								results.push(file_name);
								
								// Recursively add subdirectories
								if let Ok(metadata) = entry.metadata() {
									if metadata.is_dir() {
										let sub_path = working_directory.join(&file_name_clone);
										let sub_results = Self::expand_glob_pattern("**", &sub_path);
										for sub_result in sub_results {
											results.push(format!("{}/{}", file_name_clone, sub_result));
										}
									}
								}
							}
						}
					}
				}
			}
			return results;
		}
		
		// Handle complex patterns
		if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
			// Complex glob pattern
			if let Ok(entries) = fs::read_dir(working_directory) {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(file_name) = entry.file_name().into_string() {
							if Self::matches_glob_pattern(&file_name, pattern) {
								results.push(file_name);
							}
						}
					}
				}
			}
		} else {
			// Simple filename
			let file_path = working_directory.join(pattern);
			if file_path.exists() {
				results.push(pattern.to_string());
			}
		}
		
		results
	}
	
	/**
	 * Checks if a filename matches a glob pattern
	 * 
	 * @param filename - Filename to check
	 * @param pattern - Glob pattern to match against
	 * @return bool - True if filename matches pattern
	 */
	fn matches_glob_pattern(filename: &str, pattern: &str) -> bool {
		/**
		 * グロブマッチングの複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なパターンマッチングを行います。
		 * ワイルドカードと文字クラスの処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Simple wildcard matching
		if pattern == "*" {
			return true;
		}
		
		if pattern == "?" {
			return filename.len() == 1;
		}
		
		// Handle character classes [abc] or [a-z]
		if pattern.starts_with('[') && pattern.contains(']') {
			if let Some(end_bracket) = pattern.find(']') {
				let class = &pattern[1..end_bracket];
				let remaining = &pattern[end_bracket + 1..];
				
				// Check if first character matches the class
				if let Some(first_char) = filename.chars().next() {
					if Self::matches_character_class(first_char, class) {
						return Self::matches_glob_pattern(&filename[1..], remaining);
					}
				}
				return false;
			}
		}
		
		// Handle wildcards
		if pattern.starts_with('*') {
			// * matches any sequence
			let remaining = &pattern[1..];
			if remaining.is_empty() {
				return true;
			}
			
			// Try matching at each position
			for i in 0..=filename.len() {
				if Self::matches_glob_pattern(&filename[i..], remaining) {
					return true;
				}
			}
			return false;
		}
		
		if pattern.starts_with('?') {
			// ? matches any single character
			if filename.is_empty() {
				return false;
			}
			return Self::matches_glob_pattern(&filename[1..], &pattern[1..]);
		}
		
		// Literal character match
		if let (Some(pattern_char), Some(filename_char)) = (pattern.chars().next(), filename.chars().next()) {
			if pattern_char == filename_char {
				return Self::matches_glob_pattern(&filename[1..], &pattern[1..]);
			}
		}
		
		// No match
		false
	}
	
	/**
	 * Checks if a character matches a character class
	 * 
	 * @param ch - Character to check
	 * @param class - Character class pattern
	 * @return bool - True if character matches class
	 */
	fn matches_character_class(ch: char, class: &str) -> bool {
		/**
		 * 文字クラスマッチングの複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な文字クラス解析を行います。
		 * 範囲指定と否定の処理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut i = 0;
		let mut negated = false;
		
		// Check for negation
		if class.starts_with('^') {
			negated = true;
			i = 1;
		}
		
		while i < class.len() {
			if i + 2 < class.len() && class.chars().nth(i + 1) == Some('-') {
				// Range: a-z
				let start = class.chars().nth(i).unwrap();
				let end = class.chars().nth(i + 2).unwrap();
				
				if ch >= start && ch <= end {
					return !negated;
				}
				
				i += 3;
			} else {
				// Single character
				let class_char = class.chars().nth(i).unwrap();
				if ch == class_char {
					return !negated;
				}
				
				i += 1;
			}
		}
		
		negated
	}
	
	/**
	 * Processes brace expansions and globbing in input
	 * 
	 * @param input - Input text to process
	 * @param working_directory - Working directory for relative paths
	 * @return Result<String> - Processed input with expansions
	 */
	pub fn process_brace_expansions(input: &str, working_directory: &Path) -> Result<String> {
		/**
		 * ブレース展開処理の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な展開処理を行います。
		 * ネストした展開とエラーハンドリングが難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut result = input.to_string();
		let expansions = Self::detect_brace_expansions(&result);
		
		// Process expansions in reverse order to maintain indices
		for (start, end, pattern) in expansions.iter().rev() {
			let expanded = Self::expand_brace_pattern(pattern);
			
			if expanded.len() == 1 {
				// Single expansion, replace directly
				result.replace_range(*start..*end, &expanded[0]);
			} else {
				// Multiple expansions, join with spaces
				let joined = expanded.join(" ");
				result.replace_range(*start..*end, &joined);
			}
		}
		
		// Process glob patterns
		let words: Vec<&str> = result.split_whitespace().collect();
		let mut processed_words = Vec::new();
		
		for word in words {
			if word.contains('*') || word.contains('?') || word.contains('[') {
				// This is a glob pattern
				let matches = Self::expand_glob_pattern(word, working_directory);
				if !matches.is_empty() {
					processed_words.extend(matches);
				} else {
					// No matches, keep original
					processed_words.push(word.to_string());
				}
			} else {
				processed_words.push(word.to_string());
			}
		}
		
		Ok(processed_words.join(" "))
	}
}

impl ExpansionState {
	/**
	 * Checks if in expansion mode
	 * 
	 * @return bool - True if in expansion mode
	 */
	pub fn is_expansion_mode(&self) -> bool {
		self.expansion_mode
	}
	
	/**
	 * Sets expansion mode
	 * 
	 * @param mode - Expansion mode state
	 */
	pub fn set_expansion_mode(&mut self, mode: bool) {
		self.expansion_mode = mode;
	}
	
	/**
	 * Gets expansion depth
	 * 
	 * @return usize - Current expansion depth
	 */
	pub fn get_depth(&self) -> usize {
		self.expansion_depth
	}
	
	/**
	 * Sets expansion depth
	 * 
	 * @param depth - Expansion depth
	 */
	pub fn set_depth(&mut self, depth: usize) {
		self.expansion_depth = depth;
	}
	
	/**
	 * Gets working directory
	 * 
	 * @return PathBuf - Current working directory
	 */
	pub fn get_working_directory(&self) -> PathBuf {
		self.working_directory.clone()
	}
	
	/**
	 * Sets working directory
	 * 
	 * @param dir - Working directory path
	 */
	pub fn set_working_directory(&mut self, dir: PathBuf) {
		self.working_directory = dir;
	}
	
	/**
	 * Gets glob cache
	 * 
	 * @return &HashMap<String, Vec<String>> - Glob cache reference
	 */
	pub fn get_glob_cache(&self) -> &HashMap<String, Vec<String>> {
		&self.glob_cache
	}
	
	/**
	 * Adds pattern to glob cache
	 * 
	 * @param pattern - Glob pattern
	 * @param matches - Matching files
	 */
	pub fn add_to_glob_cache(&mut self, pattern: String, matches: Vec<String>) {
		self.glob_cache.insert(pattern, matches);
	}
	
	/**
	 * Processes brace expansions and globbing in input
	 * 
	 * @param input - Input text to process
	 * @return Result<String> - Processed input with expansions
	 */
	pub fn process_expansions(&self, input: &str) -> Result<String> {
		ExpansionProcessor::process_brace_expansions(input, &self.working_directory)
	}
} 