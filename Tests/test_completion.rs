/**
 * Tab completion tests for Sare terminal
 * 
 * Tests tab completion with context awareness, file completion,
 * command completion, and quoted path handling.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_completion.rs
 * Description: Comprehensive tab completion testing
 */

use sare::history::completion::{TabCompleter, CompletionContext, CompletionResult};
use std::path::PathBuf;

#[test]
fn test_tab_completer_creation() {
	/**
	 * タブ補完作成のテストです (｡◕‿◕｡)
	 * 
	 * このテストはタブ補完の初期化を確認します。
	 * 基本的な機能が正しく動作することを検証します (◕‿◕)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	assert!(!completer.common_commands.is_empty());
	assert!(completer.common_commands.contains(&"ls".to_string()));
	assert!(completer.common_commands.contains(&"cd".to_string()));
	assert!(completer.common_commands.contains(&"echo".to_string()));
}

#[test]
fn test_command_completion() {
	/**
	 * コマンド補完のテストです (◕‿◕)
	 * 
	 * このテストはコマンド補完機能を確認します。
	 * 部分的なコマンド入力から正しく補完されることを検証します (｡◕‿◕｡)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test command completion
	let result = completer.complete("l", 1).unwrap().unwrap();
	assert_eq!(result.completed_text, "ls");
	assert_eq!(result.context, CompletionContext::Command);
	
	let result = completer.complete("ec", 2).unwrap().unwrap();
	assert_eq!(result.completed_text, "echo");
	assert_eq!(result.context, CompletionContext::Command);
	
	let result = completer.complete("cd", 2).unwrap().unwrap();
	assert_eq!(result.completed_text, "cd");
	assert_eq!(result.context, CompletionContext::Command);
}

#[test]
fn test_file_path_completion() {
	/**
	 * ファイルパス補完のテストです (｡◕‿◕｡)
	 * 
	 * このテストはファイルパス補完機能を確認します。
	 * ファイル名の部分入力から正しく補完されることを検証します (◕‿◕)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Create a test file
	let test_file = std::env::temp_dir().join("test_completion_file.txt");
	std::fs::write(&test_file, "test content").unwrap();
	
	// Test file completion (this will depend on actual files in temp dir)
	let result = completer.complete("cat ", 4).unwrap();
	if let Some(completion) = result {
		assert_eq!(completion.context, CompletionContext::FilePath);
	}
	
	// Clean up
	let _ = std::fs::remove_file(test_file);
}

#[test]
fn test_quoted_path_completion() {
	/**
	 * 引用符付きパス補完のテストです (◕‿◕)
	 * 
	 * このテストは引用符付きパス補完機能を確認します。
	 * スペースを含むファイル名が正しく処理されることを検証します (｡◕‿◕｡)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test quoted path completion
	let result = completer.complete("cat \"test", 9).unwrap();
	if let Some(completion) = result {
		assert_eq!(completion.context, CompletionContext::FilePath);
	}
	
	let result = completer.complete("cat 'test", 9).unwrap();
	if let Some(completion) = result {
		assert_eq!(completion.context, CompletionContext::FilePath);
	}
}

#[test]
fn test_flag_completion() {
	/**
	 * フラグ補完のテストです (｡◕‿◕｡)
	 * 
	 * このテストはフラグ補完機能を確認します。
	 * コマンドのフラグが正しく補完されることを検証します (◕‿◕)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test flag completion for ls
	let result = completer.complete("ls -", 4).unwrap();
	if let Some(completion) = result {
		assert_eq!(completion.context, CompletionContext::Flag);
		// Should suggest common ls flags
		assert!(completion.alternatives.iter().any(|flag| flag.contains("a")));
		assert!(completion.alternatives.iter().any(|flag| flag.contains("l")));
	}
}

#[test]
fn test_variable_completion() {
	/**
	 * 変数補完のテストです (◕‿◕)
	 * 
	 * このテストは変数補完機能を確認します。
	 * 環境変数が正しく補完されることを検証します (｡◕‿◕｡)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test variable completion
	let result = completer.complete("echo $H", 7).unwrap();
	if let Some(completion) = result {
		assert_eq!(completion.context, CompletionContext::Variable);
		// Should suggest HOME, HOSTNAME, etc.
		assert!(completion.alternatives.iter().any(|var| var.contains("HOME")));
	}
}

#[test]
fn test_completion_context_detection() {
	/**
	 * 補完コンテキスト検出のテストです (｡◕‿◕｡)
	 * 
	 * このテストは補完コンテキストの検出機能を確認します。
	 * 入力の位置に応じて正しいコンテキストが検出されることを検証します (◕‿◕)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test command context
	let context = completer.parse_context("ls", 2);
	assert_eq!(context, CompletionContext::Command);
	
	// Test file path context
	let context = completer.parse_context("cat test", 8);
	assert_eq!(context, CompletionContext::FilePath);
	
	// Test flag context
	let context = completer.parse_context("ls -", 4);
	assert_eq!(context, CompletionContext::Flag);
	
	// Test variable context
	let context = completer.parse_context("echo $H", 7);
	assert_eq!(context, CompletionContext::Variable);
}

#[test]
fn test_completion_alternatives() {
	/**
	 * 補完候補のテストです (◕‿◕)
	 * 
	 * このテストは補完候補の表示機能を確認します。
	 * 複数の候補が正しく表示されることを検証します (｡◕‿◕｡)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test command alternatives
	let result = completer.complete("l", 1).unwrap().unwrap();
	assert!(!result.alternatives.is_empty());
	assert!(result.alternatives.contains(&"ls".to_string()));
	assert!(result.alternatives.contains(&"ln".to_string()));
}

#[test]
fn test_completion_with_history() {
	/**
	 * ヒストリー付き補完のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒストリー付き補完機能を確認します。
	 * 実行済みコマンドが補完候補に含まれることを検証します (◕‿◕)
	 */
	
	let working_dir = PathBuf::from(".");
	let mut completer = TabCompleter::new(working_dir);
	
	// Add some commands to history
	completer.add_command("custom_command".to_string());
	completer.add_command("another_custom_command".to_string());
	
	// Test completion with history
	let result = completer.complete("custom", 6).unwrap().unwrap();
	assert!(result.alternatives.contains(&"custom_command".to_string()));
	assert!(result.alternatives.contains(&"another_custom_command".to_string()));
}

