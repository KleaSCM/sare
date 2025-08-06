/**
 * Hot reload system for Sare terminal
 * 
 * This module provides comprehensive hot reload capabilities including
 * configuration hot reload, runtime configuration changes, and file watching.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: hot_reload.rs
 * Description: Hot reload system with runtime configuration changes
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Duration, interval};
use std::time::SystemTime;

/**
 * File change event
 * 
 * ファイル変更イベントです。
 * ファイルの変更を検知する
 * イベントです。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChangeEvent {
	/// File was created
	Created { path: String, timestamp: SystemTime },
	/// File was modified
	Modified { path: String, timestamp: SystemTime },
	/// File was deleted
	Deleted { path: String, timestamp: SystemTime },
	/// File was renamed
	Renamed { old_path: String, new_path: String, timestamp: SystemTime },
}

/**
 * Configuration change event
 * 
 * 設定変更イベントです。
 * 設定の変更を検知する
 * イベントです。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeEvent {
	/// Theme was changed
	ThemeChanged { old_theme: String, new_theme: String },
	/// Key binding was changed
	KeyBindingChanged { binding_id: String, action: String },
	/// Plugin was enabled/disabled
	PluginToggled { plugin_name: String, enabled: bool },
	/// UI setting was changed
	UiSettingChanged { setting: String, value: serde_json::Value },
	/// Terminal setting was changed
	TerminalSettingChanged { setting: String, value: serde_json::Value },
	/// Custom configuration change
	Custom { change_type: String, data: serde_json::Value },
}

/**
 * Hot reload watcher
 * 
 * ホットリロードウォッチャーです。
 * ファイルの変更を監視し、
 * 設定の更新を管理します。
 */
pub struct HotReloadWatcher {
	/// Watched files
	watched_files: Arc<RwLock<HashMap<String, FileWatcher>>>,
	/// Change event sender
	change_sender: mpsc::UnboundedSender<FileChangeEvent>,
	/// Change event receiver
	change_receiver: mpsc::UnboundedReceiver<FileChangeEvent>,
	/// Configuration change callbacks
	config_callbacks: Arc<RwLock<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>,
	/// Hot reload enabled
	enabled: bool,
	/// Watch interval
	watch_interval: Duration,
}

/**
 * File watcher
 * 
 * ファイルウォッチャーです。
 * 個別のファイルの変更を
 * 監視します。
 */
struct FileWatcher {
	/// File path
	path: String,
	/// Last modification time
	last_modified: SystemTime,
	/// File size
	file_size: u64,
	/// File hash
	file_hash: String,
}

impl FileWatcher {
	/**
	 * Creates a new file watcher
	 * 
	 * @param path - File path
	 * @return Result<FileWatcher> - New file watcher
	 */
	pub fn new(path: &str) -> Result<Self> {
		let metadata = std::fs::metadata(path)?;
		let last_modified = metadata.modified()?;
		let file_size = metadata.len();
		let file_hash = Self::calculate_file_hash(path)?;
		
		Ok(Self {
			path: path.to_string(),
			last_modified,
			file_size,
			file_hash,
		})
	}
	
	/**
	 * Checks if file has changed
	 * 
	 * @return Result<bool> - Whether file has changed
	 */
	pub fn has_changed(&self) -> Result<bool> {
		if !Path::new(&self.path).exists() {
			return Ok(true); // File was deleted
		}
		
		let metadata = std::fs::metadata(&self.path)?;
		let current_modified = metadata.modified()?;
		let current_size = metadata.len();
		let current_hash = Self::calculate_file_hash(&self.path)?;
		
		Ok(current_modified != self.last_modified || 
		   current_size != self.file_size || 
		   current_hash != self.file_hash)
	}
	
	/**
	 * Updates watcher state
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub fn update_state(&mut self) -> Result<()> {
		if Path::new(&self.path).exists() {
			let metadata = std::fs::metadata(&self.path)?;
			self.last_modified = metadata.modified()?;
			self.file_size = metadata.len();
			self.file_hash = Self::calculate_file_hash(&self.path)?;
		}
		Ok(())
	}
	
	/**
	 * Calculates file hash
	 * 
	 * @param path - File path
	 * @return Result<String> - File hash
	 */
	fn calculate_file_hash(path: &str) -> Result<String> {
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hash, Hasher};
		
		let content = std::fs::read_to_string(path)?;
		let mut hasher = DefaultHasher::new();
		content.hash(&mut hasher);
		Ok(format!("{:x}", hasher.finish()))
	}
}

