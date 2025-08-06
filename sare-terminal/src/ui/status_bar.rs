/**
 * Status bar for Sare terminal
 * 
 * This module provides a status bar widget that displays
 * system information, process status, and other useful data.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: status_bar.rs
 * Description: Status bar widget with system information
 */

use anyhow::Result;
use std::time::Instant;
use super::widgets::{Widget, WidgetRect, WidgetStyle, WidgetEvent};

/**
 * Status bar widget
 * 
 * ステータスバーウィジェットです。
 * システム情報とステータスを
 * 表示します。
 */
pub struct StatusBar {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Widget visibility
	visible: bool,
	/// Status items
	status_items: Vec<StatusItem>,
	/// Last update time
	last_update: Instant,
}

/**
 * Status item
 * 
 * ステータスアイテムです。
 * 個別のステータス情報を
 * 管理します。
 */
#[derive(Debug, Clone)]
pub struct StatusItem {
	/// Item key
	pub key: String,
	/// Item value
	pub value: String,
	/// Item color
	pub color: u32,
	/// Item priority
	pub priority: u32,
}

impl StatusBar {
	/**
	 * Creates a new status bar
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @return StatusBar - New status bar
	 */
	pub fn new(id: String, rect: WidgetRect) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			visible: true,
			status_items: Vec::new(),
			last_update: Instant::now(),
		}
	}
	
	/**
	 * Adds a status item
	 * 
	 * @param key - Item key
	 * @param value - Item value
	 * @param color - Item color
	 * @param priority - Item priority
	 */
	pub fn add_status_item(&mut self, key: String, value: String, color: u32, priority: u32) {
		self.status_items.push(StatusItem {
			key,
			value,
			color,
			priority,
		});
		
		// Sort by priority
		self.status_items.sort_by(|a, b| b.priority.cmp(&a.priority));
	}
	
	/**
	 * Updates a status item
	 * 
	 * @param key - Item key
	 * @param value - New value
	 */
	pub fn update_status_item(&mut self, key: &str, value: String) {
		for item in &mut self.status_items {
			if item.key == key {
				item.value = value;
				break;
			}
		}
	}
	
	/**
	 * Removes a status item
	 * 
	 * @param key - Item key to remove
	 */
	pub fn remove_status_item(&mut self, key: &str) {
		self.status_items.retain(|item| item.key != key);
	}
	
	/**
	 * Renders the status bar
	 * 
	 * @return String - Rendered status bar
	 */
	fn render_status_bar(&self) -> String {
		let mut result = String::new();
		let width = self.rect.width as usize;
		
		// Add border if specified
		if self.style.border_style != super::widgets::BorderStyle::None {
			result.push_str(&self.render_border());
		}
		
		// Render status items
		let mut current_pos = 0;
		for item in &self.status_items {
			let item_text = format!("{}: {}", item.key, item.value);
			if current_pos + item_text.len() < width {
				result.push_str(&item_text);
				result.push(' ');
				current_pos += item_text.len() + 1;
			} else {
				break;
			}
		}
		
		// Pad with spaces
		while current_pos < width {
			result.push(' ');
			current_pos += 1;
		}
		
		result
	}
	
	/**
	 * Renders the border
	 * 
	 * @return String - Rendered border
	 */
	fn render_border(&self) -> String {
		match self.style.border_style {
			super::widgets::BorderStyle::Single => {
				let mut border = String::new();
				let width = self.rect.width as usize;
				
				// Top border
				border.push('┌');
				for _ in 0..width {
					border.push('─');
				}
				border.push('┐');
				border.push('\n');
				
				// Content with side borders
				border.push('│');
				border.push_str(&self.render_status_bar());
				border.push('│');
				
				border
			}
			super::widgets::BorderStyle::Double => {
				let mut border = String::new();
				let width = self.rect.width as usize;
				
				// Top border
				border.push('╔');
				for _ in 0..width {
					border.push('═');
				}
				border.push('╗');
				border.push('\n');
				
				// Content with side borders
				border.push('║');
				border.push_str(&self.render_status_bar());
				border.push('║');
				
				border
			}
			super::widgets::BorderStyle::Rounded => {
				let mut border = String::new();
				let width = self.rect.width as usize;
				
				// Top border
				border.push('╭');
				for _ in 0..width {
					border.push('─');
				}
				border.push('╮');
				border.push('\n');
				
				// Content with side borders
				border.push('│');
				border.push_str(&self.render_status_bar());
				border.push('│');
				
				border
			}
			super::widgets::BorderStyle::Dashed => {
				let mut border = String::new();
				let width = self.rect.width as usize;
				
				// Top border
				border.push('┌');
				for i in 0..width {
					if i % 2 == 0 {
						border.push('─');
					} else {
						border.push(' ');
					}
				}
				border.push('┐');
				border.push('\n');
				
				// Content with side borders
				border.push('│');
				border.push_str(&self.render_status_bar());
				border.push('│');
				
				border
			}
			super::widgets::BorderStyle::None => String::new(),
		}
	}
}

