/**
 * Plugin system for Sare terminal
 * 
 * This module provides extensible plugin architecture with
 * plugin loading, plugin API, and hot-reloadable plugins.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: plugin_system.rs
 * Description: Plugin system with extensible architecture
 */

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::Path;

/**
 * Plugin metadata
 * 
 * プラグインメタデータです。
 * プラグインの基本情報を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
	/// Plugin name
	pub name: String,
	/// Plugin version
	pub version: String,
	/// Plugin description
	pub description: String,
	/// Plugin author
	pub author: String,
	/// Plugin license
	pub license: String,
	/// Plugin dependencies
	pub dependencies: Vec<String>,
	/// Plugin entry point
	pub entry_point: String,
	/// Plugin configuration schema
	pub config_schema: Option<serde_json::Value>,
}

/**
 * Plugin state
 * 
 * プラグイン状態です。
 * プラグインの実行状態を
 * 管理します。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginState {
	/// Plugin is loaded but not initialized
	Loaded,
	/// Plugin is initialized and ready
	Initialized,
	/// Plugin is running
	Running,
	/// Plugin is paused
	Paused,
	/// Plugin is stopped
	Stopped,
	/// Plugin encountered an error
	Error(String),
}

/**
 * Plugin interface trait
 * 
 * プラグインインターフェーストレイトです。
 * すべてのプラグインが実装する
 * 基本インターフェースです。
 */
#[async_trait::async_trait]
pub trait PluginInterface: Send + Sync {
	/// Plugin metadata
	fn metadata(&self) -> &PluginMetadata;
	
	/// Initialize the plugin
	async fn initialize(&mut self, config: &serde_json::Value) -> Result<()>;
	
	/// Start the plugin
	async fn start(&mut self) -> Result<()>;
	
	/// Stop the plugin
	async fn stop(&mut self) -> Result<()>;
	
	/// Pause the plugin
	async fn pause(&mut self) -> Result<()>;
	
	/// Resume the plugin
	async fn resume(&mut self) -> Result<()>;
	
	/// Get plugin state
	fn state(&self) -> PluginState;
	
	/// Handle terminal event
	async fn handle_event(&mut self, event: &PluginEvent) -> Result<()>;
	
	/// Get plugin commands
	fn commands(&self) -> Vec<PluginCommand>;
	
	/// Execute plugin command
	async fn execute_command(&mut self, command: &str, args: &[String]) -> Result<String>;
}

/**
 * Plugin event
 * 
 * プラグインイベントです。
 * プラグイン間の通信に使用される
 * イベントです。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
	/// Terminal resize event
	TerminalResize { width: u32, height: u32 },
	/// Key press event
	KeyPress { key: String, modifiers: Vec<String> },
	/// Mouse event
	MouseEvent { x: i32, y: i32, button: u8, pressed: bool },
	/// Text input event
	TextInput { text: String },
	/// Command execution event
	CommandExecuted { command: String, output: String },
	/// Theme change event
	ThemeChanged { theme: String },
	/// Custom plugin event
	Custom { plugin: String, event_type: String, data: serde_json::Value },
}

/**
 * Plugin command
 * 
 * プラグインコマンドです。
 * プラグインが提供する
 * コマンドです。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
	/// Command name
	pub name: String,
	/// Command description
	pub description: String,
	/// Command usage
	pub usage: String,
	/// Command arguments
	pub arguments: Vec<PluginArgument>,
	/// Command enabled status
	pub enabled: bool,
}

/**
 * Plugin argument
 * 
 * プラグイン引数です。
 * プラグインコマンドの
 * 引数定義です。
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginArgument {
	/// Argument name
	pub name: String,
	/// Argument description
	pub description: String,
	/// Argument type
	pub arg_type: String,
	/// Argument required status
	pub required: bool,
	/// Argument default value
	pub default: Option<serde_json::Value>,
}

/**
 * Plugin manager
 * 
 * プラグインマネージャーです。
 * プラグインの読み込み、管理、実行を
 * 担当します。
 */
