/**
 * Command substitution tests for Sare terminal
 * 
 * Tests command substitution support including $(command) and `command`
 * syntax with nested substitution and error handling.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_substitution.rs
 * Description: Comprehensive command substitution testing
 */

use sare::gui::substitution::{SubstitutionState, SubstitutionProcessor};

#[test]
fn test_substitution_state_creation() {
	/**
	 * コマンド置換状態作成のテストです (｡◕‿◕｡)
	 * 
	 * このテストはコマンド置換状態の初期化を確認します。
	 * 基本的な機能が正しく動作することを検証します (◕‿◕)
	 */
	
	let state = SubstitutionState::default();
	assert!(!state.is_substitution_mode());
	assert_eq!(state.get_depth(), 0);
	assert!(state.get_buffer().is_empty());
}

#[test]
fn test_basic_command_substitution() {
	/**
	 * 基本的なコマンド置換のテストです (◕‿◕)
	 * 
	 * このテストは基本的なコマンド置換機能を確認します。
	 * $(command)構文が正しく処理されることを検証します (｡◕‿◕｡)
	 */
	
	// Test $(command) syntax
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo $(date)");
	assert_eq!(substitutions.len(), 1);
	let (start, end, command) = &substitutions[0];
	assert_eq!(*start, 5);
	assert_eq!(*end, 12);
	assert_eq!(command, "date");
	
	// Test `command` syntax
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo `date`");
	assert_eq!(substitutions.len(), 1);
	let (start, end, command) = &substitutions[0];
	assert_eq!(*start, 5);
	assert_eq!(*end, 12);
	assert_eq!(command, "date");
}

#[test]
fn test_nested_command_substitution() {
	/**
	 * ネストしたコマンド置換のテストです (｡◕‿◕｡)
	 * 
	 * このテストはネストしたコマンド置換機能を確認します。
	 * 複数の置換が正しく処理されることを検証します (◕‿◕)
	 */
	
	// Test nested $(command) syntax
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo $(echo $(date))");
	assert_eq!(substitutions.len(), 2);
	
	// Test nested backticks
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo `echo `date``");
	assert_eq!(substitutions.len(), 2);
	
	// Test mixed syntax
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo $(echo `date`)");
	assert_eq!(substitutions.len(), 2);
}

#[test]
fn test_command_execution() {
	/**
	 * コマンド実行のテストです (◕‿◕)
	 * 
	 * このテストはコマンド実行機能を確認します。
	 * コマンドが正しく実行されることを検証します (｡◕‿◕｡)
	 */
	
	// Test simple command execution
	let result = SubstitutionProcessor::execute_substitution_command("echo hello");
	assert!(result.is_ok());
	let output = result.unwrap();
	assert!(output.contains("hello"));
	
	// Test command with arguments
	let result = SubstitutionProcessor::execute_substitution_command("echo hello world");
	assert!(result.is_ok());
	let output = result.unwrap();
	assert!(output.contains("hello world"));
	
	// Test command that produces error
	let result = SubstitutionProcessor::execute_substitution_command("nonexistent_command");
	assert!(result.is_ok()); // Should not panic, just return empty string
}

#[test]
fn test_substitution_processing() {
	/**
	 * 置換処理のテストです (｡◕‿◕｡)
	 * 
	 * このテストは置換処理機能を確認します。
	 * 置換が正しく処理されることを検証します (◕‿◕)
	 */
	
	let mut state = SubstitutionState::default();
	
	// Test simple substitution
	let result = state.process_substitutions("echo $(echo hello)");
	assert!(result.is_ok());
	let processed = result.unwrap();
	assert!(processed.contains("hello"));
	
	// Test multiple substitutions
	let result = state.process_substitutions("echo $(echo hello) $(echo world)");
	assert!(result.is_ok());
	let processed = result.unwrap();
	assert!(processed.contains("hello"));
	assert!(processed.contains("world"));
	
	// Test no substitutions
	let result = state.process_substitutions("echo hello world");
	assert!(result.is_ok());
	let processed = result.unwrap();
	assert_eq!(processed, "echo hello world");
}

#[test]
fn test_substitution_state_management() {
	/**
	 * 置換状態管理のテストです (◕‿◕)
	 * 
	 * このテストは置換状態管理機能を確認します。
	 * 状態の切り替えが正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	let mut state = SubstitutionState::default();
	
	// Test state transitions
	state.set_substitution_mode(true);
	state.set_depth(2);
	state.set_buffer("test buffer".to_string());
	
	assert!(state.is_substitution_mode());
	assert_eq!(state.get_depth(), 2);
	assert_eq!(state.get_buffer(), "test buffer");
	
	// Test reset
	state.set_substitution_mode(false);
	state.set_depth(0);
	state.set_buffer("".to_string());
	
	assert!(!state.is_substitution_mode());
	assert_eq!(state.get_depth(), 0);
	assert!(state.get_buffer().is_empty());
}

#[test]
fn test_complex_substitution_scenarios() {
	/**
	 * 複雑な置換シナリオのテストです (｡◕‿◕｡)
	 * 
	 * このテストは複雑な置換シナリオを確認します。
	 * 複雑なコマンドが正しく処理されることを検証します (◕‿◕)
	 */
	
	let mut state = SubstitutionState::default();
	
	// Test command with pipes
	let result = state.process_substitutions("echo $(ls | grep .txt)");
	assert!(result.is_ok());
	
	// Test command with redirection
	let result = state.process_substitutions("echo $(echo hello > /tmp/test.txt && cat /tmp/test.txt)");
	assert!(result.is_ok());
	
	// Test command with variables
	let result = state.process_substitutions("echo $(echo $HOME)");
	assert!(result.is_ok());
	
	// Clean up
	let _ = std::fs::remove_file("/tmp/test.txt");
}

