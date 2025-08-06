/**
 * Drag and drop for Sare terminal
 * 
 * This module provides drag and drop functionality including
 * file drag and drop and content drag and drop support.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: drag_drop.rs
 * Description: Drag and drop manager with file and content support
 */

use anyhow::Result;
use std::collections::HashMap;
use super::widgets::{WidgetEvent, MouseButton};

/**
 * Drag data types
 * 
 * ドラッグデータタイプです。
 * ドラッグ可能なデータの
 * 種類を定義します。
 */
#[derive(Debug, Clone)]
pub enum DragDataType {
	/// File data
	File { path: String },
	/// Text data
	Text { content: String },
	/// URL data
	Url { url: String },
	/// Custom data
	Custom { mime_type: String, data: Vec<u8> },
}

/**
 * Drag data
 * 
 * ドラッグデータです。
 * ドラッグ操作のデータを
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct DragData {
	/// Data type
	pub data_type: DragDataType,
	/// Data source
	pub source: String,
	/// Data description
	pub description: String,
}

/**
 * Drop target
 * 
 * ドロップターゲットです。
 * ドロップ可能な領域を
 * 定義します。
 */
#[derive(Debug, Clone)]
pub struct DropTarget {
	/// Target ID
	pub id: String,
	/// Target bounds
	pub x: u32,
	/// Target bounds
	pub y: u32,
	/// Target bounds
	pub width: u32,
	/// Target bounds
	pub height: u32,
	/// Target enabled state
	pub enabled: bool,
	/// Target callback
	pub callback: Option<Box<dyn Fn(DragData) + Send + Sync>>,
}

/**
 * Drag and drop manager
 * 
 * ドラッグアンドドロップマネージャーです。
 * ファイルドラッグアンドドロップと
 * コンテンツドラッグアンドドロップを提供します。
 */
pub struct DragDropManager {
	/// Active drag data
	active_drag: Option<DragData>,
	/// Drag position
	drag_x: u32,
	/// Drag position
	drag_y: u32,
	/// Drag active state
	drag_active: bool,
	/// Drop targets
	drop_targets: HashMap<String, DropTarget>,
	/// Drag callbacks
	drag_callbacks: HashMap<String, Box<dyn Fn(DragData) + Send + Sync>>,
}

impl DragDropManager {
	/**
	 * Creates a new drag and drop manager
	 * 
	 * @return DragDropManager - New drag and drop manager
	 */
	pub fn new() -> Self {
		Self {
			active_drag: None,
			drag_x: 0,
			drag_y: 0,
			drag_active: false,
			drop_targets: HashMap::new(),
			drag_callbacks: HashMap::new(),
		}
	}
	
	/**
	 * Initializes the drag and drop manager
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn initialize(&mut self) -> Result<()> {
		// Initialize default callbacks
		self.drag_callbacks.insert("file_drop".to_string(), Box::new(|data| {
			match &data.data_type {
				DragDataType::File { path } => {
					println!("File dropped: {}", path);
				}
				_ => {
					println!("Unknown data type dropped");
				}
			}
		}));
		
		self.drag_callbacks.insert("text_drop".to_string(), Box::new(|data| {
			match &data.data_type {
				DragDataType::Text { content } => {
					println!("Text dropped: {}", content);
				}
				_ => {
					println!("Unknown data type dropped");
				}
			}
		}));
		
		Ok(())
	}
	
	/**
	 * Starts a drag operation
	 * 
	 * @param data - Drag data
	 * @param x - X coordinate
	 * @param y - Y coordinate
	 * @return Result<()> - Success or error status
	 */
	pub async fn start_drag(&mut self, data: DragData, x: u32, y: u32) -> Result<()> {
		self.active_drag = Some(data);
		self.drag_x = x;
		self.drag_y = y;
		self.drag_active = true;
		
		Ok(())
	}
	
	/**
	 * Ends the drag operation
	 * 
	 * @return Result<()> - Success or error status
	 */
	pub async fn end_drag(&mut self) -> Result<()> {
		self.drag_active = false;
		self.active_drag = None;
		
		Ok(())
	}
	
