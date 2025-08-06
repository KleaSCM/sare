/**
 * Toolbar for Sare terminal
 * 
 * This module provides a toolbar widget with customizable
 * buttons and quick access to common functions.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: toolbar.rs
 * Description: Toolbar widget with customizable buttons
 */

use anyhow::Result;
use super::widgets::{Widget, WidgetRect, WidgetStyle, WidgetEvent, MouseButton};

/**
 * Toolbar button
 * 
 * ツールバーボタンです。
 * ツールバーの個別ボタンを
 * 管理します。
 */
pub struct ToolbarItem {
	/// Item ID
	pub id: String,
	/// Item text
	pub text: String,
	/// Item icon
	pub icon: Option<String>,
	/// Item enabled state
	pub enabled: bool,
	/// Item callback
	pub callback: Option<Box<dyn Fn() + Send + Sync>>,
}

/**
 * Toolbar widget
 * 
 * ツールバーウィジェットです。
 * カスタマイズ可能なボタンと
 * クイックアクセス機能を提供します。
 */
pub struct Toolbar {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Widget visibility
	visible: bool,
	/// Toolbar buttons
	buttons: Vec<ToolbarItem>,
	/// Selected button index
	selected_index: Option<usize>,
}

impl Toolbar {
	/**
	 * Creates a new toolbar
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @return Toolbar - New toolbar
	 */
	pub fn new(id: String, rect: WidgetRect) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			visible: true,
			buttons: Vec::new(),
			selected_index: None,
		}
	}
	
	/**
	 * Adds a button to the toolbar
	 * 
	 * @param button - Toolbar button
	 */
	pub fn add_button(&mut self, button: ToolbarItem) {
		self.buttons.push(button);
	}
	
	/**
	 * Removes a button from the toolbar
	 * 
	 * @param id - Button ID to remove
	 */
	pub fn remove_button(&mut self, id: &str) {
		self.buttons.retain(|button| button.id != id);
	}
	
	/**
	 * Gets a button by ID
	 * 
	 * @param id - Button ID
	 * @return Option<&ToolbarButton> - Button reference
	 */
	pub fn get_button(&self, id: &str) -> Option<&ToolbarItem> {
		self.buttons.iter().find(|button| button.id == id)
	}
	
	/**
	 * Gets a mutable button by ID
	 * 
	 * @param id - Button ID
	 * @return Option<&mut ToolbarButton> - Mutable button reference
	 */
	pub fn get_button_mut(&mut self, id: &str) -> Option<&mut ToolbarItem> {
		self.buttons.iter_mut().find(|button| button.id == id)
	}
	
	/**
	 * Renders the toolbar
	 * 
	 * @return String - Rendered toolbar
	 */
	fn render_toolbar(&self) -> String {
		let mut result = String::new();
		let width = self.rect.width as usize;
		
		// Add border if specified
		if self.style.border_style != super::widgets::BorderStyle::None {
			result.push_str(&self.render_border());
		}
		
		// Render buttons
		let mut current_pos = 0;
		for (index, button) in self.buttons.iter().enumerate() {
			if !button.enabled {
				continue;
			}
			
			let button_text = if let Some(icon) = &button.icon {
				format!("{} {}", icon, button.text)
			} else {
				button.text.clone()
			};
			
			// Add selection indicator
			let button_display = if Some(index) == self.selected_index {
				format!("[{}]", button_text)
			} else {
				button_text
			};
			
			if current_pos + button_display.len() + 1 < width {
				result.push_str(&button_display);
				result.push(' ');
				current_pos += button_display.len() + 1;
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
				border.push_str(&self.render_toolbar());
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
				border.push_str(&self.render_toolbar());
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
				border.push_str(&self.render_toolbar());
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
				border.push_str(&self.render_toolbar());
				border.push('│');
				
				border
			}
			super::widgets::BorderStyle::None => String::new(),
		}
	}
	
	/**
	 * Handles button click
	 * 
	 * @param index - Button index
	 * @return Result<bool> - Whether click was handled
	 */
	fn handle_button_click(&mut self, index: usize) -> Result<bool> {
		if index < self.buttons.len() {
			let button = &mut self.buttons[index];
			if button.enabled {
				// Execute callback if available
				if let Some(callback) = &button.callback {
					callback();
				}
				
				self.selected_index = Some(index);
				return Ok(true);
			}
		}
		
		Ok(false)
	}
}

impl Widget for Toolbar {
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
		
		Ok(self.render_toolbar())
	}
	
	fn handle_event(&mut self, event: WidgetEvent) -> Result<bool> {
		match event {
			WidgetEvent::Click { x, y, button: MouseButton::Left } => {
				// Calculate which button was clicked
				let button_width = self.rect.width as usize / self.buttons.len().max(1);
				let button_index = (x as usize / button_width).min(self.buttons.len().saturating_sub(1));
				
				return self.handle_button_click(button_index);
			}
			_ => Ok(false),
		}
	}
	
	fn update(&mut self) -> Result<bool> {
		// Toolbar doesn't need regular updates
		Ok(false)
	}
	
	fn is_visible(&self) -> bool {
		self.visible
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
	}
} 