/**
 * History navigation functionality for Sare terminal
 * 
 * This module provides history navigation features including
 * up/down navigation, reverse search, and state management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: navigation.rs
 * Description: History navigation with proper state management
 */

use anyhow::Result;
use super::{HistoryManager, HistoryEntry};

/**
 * History navigation state
 * 
 * Manages the current state of history navigation
 * including search mode, query, and original input.
 */
#[derive(Debug, Clone)]
pub struct HistoryNavigationState {
	/// Current history index
	pub history_index: Option<usize>,
	/// Whether in search mode
	pub search_mode: bool,
	/// Current search query
	pub search_query: String,
	/// Original input before navigation
	pub original_input: String,
}

impl Default for HistoryNavigationState {
	fn default() -> Self {
		Self {
			history_index: None,
			search_mode: false,
			search_query: String::new(),
			original_input: String::new(),
		}
	}
}

/**
 * History navigator that handles navigation logic
 * 
 * Provides functionality for navigating through command history
 * with proper state management and search capabilities.
 */
pub struct HistoryNavigator {
	/// History manager
	history_manager: HistoryManager,
	/// Navigation state
	state: HistoryNavigationState,
}

impl HistoryNavigator {
	/**
	 * Creates a new history navigator
	 * 
	 * @param history_manager - History manager instance
	 * @return HistoryNavigator - New history navigator
	 */
	pub fn new(history_manager: HistoryManager) -> Self {
		Self {
			history_manager,
			state: HistoryNavigationState::default(),
		}
	}
	
	/**
	 * Navigates history up (older commands)
	 * 
	 * @param current_input - Current input to save if first navigation
	 * @return Option<String> - Command from history or None
	 */
	pub fn navigate_up(&mut self, current_input: &str) -> Option<String> {
		/**
		 * 履歴を上に移動する関数です
		 * 
		 * コマンド履歴を古い順に移動して、指定されたインデックスの
		 * コマンドを返します。初回移動時は現在の入力を保存します。
		 * 
		 * 履歴が空の場合はNoneを返し、インデックスが範囲外の場合は
		 * 適切に調整して履歴内のコマンドを返します
		 */
		
		let history = self.history_manager.get_history();
		if history.is_empty() {
			return None;
		}
		
		let current_index = self.state.history_index.unwrap_or(history.len());
		
		if current_index > 0 {
			self.state.history_index = Some(current_index - 1);
			
			if self.state.original_input.is_empty() {
				self.state.original_input = current_input.to_string();
			}
			
			if let Some(entry) = history.get(current_index - 1) {
				return Some(entry.command.clone());
			}
		}
		
		None
	}
	
	/**
	 * Navigates history down (newer commands)
	 * 
	 * @return Option<String> - Command from history or original input
	 */
	pub fn navigate_down(&mut self) -> Option<String> {
		let history = self.history_manager.get_history();
		if history.is_empty() {
			return None;
		}
		
		let current_index = self.state.history_index.unwrap_or(history.len());
		
		if current_index < history.len() - 1 {
			self.state.history_index = Some(current_index + 1);
			if let Some(entry) = history.get(current_index + 1) {
				return Some(entry.command.clone());
			}
		} else {
			self.state.history_index = None;
			let original = self.state.original_input.clone();
			self.state.original_input.clear();
			return Some(original);
		}
		
		None
	}
	
	/**
	 * Starts reverse incremental search (Ctrl+R)
	 * 
	 * @param current_input - Current input to save
	 */
	pub fn start_reverse_search(&mut self, current_input: &str) {
		self.state.search_mode = true;
		self.state.search_query.clear();
		self.state.history_index = None;
		self.state.original_input = current_input.to_string();
	}
	
	/**
	 * Performs reverse incremental search
	 * 
	 * @param query - Search query to add
	 * @return Option<String> - Matching command or None
	 */
	pub fn perform_reverse_search(&mut self, query: &str) -> Option<String> {
		self.state.search_query.push_str(query);
		
		let history = self.history_manager.get_history();
		let search_query = &self.state.search_query;
		
		if search_query.is_empty() {
			return None;
		}
		
		for (i, entry) in history.iter().enumerate().rev() {
			if entry.command.contains(search_query) {
				self.state.history_index = Some(i);
				return Some(entry.command.clone());
			}
		}
		
		None
	}
	
	/**
	 * Exits history search mode
	 * 
	 * @return String - Original input to restore
	 */
	pub fn exit_search(&mut self) -> String {
		self.state.search_mode = false;
		self.state.search_query.clear();
		self.state.history_index = None;
		
		let original = self.state.original_input.clone();
		self.state.original_input.clear();
		original
	}
	
	/**
	 * Adds a command to history
	 * 
	 * @param command - Command to add
	 * @param exit_code - Exit code of the command
	 */
	pub fn add_command(&mut self, command: String, exit_code: Option<i32>) {
		self.history_manager.add_command(command, exit_code);
		self.reset_navigation();
	}
	
	/**
	 * Resets navigation state
	 */
	pub fn reset_navigation(&mut self) {
		self.state.history_index = None;
		self.state.search_mode = false;
		self.state.search_query.clear();
		self.state.original_input.clear();
	}
	
	/**
	 * Gets history display for history command
	 * 
	 * @return String - Formatted history display
	 */
	pub fn get_history_display(&self) -> String {
		let history = self.history_manager.get_history();
		let mut display = String::new();
		
		for (i, entry) in history.iter().enumerate() {
			display.push_str(&format!("{:4}  {}\n", i + 1, entry.command));
		}
		
		display
	}
	
	/**
	 * Gets current navigation state
	 * 
	 * @return &HistoryNavigationState - Current navigation state
	 */
	pub fn state(&self) -> &HistoryNavigationState {
		&self.state
	}
	
	/**
	 * Gets history manager reference
	 * 
	 * @return &HistoryManager - History manager reference
	 */
	pub fn history_manager(&self) -> &HistoryManager {
		&self.history_manager
	}
} 