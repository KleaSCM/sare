/**
 * Configuration system for Sare terminal
 * 
 * This module provides comprehensive configuration capabilities including
 * theme engine, key bindings, plugins, and configuration file management.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Configuration system with theme engine and key bindings
 */

pub mod theme_engine;
pub mod plugin_system;
pub mod key_binding_system;
pub mod config_files;
pub mod hot_reload;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use theme_engine::{Theme, ThemeEngine};
use plugin_system::{PluginManager, PluginEvent};
use key_binding_system::{KeyBindingManager, KeyBinding as KeyBindingSystem, KeyCombination, KeyModifier};
use config_files::{ConfigFileManager, ConfigFormat, ConfigSchema, ConfigValidationError};
use hot_reload::{HotReloadWatcher, RuntimeConfigManager, ConfigChangeEvent, FileChangeEvent};

/**
 * Key binding definition
 * 
 * キーバインドの定義です。
 * ショートカットキーの設定を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
	/// Key combination (e.g., "Ctrl+C", "Alt+F4")
	pub key: String,
	/// Action to perform
	pub action: String,
	/// Description of the action
	pub description: String,
	/// Whether the binding is enabled
	pub enabled: bool,
}

/**
 * Plugin configuration
 * 
 * プラグイン設定です。
 * プラグインの読み込みと設定を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
	/// Plugin name
	pub name: String,
	/// Plugin path
	pub path: String,
	/// Plugin enabled status
	pub enabled: bool,
	/// Plugin configuration
	pub config: HashMap<String, serde_json::Value>,
}

/**
 * Main configuration structure
 * 
 * メイン設定構造体です。
 * すべての設定を統合して
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	/// Current theme name
	pub theme: String,
	/// Key bindings
	pub key_bindings: HashMap<String, KeyBinding>,
	/// Plugin configurations
	pub plugins: Vec<PluginConfig>,
	/// Terminal settings
	pub terminal: TerminalConfig,
	/// UI settings
	pub ui: UiConfig,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			theme: "default-dark".to_string(),
			key_bindings: Self::default_key_bindings(),
			plugins: Vec::new(),
			terminal: TerminalConfig::default(),
			ui: UiConfig::default(),
		}
	}
}

impl Config {
	/**
	 * Creates default key bindings
	 * 
	 * @return HashMap<String, KeyBinding> - Default key bindings
	 */
	fn default_key_bindings() -> HashMap<String, KeyBinding> {
		let mut bindings = HashMap::new();
		
		// Terminal control
		bindings.insert("new-tab".to_string(), KeyBinding {
			key: "Ctrl+T".to_string(),
			action: "new-tab".to_string(),
			description: "Create new tab".to_string(),
			enabled: true,
		});
		
		bindings.insert("close-tab".to_string(), KeyBinding {
			key: "Ctrl+W".to_string(),
			action: "close-tab".to_string(),
			description: "Close current tab".to_string(),
			enabled: true,
		});
		
		bindings.insert("next-tab".to_string(), KeyBinding {
			key: "Ctrl+Tab".to_string(),
			action: "next-tab".to_string(),
			description: "Switch to next tab".to_string(),
			enabled: true,
		});
		
		bindings.insert("prev-tab".to_string(), KeyBinding {
			key: "Ctrl+Shift+Tab".to_string(),
			action: "prev-tab".to_string(),
			description: "Switch to previous tab".to_string(),
			enabled: true,
		});
		
		// Theme switching
		bindings.insert("next-theme".to_string(), KeyBinding {
			key: "Ctrl+Shift+T".to_string(),
			action: "next-theme".to_string(),
			description: "Switch to next theme".to_string(),
			enabled: true,
		});
		
		bindings.insert("prev-theme".to_string(), KeyBinding {
			key: "Ctrl+Shift+R".to_string(),
			action: "prev-theme".to_string(),
			description: "Switch to previous theme".to_string(),
			enabled: true,
		});
		
		// Pane management
		bindings.insert("split-horizontal".to_string(), KeyBinding {
			key: "Ctrl+Shift+E".to_string(),
			action: "split-horizontal".to_string(),
			description: "Split pane horizontally".to_string(),
			enabled: true,
		});
		
		bindings.insert("split-vertical".to_string(), KeyBinding {
			key: "Ctrl+Shift+O".to_string(),
			action: "split-vertical".to_string(),
			description: "Split pane vertically".to_string(),
			enabled: true,
		});
		
		bindings.insert("close-pane".to_string(), KeyBinding {
			key: "Ctrl+Shift+W".to_string(),
			action: "close-pane".to_string(),
			description: "Close current pane".to_string(),
			enabled: true,
		});
		
		// Navigation
		bindings.insert("focus-next".to_string(), KeyBinding {
			key: "Ctrl+Shift+Arrow".to_string(),
			action: "focus-next".to_string(),
			description: "Focus next pane".to_string(),
			enabled: true,
		});
		
		bindings.insert("focus-prev".to_string(), KeyBinding {
			key: "Ctrl+Shift+Arrow".to_string(),
			action: "focus-prev".to_string(),
			description: "Focus previous pane".to_string(),
			enabled: true,
		});
		
		// Zoom
		bindings.insert("zoom-in".to_string(), KeyBinding {
			key: "Ctrl+Plus".to_string(),
			action: "zoom-in".to_string(),
			description: "Zoom in".to_string(),
			enabled: true,
		});
		
		bindings.insert("zoom-out".to_string(), KeyBinding {
			key: "Ctrl+Minus".to_string(),
			action: "zoom-out".to_string(),
			description: "Zoom out".to_string(),
			enabled: true,
		});
		
		bindings.insert("zoom-reset".to_string(), KeyBinding {
			key: "Ctrl+0".to_string(),
			action: "zoom-reset".to_string(),
			description: "Reset zoom".to_string(),
			enabled: true,
		});
		
		bindings
	}
}

