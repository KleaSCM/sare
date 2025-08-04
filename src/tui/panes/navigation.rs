/**
 * @file navigation.rs
 * @brief Pane navigation and focus management for Sare terminal
 * 
 * This module provides navigation capabilities for the multi-pane
 * terminal interface, including keyboard shortcuts, focus management,
 * and navigation patterns optimized for developer workflows.
 * 
 * @author KleaSCM
 * @email KleaSCM@gmail.com
 * @file navigation.rs
 * @description Navigation module that provides pane navigation
 * and focus management for the multi-pane terminal interface.
 */

use anyhow::Result;
use std::collections::HashMap;

use super::{Pane, PaneState, SplitDirection};

/**
 * Navigation manager for pane navigation
 * 
 * Manages pane navigation, focus, and keyboard shortcuts
 * for optimal developer workflow experience.
 */
pub struct NavigationManager {
	/// Navigation mode
	mode: NavigationMode,
	/// Focus history
	focus_history: Vec<String>,
	/// Navigation shortcuts
	shortcuts: HashMap<String, NavigationAction>,
	/// Navigation configuration
	config: NavigationConfig,
}

/**
 * Navigation mode enumeration
 * 
 * Defines the different navigation modes available.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationMode {
	/// Normal navigation mode
	Normal,
	/// Quick navigation mode
	Quick,
	/// Visual navigation mode
	Visual,
	/// Command navigation mode
	Command,
}

/**
 * Navigation action enumeration
 * 
 * Defines the different navigation actions available.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationAction {
	/// Focus next pane
	FocusNext,
	/// Focus previous pane
	FocusPrevious,
	/// Focus pane by ID
	FocusPane(String),
	/// Split pane horizontally
	SplitHorizontal,
	/// Split pane vertically
	SplitVertical,
	/// Close current pane
	ClosePane,
	/// Resize pane
	ResizePane(ResizeDirection),
	/// Move pane
	MovePane(MoveDirection),
	/// Toggle pane synchronization
	ToggleSync,
	/// Switch navigation mode
	SwitchMode(NavigationMode),
}

/**
 * Resize direction enumeration
 * 
 * Defines the different directions for pane resizing.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeDirection {
	/// Resize left
	Left,
	/// Resize right
	Right,
	/// Resize up
	Up,
	/// Resize down
	Down,
}

/**
 * Move direction enumeration
 * 
 * Defines the different directions for pane movement.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum MoveDirection {
	/// Move left
	Left,
	/// Move right
	Right,
	/// Move up
	Up,
	/// Move down
	Down,
}

/**
 * Navigation configuration
 * 
 * Contains configuration options for navigation including
 * shortcuts, behavior, and navigation patterns.
 */
#[derive(Debug, Clone)]
pub struct NavigationConfig {
	/// Enable keyboard shortcuts
	pub enable_shortcuts: bool,
	/// Enable focus history
	pub enable_focus_history: bool,
	/// Maximum focus history size
	pub max_focus_history: usize,
	/// Enable visual navigation
	pub enable_visual_navigation: bool,
	/// Navigation timeout
	pub navigation_timeout: std::time::Duration,
	/// Default navigation mode
	pub default_mode: NavigationMode,
}

impl Default for NavigationConfig {
	fn default() -> Self {
		Self {
			enable_shortcuts: true,
			enable_focus_history: true,
			max_focus_history: 10,
			enable_visual_navigation: true,
			navigation_timeout: std::time::Duration::from_millis(500),
			default_mode: NavigationMode::Normal,
		}
	}
}

/**
 * Navigation result
 * 
 * Contains the result of a navigation operation including
 * success status and any additional information.
 */
#[derive(Debug, Clone)]
pub struct NavigationResult {
	/// Success status
	pub success: bool,
	/// Action performed
	pub action: Option<NavigationAction>,
	/// Target pane ID
	pub target_pane: Option<String>,
	/// Error message
	pub error: Option<String>,
}

