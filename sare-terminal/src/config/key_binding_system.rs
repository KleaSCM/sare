/**
 * Key binding system for Sare terminal
 * 
 * This module provides comprehensive key binding capabilities including
 * customizable shortcuts, key binding configuration, and dynamic binding.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: key_binding_system.rs
 * Description: Key binding system with customizable shortcuts
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/**
 * Key combination
 * 
 * キーコンビネーションです。
 * 複数のキーの組み合わせを
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyCombination {
	/// Primary key
	pub key: String,
	/// Modifier keys
	pub modifiers: Vec<KeyModifier>,
	/// Key state
	pub state: KeyState,
}

impl KeyCombination {
	/**
	 * Creates a new key combination
	 * 
	 * @param key - Primary key
	 * @param modifiers - Modifier keys
	 * @return KeyCombination - New key combination
	 */
	pub fn new(key: &str, modifiers: Vec<KeyModifier>) -> Self {
		Self {
			key: key.to_string(),
			modifiers,
			state: KeyState::Pressed,
		}
	}
	
	/**
	 * Creates a key combination from string
	 * 
	 * @param key_str - Key string (e.g., "Ctrl+C", "Alt+F4")
	 * @return Result<KeyCombination> - Parsed key combination
	 */
	pub fn from_string(key_str: &str) -> Result<Self> {
		let parts: Vec<&str> = key_str.split('+').collect();
		if parts.is_empty() {
			return Err(anyhow::anyhow!("Invalid key combination: {}", key_str));
		}
		
		let mut modifiers = Vec::new();
		let mut key = String::new();
		
		for (i, part) in parts.iter().enumerate() {
			if i == parts.len() - 1 {
				// Last part is the key
				key = part.to_string();
			} else {
				// Other parts are modifiers
				match part.to_lowercase().as_str() {
					"ctrl" | "control" => modifiers.push(KeyModifier::Ctrl),
					"alt" => modifiers.push(KeyModifier::Alt),
					"shift" => modifiers.push(KeyModifier::Shift),
					"super" | "cmd" | "command" => modifiers.push(KeyModifier::Super),
					"meta" => modifiers.push(KeyModifier::Meta),
					_ => return Err(anyhow::anyhow!("Unknown modifier: {}", part)),
				}
			}
		}
		
		Ok(Self {
			key,
			modifiers,
			state: KeyState::Pressed,
		})
	}
	
	/**
	 * Converts to string representation
	 * 
	 * @return String - String representation
	 */
	pub fn to_string(&self) -> String {
		let mut parts = Vec::new();
		
		// Add modifiers
		for modifier in &self.modifiers {
			parts.push(match modifier {
				KeyModifier::Ctrl => "Ctrl",
				KeyModifier::Alt => "Alt",
				KeyModifier::Shift => "Shift",
				KeyModifier::Super => "Super",
				KeyModifier::Meta => "Meta",
			}.to_string());
		}
		
		// Add key
		parts.push(self.key.clone());
		
		parts.join("+")
	}
}