	/**
	 * Updates drag position
	 * 
	 * @param x - New X coordinate
	 * @param y - New Y coordinate
	 */
	pub fn update_drag_position(&mut self, x: u32, y: u32) {
		self.drag_x = x;
		self.drag_y = y;
	}
	
	/**
	 * Adds a drop target
	 * 
	 * @param target - Drop target
	 */
	pub fn add_drop_target(&mut self, target: DropTarget) {
		self.drop_targets.insert(target.id.clone(), target);
	}
	
	/**
	 * Removes a drop target
	 * 
	 * @param id - Target ID to remove
	 */
	pub fn remove_drop_target(&mut self, id: &str) {
		self.drop_targets.remove(id);
	}
	
	/**
	 * Checks if drag is active
	 * 
	 * @return bool - Whether drag is active
	 */
	pub fn is_active(&self) -> bool {
		self.drag_active
	}
	
	/**
	 * Renders the drag and drop overlay
	 * 
	 * @return Result<String> - Rendered overlay content
	 */
	pub fn render(&self) -> Result<String> {
		if !self.drag_active {
			return Ok(String::new());
		}
		
		let mut result = String::new();
		
		// Position cursor at drag location
		result.push_str(&format!("\x1b[{};{}H", self.drag_y + 1, self.drag_x + 1));
		
		// Render drag indicator
		if let Some(drag_data) = &self.active_drag {
			match &drag_data.data_type {
				DragDataType::File { path } => {
					result.push_str("📁 "); // File icon
					result.push_str(&path);
				}
				DragDataType::Text { content } => {
					result.push_str("📝 "); // Text icon
					result.push_str(&content);
				}
				DragDataType::Url { url } => {
					result.push_str("🔗 "); // URL icon
					result.push_str(&url);
				}
				DragDataType::Custom { mime_type, data: _ } => {
					result.push_str("📦 "); // Custom icon
					result.push_str(&mime_type);
				}
			}
		}
		
		// Render drop targets
		for target in self.drop_targets.values() {
			if target.enabled {
				result.push_str(&format!("\x1b[{};{}H", target.y + 1, target.x + 1));
				result.push_str("┌");
				for _ in 0..target.width {
					result.push('─');
				}
				result.push_str("┐");
				result.push('\n');
				
				result.push_str(&format!("\x1b[{};{}H", target.y + 2, target.x + 1));
				result.push_str("│");
				result.push_str(&target.id);
				for _ in target.id.len()..target.width as usize {
					result.push(' ');
				}
				result.push_str("│");
				result.push('\n');
				
				result.push_str(&format!("\x1b[{};{}H", target.y + 3, target.x + 1));
				result.push_str("└");
				for _ in 0..target.width {
					result.push('─');
				}
				result.push_str("┘");
			}
		}
		
		Ok(result)
	}
	
	/**
	 * Handles drag and drop events
	 * 
	 * @param event - Widget event
	 * @return Result<bool> - Whether event was handled
	 */
	pub async fn handle_event(&self, event: &WidgetEvent) -> Result<bool> {
		match event {
			WidgetEvent::Click { x, y, button: MouseButton::Left } => {
				if self.drag_active {
					// Check if click is on a drop target
					for target in self.drop_targets.values() {
						if target.enabled &&
						   *x >= target.x && *x < target.x + target.width &&
						   *y >= target.y && *y < target.y + target.height {
							// Execute drop callback
							if let Some(callback) = &target.callback {
								if let Some(drag_data) = &self.active_drag {
									callback(drag_data.clone());
								}
							}
							
							return Ok(true);
						}
					}
					
					// Click outside drop targets, end drag
					return Ok(true);
				}
			}
			WidgetEvent::Hover { x, y } => {
				if self.drag_active {
					// Update drag position
					return Ok(true);
				}
			}
			_ => {}
		}
		
		Ok(false)
	}
	
	/**
	 * Updates the drag and drop manager
	 * 
	 * @return Result<bool> - Whether manager needs redraw
	 */
	pub async fn update(&self) -> Result<bool> {
		// Drag and drop manager doesn't need regular updates
		Ok(false)
	}
} 