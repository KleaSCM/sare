/**
 * Context menu for Sare terminal
 * 
 * This module provides context menu functionality with
 * right-click menus and context-aware options.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: context_menu.rs
 * Description: Context menu manager with right-click menus
 */

use anyhow::Result;
use std::collections::HashMap;
use super::widgets::{WidgetEvent, MouseButton};

/**
 * Menu item
 * 
 * メニューアイテムです。
 * コンテキストメニューの
 * 個別アイテムを管理します。
 */
#[derive(Debug, Clone)]
pub struct MenuItem {
	/// Item ID
	pub id: String,
	/// Item text
	pub text: String,
	/// Item icon
	pub icon: String,
	/// Item enabled state
	pub enabled: bool,
	/// Item separator
	pub separator: bool,
	/// Item callback
	pub callback: Option<Box<dyn Fn() + Send + Sync>>,
}

/**
 * Context menu manager
 * 
 * コンテキストメニューマネージャーです。
 * 右クリックメニューと
 * コンテキスト対応オプションを提供します。
 */
pub struct ContextMenuManager {
	/// Active menu items
	active_items: Vec<MenuItem>,
	/// Menu position
	menu_x: u32,
	/// Menu position
	menu_y: u32,
	/// Menu visible state
	visible: bool,
	/// Selected item index
	selected_index: Option<usize>,
	/// Menu callbacks
	callbacks: HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

impl ContextMenuManager {
	/**
	 * Creates a new context menu manager
	 * 
	 * @return ContextMenuManager - New context menu manager
	 */
	pub fn new() -> Self {
		Self {
			active_items: Vec::new(),
			menu_x: 0,
			menu_y: 0,
			visible: false,
			selected_index: None,
			callbacks: HashMap::new(),
		}
	}
	
	/**
	 * Initializes the context menu manager
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		// Initialize default callbacks
		self.callbacks.insert("copy".to_string(), Box::new(|| {
			println!("Copy action triggered");
		}));
		
		self.callbacks.insert("paste".to_string(), Box::new(|| {
			println!("Paste action triggered");
		}));
		
		self.callbacks.insert("cut".to_string(), Box::new(|| {
			println!("Cut action triggered");
		}));
		
		self.callbacks.insert("select_all".to_string(), Box::new(|| {
			println!("Select all action triggered");
		}));
		
		Ok(())
	}
	
	/**
	 * Shows a context menu
	 * 
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @param items - Menu items
	 * @return Result<()> - Success or error status
	 */
	pub async fn show_menu(&mut self, x: u32, y: u32, items: Vec<MenuItem>) -> Result<()> {
		self.active_items = items;
		self.menu_x = x;
		self.menu_y = y;
		self.visible = true;
		self.selected_index = None;
		
		Ok(())
	}
	
	/**
	 * Hides the context menu
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn hide_menu(&mut self) -> Result<()> {
		self.visible = false;
		self.active_items.clear();
		self.selected_index = None;
		
		Ok(())
	}
	
	/**
	 * Checks if menu is active
	 * 
	 * @return bool - Whether menu is active
	 */
	pub fn is_active(&self) -> bool {
		self.visible
	}
	
	/**
	 * Renders the context menu
	 * 
	 * @return Result<String> - Rendered menu content
	 */
	pub fn render(&self) -> Result<String> {
		if !self.visible {
			return Ok(String::new());
		}
		
		let mut result = String::new();
		
		// Position cursor at menu location
		result.push_str(&format!("\x1b[{};{}H", self.menu_y + 1, self.menu_x + 1));
		
		// Render menu border
		result.push('┌');
		for _ in 0..30 {
			result.push('─');
		}
		result.push('┐');
		result.push('\n');
		
		// Render menu items
		for (index, item) in self.active_items.iter().enumerate() {
			if item.separator {
				result.push('├');
				for _ in 0..30 {
					result.push('─');
				}
				result.push('┤');
			} else {
				let selected = Some(index) == self.selected_index;
				let prefix = if selected { "▶ " } else { "  " };
				let icon = if !item.icon.is_empty() { &item.icon } else { " " };
				let text = if item.enabled { &item.text } else { "Disabled" };
				
				result.push('│');
				result.push_str(&format!("{}{} {}", prefix, icon, text));
				
				// Pad to border width
				let item_width = prefix.len() + icon.len() + 1 + text.len();
				for _ in item_width..30 {
					result.push(' ');
				}
				result.push('│');
			}
			result.push('\n');
		}
		
		// Render bottom border
		result.push('└');
		for _ in 0..30 {
			result.push('─');
		}
		result.push('┘');
		
		Ok(result)
	}
	
	/**
	 * Handles context menu events
	 * 
	 * @param event - Widget event
	 * @return Result<bool> - Whether event was handled
	 */
	pub async fn handle_event(&self, event: &WidgetEvent) -> Result<bool> {
		if !self.visible {
			return Ok(false);
		}
		
		match event {
			WidgetEvent::Click { x, y, button: MouseButton::Left } => {
				// Check if click is within menu bounds
				if *x >= self.menu_x && *x < self.menu_x + 30 &&
				   *y >= self.menu_y && *y < self.menu_y + self.active_items.len() as u32 {
					let item_index = (*y - self.menu_y) as usize;
					if item_index < self.active_items.len() {
						let item = &self.active_items[item_index];
						if item.enabled && !item.separator {
							// Execute callback if available
							if let Some(callback) = &item.callback {
								callback();
							}
							
							return Ok(true);
						}
					}
				} else {
					// Click outside menu, hide it
					return Ok(true);
				}
			}
			WidgetEvent::KeyPress { key } => {
				match key.as_str() {
					"Escape" => {
						// Hide menu on escape
						return Ok(true);
					}
					"ArrowDown" => {
						// Navigate down
						if let Some(current) = self.selected_index {
							let next = (current + 1) % self.active_items.len();
							if next < self.active_items.len() {
								return Ok(true);
							}
						} else if !self.active_items.is_empty() {
							return Ok(true);
						}
					}
					"ArrowUp" => {
						// Navigate up
						if let Some(current) = self.selected_index {
							let prev = if current == 0 {
								self.active_items.len() - 1
							} else {
								current - 1
							};
							return Ok(true);
						} else if !self.active_items.is_empty() {
							return Ok(true);
						}
					}
					"Enter" => {
						// Select current item
						if let Some(index) = self.selected_index {
							if index < self.active_items.len() {
								let item = &self.active_items[index];
								if item.enabled && !item.separator {
									// Execute callback if available
									if let Some(callback) = &item.callback {
										callback();
									}
									
									return Ok(true);
								}
							}
						}
					}
					_ => {}
				}
			}
			_ => {}
		}
		
		Ok(false)
	}
	
	/**
	 * Updates the context menu
	 * 
	 * @return Result<bool> - Whether menu needs redraw
	 */
	pub async fn update(&self) -> Result<bool> {
		// Context menu doesn't need regular updates
		Ok(false)
	}
} 