pub struct PluginManager {
	/// Loaded plugins
	plugins: Arc<RwLock<HashMap<String, Box<dyn PluginInterface>>>>,
	/// Plugin configurations
	configs: Arc<RwLock<HashMap<String, serde_json::Value>>>,
	/// Plugin directory
	plugin_dir: String,
	/// Plugin event handlers
	event_handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(PluginEvent) + Send + Sync>>>>>,
}

impl PluginManager {
	/**
	 * Creates a new plugin manager
	 * 
	 * @return PluginManager - New plugin manager
	 */
	pub fn new() -> Self {
		let plugin_dir = format!("{}/.sare/plugins", dirs::home_dir().unwrap_or_default().display());
		
		Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			configs: Arc::new(RwLock::new(HashMap::new())),
			plugin_dir,
			event_handlers: Arc::new(RwLock::new(HashMap::new())),
		}
	}
	
	/**
	 * Loads all plugins from plugin directory
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn load_plugins(&self) -> Result<()> {
		// Create plugin directory if it doesn't exist
		if !Path::new(&self.plugin_dir).exists() {
			tokio::fs::create_dir_all(&self.plugin_dir).await?;
		}
		
		// Scan for plugin files
		let mut entries = tokio::fs::read_dir(&self.plugin_dir).await?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.is_file() && path.extension().map_or(false, |ext| ext == "so") {
				if let Err(e) = self.load_plugin(&path).await {
					eprintln!("⚠️ Failed to load plugin {}: {}", path.display(), e);
				}
			}
		}
		
		Ok(())
	}
	
	/**
	 * Loads a specific plugin
	 * 
	 * @param plugin_path - Path to plugin file
	 * @return Result<()> - Success or error status
	 */
	pub async fn load_plugin(&self, plugin_path: &Path) -> Result<()> {
		// Load plugin library
		unsafe {
			// This is a simplified implementation
			// In a real implementation, you would use libloading or similar
			// to dynamically load shared libraries
			
			// For now, we'll create a mock plugin
			let plugin_name = plugin_path.file_stem()
				.and_then(|s| s.to_str())
				.ok_or_else(|| anyhow::anyhow!("Invalid plugin path"))?;
			
			let plugin = self.create_mock_plugin(plugin_name)?;
			let mut plugins = self.plugins.write().await;
			plugins.insert(plugin_name.to_string(), plugin);
			
			println!("🔌 Loaded plugin: {}", plugin_name);
		}
		
		Ok(())
	}
	
	/**
	 * Creates a mock plugin for demonstration
	 * 
	 * @param name - Plugin name
	 * @return Result<Box<dyn PluginInterface>> - Mock plugin
	 */
	fn create_mock_plugin(&self, name: &str) -> Result<Box<dyn PluginInterface>> {
		Ok(Box::new(MockPlugin::new(name)))
	}
	
	/**
	 * Initializes all loaded plugins
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize_plugins(&self) -> Result<()> {
		let mut plugins = self.plugins.write().await;
		let configs = self.configs.read().await;
		
		for (name, plugin) in plugins.iter_mut() {
			let config = configs.get(name).unwrap_or(&serde_json::Value::Null);
			if let Err(e) = plugin.initialize(config).await {
				eprintln!("⚠️ Failed to initialize plugin {}: {}", name, e);
			} else {
				println!("🚀 Initialized plugin: {}", name);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Starts all plugins
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn start_plugins(&self) -> Result<()> {
		let mut plugins = self.plugins.write().await;
		
		for (name, plugin) in plugins.iter_mut() {
			if let Err(e) = plugin.start().await {
				eprintln!("⚠️ Failed to start plugin {}: {}", name, e);
			} else {
				println!("▶️ Started plugin: {}", name);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Stops all plugins
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn stop_plugins(&self) -> Result<()> {
		let mut plugins = self.plugins.write().await;
		
		for (name, plugin) in plugins.iter_mut() {
			if let Err(e) = plugin.stop().await {
				eprintln!("⚠️ Failed to stop plugin {}: {}", name, e);
			} else {
				println!("⏹️ Stopped plugin: {}", name);
			}
		}
		
		Ok(())
	}
	
	/**
	 * Gets all loaded plugins
	 * 
	 * @return Vec<String> - List of plugin names
	 */
	pub async fn get_plugin_names(&self) -> Vec<String> {
		let plugins = self.plugins.read().await;
		plugins.keys().cloned().collect()
	}
	
	/**
	 * Gets plugin by name
	 * 
	 * @param name - Plugin name
	 * @return Option<&dyn PluginInterface> - Plugin reference
	 */
	pub async fn get_plugin(&self, name: &str) -> Option<Box<dyn PluginInterface>> {
		let plugins = self.plugins.read().await;
		plugins.get(name).map(|p| p.clone())
	}
	
	/**
	 * Registers an event handler
	 * 
	 * @param event_type - Event type to handle
	 * @param handler - Event handler function
	 */
	pub async fn register_event_handler<F>(&self, event_type: &str, handler: F)
	where
		F: Fn(PluginEvent) + Send + Sync + 'static,
	{
		let mut handlers = self.event_handlers.write().await;
		handlers.entry(event_type.to_string())
			.or_insert_with(Vec::new)
			.push(Box::new(handler));
	}
	
	/**
	 * Broadcasts an event to all plugins
	 * 
	 * @param event - Event to broadcast
	 * @return Result<()> - Success or error status
	 */
	pub async fn broadcast_event(&self, event: PluginEvent) -> Result<()> {
		let mut plugins = self.plugins.write().await;
		
		for (name, plugin) in plugins.iter_mut() {
			if let Err(e) = plugin.handle_event(&event).await {
				eprintln!("⚠️ Plugin {} failed to handle event: {}", name, e);
			}
		}
		
		// Notify event handlers
		let handlers = self.event_handlers.read().await;
		if let Some(event_handlers) = handlers.get(&event_type(&event)) {
			for handler in event_handlers {
				handler(event.clone());
			}
		}
		
		Ok(())
	}
	
	/**
	 * Executes a plugin command
	 * 
	 * @param plugin_name - Plugin name
	 * @param command - Command to execute
	 * @param args - Command arguments
	 * @return Result<String> - Command output
	 */
	pub async fn execute_plugin_command(&self, plugin_name: &str, command: &str, args: &[String]) -> Result<String> {
		let mut plugins = self.plugins.write().await;
		
		if let Some(plugin) = plugins.get_mut(plugin_name) {
			plugin.execute_command(command, args).await
		} else {
			Err(anyhow::anyhow!("Plugin not found: {}", plugin_name))
		}
	}
	
	/**
	 * Gets all available commands from all plugins
	 * 
	 * @return HashMap<String, Vec<PluginCommand>> - Plugin commands
	 */
	pub async fn get_all_commands(&self) -> HashMap<String, Vec<PluginCommand>> {
		let plugins = self.plugins.read().await;
		let mut all_commands = HashMap::new();
		
		for (name, plugin) in plugins.iter() {
			all_commands.insert(name.clone(), plugin.commands());
		}
		
		all_commands
	}
}