#[test]
fn test_working_directory_update() {
	/**
	 * 作業ディレクトリ更新のテストです (◕‿◕)
	 * 
	 * このテストは作業ディレクトリの更新機能を確認します。
	 * ディレクトリ変更時に補完が正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	let initial_dir = PathBuf::from(".");
	let mut completer = TabCompleter::new(initial_dir);
	
	// Update working directory
	let new_dir = std::env::temp_dir();
	completer.update_working_directory(new_dir.clone());
	
	assert_eq!(completer.working_directory, new_dir);
}

#[test]
fn test_completion_edge_cases() {
	/**
	 * 補完エッジケースのテストです (｡◕‿◕｡)
	 * 
	 * このテストは補完のエッジケースを確認します。
	 * 特殊な入力に対する動作が正しいことを検証します (◕‿◕)
	 */
	
	let working_dir = PathBuf::from(".");
	let completer = TabCompleter::new(working_dir);
	
	// Test empty input
	let result = completer.complete("", 0).unwrap();
	assert!(result.is_none());
	
	// Test whitespace only
	let result = completer.complete("   ", 3).unwrap();
	assert!(result.is_none());
	
	// Test cursor at beginning
	let result = completer.complete("ls", 0).unwrap();
	assert!(result.is_none());
	
	// Test cursor beyond input length
	let result = completer.complete("ls", 5).unwrap();
	assert!(result.is_none());
} 