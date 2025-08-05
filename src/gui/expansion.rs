
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
		 * ブレース展開パターンを検出する関数です
		 * 
		 * 入力文字列から {pattern} 形式のブレース展開を見つけて、
		 * 開始位置、終了位置、パターン内容を返します。
		 * 
		 * ネストしたブレースも正しく処理して、深さを追跡しながら
		 * 各展開パターンの位置を正確に特定します 
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
		 * ブレースパターンを展開する関数です
		 * 
		 * 数値範囲 {1..5}、カンマリスト {a,b,c}、単一パターンの
		 * 3種類の展開タイプを処理します。
		 * 
		 * 数値範囲は指定された範囲の数字を生成し、カンマリストは
		 * 各要素を個別に展開して結果を返します 
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
		 * 数値範囲を展開する関数です
		 * 
		 * {1..5} や {1..10..2} 形式の数値範囲を解析して、
		 * 指定された範囲の数字を順番に生成します。
		 * 
		 * 開始値、終了値、オプションのステップ値を処理して、
		 * 正の方向と負の方向の両方の範囲をサポートします 
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
		 * カンマリストを展開する関数です (｡◕‿◕｡)
		 * 
		 * {a,b,c} 形式のカンマ区切りリストを解析して、
		 * 各要素を個別の文字列として展開します。
		 * 
		 * ネストしたブレース内のカンマは無視して、トップレベルの
		 * カンマのみで分割するため、深さを追跡します 
		 */
		
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
		 * グロブパターンを展開する関数です (｡◕‿◕｡)
		 * 
		 * ワイルドカード (*, ?, **) や文字クラス ([abc]) を含む
		 * ファイルパターンを解析して、マッチするファイル名を返します。
		 * 
		 * 現在のディレクトリ内のファイルを検索して、パターンに
		 * 一致するファイル名のリストを生成します (◕‿◕)
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
		 * グロブパターンとファイル名をマッチングする関数です
		 * 
		 * ワイルドカード (*, ?) や文字クラス ([abc]) を含むパターンと
		 * ファイル名が一致するかを判定します。
		 * 
		 * 再帰的にパターンを処理して、文字クラス、ワイルドカード、
		 * リテラル文字の各要素を順番にマッチングします
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
		 * 文字クラスと文字をマッチングする関数です
		 * 
		 * [abc] や [a-z] や [^abc] 形式の文字クラスを解析して、
		 * 指定された文字がクラスに含まれるかを判定します。
		 * 
		 * 文字範囲 (a-z)、個別文字、否定文字クラス (^) を
		 * 順番に処理してマッチング結果を返します 
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
		 * ブレース展開とグロブ展開を処理する関数です
		 * 
		 * 入力文字列内の {pattern} 形式のブレース展開を検出して展開し、
		 * その後ワイルドカードを含む単語をグロブ展開で処理します。
		 * 
		 * 逆順で展開を処理することで文字列のインデックスを維持し、
		 * 最終的に展開された文字列を返します 
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
	/**
	 * Checks if expansion mode is currently active
	 * 
	 * @return bool - True if expansion mode is enabled
	 */
	pub fn is_expansion_mode(&self) -> bool {
		self.expansion_mode
	}
	
	/**
	 * Sets the expansion mode state
	 * 
	 * @param mode - Whether to enable or disable expansion mode
	 */
	pub fn set_expansion_mode(&mut self, mode: bool) {
		self.expansion_mode = mode;
	}
	
	/**
	 * Gets the current expansion depth for nested patterns
	 * 
	 * @return usize - Current expansion depth
	 */
	pub fn get_depth(&self) -> usize {
		self.expansion_depth
	}
	
	/**
	 * Sets the expansion depth for nested pattern processing
	 * 
	 * @param depth - New expansion depth value
	 */
	pub fn set_depth(&mut self, depth: usize) {
		self.expansion_depth = depth;
	}
	
	/**
	 * Gets the current working directory for relative path resolution
	 * 
	 * @return PathBuf - Current working directory path
	 */
	pub fn get_working_directory(&self) -> PathBuf {
		self.working_directory.clone()
	}
	
	/**
	 * Sets the working directory for relative path resolution
	 * 
	 * @param dir - New working directory path
	 */
	pub fn set_working_directory(&mut self, dir: PathBuf) {
		self.working_directory = dir;
	}
	
	/**
	 * Gets the glob pattern cache for performance optimization
	 * 
	 * @return &HashMap<String, Vec<String>> - Reference to glob cache
	 */
	pub fn get_glob_cache(&self) -> &HashMap<String, Vec<String>> {
		&self.glob_cache
	}
	
	/**
	 * Adds a pattern and its matches to the glob cache
	 * 
	 * @param pattern - Glob pattern string
	 * @param matches - List of matching file names
	 */
	pub fn add_to_glob_cache(&mut self, pattern: String, matches: Vec<String>) {
		self.glob_cache.insert(pattern, matches);
	}
	
	/**
	 * Processes brace and glob expansions using current state
	 * 
	 * @param input - Input string to process
	 * @return Result<String> - Processed string or error
	 */
	pub fn process_expansions(&self, input: &str) -> Result<String> {
		ExpansionProcessor::process_brace_expansions(input, &self.working_directory)
	}
} 