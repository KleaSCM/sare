/**
 * Advanced UI features for Sare terminal
 * 
 * This module provides advanced UI features including custom widgets,
 * status bar, toolbar, context menus, and drag and drop functionality.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: mod.rs
 * Description: Advanced UI features for terminal emulator
 */

pub mod widgets;
pub mod status_bar;
pub mod toolbar;
pub mod context_menu;
pub mod drag_drop;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use widgets::Widget;

/**
 * UI manager for Sare terminal
 * 
 * UIマネージャーです。
 * すべてのUI機能を
 * 統合管理します。
 */
pub struct UiManager {
	/// Widget manager
	widget_manager: Arc<widgets::WidgetManager>,
	/// Status bar
	status_bar: Option<status_bar::StatusBar>,
	/// Toolbar
	toolbar: Option<toolbar::Toolbar>,
	/// Context menu manager
	context_menu_manager: context_menu::ContextMenuManager,
	/// Drag and drop manager
	drag_drop_manager: drag_drop::DragDropManager,
	/// UI configuration
	config: UiConfig,
}

/**
 * UI configuration
 * 
 * UI設定です。
 * UI機能の設定を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct UiConfig {
	/// Enable status bar
	pub enable_status_bar: bool,
	/// Enable toolbar
	pub enable_toolbar: bool,
	/// Enable context menus
	pub enable_context_menus: bool,
	/// Enable drag and drop
	pub enable_drag_drop: bool,
	/// Status bar height
	pub status_bar_height: u32,
	/// Toolbar height
	pub toolbar_height: u32,
	/// Widget update interval in milliseconds
	pub widget_update_interval: u64,
}

impl Default for UiConfig {
	fn default() -> Self {
		Self {
			enable_status_bar: true,
			enable_toolbar: true,
			enable_context_menus: true,
			enable_drag_drop: true,
			status_bar_height: 1,
			toolbar_height: 1,
			widget_update_interval: 100,
		}
	}
}

impl UiManager {
	/**
	 * Creates a new UI manager
	 * 
	 * @param config - UI configuration
	 * @return UiManager - New UI manager
	 */
	pub fn new(config: UiConfig) -> Self {
		Self {
			widget_manager: Arc::new(widgets::WidgetManager::new()),
			status_bar: None,
			toolbar: None,
			context_menu_manager: context_menu::ContextMenuManager::new(),
			drag_drop_manager: drag_drop::DragDropManager::new(),
			config,
		}
	}
	
	/**
	 * Initializes the UI manager
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		// Initialize status bar if enabled
		if self.config.enable_status_bar {
			self.status_bar = Some(status_bar::StatusBar::new(
				"status_bar".to_string(),
				widgets::WidgetRect::new(0, 0, 80, self.config.status_bar_height)
			));
		}
		
		// Initialize toolbar if enabled
		if self.config.enable_toolbar {
			self.toolbar = Some(toolbar::Toolbar::new(
				"toolbar".to_string(),
				widgets::WidgetRect::new(0, 0, 80, self.config.toolbar_height)
			));
		}
		
		// Initialize context menu manager
		self.context_menu_manager.initialize().await?;
		
		// Initialize drag and drop manager
		self.drag_drop_manager.initialize().await?;
		
		Ok(())
	}
	
	/**
	 * Gets the widget manager
	 * 
	 * @return &Arc<widgets::WidgetManager> - Widget manager reference
	 */
	pub fn widget_manager(&self) -> &Arc<widgets::WidgetManager> {
		&self.widget_manager
	}
	
	/**
	 * Gets the status bar
	 * 
	 * @return Option<&status_bar::StatusBar> - Status bar reference
	 */
	pub fn status_bar(&self) -> Option<&status_bar::StatusBar> {
		self.status_bar.as_ref()
	}
	
	/**
	 * Gets the toolbar
	 * 
	 * @return Option<&toolbar::Toolbar> - Toolbar reference
	 */
	pub fn toolbar(&self) -> Option<&toolbar::Toolbar> {
		self.toolbar.as_ref()
	}
	
	/**
	 * Gets the context menu manager
	 * 
	 * @return &context_menu::ContextMenuManager - Context menu manager reference
	 */
	pub fn context_menu_manager(&self) -> &context_menu::ContextMenuManager {
		&self.context_menu_manager
	}
	
	/**
	 * Gets the drag and drop manager
	 * 
	 * @return &drag_drop::DragDropManager - Drag and drop manager reference
	 */
	pub fn drag_drop_manager(&self) -> &drag_drop::DragDropManager {
		&self.drag_drop_manager
	}
	
	/**
	 * Renders the entire UI
	 * 
	 * @return Result<String> - Rendered UI content
	 */
	pub async fn render(&self) -> Result<String> {
		let mut result = String::new();
		
		// Render toolbar
		if let Some(toolbar) = &self.toolbar {
			result.push_str(&toolbar.render()?);
			result.push('\n');
		}
		
		// Render main content area (widgets)
		result.push_str(&self.widget_manager.render_all().await?);
		
		// Render status bar
		if let Some(status_bar) = &self.status_bar {
			result.push_str(&status_bar.render()?);
		}
		
		// Render context menu if active
		if self.context_menu_manager.is_active() {
			result.push_str(&self.context_menu_manager.render()?);
		}
		
		// Render drag and drop overlay if active
		if self.drag_drop_manager.is_active() {
			result.push_str(&self.drag_drop_manager.render()?);
		}
		
		Ok(result)
	}
	
	/**
	 * Handles UI events
	 * 
	 * @param event - UI event
	 * @return Result<bool> - Whether event was handled
	 */
	pub async fn handle_event(&self, event: widgets::WidgetEvent) -> Result<bool> {
		// Handle widget events
		if self.widget_manager.handle_event(event.clone()).await? {
			return Ok(true);
		}
		
		// Handle context menu events
		if self.context_menu_manager.handle_event(&event).await? {
			return Ok(true);
		}
		
		// Handle drag and drop events
		if self.drag_drop_manager.handle_event(&event).await? {
			return Ok(true);
		}
		
		// Handle toolbar events (simplified for now)
		// TODO: Implement proper mutable toolbar event handling
		
		// Handle status bar events (simplified for now)
		// TODO: Implement proper mutable status bar event handling
		
		Ok(false)
	}
	
	/**
	 * Updates the UI
	 * 
	 * @return Result<bool> - Whether UI needs redraw
	 */
	pub async fn update(&self) -> Result<bool> {
		let mut needs_redraw = false;
		
		// Update widgets
		if self.widget_manager.update_all().await? {
			needs_redraw = true;
		}
		
		// Update status bar (simplified for now)
		// TODO: Implement proper mutable status bar updating
		
		// Update toolbar (simplified for now)
		// TODO: Implement proper mutable toolbar updating
		
		// Update context menu manager
		if self.context_menu_manager.update().await? {
			needs_redraw = true;
		}
		
		// Update drag and drop manager
		if self.drag_drop_manager.update().await? {
			needs_redraw = true;
		}
		
		Ok(needs_redraw)
	}
	
	/**
	 * Shows a context menu
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param items - Menu items
	 * @return Result<()> - Success or error status
	 */
	pub async fn show_context_menu(&mut self, x: u32, y: u32, items: Vec<context_menu::MenuItem>) -> Result<()> {
		self.context_menu_manager.show_menu(x, y, items).await
	}
	
	/**
	 * Hides the context menu
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn hide_context_menu(&mut self) -> Result<()> {
		self.context_menu_manager.hide_menu().await
	}
	
	/**
	 * Starts drag operation
	 * 
	 * @param data - Drag data
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @return Result<()> - Success or error status
	 */
	pub async fn start_drag(&mut self, data: drag_drop::DragData, x: u32, y: u32) -> Result<()> {
		self.drag_drop_manager.start_drag(data, x, y).await
	}
	
	/**
	 * Ends drag operation
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn end_drag(&mut self) -> Result<()> {
		self.drag_drop_manager.end_drag().await
	}
	
	/**
	 * Gets UI configuration
	 * 
	 * @return &UiConfig - UI configuration
	 */
	pub fn config(&self) -> &UiConfig {
		&self.config
	}
	
	/**
	 * Updates UI configuration
	 * 
	 * @param config - New UI configuration
	 */
	pub fn update_config(&mut self, config: UiConfig) {
		self.config = config;
	}
} 