/**
 * Brace expansion and globbing tests for Sare terminal
 * 
 * Tests brace expansion and globbing support including numeric ranges,
 * comma lists, glob patterns, and advanced matching with caching.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_expansion.rs
 * Description: Comprehensive brace expansion and globbing testing
 */

use sare::gui::expansion::{ExpansionState, ExpansionProcessor};
use std::path::PathBuf;

#[test]
fn test_expansion_state_creation() {
	/**
	 * 展開状態作成のテストです (｡◕‿◕｡)
	 * 
	 * このテストは展開状態の初期化を確認します。
	 * 基本的な機能が正しく動作することを検証します (◕‿◕)
	 */
	
	let state = ExpansionState::default();
	assert!(!state.is_expansion_mode());
	assert_eq!(state.get_depth(), 0);
	assert!(state.get_glob_cache().is_empty());
	assert!(state.get_working_directory().exists());
}

#[test]
fn test_brace_expansion_detection() {
	/**
	 * ブレース展開検出のテストです (◕‿◕)
	 * 
	 * このテストはブレース展開検出機能を確認します。
	 * 正しい構文が検出されることを検証します (｡◕‿◕｡)
	 */
	
	// Test basic brace expansion
	let expansions = ExpansionProcessor::detect_brace_expansions("echo {a,b,c}");
	assert_eq!(expansions.len(), 1);
	let (start, end, pattern) = &expansions[0];
	assert_eq!(*start, 5);
	assert_eq!(*end, 12);
	assert_eq!(pattern, "a,b,c");
	
	// Test numeric range expansion
	let expansions = ExpansionProcessor::detect_brace_expansions("echo {1..5}");
	assert_eq!(expansions.len(), 1);
	let (start, end, pattern) = &expansions[0];
	assert_eq!(pattern, "1..5");
	
	// Test no expansion
	let expansions = ExpansionProcessor::detect_brace_expansions("echo hello");
	assert_eq!(expansions.len(), 0);
}

#[test]
fn test_numeric_range_expansion() {
	/**
	 * 数値範囲展開のテストです (｡◕‿◕｡)
	 * 
	 * このテストは数値範囲展開機能を確認します。
	 * 数値範囲が正しく展開されることを検証します (◕‿◕)
	 */
	
	// Test basic range
	let expanded = ExpansionProcessor::expand_brace_pattern("1..5");
	assert_eq!(expanded, vec!["1", "2", "3", "4", "5"]);
	
	// Test range with step
	let expanded = ExpansionProcessor::expand_brace_pattern("1..10..2");
	assert_eq!(expanded, vec!["1", "3", "5", "7", "9"]);
	
	// Test reverse range
	let expanded = ExpansionProcessor::expand_brace_pattern("5..1");
	assert_eq!(expanded, vec!["5", "4", "3", "2", "1"]);
	
	// Test single number
	let expanded = ExpansionProcessor::expand_brace_pattern("42");
	assert_eq!(expanded, vec!["42"]);
}

#[test]
fn test_comma_list_expansion() {
	/**
	 * カンマリスト展開のテストです (◕‿◕)
	 * 
	 * このテストはカンマリスト展開機能を確認します。
	 * カンマ区切りリストが正しく展開されることを検証します (｡◕‿◕｡)
	 */
	
	// Test basic comma list
	let expanded = ExpansionProcessor::expand_brace_pattern("a,b,c");
	assert_eq!(expanded, vec!["a", "b", "c"]);
	
	// Test comma list with spaces
	let expanded = ExpansionProcessor::expand_brace_pattern("dev, test, prod");
	assert_eq!(expanded, vec!["dev", "test", "prod"]);
	
	// Test single item
	let expanded = ExpansionProcessor::expand_brace_pattern("single");
	assert_eq!(expanded, vec!["single"]);
	
	// Test nested braces in comma list
	let expanded = ExpansionProcessor::expand_brace_pattern("a,{1..3},c");
	assert_eq!(expanded, vec!["a", "{1..3}", "c"]);
}

