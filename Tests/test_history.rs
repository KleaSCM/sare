/**
 * Command history tests for Sare terminal
 * 
 * Tests command history navigation, search, and persistence
 * features to ensure proper functionality.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_history.rs
 * Description: Comprehensive history feature testing
 */

use sare::history::{HistoryManager, HistoryNavigator, HistoryNavigationState};
use std::path::PathBuf;

#[test]
fn test_history_manager_creation() {
	/**
	 * ヒストリーマネージャー作成のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒストリーマネージャーの初期化を確認します。
	 * 基本的な機能が正しく動作することを検証します (◕‿◕)
	 */
	
	let history_manager = HistoryManager::new().unwrap();
	assert_eq!(history_manager.history.len(), 0);
	assert_eq!(history_manager.max_entries, 1000);
}

#[test]
fn test_add_commands_to_history() {
	/**
	 * コマンド追加のテストです (◕‿◕)
	 * 
	 * このテストはコマンドの追加機能を確認します。
	 * ヒストリーに正しくコマンドが保存されることを検証します (｡◕‿◕｡)
	 */
	
	let mut history_manager = HistoryManager::new().unwrap();
	
	// Add some test commands
	history_manager.add_command("ls -la".to_string(), None);
	history_manager.add_command("cd /home".to_string(), None);
	history_manager.add_command("echo 'hello world'".to_string(), None);
	
	assert_eq!(history_manager.history.len(), 3);
	assert_eq!(history_manager.history[0].command, "ls -la");
	assert_eq!(history_manager.history[1].command, "cd /home");
	assert_eq!(history_manager.history[2].command, "echo 'hello world'");
}

#[test]
fn test_history_navigation() {
	/**
	 * ヒストリー操作のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒストリー操作機能を確認します。
	 * 上下キーでの操作が正しく動作することを検証します (◕‿◕)
	 */
	
	let mut navigator = HistoryNavigator::new(HistoryManager::new().unwrap());
	
	// Add some commands
	navigator.add_command("first command".to_string());
	navigator.add_command("second command".to_string());
	navigator.add_command("third command".to_string());
	
	// Test navigation up
	let result = navigator.navigate_up().unwrap();
	assert_eq!(result.command, "third command");
	
	let result = navigator.navigate_up().unwrap();
	assert_eq!(result.command, "second command");
	
	let result = navigator.navigate_up().unwrap();
	assert_eq!(result.command, "first command");
	
	// Test navigation down
	let result = navigator.navigate_down().unwrap();
	assert_eq!(result.command, "second command");
	
	let result = navigator.navigate_down().unwrap();
	assert_eq!(result.command, "third command");
}

#[test]
fn test_reverse_search() {
	/**
	 * 逆方向検索のテストです (◕‿◕)
	 * 
	 * このテストは逆方向検索機能を確認します。
	 * Ctrl+Rでの検索が正しく動作することを検証します (｡◕‿◕｡)
	 */
	
	let mut navigator = HistoryNavigator::new(HistoryManager::new().unwrap());
	
	// Add commands with different patterns
	navigator.add_command("ls -la".to_string());
	navigator.add_command("cd /home/user".to_string());
	navigator.add_command("echo 'hello'".to_string());
	navigator.add_command("ls -la /tmp".to_string());
	
	// Test reverse search for "ls"
	let result = navigator.perform_reverse_search("ls").unwrap();
	assert_eq!(result.command, "ls -la /tmp");
	
	// Test reverse search for "cd"
	let result = navigator.perform_reverse_search("cd").unwrap();
	assert_eq!(result.command, "cd /home/user");
}

#[test]
fn test_history_persistence() {
	/**
	 * ヒストリー永続化のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒストリーの永続化機能を確認します。
	 * ファイルへの保存と読み込みが正しく動作することを検証します (◕‿◕)
	 */
	
	let temp_file = std::env::temp_dir().join("test_history.txt");
	let mut history_manager = HistoryManager::with_config(100, temp_file.clone()).unwrap();
	
	// Add commands
	history_manager.add_command("test command 1".to_string(), None);
	history_manager.add_command("test command 2".to_string(), None);
	
	// Save to file
	history_manager.save_history().unwrap();
	
	// Create new manager and load from file
	let mut new_manager = HistoryManager::with_config(100, temp_file).unwrap();
	new_manager.load_history().unwrap();
	
	assert_eq!(new_manager.history.len(), 2);
	assert_eq!(new_manager.history[0].command, "test command 1");
	assert_eq!(new_manager.history[1].command, "test command 2");
	
	// Clean up
	let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_history_max_entries() {
	/**
	 * ヒストリー最大エントリー数のテストです (◕‿◕)
	 * 
	 * このテストはヒストリーの最大エントリー数制限を確認します。
	 * 制限を超えた場合の動作が正しいことを検証します (｡◕‿◕｡)
	 */
	
	let mut history_manager = HistoryManager::with_config(3, PathBuf::from("/tmp/test")).unwrap();
	
	// Add more commands than max_entries
	history_manager.add_command("command 1".to_string(), None);
	history_manager.add_command("command 2".to_string(), None);
	history_manager.add_command("command 3".to_string(), None);
	history_manager.add_command("command 4".to_string(), None);
	history_manager.add_command("command 5".to_string(), None);
	
	// Should only keep the last 3 commands
	assert_eq!(history_manager.history.len(), 3);
	assert_eq!(history_manager.history[0].command, "command 3");
	assert_eq!(history_manager.history[1].command, "command 4");
	assert_eq!(history_manager.history[2].command, "command 5");
}

#[test]
fn test_history_search_mode() {
	/**
	 * ヒストリー検索モードのテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒストリー検索モードの切り替えを確認します。
	 * 検索モードの開始と終了が正しく動作することを検証します (◕‿◕)
	 */
	
	let mut navigator = HistoryNavigator::new(HistoryManager::new().unwrap());
	
	// Test starting search mode
	navigator.start_reverse_search();
	assert!(navigator.state().search_mode);
	
	// Test exiting search mode
	navigator.exit_search();
	assert!(!navigator.state().search_mode);
}

#[test]
fn test_history_with_timestamps() {
	/**
	 * タイムスタンプ付きヒストリーのテストです (◕‿◕)
	 * 
	 * このテストはタイムスタンプ付きヒストリー機能を確認します。
	 * タイムスタンプが正しく記録されることを検証します (｡◕‿◕｡)
	 */
	
	let mut history_manager = HistoryManager::new().unwrap();
	
	// Add command with timestamp
	history_manager.add_command("timestamped command".to_string(), None);
	
	assert_eq!(history_manager.history.len(), 1);
	assert!(history_manager.history[0].timestamp > 0);
}

#[test]
fn test_history_display() {
	/**
	 * ヒストリー表示のテストです (｡◕‿◕｡)
	 * 
	 * このテストはヒストリー表示機能を確認します。
	 * ヒストリーが正しい形式で表示されることを検証します (◕‿◕)
	 */
	
	let mut history_manager = HistoryManager::new().unwrap();
	
	// Add some commands
	history_manager.add_command("ls".to_string(), None);
	history_manager.add_command("cd /home".to_string(), None);
	history_manager.add_command("echo hello".to_string(), None);
	
	let display = history_manager.get_history_display();
	
	// Should contain all commands
	assert!(display.contains("ls"));
	assert!(display.contains("cd /home"));
	assert!(display.contains("echo hello"));
	assert!(display.contains("1"));
	assert!(display.contains("2"));
	assert!(display.contains("3"));
} 