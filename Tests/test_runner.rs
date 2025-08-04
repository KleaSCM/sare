/**
 * Unified test runner for Sare terminal
 * 
 * This module provides a comprehensive test suite that merges all
 * existing tests into a single runner with detailed output reporting,
 * test categorization, and failure analysis.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_runner.rs
 * Description: Unified test runner with comprehensive reporting
 */

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/**
 * Test result information
 * 
 * Contains detailed information about test execution including
 * success status, execution time, and error details.
 */
#[derive(Debug, Clone)]
pub struct TestResult {
	/// Test name
	pub name: String,
	/// Test category
	pub category: String,
	/// Success status
	pub success: bool,
	/// Execution time in milliseconds
	pub execution_time: u64,
	/// Error message if failed
	pub error_message: Option<String>,
	/// Test description
	pub description: String,
}

/**
 * Test suite summary
 * 
 * Contains summary information about test suite execution
 * including pass/fail counts and timing information.
 */
#[derive(Debug, Clone)]
pub struct TestSuiteSummary {
	/// Total tests run
	pub total_tests: usize,
	/// Passed tests
	pub passed_tests: usize,
	/// Failed tests
	pub failed_tests: usize,
	/// Total execution time
	pub total_time: Duration,
	/// Results by category
	pub results_by_category: HashMap<String, (usize, usize)>, // (passed, failed)
	/// Failed test details
	pub failed_tests_details: Vec<TestResult>,
}

/**
 * Test runner for unified test execution
 * 
 * Provides functionality to run all tests with comprehensive
 * reporting and detailed output analysis.
 */
pub struct TestRunner {
	/// Test results
	results: Arc<Mutex<Vec<TestResult>>>,
	/// Test suite summary
	summary: Arc<Mutex<TestSuiteSummary>>,
}