/**
 * Gets event type from event
 * 
 * @param event - Plugin event
 * @return String - Event type
 */
fn event_type(event: &PluginEvent) -> String {
	match event {
		PluginEvent::TerminalResize { .. } => "terminal_resize".to_string(),
		PluginEvent::KeyPress { .. } => "key_press".to_string(),
		PluginEvent::MouseEvent { .. } => "mouse_event".to_string(),
		PluginEvent::TextInput { .. } => "text_input".to_string(),
		PluginEvent::CommandExecuted { .. } => "command_executed".to_string(),
		PluginEvent::ThemeChanged { .. } => "theme_changed".to_string(),
		PluginEvent::Custom { event_type, .. } => event_type.clone(),
	}
}

/**
 * Mock plugin implementation
 * 
 * モックプラグイン実装です。
 * デモンストレーション用の
 * サンプルプラグインです。
 */
pub struct MockPlugin {
	metadata: PluginMetadata,
	state: PluginState,
	config: serde_json::Value,
}

impl MockPlugin {
	/**
	 * Creates a new mock plugin
	 * 
	 * @param name - Plugin name
	 * @return MockPlugin - New mock plugin
	 */
	pub fn new(name: &str) -> Self {
		Self {
			metadata: PluginMetadata {
				name: name.to_string(),
				version: "1.0.0".to_string(),
				description: format!("Mock plugin for {}", name),
				author: "Sare Team".to_string(),
				license: "MIT".to_string(),
				dependencies: Vec::new(),
				entry_point: "main".to_string(),
				config_schema: None,
			},
			state: PluginState::Loaded,
			config: serde_json::Value::Null,
		}
	}
}