/**
 * Terminal configuration
 * 
 * ターミナル設定です。
 * ターミナルの動作に関する
 * 設定を管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
	/// Default shell
	pub default_shell: String,
	/// Scrollback buffer size
	pub scrollback_size: usize,
	/// Enable bell sound
	pub bell_enabled: bool,
	/// Bell sound file
	pub bell_sound: String,
	/// Cursor blink rate
	pub cursor_blink_rate: u32,
	/// Cursor shape
	pub cursor_shape: String,
	/// Enable mouse support
	pub mouse_enabled: bool,
	/// Enable bracketed paste
	pub bracketed_paste: bool,
	/// Enable focus reporting
	pub focus_reporting: bool,
}

impl Default for TerminalConfig {
	fn default() -> Self {
		Self {
			default_shell: "/bin/bash".to_string(),
			scrollback_size: 10000,
			bell_enabled: true,
			bell_sound: "/usr/share/sounds/freedesktop/stereo/complete.oga".to_string(),
			cursor_blink_rate: 530,
			cursor_shape: "block".to_string(),
			mouse_enabled: true,
			bracketed_paste: true,
			focus_reporting: true,
		}
	}
}

/**
 * UI configuration
 * 
 * UI設定です。
 * ユーザーインターフェースの
 * 設定を管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
	/// Window width
	pub window_width: u32,
	/// Window height
	pub window_height: u32,
	/// Window title
	pub window_title: String,
	/// Enable status bar
	pub status_bar_enabled: bool,
	/// Enable tab bar
	pub tab_bar_enabled: bool,
	/// Enable scrollbar
	pub scrollbar_enabled: bool,
	/// Animation duration
	pub animation_duration: f32,
	/// Enable smooth scrolling
	pub smooth_scrolling: bool,
	/// Enable hardware acceleration
	pub hardware_acceleration: bool,
}

impl Default for UiConfig {
	fn default() -> Self {
		Self {
			window_width: 1024,
			window_height: 768,
			window_title: "Sare Terminal".to_string(),
			status_bar_enabled: true,
			tab_bar_enabled: true,
			scrollbar_enabled: true,
			animation_duration: 0.3,
			smooth_scrolling: true,
			hardware_acceleration: true,
		}
	}
}

/**
 * Configuration manager
 * 
 * 設定マネージャーです。
 * 設定の読み込み、保存、更新を
 * 管理します。
 */
pub struct ConfigManager {
	/// Current configuration
	config: Arc<RwLock<Config>>,
	/// Theme engine
	theme_engine: Arc<ThemeEngine>,
	/// Plugin manager
	plugin_manager: Arc<PluginManager>,
	/// Key binding manager
	key_binding_manager: Arc<KeyBindingManager>,
	/// Configuration file manager
	config_file_manager: Arc<ConfigFileManager>,
	/// Runtime configuration manager
	runtime_config_manager: Arc<RuntimeConfigManager>,
	/// Configuration file path
	config_path: String,
	/// Hot reload enabled
	hot_reload: bool,
}

