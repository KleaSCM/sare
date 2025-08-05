/**
 * Terminal features tests
 * 
 * This module provides comprehensive tests for the terminal features
 * system, verifying that all feature managers work correctly after
 * the rename from advanced to features.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: test_features.rs
 * Description: Tests for terminal features system
 */

use sare_terminal::features::{TerminalFeatures, TerminalFeaturesStatus};

/**
 * Test terminal features creation
 */
#[test]
fn test_terminal_features_creation() {
	let features = TerminalFeatures::new();
	assert!(features.is_ok());
	
	let features = features.unwrap();
	assert!(features.image_manager().is_some());
	assert!(features.hyperlink_manager().is_some());
	assert!(features.semantic_manager().is_some());
	assert!(features.search_manager().is_some());
	assert!(features.selection_manager().is_some());
	assert!(features.paste_protection_manager().is_some());
	assert!(features.input_method_manager().is_some());
}

/**
 * Test terminal features initialization
 */
#[tokio::test]
async fn test_terminal_features_initialization() {
	let features = TerminalFeatures::new().unwrap();
	
	// Test initialization
	let result = features.initialize().await;
	assert!(result.is_ok());
	
	// Test shutdown
	let result = features.shutdown().await;
	assert!(result.is_ok());
}

/**
 * Test terminal features status
 */
#[tokio::test]
async fn test_terminal_features_status() {
	let features = TerminalFeatures::new().unwrap();
	
	// Initialize features
	features.initialize().await.unwrap();
	
	// Get status
	let status = features.get_status().await;
	assert!(status.is_ok());
	
	let status = status.unwrap();
	assert_eq!(status.image_count, 0);
	assert_eq!(status.hyperlink_count, 0);
	assert_eq!(status.semantic_count, 0);
	assert_eq!(status.search_count, 0);
	assert_eq!(status.selection_count, 0);
	assert_eq!(status.paste_protection_count, 0);
	assert_eq!(status.input_method_count, 0);
}

/**
 * Test terminal features manager access
 */
#[test]
fn test_terminal_features_manager_access() {
	let features = TerminalFeatures::new().unwrap();
	
	// Test image manager access
	let image_manager = features.image_manager();
	assert!(image_manager.is_some());
	
	// Test hyperlink manager access
	let hyperlink_manager = features.hyperlink_manager();
	assert!(hyperlink_manager.is_some());
	
	// Test semantic manager access
	let semantic_manager = features.semantic_manager();
	assert!(semantic_manager.is_some());
	
	// Test search manager access
	let search_manager = features.search_manager();
	assert!(search_manager.is_some());
	
	// Test selection manager access
	let selection_manager = features.selection_manager();
	assert!(selection_manager.is_some());
	
	// Test paste protection manager access
	let paste_protection_manager = features.paste_protection_manager();
	assert!(paste_protection_manager.is_some());
	
	// Test input method manager access
	let input_method_manager = features.input_method_manager();
	assert!(input_method_manager.is_some());
}

/**
 * Test terminal features status structure
 */
#[test]
fn test_terminal_features_status_structure() {
	let status = TerminalFeaturesStatus {
		image_count: 5,
		hyperlink_count: 10,
		semantic_count: 15,
		search_count: 3,
		selection_count: 2,
		paste_protection_count: 1,
		input_method_count: 4,
		timestamp: chrono::Utc::now(),
	};
	
	assert_eq!(status.image_count, 5);
	assert_eq!(status.hyperlink_count, 10);
	assert_eq!(status.semantic_count, 15);
	assert_eq!(status.search_count, 3);
	assert_eq!(status.selection_count, 2);
	assert_eq!(status.paste_protection_count, 1);
	assert_eq!(status.input_method_count, 4);
}

/**
 * Run all terminal features tests
 */
pub fn run_terminal_features_tests() -> Vec<(&'static str, bool)> {
	let mut results = Vec::new();
	
	let tests = vec![
		("test_terminal_features_creation", test_terminal_features_creation),
		("test_terminal_features_initialization", test_terminal_features_initialization),
		("test_terminal_features_status", test_terminal_features_status),
		("test_terminal_features_manager_access", test_terminal_features_manager_access),
		("test_terminal_features_status_structure", test_terminal_features_status_structure),
	];
	
	for (name, test_fn) in tests {
		let result = std::panic::catch_unwind(test_fn).is_ok();
		results.push((name, result));
	}
	
	results
} 