impl HotReloadWatcher {
	/**
	 * Creates a new hot reload watcher
	 * 
	 * @return HotReloadWatcher - New hot reload watcher
	 */
	pub fn new() -> Self {
		let (change_sender, change_receiver) = mpsc::unbounded_channel();
		
		Self {
			watched_files: Arc::new(RwLock::new(HashMap::new())),
			change_sender,
			change_receiver,
			config_callbacks: Arc::new(RwLock::new(Vec::new())),
			enabled: true,
			watch_interval: Duration::from_millis(1000), // 1 second
		}
	}
	
	/**
	 * Starts the hot reload watcher
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn start(&mut self) -> Result<()> {
		println!("🔄 Starting hot reload watcher...");
		
		// Start file watching task
		let watched_files = self.watched_files.clone();
		let change_sender = self.change_sender.clone();
		let watch_interval = self.watch_interval;
		
		tokio::spawn(async move {
			let mut interval = interval(watch_interval);
			
			loop {
				interval.tick().await;
				
				let mut files = watched_files.write().await;
				for (path, watcher) in files.iter_mut() {
					if let Ok(has_changed) = watcher.has_changed() {
						if has_changed {
							if let Err(e) = change_sender.send(FileChangeEvent::Modified {
								path: path.clone(),
								timestamp: SystemTime::now(),
							}) {
								eprintln!("⚠️ Failed to send file change event: {}", e);
							}
							
							// Update watcher state
							if let Err(e) = watcher.update_state() {
								eprintln!("⚠️ Failed to update watcher state: {}", e);
							}
						}
					}
				}
			}
		});
		
		// Start event processing task
		let config_callbacks = self.config_callbacks.clone();
		let mut change_receiver = std::mem::replace(&mut self.change_receiver, mpsc::unbounded_channel().1);
		
		tokio::spawn(async move {
			while let Some(event) = change_receiver.recv().await {
				Self::process_file_change(event, &config_callbacks).await;
			}
		});
		
		Ok(())
	}
	
	/**
	 * Stops the hot reload watcher
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn stop(&mut self) -> Result<()> {
		println!("⏹️ Stopping hot reload watcher...");
		self.enabled = false;
		Ok(())
	}
	
	/**
	 * Adds a file to watch
	 * 
	 * @param path - File path to watch
	 * @return Result<()> - Success or error status
	 */
	pub async fn watch_file(&self, path: &str) -> Result<()> {
		let watcher = FileWatcher::new(path)?;
		let mut watched_files = self.watched_files.write().await;
		watched_files.insert(path.to_string(), watcher);
		
		println!("👁️ Watching file: {}", path);
		Ok(())
	}
	
	/**
	 * Removes a file from watching
	 * 
	 * @param path - File path to stop watching
	 * @return Result<()> - Success or error status
	 */
	pub async fn unwatch_file(&self, path: &str) -> Result<()> {
		let mut watched_files = self.watched_files.write().await;
		watched_files.remove(path);
		
		println!("👁️ Stopped watching file: {}", path);
		Ok(())
	}
	
	/**
	 * Gets list of watched files
	 * 
	 * @return Vec<String> - List of watched files
	 */
	pub async fn get_watched_files(&self) -> Vec<String> {
		let watched_files = self.watched_files.read().await;
		watched_files.keys().cloned().collect()
	}
	
	/**
	 * Registers a configuration change callback
	 * 
	 * @param callback - Callback function
	 */
	pub async fn register_config_callback<F>(&self, callback: F)
	where
		F: Fn(ConfigChangeEvent) + Send + Sync + 'static,
	{
		let mut callbacks = self.config_callbacks.write().await;
		callbacks.push(Box::new(callback));
	}
	
	/**
	 * Processes file change event
	 * 
	 * @param event - File change event
	 * @param callbacks - Configuration callbacks
	 */
	async fn process_file_change(event: FileChangeEvent, callbacks: &Arc<RwLock<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>) {
		match event {
			FileChangeEvent::Modified { path, timestamp } => {
				println!("📝 File modified: {} at {:?}", path, timestamp);
				
				// Determine what type of configuration file changed
				if path.ends_with("config.json") {
					Self::handle_config_change(&path, callbacks).await;
				} else if path.ends_with("theme.json") {
					Self::handle_theme_change(&path, callbacks).await;
				} else if path.ends_with("keybindings.json") {
					Self::handle_keybinding_change(&path, callbacks).await;
				} else if path.ends_with("plugins.json") {
					Self::handle_plugin_change(&path, callbacks).await;
				}
			}
			FileChangeEvent::Created { path, timestamp } => {
				println!("📄 File created: {} at {:?}", path, timestamp);
			}
			FileChangeEvent::Deleted { path, timestamp } => {
				println!("🗑️ File deleted: {} at {:?}", path, timestamp);
			}
			FileChangeEvent::Renamed { old_path, new_path, timestamp } => {
				println!("🔄 File renamed: {} -> {} at {:?}", old_path, new_path, timestamp);
			}
		}
	}
	
