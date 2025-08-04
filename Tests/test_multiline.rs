/**
 * Multiline input tests for Sare terminal
 * 
 * Tests multiline input support including continuation lines,
 * quotes, pipes, and visual indicators.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_multiline.rs
 * Description: Comprehensive multiline input testing
 */

use sare::gui::multiline::{MultilineState, MultilineProcessor};

#[test]
fn test_multiline_state_creation() {
	/**
	 * マルチライン状態作成のテストです (｡◕‿◕｡)
	 * 
	 * このテストはマルチライン状態の初期化を確認します。
	 * 基本的な機能が正しく動作することを検証します (◕‿◕)
	 */
	
	let state = MultilineState::default();
	assert!(!state.is_multiline());
	assert!(state.get_continuation_char().is_none());
	assert!(state.multiline_prompt.is_empty());
}

#[test]
fn test_backslash_continuation() {
	/**
	 * バックスラッシュ継続のテストです (◕‿◕)
	 * 
	 * このテストはバックスラッシュ継続機能を確認します。
	 * 行末のバックスラッシュが正しく処理されることを検証します (｡◕‿◕｡)
	 */
	
	let mut state = MultilineState::default();
	
	// Test backslash continuation
	state.update("echo hello \\");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('\\'));
	assert_eq!(state.multiline_prompt, "> ");
	
	// Test completion
	state.update("echo hello \\\nworld");
	assert!(!state.is_multiline());
	assert!(state.get_continuation_char().is_none());
}

#[test]
fn test_pipe_continuation() {
	/**
	 * パイプ継続のテストです (｡◕‿◕｡)
	 * 
	 * このテストはパイプ継続機能を確認します。
	 * 行末のパイプが正しく処理されることを検証します (◕‿◕)
	 */
	
	let mut state = MultilineState::default();
	
	// Test pipe continuation
	state.update("ls -la |");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('|'));
	assert_eq!(state.multiline_prompt, "| ");
	
	// Test completion
	state.update("ls -la |\ngrep .txt");
	assert!(!state.is_multiline());
	assert!(state.get_continuation_char().is_none());
}

#[test]
fn test_quote_continuation() {
	/**
	 * 引用符継続のテストです (◕‿◕)
	 * 
	 * このテストは引用符継続機能を確認します。
	 * 未閉じの引用符が正しく処理されることを検証します (｡◕‿◕｡)
	 */
	
	let mut state = MultilineState::default();
	
	// Test single quote continuation
	state.update("echo 'hello");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('\''));
	assert_eq!(state.multiline_prompt, "'> ");
	
	// Test double quote continuation
	state.update("echo \"hello");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('"'));
	assert_eq!(state.multiline_prompt, "\"> ");
	
	// Test completion
	state.update("echo 'hello\nworld'");
	assert!(!state.is_multiline());
	assert!(state.get_continuation_char().is_none());
}

#[test]
fn test_parenthesis_continuation() {
	/**
	 * 括弧継続のテストです (｡◕‿◕｡)
	 * 
	 * このテストは括弧継続機能を確認します。
	 * 未閉じの括弧が正しく処理されることを検証します (◕‿◕)
	 */
	
	let mut state = MultilineState::default();
	
	// Test parenthesis continuation
	state.update("echo (hello");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('('));
	assert_eq!(state.multiline_prompt, "(> ");
	
	// Test brace continuation
	state.update("echo {hello");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('{'));
	assert_eq!(state.multiline_prompt, "{> ");
	
	// Test bracket continuation
	state.update("echo [hello");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('['));
	assert_eq!(state.multiline_prompt, "[> ");
}