#[test]
fn test_glob_pattern_expansion() {
	/**
	 * グロブパターン展開のテストです (｡◕‿◕｡)
	 * 
	 * このテストはグロブパターン展開機能を確認します。
	 * グロブパターンが正しく展開されることを検証します (◕‿◕)
	 */
	
	let working_dir = std::env::temp_dir();
	
	// Create test files
	let test_files = ["test1.txt", "test2.txt", "test3.rs", "README.md"];
	for file in &test_files {
		let file_path = working_dir.join(file);
		std::fs::write(file_path, "test content").unwrap();
	}
	
	// Test basic glob pattern
	let matches = ExpansionProcessor::expand_glob_pattern("*.txt", &working_dir);
	assert!(matches.contains(&"test1.txt".to_string()));
	assert!(matches.contains(&"test2.txt".to_string()));
	assert!(!matches.contains(&"test3.rs".to_string()));
	
	// Test pattern with no matches
	let matches = ExpansionProcessor::expand_glob_pattern("*.nonexistent", &working_dir);
	assert!(matches.is_empty());
	
	// Test exact filename
	let matches = ExpansionProcessor::expand_glob_pattern("README.md", &working_dir);
	assert_eq!(matches, vec!["README.md"]);
	
	// Clean up
	for file in &test_files {
		let file_path = working_dir.join(file);
		let _ = std::fs::remove_file(file_path);
	}
}

#[test]
fn test_glob_pattern_matching() {
	/**
	 * グロブパターンマッチングのテストです (◕‿◕)
	 * 
	 * このテストはグロブパターンマッチング機能を確認します。
	 * パターンマッチングが正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	// Test wildcard matching
	assert!(ExpansionProcessor::matches_glob_pattern("test.txt", "*"));
	assert!(ExpansionProcessor::matches_glob_pattern("test.txt", "*.txt"));
	assert!(ExpansionProcessor::matches_glob_pattern("test.txt", "test.*"));
	assert!(!ExpansionProcessor::matches_glob_pattern("test.txt", "*.rs"));
	
	// Test single character matching
	assert!(ExpansionProcessor::matches_glob_pattern("a", "?"));
	assert!(ExpansionProcessor::matches_glob_pattern("ab", "a?"));
	assert!(!ExpansionProcessor::matches_glob_pattern("abc", "?"));
	
	// Test character classes
	assert!(ExpansionProcessor::matches_glob_pattern("a", "[abc]"));
	assert!(ExpansionProcessor::matches_glob_pattern("b", "[abc]"));
	assert!(!ExpansionProcessor::matches_glob_pattern("d", "[abc]"));
	
	// Test character ranges
	assert!(ExpansionProcessor::matches_glob_pattern("a", "[a-z]"));
	assert!(ExpansionProcessor::matches_glob_pattern("z", "[a-z]"));
	assert!(!ExpansionProcessor::matches_glob_pattern("A", "[a-z]"));
	
	// Test negated character classes
	assert!(ExpansionProcessor::matches_glob_pattern("d", "[^abc]"));
	assert!(!ExpansionProcessor::matches_glob_pattern("a", "[^abc]"));
}

#[test]
fn test_complex_expansion_scenarios() {
	/**
	 * 複雑な展開シナリオのテストです (｡◕‿◕｡)
	 * 
	 * このテストは複雑な展開シナリオを確認します。
	 * 複雑なパターンが正しく処理されることを検証します (◕‿◕)
	 */
	
	let working_dir = std::env::temp_dir();
	
	// Test combination of brace expansion and globbing
	let result = ExpansionProcessor::process_brace_expansions("ls {*.txt,*.rs}", &working_dir);
	assert!(result.is_ok());
	
	// Test nested expansions
	let result = ExpansionProcessor::process_brace_expansions("echo {a,{1..3},c}", &working_dir);
	assert!(result.is_ok());
	let processed = result.unwrap();
	assert!(processed.contains("a"));
	assert!(processed.contains("1"));
	assert!(processed.contains("2"));
	assert!(processed.contains("3"));
	assert!(processed.contains("c"));
	
	// Test multiple expansions
	let result = ExpansionProcessor::process_brace_expansions("echo {1..3} {a,b,c}", &working_dir);
	assert!(result.is_ok());
	let processed = result.unwrap();
	assert!(processed.contains("1"));
	assert!(processed.contains("2"));
	assert!(processed.contains("3"));
	assert!(processed.contains("a"));
	assert!(processed.contains("b"));
	assert!(processed.contains("c"));
}

