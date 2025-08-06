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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use libc::statvfs;
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
	
	/**
	 * Gets system load average
	 * 
	 * @return Result<String> - Load average string
	 */
	fn get_load_average(&self) -> Result<String> {
		// Read load average from /proc/loadavg
		let loadavg_content = std::fs::read_to_string("/proc/loadavg")?;
		let parts: Vec<&str> = loadavg_content.split_whitespace().collect();
		
		if parts.len() >= 3 {
			let load_1min = parts[0];
			let load_5min = parts[1];
			let load_15min = parts[2];
			Ok(format!("{}/{}/{}", load_1min, load_5min, load_15min))
		} else {
			Ok("0.00/0.00/0.00".to_string())
		}
	}
	
	/**
	 * Gets system uptime
	 * 
	 * @return Result<String> - Uptime string
	 */
	fn get_uptime(&self) -> Result<String> {
		// Read uptime from /proc/uptime
		let uptime_content = std::fs::read_to_string("/proc/uptime")?;
		let uptime_seconds: f64 = uptime_content
			.split_whitespace()
			.next()
			.and_then(|s| s.parse::<f64>().ok())
			.unwrap_or(0.0);
		
		let days = (uptime_seconds / 86400.0) as u64;
		let hours = ((uptime_seconds % 86400.0) / 3600.0) as u64;
		let minutes = ((uptime_seconds % 3600.0) / 60.0) as u64;
		
		if days > 0 {
			Ok(format!("{}d {}h {}m", days, hours, minutes))
		} else if hours > 0 {
			Ok(format!("{}h {}m", hours, minutes))
		} else {
			Ok(format!("{}m", minutes))
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
		
		// Update load average
		if let Ok(load_avg) = self.get_load_average() {
			self.update_status_item("Load", load_avg);
		}
		
		// Update uptime
		if let Ok(uptime) = self.get_uptime() {
			self.update_status_item("Uptime", uptime);
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
		// Read CPU stats from /proc/stat
		let stat_content = std::fs::read_to_string("/proc/stat")?;
		let first_line = stat_content.lines().next().ok_or_else(|| anyhow::anyhow!("No CPU stats found"))?;
		
		// Parse CPU time values
		let parts: Vec<u64> = first_line
			.split_whitespace()
			.skip(1) // Skip "cpu" identifier
			.filter_map(|s| s.parse::<u64>().ok())
			.collect();
		
		if parts.len() < 4 {
			return Ok(0);
		}
		
		// Calculate CPU usage
		let total = parts.iter().sum::<u64>();
		let idle = parts[3];
		let usage = if total > 0 {
			((total - idle) * 100) / total
		} else {
			0
		};
		
		Ok(usage as u32)
	}
	
	/**
	 * Gets memory usage
	 * 
	 * @return Result<u32> - Memory usage percentage
	 */
	fn get_memory_usage(&self) -> Result<u32> {
		// Read memory info from /proc/meminfo
		let meminfo_content = std::fs::read_to_string("/proc/meminfo")?;
		let mut total_mem = 0u64;
		let mut available_mem = 0u64;
		
		for line in meminfo_content.lines() {
			if line.starts_with("MemTotal:") {
				total_mem = line
					.split_whitespace()
					.nth(1)
					.and_then(|s| s.parse::<u64>().ok())
					.unwrap_or(0);
			} else if line.starts_with("MemAvailable:") {
				available_mem = line
					.split_whitespace()
					.nth(1)
					.and_then(|s| s.parse::<u64>().ok())
					.unwrap_or(0);
			}
		}
		
		if total_mem > 0 {
			let used_mem = total_mem - available_mem;
			let usage = (used_mem * 100) / total_mem;
			Ok(usage as u32)
		} else {
			Ok(0)
		}
	}
	
	/**
	 * Gets disk usage
	 * 
	 * @return Result<u32> - Disk usage percentage
	 */
	fn get_disk_usage(&self) -> Result<u32> {
		// Get disk usage for current directory
		let current_dir = std::env::current_dir()?;
		let mut statvfs_buf = std::mem::MaybeUninit::<libc::statvfs>::uninit();
		let result = unsafe { 
			statvfs(
				current_dir.to_string_lossy().as_ptr() as *const i8,
				statvfs_buf.as_mut_ptr()
			) 
		};
		
		if result == 0 {
			let statvfs = unsafe { statvfs_buf.assume_init() };
			let total_blocks = statvfs.f_blocks as u64;
			let free_blocks = statvfs.f_bavail as u64;
			let block_size = statvfs.f_frsize as u64;
			
			let total_bytes = total_blocks * block_size;
			let free_bytes = free_blocks * block_size;
			let used_bytes = total_bytes - free_bytes;
			
			let usage = if total_bytes > 0 {
				(used_bytes * 100) / total_bytes
			} else {
				0
			};
			Ok(usage as u32)
		} else {
			Ok(0)
		}
	}
	
	/**
	 * Gets network status
	 * 
	 * @return Result<String> - Network status string
	 */
	fn get_network_status(&self) -> Result<String> {
		// Check network interfaces
		let interfaces = std::fs::read_dir("/sys/class/net")?;
		let mut active_interfaces = Vec::new();
		
		for entry in interfaces {
			if let Ok(entry) = entry {
				let interface_name = entry.file_name().to_string_lossy().to_string();
				
				// Skip loopback and virtual interfaces
				if !interface_name.starts_with("lo") && 
				   !interface_name.starts_with("docker") &&
				   !interface_name.starts_with("veth") {
					
					// Check if interface is up
					if let Ok(operstate) = std::fs::read_to_string(format!("/sys/class/net/{}/operstate", interface_name)) {
						if operstate.trim() == "up" {
							active_interfaces.push(interface_name);
						}
					}
				}
			}
		}
		
		if active_interfaces.is_empty() {
			Ok("Disconnected".to_string())
		} else {
			Ok(format!("Connected ({})", active_interfaces.join(", ")))
		}
	}
} 