#[test]
fn test_multiline_processor() {
	/**
	 * マルチラインプロセッサーのテストです (◕‿◕)
	 * 
	 * このテストはマルチラインプロセッサー機能を確認します。
	 * 継続文字の検出が正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	// Test backslash detection
	let (needs_continuation, continuation_char) = MultilineProcessor::check_multiline_continuation("echo hello \\");
	assert!(needs_continuation);
	assert_eq!(continuation_char, Some('\\'));
	
	// Test pipe detection
	let (needs_continuation, continuation_char) = MultilineProcessor::check_multiline_continuation("ls -la |");
	assert!(needs_continuation);
	assert_eq!(continuation_char, Some('|'));
	
	// Test quote detection
	let (needs_continuation, continuation_char) = MultilineProcessor::check_multiline_continuation("echo 'hello");
	assert!(needs_continuation);
	assert_eq!(continuation_char, Some('\''));
	
	// Test no continuation
	let (needs_continuation, continuation_char) = MultilineProcessor::check_multiline_continuation("echo hello");
	assert!(!needs_continuation);
	assert_eq!(continuation_char, None);
}

#[test]
fn test_multiline_prompt_generation() {
	/**
	 * マルチラインプロンプト生成のテストです (｡◕‿◕｡)
	 * 
	 * このテストはマルチラインプロンプト生成機能を確認します。
	 * 継続文字に応じて正しいプロンプトが生成されることを検証します (◕‿◕)
	 */
	
	let mut state = MultilineState::default();
	
	// Test backslash prompt
	state.update("echo hello \\");
	assert_eq!(state.multiline_prompt, "> ");
	
	// Test pipe prompt
	state.update("ls -la |");
	assert_eq!(state.multiline_prompt, "| ");
	
	// Test single quote prompt
	state.update("echo 'hello");
	assert_eq!(state.multiline_prompt, "'> ");
	
	// Test double quote prompt
	state.update("echo \"hello");
	assert_eq!(state.multiline_prompt, "\"> ");
	
	// Test parenthesis prompt
	state.update("echo (hello");
	assert_eq!(state.multiline_prompt, "(> ");
	
	// Test brace prompt
	state.update("echo {hello");
	assert_eq!(state.multiline_prompt, "{> ");
	
	// Test bracket prompt
	state.update("echo [hello");
	assert_eq!(state.multiline_prompt, "[> ");
}

#[test]
fn test_complex_multiline_commands() {
	/**
	 * 複雑なマルチラインコマンドのテストです (◕‿◕)
	 * 
	 * このテストは複雑なマルチラインコマンド機能を確認します。
	 * 複数の継続文字が組み合わされた場合の動作を検証します (｡◕‿◕｡)
	 */
	
	let mut state = MultilineState::default();
	
	// Test complex command with multiple continuations
	state.update("echo 'hello world' |");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('|'));
	
	state.update("echo 'hello world' |\ngrep 'hello' |");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('|'));
	
	state.update("echo 'hello world' |\ngrep 'hello' |\nwc -l");
	assert!(!state.is_multiline());
	assert!(state.get_continuation_char().is_none());
}

#[test]
fn test_multiline_state_transitions() {
	/**
	 * マルチライン状態遷移のテストです (｡◕‿◕｡)
	 * 
	 * このテストはマルチライン状態の遷移を確認します。
	 * 状態の切り替えが正しく動作することを検証します (◕‿◕)
	 */
	
	let mut state = MultilineState::default();
	
	// Start multiline
	state.set_multiline(true);
	state.set_continuation_char(Some('\\'));
	state.set_prompt("> ".to_string());
	
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('\\'));
	assert_eq!(state.multiline_prompt, "> ");
	
	// End multiline
	state.set_multiline(false);
	state.set_continuation_char(None);
	state.set_prompt("".to_string());
	
	assert!(!state.is_multiline());
	assert!(state.get_continuation_char().is_none());
	assert!(state.multiline_prompt.is_empty());
}

#[test]
fn test_multiline_edge_cases() {
	/**
	 * マルチラインエッジケースのテストです (◕‿◕)
	 * 
	 * このテストはマルチラインのエッジケースを確認します。
	 * 特殊な入力に対する動作が正しいことを検証します (｡◕‿◕｡)
	 */
	
	let mut state = MultilineState::default();
	
	// Test empty input
	state.update("");
	assert!(!state.is_multiline());
	
	// Test whitespace only
	state.update("   ");
	assert!(!state.is_multiline());
	
	// Test backslash in middle of line
	state.update("echo hello\\world");
	assert!(!state.is_multiline());
	
	// Test pipe in middle of line
	state.update("echo hello|world");
	assert!(!state.is_multiline());
	
	// Test quote in middle of line
	state.update("echo 'hello' world");
	assert!(!state.is_multiline());
}

#[test]
fn test_multiline_nested_quotes() {
	/**
	 * ネストした引用符のテストです (｡◕‿◕｡)
	 * 
	 * このテストはネストした引用符の処理を確認します。
	 * 複雑な引用符の組み合わせが正しく処理されることを検証します (◕‿◕)
	 */
	
	let mut state = MultilineState::default();
	
	// Test nested quotes
	state.update("echo 'hello \"world\"'");
	assert!(!state.is_multiline());
	
	state.update("echo \"hello 'world'\"");
	assert!(!state.is_multiline());
	
	// Test unclosed nested quotes
	state.update("echo 'hello \"world'");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('\''));
	
	state.update("echo \"hello 'world\"");
	assert!(state.is_multiline());
	assert_eq!(state.get_continuation_char(), Some('"'));
} 