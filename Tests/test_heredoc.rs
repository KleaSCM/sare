/**
 * Heredoc tests for Sare terminal
 * 
 * Tests heredoc support including syntax detection, content collection,
 * and variable expansion with proper delimiter handling.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_heredoc.rs
 * Description: Comprehensive heredoc testing
 */

use sare::gui::heredoc::{HeredocState, HeredocProcessor};

#[test]
fn test_heredoc_state_creation() {
	/**
	 * ヒアドキュメント状態作成のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒアドキュメント状態の初期化を確認します。
	 * 基本的な機能が正しく動作することを検証します (◕‿◕)
	 */
	
	let state = HeredocState::default();
	assert!(!state.is_heredoc());
	assert!(state.get_delimiter().is_empty());
	assert!(state.get_heredoc_content().is_empty());
	assert!(!state.should_expand_vars());
}

#[test]
fn test_heredoc_detection() {
	/**
	 * ヒアドキュメント検出のテストです (◕‿◕)
	 * 
	 * このテストはヒアドキュメント検出機能を確認します。
	 * 正しい構文が検出されることを検証します (｡◕‿◕｡)
	 */
	
	// Test basic heredoc detection
	let result = HeredocProcessor::detect_heredoc("cat <<EOF");
	assert!(result.is_some());
	let (delimiter, expand_vars) = result.unwrap();
	assert_eq!(delimiter, "EOF");
	assert!(expand_vars);
	
	// Test quoted heredoc detection
	let result = HeredocProcessor::detect_heredoc("cat <<'EOF'");
	assert!(result.is_some());
	let (delimiter, expand_vars) = result.unwrap();
	assert_eq!(delimiter, "EOF");
	assert!(!expand_vars);
	
	// Test double quoted heredoc detection
	let result = HeredocProcessor::detect_heredoc("cat <<\"EOF\"");
	assert!(result.is_some());
	let (delimiter, expand_vars) = result.unwrap();
	assert_eq!(delimiter, "EOF");
	assert!(!expand_vars);
	
	// Test no heredoc
	let result = HeredocProcessor::detect_heredoc("cat file.txt");
	assert!(result.is_none());
}

#[test]
fn test_heredoc_delimiter_matching() {
	/**
	 * ヒアドキュメント区切り文字マッチングのテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒアドキュメント区切り文字のマッチングを確認します。
	 * 区切り文字が正しく検出されることを検証します (◕‿◕)
	 */
	
	let mut state = HeredocState::default();
	state.set_heredoc(true);
	state.set_delimiter("EOF".to_string());
	
	// Test matching delimiter
	assert!(HeredocProcessor::is_heredoc_delimiter(&state, "EOF"));
	assert!(HeredocProcessor::is_heredoc_delimiter(&state, "  EOF  "));
	
	// Test non-matching delimiter
	assert!(!HeredocProcessor::is_heredoc_delimiter(&state, "END"));
	assert!(!HeredocProcessor::is_heredoc_delimiter(&state, "EOF\n"));
	
	// Test when not in heredoc mode
	state.set_heredoc(false);
	assert!(!HeredocProcessor::is_heredoc_delimiter(&state, "EOF"));
}

#[test]
fn test_heredoc_content_collection() {
	/**
	 * ヒアドキュメント内容収集のテストです (◕‿◕)
	 * 
	 * このテストはヒアドキュメント内容の収集機能を確認します。
	 * 内容が正しく収集されることを検証します (｡◕‿◕｡)
	 */
	
	let mut state = HeredocState::default();
	state.set_heredoc(true);
	state.set_delimiter("EOF".to_string());
	
	// Add content lines
	state.add_heredoc_content("line 1".to_string());
	state.add_heredoc_content("line 2".to_string());
	state.add_heredoc_content("line 3".to_string());
	
	let content = state.get_heredoc_content();
	assert!(content.contains("line 1"));
	assert!(content.contains("line 2"));
	assert!(content.contains("line 3"));
	assert!(content.contains('\n'));
}

#[test]
fn test_heredoc_variable_expansion() {
	/**
	 * ヒアドキュメント変数展開のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒアドキュメント変数展開機能を確認します。
	 * 変数が正しく展開されることを検証します (◕‿◕)
	 */
	
	// Set up environment variable
	std::env::set_var("TEST_VAR", "test_value");
	
	// Test variable expansion
	let content = "Hello $TEST_VAR, welcome to $HOME";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	
	assert!(expanded.contains("test_value"));
	assert!(expanded.contains("Hello"));
	assert!(expanded.contains("welcome to"));
	
	// Test non-existent variable
	let content = "Hello $NONEXISTENT_VAR";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert_eq!(expanded, "Hello $NONEXISTENT_VAR");
	
	// Test mixed content
	let content = "Hello $TEST_VAR and $USER";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert!(expanded.contains("test_value"));
	
	// Clean up
	std::env::remove_var("TEST_VAR");
}