	/**
	 * Handles configuration file change
	 * 
	 * @param path - Configuration file path
	 * @param callbacks - Configuration callbacks
	 */
	async fn handle_config_change(path: &str, callbacks: &Arc<RwLock<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>) {
		// Load and parse the new configuration
		if let Ok(content) = tokio::fs::read_to_string(path).await {
			if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
				// Extract theme change
				if let Some(theme) = config.get("theme") {
					if let Some(theme_str) = theme.as_str() {
						let callbacks = callbacks.read().await;
						for callback in callbacks.iter() {
							callback(ConfigChangeEvent::ThemeChanged {
								old_theme: "unknown".to_string(),
								new_theme: theme_str.to_string(),
							});
						}
					}
				}
				
				// Extract UI setting changes
				if let Some(ui) = config.get("ui") {
					if let Some(ui_obj) = ui.as_object() {
						for (setting, value) in ui_obj {
							let callbacks = callbacks.read().await;
							for callback in callbacks.iter() {
								callback(ConfigChangeEvent::UiSettingChanged {
									setting: setting.clone(),
									value: value.clone(),
								});
							}
						}
					}
				}
				
				// Extract terminal setting changes
				if let Some(terminal) = config.get("terminal") {
					if let Some(terminal_obj) = terminal.as_object() {
						for (setting, value) in terminal_obj {
							let callbacks = callbacks.read().await;
							for callback in callbacks.iter() {
								callback(ConfigChangeEvent::TerminalSettingChanged {
									setting: setting.clone(),
									value: value.clone(),
								});
							}
						}
					}
				}
			}
		}
	}
	
	/**
	 * Handles theme file change
	 * 
	 * @param path - Theme file path
	 * @param callbacks - Configuration callbacks
	 */
	async fn handle_theme_change(path: &str, callbacks: &Arc<RwLock<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>) {
		println!("🎨 Theme file changed: {}", path);
		
		// Extract theme name from path
		if let Some(file_name) = Path::new(path).file_stem() {
			if let Some(theme_name) = file_name.to_str() {
				let callbacks = callbacks.read().await;
				for callback in callbacks.iter() {
					callback(ConfigChangeEvent::ThemeChanged {
						old_theme: "unknown".to_string(),
						new_theme: theme_name.to_string(),
					});
				}
			}
		}
	}
	
	/**
	 * Handles keybinding file change
	 * 
	 * @param path - Keybinding file path
	 * @param callbacks - Configuration callbacks
	 */
	async fn handle_keybinding_change(path: &str, callbacks: &Arc<RwLock<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>) {
		println!("⌨️ Keybinding file changed: {}", path);
		
		// Load and parse the new keybindings
		if let Ok(content) = tokio::fs::read_to_string(path).await {
			if let Ok(bindings) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
				for (binding_id, binding_data) in bindings {
					if let Some(action) = binding_data.get("action") {
						if let Some(action_str) = action.as_str() {
							let callbacks = callbacks.read().await;
							for callback in callbacks.iter() {
								callback(ConfigChangeEvent::KeyBindingChanged {
									binding_id: binding_id.clone(),
									action: action_str.to_string(),
								});
							}
						}
					}
				}
			}
		}
	}
	
	/**
	 * Handles plugin file change
	 * 
	 * @param path - Plugin file path
	 * @param callbacks - Configuration callbacks
	 */
	async fn handle_plugin_change(path: &str, callbacks: &Arc<RwLock<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>) {
		println!("🔌 Plugin file changed: {}", path);
		
		// Load and parse the new plugin configuration
		if let Ok(content) = tokio::fs::read_to_string(path).await {
			if let Ok(plugins) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
				for plugin in plugins {
					if let Some(plugin_obj) = plugin.as_object() {
						if let (Some(name), Some(enabled)) = (plugin_obj.get("name"), plugin_obj.get("enabled")) {
							if let (Some(name_str), Some(enabled_bool)) = (name.as_str(), enabled.as_bool()) {
								let callbacks = callbacks.read().await;
								for callback in callbacks.iter() {
									callback(ConfigChangeEvent::PluginToggled {
										plugin_name: name_str.to_string(),
										enabled: enabled_bool,
									});
								}
							}
						}
					}
				}
			}
		}
	}
	
	/**
	 * Triggers a configuration change manually
	 * 
	 * @param event - Configuration change event
	 */
	pub async fn trigger_config_change(&self, event: ConfigChangeEvent) {
		let callbacks = self.config_callbacks.read().await;
		for callback in callbacks.iter() {
			callback(event.clone());
		}
	}
	
	/**
	 * Gets hot reload statistics
	 * 
	 * @return HashMap<String, usize> - Statistics
	 */
	pub async fn get_stats(&self) -> HashMap<String, usize> {
		let watched_files = self.watched_files.read().await;
		let mut stats = HashMap::new();
		
		stats.insert("watched_files".to_string(), watched_files.len());
		stats.insert("config_callbacks".to_string(), self.config_callbacks.read().await.len());
		stats.insert("enabled".to_string(), if self.enabled { 1 } else { 0 });
		
		stats
	}
}