impl TestRunner {
	/**
	 * Creates a new test runner
	 * 
	 * @return TestRunner - New test runner instance
	 */
	pub fn new() -> Self {
		/**
		 * テストランナー初期化の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なテスト管理を行います。
		 * 複数のテストカテゴリ管理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		Self {
			results: Arc::new(Mutex::new(Vec::new())),
			summary: Arc::new(Mutex::new(TestSuiteSummary {
				total_tests: 0,
				passed_tests: 0,
				failed_tests: 0,
				total_time: Duration::new(0, 0),
				results_by_category: HashMap::new(),
				failed_tests_details: Vec::new(),
			})),
		}
	}
	
	/**
	 * Runs a single test with timing and error handling
	 * 
	 * @param test_fn - Test function to run
	 * @param name - Test name
	 * @param category - Test category
	 * @param description - Test description
	 * @return TestResult - Test execution result
	 */
	fn run_single_test<F>(&self, test_fn: F, name: &str, category: &str, description: &str) -> TestResult 
	where
		F: FnOnce() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + std::panic::UnwindSafe,
	{
		let start_time = Instant::now();
		let result = std::panic::catch_unwind(|| {
			test_fn()
		});
		
		let execution_time = start_time.elapsed().as_millis() as u64;
		
		match result {
			Ok(Ok(())) => {
				TestResult {
					name: name.to_string(),
					category: category.to_string(),
					success: true,
					execution_time,
					error_message: None,
					description: description.to_string(),
				}
			}
			Ok(Err(e)) => {
				TestResult {
					name: name.to_string(),
					category: category.to_string(),
					success: false,
					execution_time,
					error_message: Some(e.to_string()),
					description: description.to_string(),
				}
			}
			Err(panic_info) => {
				TestResult {
					name: name.to_string(),
					category: category.to_string(),
					success: false,
					execution_time,
					error_message: Some(format!("Test panicked: {:?}", panic_info)),
					description: description.to_string(),
				}
			}
		}
	}
	
	/**
	 * Runs all completion tests
	 * 
	 * @return Vec<TestResult> - Completion test results
	 */
	fn run_completion_tests(&self) -> Vec<TestResult> {
		/**
		 * 補完テスト実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な補完テストを行います。
		 * タブ補完機能の検証が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut results = Vec::new();
		
		// Test tab completer creation
		results.push(self.run_single_test(
			|| {
				use sare::history::completion::TabCompleter;
				use std::path::PathBuf;
				
				let working_dir = PathBuf::from(".");
				let mut completer = TabCompleter::new(working_dir);
				
				// Test that completer was created successfully
				// We can't access private fields, so we test the public API
				let result = completer.complete("l", 1);
				if result.is_err() {
					return Err("Completer should handle basic completion".into());
				}
				Ok(())
			},
			"test_tab_completer_creation",
			"completion",
			"Tests tab completer initialization and basic functionality"
		));
		
		// Test command completion
		results.push(self.run_single_test(
			|| {
				use sare::history::completion::{TabCompleter, CompletionContext};
				use std::path::PathBuf;
				
				let working_dir = PathBuf::from(".");
				let mut completer = TabCompleter::new(working_dir);
				
				let result = completer.complete("l", 1).unwrap().unwrap();
				if result.completed_text != "ls" {
					return Err("Command completion failed for 'l'".into());
				}
				if result.context != CompletionContext::Command {
					return Err("Context should be Command".into());
				}
				Ok(())
			},
			"test_command_completion",
			"completion",
			"Tests command completion functionality"
		));
		
		// Test file path completion
		results.push(self.run_single_test(
			|| {
				use sare::history::completion::{TabCompleter, CompletionContext};
				use std::path::PathBuf;
				
				let working_dir = PathBuf::from(".");
				let mut completer = TabCompleter::new(working_dir);
				
				// Create a test file
				let test_file = std::env::temp_dir().join("test_completion_file.txt");
				std::fs::write(&test_file, "test content").unwrap();
				
				// Test file completion
				let result = completer.complete("cat ", 4).unwrap();
				if let Some(completion) = result {
					if completion.context != CompletionContext::FilePath {
						return Err("Context should be FilePath".into());
					}
				}
				
				// Clean up
				let _ = std::fs::remove_file(test_file);
				Ok(())
			},
			"test_file_path_completion",
			"completion",
			"Tests file path completion functionality"
		));
		
		results
	}
	
	/**
	 * Runs all history tests
	 * 
	 * @return Vec<TestResult> - History test results
	 */
	fn run_history_tests(&self) -> Vec<TestResult> {
		/**
		 * ヒストリーテスト実行の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なヒストリーテストを行います。
		 * コマンドヒストリー機能の検証が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut results = Vec::new();
		
		// Test history manager creation
		results.push(self.run_single_test(
			|| {
				use sare::history::HistoryManager;
				
				let history_manager = HistoryManager::new()?;
				if history_manager.history.len() != 0 {
					return Err("New history manager should be empty".into());
				}
				if history_manager.max_entries != 1000 {
					return Err("Default max entries should be 1000".into());
				}
				Ok(())
			},
			"test_history_manager_creation",
			"history",
			"Tests history manager initialization"
		));
		
		// Test adding commands to history
		results.push(self.run_single_test(
			|| {
				use sare::history::HistoryManager;
				
				let mut history_manager = HistoryManager::new()?;
				
				history_manager.add_command("ls -la".to_string(), None);
				history_manager.add_command("cd /home".to_string(), None);
				history_manager.add_command("echo 'hello world'".to_string(), None);
				
				if history_manager.history.len() != 3 {
					return Err("Should have 3 commands in history".into());
				}
				if history_manager.history[0].command != "ls -la" {
					return Err("First command should be 'ls -la'".into());
				}
				Ok(())
			},
			"test_add_commands_to_history",
			"history",
			"Tests adding commands to history"
		));
		
		// Test history navigation
		results.push(self.run_single_test(
			|| {
				use sare::history::{HistoryManager, HistoryNavigator};
				
				let mut navigator = HistoryNavigator::new(HistoryManager::new()?);
				
				navigator.add_command("first command".to_string(), None);
				navigator.add_command("second command".to_string(), None);
				navigator.add_command("third command".to_string(), None);
				
				let result = navigator.navigate_up("").unwrap();
				if result != "third command" {
					return Err("Navigation up should return 'third command'".into());
				}
				
				let result = navigator.navigate_up("").unwrap();
				if result != "second command" {
					return Err("Navigation up should return 'second command'".into());
				}
				Ok(())
			},
			"test_history_navigation",
			"history",
			"Tests history navigation functionality"
		));
		
		results
	}
	
	/**
	 * Runs all expansion tests
	 * 
	 * @return Vec<TestResult> - Expansion test results
	 */
	fn run_expansion_tests(&self) -> Vec<TestResult> {
		/**
		 * 展開テスト実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な展開テストを行います。
		 * ブレース展開機能の検証が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut results = Vec::new();
		
		// Test expansion state creation
		results.push(self.run_single_test(
			|| {
				use sare::gui::expansion::ExpansionState;
				
				let state = ExpansionState::default();
				if state.is_expansion_mode() {
					return Err("New expansion state should not be in expansion mode".into());
				}
				if state.get_depth() != 0 {
					return Err("New expansion state should have depth 0".into());
				}
				if !state.get_glob_cache().is_empty() {
					return Err("New expansion state should have empty glob cache".into());
				}
				Ok(())
			},
			"test_expansion_state_creation",
			"expansion",
			"Tests expansion state initialization"
		));
		
		// Test brace expansion detection
		results.push(self.run_single_test(
			|| {
				use sare::gui::expansion::ExpansionProcessor;
				
				let expansions = ExpansionProcessor::detect_brace_expansions("echo {a,b,c}");
				if expansions.len() != 1 {
					return Err("Should detect one brace expansion".into());
				}
				
				let (start, end, pattern) = &expansions[0];
				if *start != 5 || *end != 12 {
					return Err("Brace expansion position incorrect".into());
				}
				if pattern != "a,b,c" {
					return Err("Brace expansion pattern incorrect".into());
				}
				Ok(())
			},
			"test_brace_expansion_detection",
			"expansion",
			"Tests brace expansion detection"
		));
		
		// Test numeric range expansion
		results.push(self.run_single_test(
			|| {
				use sare::gui::expansion::ExpansionProcessor;
				
				let expanded = ExpansionProcessor::expand_brace_pattern("1..5");
				let expected = vec!["1", "2", "3", "4", "5"];
				if expanded != expected {
					return Err("Numeric range expansion failed".into());
				}
				
				let expanded = ExpansionProcessor::expand_brace_pattern("1..10..2");
				let expected = vec!["1", "3", "5", "7", "9"];
				if expanded != expected {
					return Err("Numeric range with step expansion failed".into());
				}
				Ok(())
			},
			"test_numeric_range_expansion",
			"expansion",
			"Tests numeric range expansion"
		));
		
		results
	}
	
	/**
	 * Runs all substitution tests
	 * 
	 * @return Vec<TestResult> - Substitution test results
	 */
	fn run_substitution_tests(&self) -> Vec<TestResult> {
		/**
		 * 置換テスト実行の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑な置換テストを行います。
		 * コマンド置換機能の検証が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut results = Vec::new();
		
		// Test substitution state creation
		results.push(self.run_single_test(
			|| {
				use sare::gui::substitution::SubstitutionState;
				
				let state = SubstitutionState::default();
				if state.is_substitution_mode() {
					return Err("New substitution state should not be in substitution mode".into());
				}
				if state.get_depth() != 0 {
					return Err("New substitution state should have depth 0".into());
				}
				Ok(())
			},
			"test_substitution_state_creation",
			"substitution",
			"Tests substitution state initialization"
		));
		
		// Test command substitution detection
		results.push(self.run_single_test(
			|| {
				use sare::gui::substitution::SubstitutionProcessor;
				
				let substitutions = SubstitutionProcessor::detect_command_substitutions("echo $(date)");
				if substitutions.len() != 1 {
					return Err("Should detect one command substitution".into());
				}
				
				let (start, end, command) = &substitutions[0];
				if *start != 5 || *end != 12 {
					return Err("Command substitution position incorrect".into());
				}
				if command != "date" {
					return Err("Command substitution command incorrect".into());
				}
				Ok(())
			},
			"test_command_substitution_detection",
			"substitution",
			"Tests command substitution detection"
		));
		
		results
	}
	
	/**
	 * Runs all heredoc tests
	 * 
	 * @return Vec<TestResult> - Heredoc test results
	 */
	fn run_heredoc_tests(&self) -> Vec<TestResult> {
		/**
		 * ヒアドキュメントテスト実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なヒアドキュメントテストを行います。
		 * ヒアドキュメント機能の検証が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		let mut results = Vec::new();
		
		// Test heredoc state creation
		results.push(self.run_single_test(
			|| {
				use sare::gui::heredoc::HeredocState;
				
				let state = HeredocState::default();
				if state.is_heredoc() {
					return Err("New heredoc state should not be in heredoc mode".into());
				}
				if !state.heredoc_delimiter.is_empty() {
					return Err("New heredoc state should have empty delimiter".into());
				}
				Ok(())
			},
			"test_heredoc_state_creation",
			"heredoc",
			"Tests heredoc state initialization"
		));
		
		// Test heredoc detection
		results.push(self.run_single_test(
			|| {
				use sare::gui::heredoc::HeredocProcessor;
				
				let heredoc_result = HeredocProcessor::detect_heredoc("cat << EOF\nhello\nEOF");
				if heredoc_result.is_none() {
					return Err("Should detect one heredoc".into());
				}
				
				let (delimiter, expand_vars) = heredoc_result.unwrap();
				if delimiter != "EOF" {
					return Err("Heredoc delimiter incorrect".into());
				}
				Ok(())
			},
			"test_heredoc_detection",
			"heredoc",
			"Tests heredoc detection"
		));
		
		results
	}
	
	/**
	 * Runs all multiline tests
	 * 
	 * @return Vec<TestResult> - Multiline test results
	 */
	fn run_multiline_tests(&self) -> Vec<TestResult> {
		/**
		 * マルチラインテスト実行の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なマルチラインテストを行います。
		 * マルチライン機能の検証が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut results = Vec::new();
		
		// Test multiline state creation
		results.push(self.run_single_test(
			|| {
				use sare::gui::multiline::MultilineState;
				
				let state = MultilineState::default();
				if state.is_multiline() {
					return Err("New multiline state should not be in multiline mode".into());
				}
				if state.continuation_char.is_some() {
					return Err("New multiline state should have no continuation char".into());
				}
				Ok(())
			},
			"test_multiline_state_creation",
			"multiline",
			"Tests multiline state initialization"
		));
		
		// Test multiline detection
		results.push(self.run_single_test(
			|| {
				use sare::gui::multiline::MultilineProcessor;
				
				let (is_multiline, continuation_char) = MultilineProcessor::check_multiline_continuation("echo \\\nhello");
				if !is_multiline {
					return Err("Should detect multiline command".into());
				}
				if continuation_char != Some('\\') {
					return Err("Continuation char should be backslash".into());
				}
				Ok(())
			},
			"test_multiline_detection",
			"multiline",
			"Tests multiline detection"
		));
		
		results
	}
	
	/**
	 * Runs all tests and generates comprehensive report
	 * 
	 * @return TestSuiteSummary - Complete test suite summary
	 */
	pub fn run_all_tests(&self) -> TestSuiteSummary {
		/**
		 * 全テスト実行の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑なテスト実行を行います。
		 * 複数カテゴリのテスト管理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		println!("🌸 Starting Sare Terminal Test Suite 🌸");
		println!("==========================================");
		
		let start_time = Instant::now();
		let mut all_results = Vec::new();
		
		// Run completion tests
		println!("\n📝 Running Completion Tests...");
		let completion_results = self.run_completion_tests();
		all_results.extend(completion_results);
		
		// Run history tests
		println!("\n📚 Running History Tests...");
		let history_results = self.run_history_tests();
		all_results.extend(history_results);
		
		// Run expansion tests
		println!("\n🔍 Running Expansion Tests...");
		let expansion_results = self.run_expansion_tests();
		all_results.extend(expansion_results);
		
		// Run substitution tests
		println!("\n🔄 Running Substitution Tests...");
		let substitution_results = self.run_substitution_tests();
		all_results.extend(substitution_results);
		
		// Run heredoc tests
		println!("\n📄 Running Heredoc Tests...");
		let heredoc_results = self.run_heredoc_tests();
		all_results.extend(heredoc_results);
		
		// Run multiline tests
		println!("\n📋 Running Multiline Tests...");
		let multiline_results = self.run_multiline_tests();
		all_results.extend(multiline_results);
		
		let total_time = start_time.elapsed();
		
		// Generate summary
		let mut results_by_category = HashMap::new();
		let mut failed_tests_details = Vec::new();
		let mut passed_tests = 0;
		let mut failed_tests = 0;
		
		for result in &all_results {
			let category_stats = results_by_category.entry(result.category.clone()).or_insert((0, 0));
			
			if result.success {
				passed_tests += 1;
				category_stats.0 += 1;
			} else {
				failed_tests += 1;
				category_stats.1 += 1;
				failed_tests_details.push(result.clone());
			}
		}
		
		let summary = TestSuiteSummary {
			total_tests: all_results.len(),
			passed_tests,
			failed_tests,
			total_time,
			results_by_category,
			failed_tests_details,
		};
		
		// Print detailed results
		self.print_detailed_results(&all_results, &summary);
		
		summary
	}
	
	/**
	 * Prints detailed test results with formatting
	 * 
	 * @param results - All test results
	 * @param summary - Test suite summary
	 */
	fn print_detailed_results(&self, results: &[TestResult], summary: &TestSuiteSummary) {
		println!("\n🌸 Test Results Summary 🌸");
		println!("==========================");
		println!("Total Tests: {}", summary.total_tests);
		println!("✅ Passed: {}", summary.passed_tests);
		println!("❌ Failed: {}", summary.failed_tests);
		println!("⏱️  Total Time: {:.2}s", summary.total_time.as_secs_f64());
		
		println!("\n📊 Results by Category:");
		for (category, (passed, failed)) in &summary.results_by_category {
			let total = passed + failed;
			let pass_rate = if total > 0 { (*passed as f64 / total as f64) * 100.0 } else { 0.0 };
			println!("  {}: {}/{} passed ({:.1}%)", category, passed, total, pass_rate);
		}
		
		if !summary.failed_tests_details.is_empty() {
			println!("\n❌ Failed Tests Details:");
			for result in &summary.failed_tests_details {
				println!("  🔴 {} ({}): {}", result.name, result.category, result.description);
				if let Some(error) = &result.error_message {
					println!("     Error: {}", error);
				}
				println!("     Time: {}ms", result.execution_time);
			}
		}
		
		println!("\n✅ Passed Tests:");
		for result in results {
			if result.success {
				println!("  🟢 {} ({}): {} - {}ms", 
					result.name, result.category, result.description, result.execution_time);
			}
		}
		
		// Final summary
		if summary.failed_tests == 0 {
			println!("\n🎉 All tests passed! 🎉");
		} else {
			println!("\n⚠️  {} tests failed. Please review the details above.", summary.failed_tests);
		}
	}
}

/**
 * Main function to run all tests
 * 
 * Creates a test runner and executes all tests with
 * comprehensive reporting and detailed output.
 */
fn main() {
	/**
	 * メイン関数の複雑な処理です (｡◕‿◕｡)
	 * 
	 * この関数は複雑なテスト実行を行います。
	 * 全テストの統合実行が難しい部分なので、
	 * 適切なエラーハンドリングで実装しています (◕‿◕)
	 */
	
	let runner = TestRunner::new();
	let summary = runner.run_all_tests();
	
	// Exit with appropriate code
	if summary.failed_tests == 0 {
		std::process::exit(0);
	} else {
		std::process::exit(1);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_runner_creation() {
		let runner = TestRunner::new();
		assert!(runner.results.lock().unwrap().is_empty());
	}
	
	#[test]
	fn test_single_test_execution() {
		let runner = TestRunner::new();
		let result = runner.run_single_test(
			|| Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()),
			"test_success",
			"test",
			"Test successful execution"
		);
		
		assert!(result.success);
		assert_eq!(result.name, "test_success");
		assert_eq!(result.category, "test");
	}
	
	#[test]
	fn test_failed_test_execution() {
		let runner = TestRunner::new();
		let result = runner.run_single_test(
			|| Err::<(), Box<dyn std::error::Error + Send + Sync>>("Test error".into()),
			"test_failure",
			"test",
			"Test failed execution"
		);
		
		assert!(!result.success);
		assert!(result.error_message.is_some());
		assert_eq!(result.name, "test_failure");
	}
} 