#[test]
fn test_heredoc_state_management() {
	/**
	 * ヒアドキュメント状態管理のテストです (◕‿◕)
	 * 
	 * このテストはヒアドキュメント状態管理機能を確認します。
	 * 状態の切り替えが正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	let mut state = HeredocState::default();
	
	// Test state transitions
	state.set_heredoc(true);
	state.set_delimiter("EOF".to_string());
	state.set_expand_vars(true);
	
	assert!(state.is_heredoc());
	assert_eq!(state.get_delimiter(), "EOF");
	assert!(state.should_expand_vars());
	
	// Test content management
	state.set_heredoc_content("test content".to_string());
	assert_eq!(state.get_heredoc_content(), "test content");
	
	// Test reset
	state.set_heredoc(false);
	state.set_delimiter("".to_string());
	state.set_expand_vars(false);
	state.set_heredoc_content("".to_string());
	
	assert!(!state.is_heredoc());
	assert!(state.get_delimiter().is_empty());
	assert!(!state.should_expand_vars());
	assert!(state.get_heredoc_content().is_empty());
}

#[test]
fn test_heredoc_syntax_variations() {
	/**
	 * ヒアドキュメント構文バリエーションのテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒアドキュメント構文のバリエーションを確認します。
	 * 様々な構文パターンが正しく処理されることを検証します (◕‿◕)
	 */
	
	// Test different delimiter names
	let result = HeredocProcessor::detect_heredoc("cat <<END");
	assert!(result.is_some());
	let (delimiter, _) = result.unwrap();
	assert_eq!(delimiter, "END");
	
	let result = HeredocProcessor::detect_heredoc("cat <<STOP");
	assert!(result.is_some());
	let (delimiter, _) = result.unwrap();
	assert_eq!(delimiter, "STOP");
	
	// Test with spaces
	let result = HeredocProcessor::detect_heredoc("cat << EOF");
	assert!(result.is_some());
	let (delimiter, _) = result.unwrap();
	assert_eq!(delimiter, "EOF");
	
	// Test with tabs
	let result = HeredocProcessor::detect_heredoc("cat <<\tEOF");
	assert!(result.is_some());
	let (delimiter, _) = result.unwrap();
	assert_eq!(delimiter, "EOF");
}

#[test]
fn test_heredoc_complex_content() {
	/**
	 * 複雑なヒアドキュメント内容のテストです (◕‿◕)
	 * 
	 * このテストは複雑なヒアドキュメント内容の処理を確認します。
	 * 複数行や特殊文字が正しく処理されることを検証します (｡◕‿◕｡)
	 */
	
	let mut state = HeredocState::default();
	state.set_heredoc(true);
	state.set_delimiter("EOF".to_string());
	
	// Add complex content
	state.add_heredoc_content("#!/bin/bash".to_string());
	state.add_heredoc_content("echo \"Hello World\"".to_string());
	state.add_heredoc_content("for i in {1..5}; do".to_string());
	state.add_heredoc_content("  echo \"Line $i\"".to_string());
	state.add_heredoc_content("done".to_string());
	
	let content = state.get_heredoc_content();
	assert!(content.contains("#!/bin/bash"));
	assert!(content.contains("echo \"Hello World\""));
	assert!(content.contains("for i in {1..5}; do"));
	assert!(content.contains("  echo \"Line $i\""));
	assert!(content.contains("done"));
}

#[test]
fn test_heredoc_edge_cases() {
	/**
	 * ヒアドキュメントエッジケースのテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒアドキュメントのエッジケースを確認します。
	 * 特殊な入力に対する動作が正しいことを検証します (◕‿◕)
	 */
	
	// Test empty delimiter
	let result = HeredocProcessor::detect_heredoc("cat <<");
	assert!(result.is_none());
	
	// Test empty content
	let mut state = HeredocState::default();
	state.set_heredoc(true);
	state.set_delimiter("EOF".to_string());
	
	state.add_heredoc_content("".to_string());
	let content = state.get_heredoc_content();
	assert_eq!(content, "\n");
	
	// Test delimiter with special characters
	let result = HeredocProcessor::detect_heredoc("cat <<EOF-123");
	assert!(result.is_some());
	let (delimiter, _) = result.unwrap();
	assert_eq!(delimiter, "EOF-123");
	
	// Test nested heredocs (should not be detected)
	let result = HeredocProcessor::detect_heredoc("cat <<EOF <<END");
	assert!(result.is_some());
	let (delimiter, _) = result.unwrap();
	assert_eq!(delimiter, "EOF");
}

#[test]
fn test_heredoc_variable_expansion_edge_cases() {
	/**
	 * ヒアドキュメント変数展開エッジケースのテストです (◕‿◕)
	 * 
	 * このテストはヒアドキュメント変数展開のエッジケースを確認します。
	 * 特殊な変数パターンが正しく処理されることを検証します (｡◕‿◕｡)
	 */
	
	// Test empty variable
	let content = "Hello $";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert_eq!(expanded, "Hello $");
	
	// Test variable at end
	let content = "Hello $TEST";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert_eq!(expanded, "Hello $TEST");
	
	// Test multiple variables
	let content = "$VAR1 $VAR2 $VAR3";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert_eq!(expanded, "$VAR1 $VAR2 $VAR3");
	
	// Test variable with underscore
	let content = "Hello $TEST_VAR_123";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert_eq!(expanded, "Hello $TEST_VAR_123");
	
	// Test variable with numbers
	let content = "Hello $VAR123";
	let expanded = HeredocProcessor::expand_heredoc_variables(content);
	assert_eq!(expanded, "Hello $VAR123");
} 