/**
 * Runtime configuration manager
 * 
 * ランタイム設定マネージャーです。
 * 実行時の設定変更を
 * 管理します。
 */
pub struct RuntimeConfigManager {
	/// Current configuration
	current_config: Arc<RwLock<serde_json::Value>>,
	/// Configuration history
	config_history: Arc<RwLock<Vec<serde_json::Value>>>,
	/// Change listeners
	change_listeners: Arc<RwLock<HashMap<String, Box<dyn Fn(serde_json::Value) + Send + Sync>>>>,
	/// Hot reload watcher
	hot_reload_watcher: Arc<HotReloadWatcher>,
}

impl RuntimeConfigManager {
	/**
	 * Creates a new runtime configuration manager
	 * 
	 * @return RuntimeConfigManager - New runtime configuration manager
	 */
	pub fn new() -> Self {
		Self {
			current_config: Arc::new(RwLock::new(serde_json::Value::Null)),
			config_history: Arc::new(RwLock::new(Vec::new())),
			change_listeners: Arc::new(RwLock::new(HashMap::new())),
			hot_reload_watcher: Arc::new(HotReloadWatcher::new()),
		}
	}
	
	/**
	 * Updates configuration at runtime
	 * 
	 * @param new_config - New configuration
	 * @return Result<()> - Success or error status
	 */
	pub async fn update_config(&self, new_config: serde_json::Value) -> Result<()> {
		// Store current config in history
		{
			let current = self.current_config.read().await;
			let mut history = self.config_history.write().await;
			history.push(current.clone());
			
			// Keep only last 10 configurations
			if history.len() > 10 {
				history.remove(0);
			}
		}
		
		// Update current configuration
		{
			let mut current = self.current_config.write().await;
			*current = new_config.clone();
		}
		
		// Notify listeners
		let listeners = self.change_listeners.read().await;
		for (name, listener) in listeners.iter() {
			println!("📢 Notifying listener '{}' of config change", name);
			listener(new_config.clone());
		}
		
		println!("⚡ Configuration updated at runtime");
		Ok(())
	}
	
	/**
	 * Gets current configuration
	 * 
	 * @return serde_json::Value - Current configuration
	 */
	pub async fn get_current_config(&self) -> serde_json::Value {
		let current = self.current_config.read().await;
		current.clone()
	}
	
	/**
	 * Reverts to previous configuration
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn revert_config(&self) -> Result<()> {
		let mut history = self.config_history.write().await;
		if let Some(previous_config) = history.pop() {
			self.update_config(previous_config).await?;
			println!("↩️ Configuration reverted to previous version");
		} else {
			return Err(anyhow::anyhow!("No previous configuration to revert to"));
		}
		
		Ok(())
	}
	
	/**
	 * Registers a configuration change listener
	 * 
	 * @param name - Listener name
	 * @param listener - Listener function
	 */
	pub async fn register_listener<F>(&self, name: &str, listener: F)
	where
		F: Fn(serde_json::Value) + Send + Sync + 'static,
	{
		let mut listeners = self.change_listeners.write().await;
		listeners.insert(name.to_string(), Box::new(listener));
		println!("👂 Registered configuration listener: {}", name);
	}
	
	/**
	 * Unregisters a configuration change listener
	 * 
	 * @param name - Listener name
	 * @return Result<()> - Success or error status
	 */
	pub async fn unregister_listener(&self, name: &str) -> Result<()> {
		let mut listeners = self.change_listeners.write().await;
		if listeners.remove(name).is_some() {
			println!("👂 Unregistered configuration listener: {}", name);
			Ok(())
		} else {
			Err(anyhow::anyhow!("Listener '{}' not found", name))
		}
	}
	
	/**
	 * Gets hot reload watcher reference
	 * 
	 * @return Arc<HotReloadWatcher> - Hot reload watcher
	 */
	pub fn hot_reload_watcher(&self) -> Arc<HotReloadWatcher> {
		self.hot_reload_watcher.clone()
	}
	
	/**
	 * Gets configuration history
	 * 
	 * @return Vec<serde_json::Value> - Configuration history
	 */
	pub async fn get_config_history(&self) -> Vec<serde_json::Value> {
		let history = self.config_history.read().await;
		history.clone()
	}
} 