#[async_trait::async_trait]
impl PluginInterface for MockPlugin {
	fn metadata(&self) -> &PluginMetadata {
		&self.metadata
	}
	
	async fn initialize(&mut self, config: &serde_json::Value) -> Result<()> {
		self.config = config.clone();
		self.state = PluginState::Initialized;
		println!("🔧 Initialized mock plugin: {}", self.metadata.name);
		Ok(())
	}
	
	async fn start(&mut self) -> Result<()> {
		self.state = PluginState::Running;
		println!("▶️ Started mock plugin: {}", self.metadata.name);
		Ok(())
	}
	
	async fn stop(&mut self) -> Result<()> {
		self.state = PluginState::Stopped;
		println!("⏹️ Stopped mock plugin: {}", self.metadata.name);
		Ok(())
	}
	
	async fn pause(&mut self) -> Result<()> {
		self.state = PluginState::Paused;
		println!("⏸️ Paused mock plugin: {}", self.metadata.name);
		Ok(())
	}
	
	async fn resume(&mut self) -> Result<()> {
		self.state = PluginState::Running;
		println!("▶️ Resumed mock plugin: {}", self.metadata.name);
		Ok(())
	}
	
	fn state(&self) -> PluginState {
		self.state.clone()
	}
	
	async fn handle_event(&mut self, event: &PluginEvent) -> Result<()> {
		match event {
			PluginEvent::TerminalResize { width, height } => {
				println!("📐 Plugin {}: Terminal resized to {}x{}", self.metadata.name, width, height);
			}
			PluginEvent::KeyPress { key, modifiers } => {
				println!("⌨️ Plugin {}: Key pressed: {} with modifiers: {:?}", self.metadata.name, key, modifiers);
			}
			PluginEvent::ThemeChanged { theme } => {
				println!("🎨 Plugin {}: Theme changed to {}", self.metadata.name, theme);
			}
			_ => {
				println!("📡 Plugin {}: Received event: {:?}", self.metadata.name, event);
			}
		}
		Ok(())
	}
	
	fn commands(&self) -> Vec<PluginCommand> {
		vec![
			PluginCommand {
				name: "status".to_string(),
				description: "Get plugin status".to_string(),
				usage: "status".to_string(),
				arguments: Vec::new(),
				enabled: true,
			},
			PluginCommand {
				name: "config".to_string(),
				description: "Get plugin configuration".to_string(),
				usage: "config".to_string(),
				arguments: Vec::new(),
				enabled: true,
			},
			PluginCommand {
				name: "test".to_string(),
				description: "Run plugin test".to_string(),
				usage: "test [test_name]".to_string(),
				arguments: vec![
					PluginArgument {
						name: "test_name".to_string(),
						description: "Name of test to run".to_string(),
						arg_type: "string".to_string(),
						required: false,
						default: Some(serde_json::Value::String("default".to_string())),
					}
				],
				enabled: true,
			},
		]
	}
	
	async fn execute_command(&mut self, command: &str, args: &[String]) -> Result<String> {
		match command {
			"status" => {
				Ok(format!("Plugin: {}\nState: {:?}\nVersion: {}", 
					self.metadata.name, self.state, self.metadata.version))
			}
			"config" => {
				Ok(format!("Configuration: {}", serde_json::to_string_pretty(&self.config)?))
			}
			"test" => {
				let test_name = args.get(0).unwrap_or(&"default".to_string()).clone();
				Ok(format!("Running test '{}' for plugin {}", test_name, self.metadata.name))
			}
			_ => {
				Err(anyhow::anyhow!("Unknown command: {}", command))
			}
		}
	}
} 