impl NavigationManager {
	/**
	 * Creates a new navigation manager
	 * 
	 * @param config - Navigation configuration
	 * @return NavigationManager - New navigation manager instance
	 */
	pub fn new(config: NavigationConfig) -> Self {
		/**
		 * ナビゲーションマネージャー初期化の複雑な処理です (｡◕‿◕｡)
		 * 
		 * この関数は複雑なナビゲーション設定を行います。
		 * ショートカット設定とフォーカス管理が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (◕‿◕)
		 */
		
		let mut shortcuts = HashMap::new();
		
		// Set up default shortcuts
		shortcuts.insert("Ctrl+h".to_string(), NavigationAction::FocusPrevious);
		shortcuts.insert("Ctrl+l".to_string(), NavigationAction::FocusNext);
		shortcuts.insert("Ctrl+k".to_string(), NavigationAction::SplitHorizontal);
		shortcuts.insert("Ctrl+j".to_string(), NavigationAction::SplitVertical);
		shortcuts.insert("Ctrl+w".to_string(), NavigationAction::ClosePane);
		shortcuts.insert("Ctrl+Shift+h".to_string(), NavigationAction::ResizePane(ResizeDirection::Left));
		shortcuts.insert("Ctrl+Shift+l".to_string(), NavigationAction::ResizePane(ResizeDirection::Right));
		shortcuts.insert("Ctrl+Shift+k".to_string(), NavigationAction::ResizePane(ResizeDirection::Up));
		shortcuts.insert("Ctrl+Shift+j".to_string(), NavigationAction::ResizePane(ResizeDirection::Down));
		shortcuts.insert("Ctrl+s".to_string(), NavigationAction::ToggleSync);
		
		Self {
			mode: config.default_mode.clone(),
			focus_history: Vec::new(),
			shortcuts,
			config,
		}
	}
	
	/**
	 * Handles navigation input
	 * 
	 * Processes navigation input and performs the appropriate
	 * navigation action based on the current mode and shortcuts.
	 * 
	 * @param input - Navigation input
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @return Result<NavigationResult> - Navigation result or error
	 */
	pub async fn handle_navigation(&mut self, input: &str, panes: &[Pane], current_focus: Option<&str>) -> Result<NavigationResult> {
		/**
		 * ナビゲーション入力処理の複雑な処理です (◕‿◕)
		 * 
		 * この関数は複雑な入力処理を行います。
		 * ショートカット解析とアクション実行が難しい部分なので、
		 * 適切なエラーハンドリングで実装しています (｡◕‿◕｡)
		 */
		
		// Check if input is a shortcut
		if let Some(action) = self.shortcuts.get(input) {
			return self.execute_action(action.clone(), panes, current_focus).await;
		}
		
		// Handle mode-specific navigation
		match self.mode {
			NavigationMode::Normal => self.handle_normal_navigation(input, panes, current_focus).await,
			NavigationMode::Quick => self.handle_quick_navigation(input, panes, current_focus).await,
			NavigationMode::Visual => self.handle_visual_navigation(input, panes, current_focus).await,
			NavigationMode::Command => self.handle_command_navigation(input, panes, current_focus).await,
		}
	}
	