impl Widget for StatusBar {
	fn id(&self) -> &str {
		&self.id
	}
	
	fn rect(&self) -> WidgetRect {
		self.rect
	}
	
	fn set_rect(&mut self, rect: WidgetRect) {
		self.rect = rect;
	}
	
	fn style(&self) -> &WidgetStyle {
		&self.style
	}
	
	fn set_style(&mut self, style: WidgetStyle) {
		self.style = style;
	}
	
	fn render(&self) -> Result<String> {
		if !self.visible {
			return Ok(String::new());
		}
		
		Ok(self.render_status_bar())
	}
	
	fn handle_event(&mut self, _event: WidgetEvent) -> Result<bool> {
		// Status bar doesn't handle events
		Ok(false)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Update status items every second
		let now = Instant::now();
		if now.duration_since(self.last_update).as_secs() >= 1 {
			self.last_update = now;
			
			// Update system information
			self.update_system_info();
			
			return Ok(true);
		}
		
		Ok(false)
	}
	
	fn is_visible(&self) -> bool {
		self.visible
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
	}
}

impl StatusBar {
	/**
	 * Updates system information
	 * 
	 * Updates status items with current system information
	 */
	fn update_system_info(&mut self) {
		// Update CPU usage
		if let Ok(cpu_usage) = self.get_cpu_usage() {
			self.update_status_item("CPU", format!("{}%", cpu_usage));
		}
		
		// Update memory usage
		if let Ok(memory_usage) = self.get_memory_usage() {
			self.update_status_item("Memory", format!("{}%", memory_usage));
		}
		
		// Update disk usage
		if let Ok(disk_usage) = self.get_disk_usage() {
			self.update_status_item("Disk", format!("{}%", disk_usage));
		}
		
		// Update network status
		if let Ok(network_status) = self.get_network_status() {
			self.update_status_item("Network", network_status);
		}
		
		// Update time
		let time = chrono::Local::now().format("%H:%M:%S").to_string();
		self.update_status_item("Time", time);
	}
	
	/**
	 * Gets CPU usage
	 * 
	 * @return Result<u32> - CPU usage percentage
	 */
	fn get_cpu_usage(&self) -> Result<u32> {
		// Simple CPU usage estimation
		// In a real implementation, this would read from /proc/stat
		Ok(25) // Placeholder
	}
	
	/**
	 * Gets memory usage
	 * 
	 * @return Result<u32> - Memory usage percentage
	 */
	fn get_memory_usage(&self) -> Result<u32> {
		// Simple memory usage estimation
		// In a real implementation, this would read from /proc/meminfo
		Ok(45) // Placeholder
	}
	
	/**
	 * Gets disk usage
	 * 
	 * @return Result<u32> - Disk usage percentage
	 */
	fn get_disk_usage(&self) -> Result<u32> {
		// Simple disk usage estimation
		// In a real implementation, this would use statvfs
		Ok(60) // Placeholder
	}
	
	/**
	 * Gets network status
	 * 
	 * @return Result<String> - Network status string
	 */
	fn get_network_status(&self) -> Result<String> {
		// Simple network status
		// In a real implementation, this would check network interfaces
		Ok("Connected".to_string()) // Placeholder
	}
} 