/**
 * Key modifier
 * 
 * キーモディファイアです。
 * Ctrl、Alt、Shiftなどの
 * 修飾キーを表します。
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyModifier {
	/// Control key
	Ctrl,
	/// Alt key
	Alt,
	/// Shift key
	Shift,
	/// Super key (Windows key, Command key)
	Super,
	/// Meta key
	Meta,
}

/**
 * Key state
 * 
 * キー状態です。
 * キーの押下状態を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyState {
	/// Key is pressed
	Pressed,
	/// Key is released
	Released,
	/// Key is held down
	Held,
}

/**
 * Key action
 * 
 * キーアクションです。
 * キーが押された時に
 * 実行されるアクションです。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAction {
	/// No action
	None,
	/// Execute command
	Command { command: String, args: Vec<String> },
	/// Switch theme
	SwitchTheme { theme: String },
	/// Create new tab
	NewTab,
	/// Close current tab
	CloseTab,
	/// Switch to next tab
	NextTab,
	/// Switch to previous tab
	PreviousTab,
	/// Split pane horizontally
	SplitHorizontal,
	/// Split pane vertically
	SplitVertical,
	/// Close current pane
	ClosePane,
	/// Focus next pane
	FocusNext,
	/// Focus previous pane
	FocusPrevious,
	/// Zoom in
	ZoomIn,
	/// Zoom out
	ZoomOut,
	/// Reset zoom
	ZoomReset,
	/// Copy selection
	Copy,
	/// Paste
	Paste,
	/// Clear screen
	ClearScreen,
	/// Show help
	ShowHelp,
	/// Quit application
	Quit,
	/// Custom action
	Custom { action: String, data: serde_json::Value },
}

/**
 * Key binding
 * 
 * キーバインドです。
 * キーコンビネーションと
 * アクションの組み合わせです。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
	/// Key combination
	pub combination: KeyCombination,
	/// Action to perform
	pub action: KeyAction,
	/// Description
	pub description: String,
	/// Whether binding is enabled
	pub enabled: bool,
	/// Binding category
	pub category: String,
	/// Binding priority
	pub priority: u32,
}

impl KeyBinding {
	/**
	 * Creates a new key binding
	 * 
	 * @param combination - Key combination
	 * @param action - Action to perform
	 * @param description - Description
	 * @return KeyBinding - New key binding
	 */
	pub fn new(combination: KeyCombination, action: KeyAction, description: &str) -> Self {
		Self {
			combination,
			action,
			description: description.to_string(),
			enabled: true,
			category: "general".to_string(),
			priority: 0,
		}
	}
}

/**
 * Key binding manager
 * 
 * キーバインドマネージャーです。
 * キーバインドの管理、実行を
 * 担当します。
 */
pub struct KeyBindingManager {
	/// Key bindings
	bindings: Arc<RwLock<HashMap<String, KeyBinding>>>,
	/// Binding categories
	categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
	/// Action handlers
	action_handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(&KeyAction) + Send + Sync>>>>,
}

impl KeyBindingManager {
	/**
	 * Creates a new key binding manager
	 * 
	 * @return KeyBindingManager - New key binding manager
	 */
	pub fn new() -> Self {
		let mut manager = Self {
			bindings: Arc::new(RwLock::new(HashMap::new())),
			categories: Arc::new(RwLock::new(HashMap::new())),
			action_handlers: Arc::new(RwLock::new(HashMap::new())),
		};
		
		// Add default bindings
		manager.add_default_bindings();
		
		manager
	}
	
	/**
	 * Adds default key bindings
	 */
	fn add_default_bindings(&mut self) {
		let default_bindings = vec![
			// Terminal control
			("new-tab", "Ctrl+T", KeyAction::NewTab, "Create new tab", "terminal"),
			("close-tab", "Ctrl+W", KeyAction::CloseTab, "Close current tab", "terminal"),
			("next-tab", "Ctrl+Tab", KeyAction::NextTab, "Switch to next tab", "terminal"),
			("prev-tab", "Ctrl+Shift+Tab", KeyAction::PreviousTab, "Switch to previous tab", "terminal"),
			
			// Theme switching
			("next-theme", "Ctrl+Shift+T", KeyAction::SwitchTheme { theme: "next".to_string() }, "Switch to next theme", "theme"),
			("prev-theme", "Ctrl+Shift+R", KeyAction::SwitchTheme { theme: "prev".to_string() }, "Switch to previous theme", "theme"),
			
			// Pane management
			("split-horizontal", "Ctrl+Shift+E", KeyAction::SplitHorizontal, "Split pane horizontally", "pane"),
			("split-vertical", "Ctrl+Shift+O", KeyAction::SplitVertical, "Split pane vertically", "pane"),
			("close-pane", "Ctrl+Shift+W", KeyAction::ClosePane, "Close current pane", "pane"),
			("focus-next", "Ctrl+Shift+Arrow", KeyAction::FocusNext, "Focus next pane", "pane"),
			("focus-prev", "Ctrl+Shift+Arrow", KeyAction::FocusPrevious, "Focus previous pane", "pane"),
			
			// Zoom
			("zoom-in", "Ctrl+Plus", KeyAction::ZoomIn, "Zoom in", "view"),
			("zoom-out", "Ctrl+Minus", KeyAction::ZoomOut, "Zoom out", "view"),
			("zoom-reset", "Ctrl+0", KeyAction::ZoomReset, "Reset zoom", "view"),
			
			// Clipboard
			("copy", "Ctrl+C", KeyAction::Copy, "Copy selection", "clipboard"),
			("paste", "Ctrl+V", KeyAction::Paste, "Paste", "clipboard"),
			
			// Other
			("clear-screen", "Ctrl+L", KeyAction::ClearScreen, "Clear screen", "terminal"),
			("show-help", "F1", KeyAction::ShowHelp, "Show help", "help"),
			("quit", "Ctrl+Q", KeyAction::Quit, "Quit application", "application"),
		];
		
		for (id, key_str, action, description, category) in default_bindings {
			if let Ok(combination) = KeyCombination::from_string(key_str) {
				let binding = KeyBinding {
					combination,
					action,
					description: description.to_string(),
					enabled: true,
					category: category.to_string(),
					priority: 0,
				};
				
				// Add to bindings
				let mut bindings = self.bindings.blocking_write();
				bindings.insert(id.to_string(), binding);
				
				// Add to categories
				let mut categories = self.categories.blocking_write();
				categories.entry(category.to_string())
					.or_insert_with(Vec::new)
					.push(id.to_string());
			}
		}
	}
	