impl ConfigManager {
	/**
	 * Creates a new configuration manager
	 * 
	 * @return ConfigManager - New configuration manager
	 */
	pub fn new() -> Self {
		let config_path = format!("{}/.sare/config.json", dirs::home_dir().unwrap_or_default().display());
		
		Self {
			config: Arc::new(RwLock::new(Config::default())),
			theme_engine: Arc::new(ThemeEngine::new()),
			plugin_manager: Arc::new(PluginManager::new()),
			key_binding_manager: Arc::new(KeyBindingManager::new()),
			config_file_manager: Arc::new(ConfigFileManager::new()),
			runtime_config_manager: Arc::new(RuntimeConfigManager::new()),
			config_path,
			hot_reload: true,
		}
	}
	
	/**
	 * Loads configuration from file
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn load_config(&self) -> Result<()> {
		if let Ok(config_data) = tokio::fs::read_to_string(&self.config_path).await {
			if let Ok(config) = serde_json::from_str::<Config>(&config_data) {
				let mut current_config = self.config.write().await;
				*current_config = config;
				
				// Apply theme
				if let Ok(()) = self.theme_engine.set_theme(&current_config.theme).await {
					println!("🎨 Applied theme: {}", current_config.theme);
				}
				
				return Ok(());
			}
		}
		
		// Create default config if file doesn't exist
		self.save_config().await?;
		Ok(())
	}
	
	/**
	 * Saves configuration to file
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn save_config(&self) -> Result<()> {
		// Create config directory if it doesn't exist
		if let Some(config_dir) = std::path::Path::new(&self.config_path).parent() {
			if !config_dir.exists() {
				tokio::fs::create_dir_all(config_dir).await?;
			}
		}
		
		let config = self.config.read().await;
		let config_json = serde_json::to_string_pretty(&*config)?;
		tokio::fs::write(&self.config_path, config_json).await?;
		
		Ok(())
	}
	
	/**
	 * Gets current configuration
	 * 
	 * @return Config - Current configuration
	 */
	pub async fn get_config(&self) -> Config {
		self.config.read().await.clone()
	}
	
	/**
	 * Updates configuration
	 * 
	 * @param new_config - New configuration
	 * @return Result<()> - Success or error status
	 */
	pub async fn update_config(&self, new_config: Config) -> Result<()> {
		{
			let mut config = self.config.write().await;
			*config = new_config;
		}
		
		// Save to file
		self.save_config().await?;
		
		// Apply theme if changed
		let config = self.config.read().await;
		if let Ok(()) = self.theme_engine.set_theme(&config.theme).await {
			println!("🎨 Applied theme: {}", config.theme);
		}
		
		Ok(())
	}
	
	/**
	 * Gets theme engine reference
	 * 
	 * @return Arc<ThemeEngine> - Theme engine reference
	 */
	pub fn theme_engine(&self) -> Arc<ThemeEngine> {
		self.theme_engine.clone()
	}
	
	/**
	 * Gets plugin manager reference
	 * 
	 * @return Arc<PluginManager> - Plugin manager reference
	 */
	pub fn plugin_manager(&self) -> Arc<PluginManager> {
		self.plugin_manager.clone()
	}
	
	/**
	 * Gets key binding manager reference
	 * 
	 * @return Arc<KeyBindingManager> - Key binding manager reference
	 */
	pub fn key_binding_manager(&self) -> Arc<KeyBindingManager> {
		self.key_binding_manager.clone()
	}
	
	/**
	 * Gets configuration file manager reference
	 * 
	 * @return Arc<ConfigFileManager> - Configuration file manager reference
	 */
	pub fn config_file_manager(&self) -> Arc<ConfigFileManager> {
		self.config_file_manager.clone()
	}
	
	/**
	 * Gets runtime configuration manager reference
	 * 
	 * @return Arc<RuntimeConfigManager> - Runtime configuration manager reference
	 */
	pub fn runtime_config_manager(&self) -> Arc<RuntimeConfigManager> {
		self.runtime_config_manager.clone()
	}
	
	/**
	 * Sets hot reload status
	 * 
	 * @param enabled - Whether hot reload is enabled
	 */
	pub fn set_hot_reload(&mut self, enabled: bool) {
		self.hot_reload = enabled;
	}
	
	/**
	 * Gets hot reload status
	 * 
	 * @return bool - Whether hot reload is enabled
	 */
	pub fn hot_reload_enabled(&self) -> bool {
		self.hot_reload
	}
} 