#[test]
fn test_expansion_state_management() {
	/**
	 * 展開状態管理のテストです (◕‿◕)
	 * 
	 * このテストは展開状態管理機能を確認します。
	 * 状態の切り替えが正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	let mut state = ExpansionState::default();
	
	// Test state transitions
	state.set_expansion_mode(true);
	state.set_depth(2);
	state.set_working_directory(PathBuf::from("/tmp"));
	
	assert!(state.is_expansion_mode());
	assert_eq!(state.get_depth(), 2);
	assert_eq!(state.get_working_directory(), PathBuf::from("/tmp"));
	
	// Test glob cache
	state.add_to_glob_cache("*.txt".to_string(), vec!["test1.txt".to_string(), "test2.txt".to_string()]);
	assert_eq!(state.get_glob_cache().len(), 1);
	assert!(state.get_glob_cache().contains_key("*.txt"));
	
	// Test reset
	state.set_expansion_mode(false);
	state.set_depth(0);
	
	assert!(!state.is_expansion_mode());
	assert_eq!(state.get_depth(), 0);
}

#[test]
fn test_expansion_edge_cases() {
	/**
	 * 展開エッジケースのテストです (｡◕‿◕｡)
	 * 
	 * このテストは展開のエッジケースを確認します。
	 * 特殊な入力に対する動作が正しいことを検証します (◕‿◕)
	 */
	
	// Test empty pattern
	let expanded = ExpansionProcessor::expand_brace_pattern("");
	assert_eq!(expanded, vec![""]);
	
	// Test invalid numeric range
	let expanded = ExpansionProcessor::expand_brace_pattern("5..1..0");
	assert_eq!(expanded, vec!["5..1..0"]);
	
	// Test invalid range format
	let expanded = ExpansionProcessor::expand_brace_pattern("1..5..2..3");
	assert_eq!(expanded, vec!["1..5..2..3"]);
	
	// Test unclosed braces
	let expansions = ExpansionProcessor::detect_brace_expansions("echo {a,b,c");
	assert_eq!(expansions.len(), 0);
	
	// Test nested unclosed braces
	let expansions = ExpansionProcessor::detect_brace_expansions("echo {a,{b,c}");
	assert_eq!(expansions.len(), 0);
}

#[test]
fn test_expansion_performance() {
	/**
	 * 展開パフォーマンスのテストです (◕‿◕)
	 * 
	 * このテストは展開パフォーマンスを確認します。
	 * 大量の展開が効率的に処理されることを検証します (｡◕‿◕｡)
	 */
	
	// Test large numeric range
	let expanded = ExpansionProcessor::expand_brace_pattern("1..100");
	assert_eq!(expanded.len(), 100);
	assert_eq!(expanded[0], "1");
	assert_eq!(expanded[99], "100");
	
	// Test large comma list
	let pattern = (1..=50).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
	let expanded = ExpansionProcessor::expand_brace_pattern(&pattern);
	assert_eq!(expanded.len(), 50);
	
	// Test multiple expansions
	let result = ExpansionProcessor::process_brace_expansions("echo {1..10} {a,b,c,d,e}", &std::env::temp_dir());
	assert!(result.is_ok());
}

#[test]
fn test_expansion_with_special_characters() {
	/**
	 * 特殊文字を含む展開のテストです (｡◕‿◕｡)
	 * 
	 * このテストは特殊文字を含む展開を確認します。
	 * 特殊文字が正しく処理されることを検証します (◕‿◕)
	 */
	
	// Test patterns with spaces
	let expanded = ExpansionProcessor::expand_brace_pattern("hello world, goodbye world");
	assert_eq!(expanded, vec!["hello world", "goodbye world"]);
	
	// Test patterns with special characters
	let expanded = ExpansionProcessor::expand_brace_pattern("file.txt,file.rs,file.md");
	assert_eq!(expanded, vec!["file.txt", "file.rs", "file.md"]);
	
	// Test patterns with quotes
	let expanded = ExpansionProcessor::expand_brace_pattern("\"hello\",\"world\"");
	assert_eq!(expanded, vec!["\"hello\"", "\"world\""]);
	
	// Test patterns with variables
	let expanded = ExpansionProcessor::expand_brace_pattern("$HOME,$PWD");
	assert_eq!(expanded, vec!["$HOME", "$PWD"]);
}

#[test]
fn test_expansion_error_handling() {
	/**
	 * 展開エラーハンドリングのテストです (◕‿◕)
	 * 
	 * このテストは展開エラーハンドリングを確認します。
	 * エラーが適切に処理されることを検証します (｡◕‿◕｡)
	 */
	
	let working_dir = std::env::temp_dir();
	
	// Test invalid glob pattern
	let result = ExpansionProcessor::process_brace_expansions("ls [invalid", &working_dir);
	assert!(result.is_ok());
	
	// Test non-existent directory
	let result = ExpansionProcessor::process_brace_expansions("ls /nonexistent/*", &working_dir);
	assert!(result.is_ok());
	
	// Test malformed brace pattern
	let result = ExpansionProcessor::process_brace_expansions("echo {1..}", &working_dir);
	assert!(result.is_ok());
	
	// Test empty brace pattern
	let result = ExpansionProcessor::process_brace_expansions("echo {}", &working_dir);
	assert!(result.is_ok());
} 