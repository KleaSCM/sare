/**
 * Core functionality tests for Sare terminal
 * 
 * This module provides tests to verify that all core terminal
 * components are properly connected and working together.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_core_functionality.rs
 * Description: Tests for core terminal functionality
 */

use sare_terminal::SareTerminal;
use sare_terminal::config::Config;

/**
 * Test terminal creation
 */
#[test]
fn test_terminal_creation() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await
		});
	
	assert!(terminal.is_ok());
}

/**
 * Test terminal initialization
 */
#[test]
fn test_terminal_initialization() {
	let config = Config::default();
	let mut terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let init_result = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			terminal.initialize().await
		});
	
	assert!(init_result.is_ok());
}

/**
 * Test UI manager access
 */
#[test]
fn test_ui_manager_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let ui_manager = terminal.ui_manager();
	assert!(ui_manager.config().enable_status_bar);
}

/**
 * Test history manager access
 */
#[test]
fn test_history_manager_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let history_manager = terminal.history_manager();
	assert!(history_manager.is_ok());
}

/**
 * Test debug manager access
 */
#[test]
fn test_debug_manager_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let debug_manager = terminal.debug_manager();
	assert!(!debug_manager.is_debug_mode());
}

/**
 * Test profiler access
 */
#[test]
fn test_profiler_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let profiler = terminal.profiler();
	let stats = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			profiler.get_statistics().await
		});
	
	assert_eq!(stats.total_samples, 0);
}

/**
 * Test logger access
 */
#[test]
fn test_logger_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let logger = terminal.logger();
	let stats = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			logger.get_statistics().await
		});
	
	assert_eq!(stats.total_entries, 0);
}

/**
 * Test error recovery access
 */
#[test]
fn test_error_recovery_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let error_recovery = terminal.error_recovery();
	let stats = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			error_recovery.get_statistics().await
		});
	
	assert_eq!(stats.total_errors, 0);
}

/**
 * Test testing framework access
 */
#[test]
fn test_testing_framework_access() {
	let config = Config::default();
	let terminal = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			SareTerminal::new(config).await.unwrap()
		});
	
	let testing_framework = terminal.testing_framework();
	let stats = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(async {
			testing_framework.get_statistics().await
		});
	
	assert_eq!(stats.total_tests, 0);
}

/**
 * Run all core functionality tests
 */
pub fn run_core_functionality_tests() -> Vec<(&'static str, bool)> {
	let mut results = Vec::new();
	
	let tests = vec![
		("test_terminal_creation", test_terminal_creation),
		("test_terminal_initialization", test_terminal_initialization),
		("test_ui_manager_access", test_ui_manager_access),
		("test_history_manager_access", test_history_manager_access),
		("test_debug_manager_access", test_debug_manager_access),
		("test_profiler_access", test_profiler_access),
		("test_logger_access", test_logger_access),
		("test_error_recovery_access", test_error_recovery_access),
		("test_testing_framework_access", test_testing_framework_access),
	];
	
	for (name, test_fn) in tests {
		let result = std::panic::catch_unwind(test_fn).is_ok();
		results.push((name, result));
	}
	
	results
} 