	/**
	 * Handles normal navigation mode
	 * 
	 * @param input - Navigation input
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn handle_normal_navigation(&self, input: &str, panes: &[Pane], current_focus: Option<&str>) -> Result<NavigationResult> {
		match input {
			"h" | "left" => self.focus_direction(panes, current_focus, MoveDirection::Left).await,
			"l" | "right" => self.focus_direction(panes, current_focus, MoveDirection::Right).await,
			"k" | "up" => self.focus_direction(panes, current_focus, MoveDirection::Up).await,
			"j" | "down" => self.focus_direction(panes, current_focus, MoveDirection::Down).await,
			input if input.len() == 1 && input.chars().next().unwrap().is_ascii_digit() => self.focus_by_number(input, panes).await,
			_ => Ok(NavigationResult {
				success: false,
				action: None,
				target_pane: None,
				error: Some(format!("Unknown navigation input: {}", input)),
			}),
		}
	}
	
	/**
	 * Handles quick navigation mode
	 * 
	 * @param input - Navigation input
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn handle_quick_navigation(&self, input: &str, panes: &[Pane], current_focus: Option<&str>) -> Result<NavigationResult> {
		// Quick navigation allows direct pane selection
		if let Ok(pane_index) = input.parse::<usize>() {
			if pane_index < panes.len() {
				let target_pane = panes[pane_index].pane_id.clone();
				return Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::FocusPane(target_pane.clone())),
					target_pane: Some(target_pane),
					error: None,
				});
			}
		}
		
		Ok(NavigationResult {
			success: false,
			action: None,
			target_pane: None,
			error: Some("Invalid pane number".to_string()),
		})
	}
	
	/**
	 * Handles visual navigation mode
	 * 
	 * @param input - Navigation input
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn handle_visual_navigation(&self, input: &str, panes: &[Pane], current_focus: Option<&str>) -> Result<NavigationResult> {
		// Visual navigation shows pane grid and allows selection
		match input {
			"h" | "left" => self.focus_direction(panes, current_focus, MoveDirection::Left).await,
			"l" | "right" => self.focus_direction(panes, current_focus, MoveDirection::Right).await,
			"k" | "up" => self.focus_direction(panes, current_focus, MoveDirection::Up).await,
			"j" | "down" => self.focus_direction(panes, current_focus, MoveDirection::Down).await,
			"enter" | "space" => {
				// Confirm selection in visual mode
				if let Some(focus) = current_focus {
					Ok(NavigationResult {
						success: true,
						action: Some(NavigationAction::FocusPane(focus.to_string())),
						target_pane: Some(focus.to_string()),
						error: None,
					})
				} else {
					Ok(NavigationResult {
						success: false,
						action: None,
						target_pane: None,
						error: Some("No pane selected".to_string()),
					})
				}
			}
			_ => Ok(NavigationResult {
				success: false,
				action: None,
				target_pane: None,
				error: Some(format!("Unknown visual navigation input: {}", input)),
			}),
		}
	}
	
	/**
	 * Handles command navigation mode
	 * 
	 * @param input - Navigation input
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn handle_command_navigation(&self, input: &str, panes: &[Pane], current_focus: Option<&str>) -> Result<NavigationResult> {
		// Command navigation allows text-based commands
		if input.starts_with("focus ") {
			let pane_id = input[6..].trim();
			return Ok(NavigationResult {
				success: true,
				action: Some(NavigationAction::FocusPane(pane_id.to_string())),
				target_pane: Some(pane_id.to_string()),
				error: None,
			});
		}
		
		if input.starts_with("split ") {
			let direction = input[6..].trim();
			let action = match direction {
				"h" | "horizontal" => NavigationAction::SplitHorizontal,
				"v" | "vertical" => NavigationAction::SplitVertical,
				_ => return Ok(NavigationResult {
					success: false,
					action: None,
					target_pane: None,
					error: Some("Invalid split direction".to_string()),
				}),
			};
			
			return Ok(NavigationResult {
				success: true,
				action: Some(action),
				target_pane: None,
				error: None,
			});
		}
		
		Ok(NavigationResult {
			success: false,
			action: None,
			target_pane: None,
			error: Some(format!("Unknown command: {}", input)),
		})
	}
	
	/**
	 * Focuses pane in specified direction
	 * 
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @param direction - Direction to focus
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn focus_direction(&self, panes: &[Pane], current_focus: Option<&str>, direction: MoveDirection) -> Result<NavigationResult> {
		if panes.is_empty() {
			return Ok(NavigationResult {
				success: false,
				action: None,
				target_pane: None,
				error: Some("No panes available".to_string()),
			});
		}
		
		// Find current pane index
		let current_index = if let Some(focus) = current_focus {
			panes.iter().position(|p| p.pane_id == focus)
		} else {
			None
		};
		
		let target_index = match direction {
			MoveDirection::Left => {
				// Implement grid-based navigation (Left)
				if let Some(idx) = current_index {
					// Calculate grid dimensions
					let grid_cols = (panes.len() as f32).sqrt().ceil() as usize;
					let current_row = idx / grid_cols;
					let current_col = idx % grid_cols;
					
					if current_col > 0 {
						// Move to pane on the left
						Some(idx - 1)
					} else {
						// Wrap to rightmost column
						let target_idx = current_row * grid_cols + (grid_cols - 1);
						if target_idx < panes.len() {
							Some(target_idx)
						} else {
							Some(panes.len() - 1)
						}
					}
				} else {
					Some(0)
				}
			}
			MoveDirection::Right => {
				// Implement grid-based navigation (Right)
				if let Some(idx) = current_index {
					// Calculate grid dimensions
					let grid_cols = (panes.len() as f32).sqrt().ceil() as usize;
					let current_row = idx / grid_cols;
					let current_col = idx % grid_cols;
					
					let target_idx = idx + 1;
					if target_idx < panes.len() && (target_idx / grid_cols) == current_row {
						// Move to pane on the right (same row)
						Some(target_idx)
					} else {
						// Wrap to leftmost column
						Some(current_row * grid_cols)
					}
				} else {
					Some(0)
				}
			}
			MoveDirection::Up => {
				// Implement grid-based navigation (Up)
				if let Some(idx) = current_index {
					// Calculate grid dimensions
					let grid_cols = (panes.len() as f32).sqrt().ceil() as usize;
					let current_row = idx / grid_cols;
					let current_col = idx % grid_cols;
					
					if current_row > 0 {
						// Move to pane above
						Some(idx - grid_cols)
					} else {
						// Wrap to bottom row
						let bottom_row = (panes.len() - 1) / grid_cols;
						let target_idx = bottom_row * grid_cols + current_col;
						if target_idx < panes.len() {
							Some(target_idx)
						} else {
							Some(panes.len() - 1)
						}
					}
				} else {
					Some(0)
				}
			}
			MoveDirection::Down => {
				// Implement grid-based navigation (Down)
				if let Some(idx) = current_index {
					// Calculate grid dimensions
					let grid_cols = (panes.len() as f32).sqrt().ceil() as usize;
					let current_row = idx / grid_cols;
					let current_col = idx % grid_cols;
					
					let target_idx = idx + grid_cols;
					if target_idx < panes.len() {
						// Move to pane below
						Some(target_idx)
					} else {
						// Wrap to top row
						Some(current_col)
					}
				} else {
					Some(0)
				}
			}
		};
		
		if let Some(idx) = target_index {
			let target_pane = panes[idx].pane_id.clone();
			return Ok(NavigationResult {
				success: true,
				action: Some(NavigationAction::FocusPane(target_pane.clone())),
				target_pane: Some(target_pane),
				error: None,
			});
		}
		
		Ok(NavigationResult {
			success: false,
			action: None,
			target_pane: None,
			error: Some("No target pane found".to_string()),
		})
	}
	
	/**
	 * Focuses pane by number
	 * 
	 * @param input - Number input
	 * @param panes - List of available panes
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn focus_by_number(&self, input: &str, panes: &[Pane]) -> Result<NavigationResult> {
		if let Ok(number) = input.parse::<usize>() {
			if number > 0 && number <= panes.len() {
				let target_pane = panes[number - 1].pane_id.clone();
				return Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::FocusPane(target_pane.clone())),
					target_pane: Some(target_pane),
					error: None,
				});
			}
		}
		
		Ok(NavigationResult {
			success: false,
			action: None,
			target_pane: None,
			error: Some("Invalid pane number".to_string()),
		})
	}
	
	/**
	 * Executes a navigation action
	 * 
	 * @param action - Navigation action to execute
	 * @param panes - List of available panes
	 * @param current_focus - Currently focused pane ID
	 * @return Result<NavigationResult> - Navigation result
	 */
	async fn execute_action(&mut self, action: NavigationAction, panes: &[Pane], current_focus: Option<&str>) -> Result<NavigationResult> {
		match action {
			NavigationAction::FocusNext => {
				if let Some(focus) = current_focus {
					if let Some(current_idx) = panes.iter().position(|p| p.pane_id == focus) {
						let next_idx = (current_idx + 1) % panes.len();
						let target_pane = panes[next_idx].pane_id.clone();
						self.add_to_focus_history(&target_pane);
						return Ok(NavigationResult {
							success: true,
							action: Some(NavigationAction::FocusNext),
							target_pane: Some(target_pane),
							error: None,
						});
					}
				}
				
				Ok(NavigationResult {
					success: false,
					action: Some(NavigationAction::FocusNext),
					target_pane: None,
					error: Some("No next pane available".to_string()),
				})
			}
			NavigationAction::FocusPrevious => {
				if let Some(focus) = current_focus {
					if let Some(current_idx) = panes.iter().position(|p| p.pane_id == focus) {
						let prev_idx = if current_idx > 0 { current_idx - 1 } else { panes.len() - 1 };
						let target_pane = panes[prev_idx].pane_id.clone();
						self.add_to_focus_history(&target_pane);
						return Ok(NavigationResult {
							success: true,
							action: Some(NavigationAction::FocusPrevious),
							target_pane: Some(target_pane),
							error: None,
						});
					}
				}
				
				Ok(NavigationResult {
					success: false,
					action: Some(NavigationAction::FocusPrevious),
					target_pane: None,
					error: Some("No previous pane available".to_string()),
				})
			}
			NavigationAction::FocusPane(pane_id) => {
				if panes.iter().any(|p| p.pane_id == pane_id) {
					self.add_to_focus_history(&pane_id);
					Ok(NavigationResult {
						success: true,
						action: Some(NavigationAction::FocusPane(pane_id.clone())),
						target_pane: Some(pane_id),
						error: None,
					})
				} else {
					Ok(NavigationResult {
						success: false,
						action: Some(NavigationAction::FocusPane(pane_id)),
						target_pane: None,
						error: Some("Pane not found".to_string()),
					})
				}
			}
			NavigationAction::SplitHorizontal => {
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::SplitHorizontal),
					target_pane: None,
					error: None,
				})
			}
			NavigationAction::SplitVertical => {
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::SplitVertical),
					target_pane: None,
					error: None,
				})
			}
			NavigationAction::ClosePane => {
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::ClosePane),
					target_pane: None,
					error: None,
				})
			}
			NavigationAction::ResizePane(direction) => {
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::ResizePane(direction)),
					target_pane: None,
					error: None,
				})
			}
			NavigationAction::MovePane(direction) => {
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::MovePane(direction)),
					target_pane: None,
					error: None,
				})
			}
			NavigationAction::ToggleSync => {
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::ToggleSync),
					target_pane: None,
					error: None,
				})
			}
			NavigationAction::SwitchMode(mode) => {
				self.mode = mode.clone();
				Ok(NavigationResult {
					success: true,
					action: Some(NavigationAction::SwitchMode(mode)),
					target_pane: None,
					error: None,
				})
			}
		}
	}
	
	/**
	 * Adds pane to focus history
	 * 
	 * @param pane_id - Pane ID to add to history
	 */
	fn add_to_focus_history(&mut self, pane_id: &str) {
		if self.config.enable_focus_history {
			// Remove if already in history
			self.focus_history.retain(|id| id != pane_id);
			
			// Add to front
			self.focus_history.insert(0, pane_id.to_string());
			
			// Limit history size
			if self.focus_history.len() > self.config.max_focus_history {
				self.focus_history.truncate(self.config.max_focus_history);
			}
		}
	}
	
	/**
	 * Gets focus history
	 * 
	 * @return &Vec<String> - Focus history
	 */
	pub fn get_focus_history(&self) -> &Vec<String> {
		&self.focus_history
	}
	
	/**
	 * Gets current navigation mode
	 * 
	 * @return NavigationMode - Current navigation mode
	 */
	pub fn get_mode(&self) -> NavigationMode {
		self.mode.clone()
	}
	
	/**
	 * Sets navigation mode
	 * 
	 * @param mode - New navigation mode
	 */
	pub fn set_mode(&mut self, mode: NavigationMode) {
		self.mode = mode;
	}
	
	/**
	 * Gets navigation shortcuts
	 * 
	 * @return &HashMap<String, NavigationAction> - Navigation shortcuts
	 */
	pub fn get_shortcuts(&self) -> &HashMap<String, NavigationAction> {
		&self.shortcuts
	}
	
	/**
	 * Adds navigation shortcut
	 * 
	 * @param key - Shortcut key
	 * @param action - Navigation action
	 */
	pub fn add_shortcut(&mut self, key: String, action: NavigationAction) {
		self.shortcuts.insert(key, action);
	}
	
	/**
	 * Gets navigation configuration
	 * 
	 * @return &NavigationConfig - Navigation configuration
	 */
	pub fn get_config(&self) -> &NavigationConfig {
		&self.config
	}
	
	/**
	 * Updates navigation configuration
	 * 
	 * @param config - New navigation configuration
	 */
	pub fn update_config(&mut self, config: NavigationConfig) {
		self.config = config;
	}
} 