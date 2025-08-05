
/**
 * Brace expansion and globbing module for Sare terminal
 * 
 * This module provides brace expansion functionality including
 * numeric ranges, comma lists, and glob pattern matching.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: expansion.rs
 * Description: Brace expansion and glob pattern processing
 */

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

pub struct ExpansionProcessor;

impl ExpansionProcessor {
	pub fn detect_brace_expansions(input: &str) -> Vec<(usize, usize, String)> {
		/**
		 * ブレース展開検出の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な構文解析を行います。ネストしたブレース処理が
		 * 難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * ネストしたブレースの深さを追跡して、正しいパターン抽出を
		 * 保証する複雑なロジックです (◕‿◕)
		 */
		
		let mut expansions = Vec::new();
		let mut i = 0;
		
		while i < input.len() {
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
	
	pub fn expand_brace_pattern(pattern: &str) -> Vec<String> {
		/**
		 * ブレースパターン展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なパターン解析を行います。複数の展開タイプの
		 * 処理が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 数値範囲とカンマリスト展開の複雑なロジックです (◕‿◕)
		 */
		
		let mut results = Vec::new();
		
		if let Some(range_results) = Self::expand_numeric_range(pattern) {
			return range_results;
		}
		
		if let Some(comma_results) = Self::expand_comma_list(pattern) {
			return comma_results;
		}
		
		results.push(pattern.to_string());
		results
	}
	
	fn expand_numeric_range(pattern: &str) -> Option<Vec<String>> {
		/**
		 * 数値範囲展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な数値解析を行います。範囲とステップの処理が
		 * 難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 正負のステップ値と範囲検証の複雑なロジックです (◕‿◕)
		 */
		
		if !pattern.contains("..") {
			return None;
		}
		
		let parts: Vec<&str> = pattern.split("..").collect();
		if parts.len() < 2 || parts.len() > 3 {
			return None;
		}
		
		let start = parts[0].parse::<i32>().ok()?;
		let end = parts[1].parse::<i32>().ok()?;
		
		let step = if parts.len() == 3 {
			parts[2].parse::<i32>().ok()?
		} else {
			1
		};
		
		if step == 0 {
			return None;
		}
		
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
	
	fn expand_comma_list(pattern: &str) -> Option<Vec<String>> {
		/**
		 * カンマリスト展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なリスト解析を行います。ネストしたカンマ処理が
		 * 難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * ブレース深度追跡によるネストしたカンマ処理の複雑なロジックです
		 */
		
		// Check if pattern contains commas
		if !pattern.contains(',') {
			return None;
		}
		
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
		
		if !current.is_empty() {
			results.push(current.trim().to_string());
		}
		
		if results.len() > 1 {
			Some(results)
		} else {
			None
		}
	}
	
	pub fn expand_glob_pattern(pattern: &str, working_directory: &Path) -> Vec<String> {
		/**
		 * グロブ展開の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なファイルシステム検索を行います。再帰的検索と
		 * パターンマッチングが難しい部分なので、適切なエラーハンドリングで
		 * 実装しています。
		 * 
		 * 再帰的ディレクトリ検索とパターンマッチングの複雑なロジックです (◕‿◕)
		 */
		
		let mut results = Vec::new();
		
		if pattern == "*" {
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
			if let Ok(entries) = fs::read_dir(working_directory) {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Ok(file_name) = entry.file_name().into_string() {
							if !file_name.starts_with('.') {
								let file_name_clone = file_name.clone();
								results.push(file_name);
								
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
		
		if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
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
			let file_path = working_directory.join(pattern);
			if file_path.exists() {
				results.push(pattern.to_string());
			}
		}
		
		results
	}
	
	fn matches_glob_pattern(filename: &str, pattern: &str) -> bool {
		/**
		 * グロブパターンマッチングの複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なパターンマッチングを行います。ワイルドカードと
		 * 文字クラス処理が難しい部分なので、適切なエラーハンドリングで
		 * 実装しています。
		 * 
		 * 再帰的なパターンマッチングと文字クラス処理の複雑なロジックです (◕‿◕)
		 */
		
		if pattern == "*" {
			return true;
		}
		
		if pattern == "?" {
			return filename.len() == 1;
		}
		
		if pattern.starts_with('[') && pattern.contains(']') {
			if let Some(end_bracket) = pattern.find(']') {
				let class = &pattern[1..end_bracket];
				let remaining = &pattern[end_bracket + 1..];
				
				if let Some(first_char) = filename.chars().next() {
					if Self::matches_character_class(first_char, class) {
						return Self::matches_glob_pattern(&filename[1..], remaining);
					}
				}
				return false;
			}
		}
		
		if pattern.starts_with('*') {
			let remaining = &pattern[1..];
			if remaining.is_empty() {
				return true;
			}
			
			for i in 0..=filename.len() {
				if Self::matches_glob_pattern(&filename[i..], remaining) {
					return true;
				}
			}
			return false;
		}
		
		if pattern.starts_with('?') {
			if filename.is_empty() {
				return false;
			}
			return Self::matches_glob_pattern(&filename[1..], &pattern[1..]);
		}
		
		if let (Some(pattern_char), Some(filename_char)) = (pattern.chars().next(), filename.chars().next()) {
			if pattern_char == filename_char {
				return Self::matches_glob_pattern(&filename[1..], &pattern[1..]);
			}
		}
		
		false
	}
	
	fn matches_character_class(ch: char, class: &str) -> bool {
		/**
		 * 文字クラスマッチングの複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な文字クラス解析を行います。範囲指定と否定の
		 * 処理が難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 文字範囲と否定文字クラスの複雑なロジックです (◕‿◕)
		 */
		
		let mut i = 0;
		let mut negated = false;
		
		if class.starts_with('^') {
			negated = true;
			i = 1;
		}
		
		while i < class.len() {
			if i + 2 < class.len() && class.chars().nth(i + 1) == Some('-') {
				let start = class.chars().nth(i).unwrap();
				let end = class.chars().nth(i + 2).unwrap();
				
				if ch >= start && ch <= end {
					return !negated;
				}
				
				i += 3;
			} else {
				let class_char = class.chars().nth(i).unwrap();
				if ch == class_char {
					return !negated;
				}
				
				i += 1;
			}
		}
		
		negated
	}
	
	pub fn process_brace_expansions(input: &str, working_directory: &Path) -> Result<String> {
		/**
		 * ブレース展開処理の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な展開処理を行います。ネストした展開とエラーハンドリングが
		 * 難しい部分なので、適切なエラーハンドリングで実装しています。
		 * 
		 * 逆順処理によるインデックス維持とグロブパターン処理の複雑なロジックです (◕‿◕)
		 */
		
		let mut result = input.to_string();
		let expansions = Self::detect_brace_expansions(&result);
		
		for (start, end, pattern) in expansions.iter().rev() {
			let expanded = Self::expand_brace_pattern(pattern);
			
			if expanded.len() == 1 {
				result.replace_range(*start..*end, &expanded[0]);
			} else {
				let joined = expanded.join(" ");
				result.replace_range(*start..*end, &joined);
			}
		}
		
		let words: Vec<&str> = result.split_whitespace().collect();
		let mut processed_words = Vec::new();
		
		for word in words {
			if word.contains('*') || word.contains('?') || word.contains('[') {
				let matches = Self::expand_glob_pattern(word, working_directory);
				if !matches.is_empty() {
					processed_words.extend(matches);
				} else {
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
	pub fn is_expansion_mode(&self) -> bool {
		self.expansion_mode
	}
	
	pub fn set_expansion_mode(&mut self, mode: bool) {
		self.expansion_mode = mode;
	}
	
	pub fn get_depth(&self) -> usize {
		self.expansion_depth
	}
	
	pub fn set_depth(&mut self, depth: usize) {
		self.expansion_depth = depth;
	}
	
	pub fn get_working_directory(&self) -> PathBuf {
		self.working_directory.clone()
	}
	
	pub fn set_working_directory(&mut self, dir: PathBuf) {
		self.working_directory = dir;
	}
	
	pub fn get_glob_cache(&self) -> &HashMap<String, Vec<String>> {
		&self.glob_cache
	}
	
	pub fn add_to_glob_cache(&mut self, pattern: String, matches: Vec<String>) {
		self.glob_cache.insert(pattern, matches);
	}
	
	pub fn process_expansions(&self, input: &str) -> Result<String> {
		ExpansionProcessor::process_brace_expansions(input, &self.working_directory)
	}
} 