	/**
	 * Adds a key binding
	 * 
	 * @param id - Binding ID
	 * @param binding - Key binding
	 * @return Result<()> - Success or error status
	 */
	pub async fn add_binding(&self, id: &str, binding: KeyBinding) -> Result<()> {
		let mut bindings = self.bindings.write().await;
		bindings.insert(id.to_string(), binding.clone());
		
		// Add to categories
		let mut categories = self.categories.write().await;
		categories.entry(binding.category.clone())
			.or_insert_with(Vec::new)
			.push(id.to_string());
		
		Ok(())
	}
	
	/**
	 * Removes a key binding
	 * 
	 * @param id - Binding ID
	 * @return Result<()> - Success or error status
	 */
	pub async fn remove_binding(&self, id: &str) -> Result<()> {
		let mut bindings = self.bindings.write().await;
		if let Some(binding) = bindings.remove(id) {
			// Remove from categories
			let mut categories = self.categories.write().await;
			if let Some(category_bindings) = categories.get_mut(&binding.category) {
				category_bindings.retain(|binding_id| binding_id != id);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Updates a key binding
	 * 
	 * @param id - Binding ID
	 * @param binding - Updated key binding
	 * @return Result<()> - Success or error status
	 */
	pub async fn update_binding(&self, id: &str, binding: KeyBinding) -> Result<()> {
		let mut bindings = self.bindings.write().await;
		bindings.insert(id.to_string(), binding);
		Ok(())
	}
	
	/**
	 * Gets a key binding
	 * 
	 * @param id - Binding ID
	 * @return Option<KeyBinding> - Key binding
	 */
	pub async fn get_binding(&self, id: &str) -> Option<KeyBinding> {
		let bindings = self.bindings.read().await;
		bindings.get(id).cloned()
	}
	
	/**
	 * Gets all key bindings
	 * 
	 * @return HashMap<String, KeyBinding> - All key bindings
	 */
	pub async fn get_all_bindings(&self) -> HashMap<String, KeyBinding> {
		let bindings = self.bindings.read().await;
		bindings.clone()
	}
	
	/**
	 * Gets bindings by category
	 * 
	 * @param category - Category name
	 * @return Vec<KeyBinding> - Category bindings
	 */
	pub async fn get_bindings_by_category(&self, category: &str) -> Vec<KeyBinding> {
		let bindings = self.bindings.read().await;
		let categories = self.categories.read().await;
		
		if let Some(binding_ids) = categories.get(category) {
			binding_ids.iter()
				.filter_map(|id| bindings.get(id).cloned())
				.collect()
		} else {
			Vec::new()
		}
	}
	
	/**
	 * Finds binding for key combination
	 * 
	 * @param combination - Key combination
	 * @return Option<KeyBinding> - Matching binding
	 */
	pub async fn find_binding(&self, combination: &KeyCombination) -> Option<KeyBinding> {
		let bindings = self.bindings.read().await;
		
		// Find binding with highest priority
		bindings.values()
			.filter(|binding| binding.enabled && binding.combination == *combination)
			.max_by_key(|binding| binding.priority)
			.cloned()
	}
	
	/**
	 * Executes key binding action
	 * 
	 * @param binding - Key binding to execute
	 * @return Result<()> - Success or error status
	 */
	pub async fn execute_binding(&self, binding: &KeyBinding) -> Result<()> {
		// Notify action handlers
		let handlers = self.action_handlers.read().await;
		if let Some(handler) = handlers.get("default") {
			handler(&binding.action);
		}
		
		// Execute action
		match &binding.action {
			KeyAction::None => {
				// Do nothing
			}
			KeyAction::Command { command, args } => {
				println!("🚀 Executing command: {} {:?}", command, args);
			}
			KeyAction::SwitchTheme { theme } => {
				println!("🎨 Switching theme: {}", theme);
			}
			KeyAction::NewTab => {
				println!("📑 Creating new tab");
			}
			KeyAction::CloseTab => {
				println!("📑 Closing current tab");
			}
			KeyAction::NextTab => {
				println!("📑 Switching to next tab");
			}
			KeyAction::PreviousTab => {
				println!("📑 Switching to previous tab");
			}
			KeyAction::SplitHorizontal => {
				println!("📐 Splitting pane horizontally");
			}
			KeyAction::SplitVertical => {
				println!("📐 Splitting pane vertically");
			}
			KeyAction::ClosePane => {
				println!("📐 Closing current pane");
			}
			KeyAction::FocusNext => {
				println!("🎯 Focusing next pane");
			}
			KeyAction::FocusPrevious => {
				println!("🎯 Focusing previous pane");
			}
			KeyAction::ZoomIn => {
				println!("🔍 Zooming in");
			}
			KeyAction::ZoomOut => {
				println!("🔍 Zooming out");
			}
			KeyAction::ZoomReset => {
				println!("🔍 Resetting zoom");
			}
			KeyAction::Copy => {
				println!("📋 Copying selection");
			}
			KeyAction::Paste => {
				println!("📋 Pasting");
			}
			KeyAction::ClearScreen => {
				println!("🧹 Clearing screen");
			}
			KeyAction::ShowHelp => {
				println!("❓ Showing help");
			}
			KeyAction::Quit => {
				println!("👋 Quitting application");
			}
			KeyAction::Custom { action, data } => {
				println!("⚙️ Custom action: {} with data: {:?}", action, data);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Registers an action handler
	 * 
	 * @param name - Handler name
	 * @param handler - Handler function
	 */
	pub async fn register_action_handler<F>(&self, name: &str, handler: F)
	where
		F: Fn(&KeyAction) + Send + Sync + 'static,
	{
		let mut handlers = self.action_handlers.write().await;
		handlers.insert(name.to_string(), Box::new(handler));
	}
	
	/**
	 * Handles key press
	 * 
	 * @param key - Pressed key
	 * @param modifiers - Active modifiers
	 * @return Result<bool> - Whether action was handled
	 */
	pub async fn handle_key_press(&self, key: &str, modifiers: &[KeyModifier]) -> Result<bool> {
		let combination = KeyCombination {
			key: key.to_string(),
			modifiers: modifiers.to_vec(),
			state: KeyState::Pressed,
		};
		
		if let Some(binding) = self.find_binding(&combination).await {
			self.execute_binding(&binding).await?;
			Ok(true)
		} else {
			Ok(false)
		}
	}
	
	/**
	 * Exports bindings to JSON
	 * 
	 * @return Result<String> - JSON string
	 */
	pub async fn export_bindings(&self) -> Result<String> {
		let bindings = self.bindings.read().await;
		Ok(serde_json::to_string_pretty(&*bindings)?)
	}
	
	/**
	 * Imports bindings from JSON
	 * 
	 * @param json - JSON string
	 * @return Result<()> - Success or error status
	 */
	pub async fn import_bindings(&self, json: &str) -> Result<()> {
		let new_bindings: HashMap<String, KeyBinding> = serde_json::from_str(json)?;
		let mut bindings = self.bindings.write().await;
		*bindings = new_bindings;
		Ok(())
	}
	
	/**
	 * Gets binding statistics
	 * 
	 * @return HashMap<String, usize> - Statistics by category
	 */
	pub async fn get_binding_stats(&self) -> HashMap<String, usize> {
		let bindings = self.bindings.read().await;
		let mut stats = HashMap::new();
		
		for binding in bindings.values() {
			*stats.entry(binding.category.clone()).or_insert(0) += 1;
		}
		
		stats
	}
} 