#[test]
fn test_substitution_edge_cases() {
	/**
	 * 置換エッジケースのテストです (◕‿◕)
	 * 
	 * このテストは置換のエッジケースを確認します。
	 * 特殊な入力に対する動作が正しいことを検証します (｡◕‿◕｡)
	 */
	
	// Test empty command
	let result = SubstitutionProcessor::execute_substitution_command("");
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), "");
	
	// Test whitespace only command
	let result = SubstitutionProcessor::execute_substitution_command("   ");
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), "");
	
	// Test unclosed substitution
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo $(date");
	assert_eq!(substitutions.len(), 0);
	
	// Test unclosed backticks
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo `date");
	assert_eq!(substitutions.len(), 0);
	
	// Test nested unclosed
	let substitutions = SubstitutionProcessor::detect_command_substitutions("echo $(echo $(date)");
	assert_eq!(substitutions.len(), 0);
}

#[test]
fn test_substitution_with_special_characters() {
	/**
	 * 特殊文字を含む置換のテストです (｡◕‿◕｡)
	 * 
	 * このテストは特殊文字を含む置換を確認します。
	 * 特殊文字が正しく処理されることを検証します (◕‿◕)
	 */
	
	// Test command with spaces
	let result = SubstitutionProcessor::execute_substitution_command("echo 'hello world'");
	assert!(result.is_ok());
	let output = result.unwrap();
	assert!(output.contains("hello world"));
	
	// Test command with quotes
	let result = SubstitutionProcessor::execute_substitution_command("echo \"hello world\"");
	assert!(result.is_ok());
	let output = result.unwrap();
	assert!(output.contains("hello world"));
	
	// Test command with special characters
	let result = SubstitutionProcessor::execute_substitution_command("echo 'hello$world'");
	assert!(result.is_ok());
	let output = result.unwrap();
	assert!(output.contains("hello$world"));
}

#[test]
fn test_substitution_error_handling() {
	/**
	 * 置換エラーハンドリングのテストです (◕‿◕)
	 * 
	 * このテストは置換エラーハンドリングを確認します。
	 * エラーが適切に処理されることを検証します (｡◕‿◕｡)
	 */
	
	let mut state = SubstitutionState::default();
	
	// Test command that fails
	let result = state.process_substitutions("echo $(nonexistent_command)");
	assert!(result.is_ok());
	let processed = result.unwrap();
	// Should replace failed command with empty string
	assert!(!processed.contains("nonexistent_command"));
	
	// Test command with syntax error
	let result = state.process_substitutions("echo $(echo 'unclosed quote)");
	assert!(result.is_ok());
	
	// Test command with invalid arguments
	let result = state.process_substitutions("echo $(ls --invalid-flag)");
	assert!(result.is_ok());
}

#[test]
fn test_substitution_performance() {
	/**
	 * 置換パフォーマンスのテストです (｡◕‿◕｡)
	 * 
	 * このテストは置換パフォーマンスを確認します。
	 * 大量の置換が効率的に処理されることを検証します (◕‿◕)
	 */
	
	let mut state = SubstitutionState::default();
	
	// Test multiple substitutions in one command
	let command = "echo $(echo 1) $(echo 2) $(echo 3) $(echo 4) $(echo 5)";
	let result = state.process_substitutions(command);
	assert!(result.is_ok());
	let processed = result.unwrap();
	
	// Should contain all numbers
	assert!(processed.contains("1"));
	assert!(processed.contains("2"));
	assert!(processed.contains("3"));
	assert!(processed.contains("4"));
	assert!(processed.contains("5"));
}

#[test]
fn test_substitution_depth_tracking() {
	/**
	 * 置換深度追跡のテストです (◕‿◕)
	 * 
	 * このテストは置換深度追跡を確認します。
	 * ネストした置換の深度が正しく追跡されることを検証します (｡◕‿◕｡)
	 */
	
	let mut state = SubstitutionState::default();
	
	// Test depth tracking
	state.set_depth(0);
	assert_eq!(state.get_depth(), 0);
	
	state.set_depth(1);
	assert_eq!(state.get_depth(), 1);
	
	state.set_depth(5);
	assert_eq!(state.get_depth(), 5);
	
	// Test depth reset
	state.set_depth(0);
	assert_eq!(state.get_depth(), 0);
} 