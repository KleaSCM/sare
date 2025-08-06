/**
 * Buttons widget for Sare terminal
 * 
 * This module provides interactive button widgets and
 * UI components for the terminal emulator.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: buttons.rs
 * Description: Interactive button widgets and UI components
 */

use anyhow::Result;
use super::{Widget, WidgetRect, WidgetStyle, WidgetEvent, MouseButton};

/**
 * Button widget
 * 
 * ボタンウィジェットです。
 * インタラクティブなボタンを
 * 提供します。
 */
pub struct Button {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Widget visibility
	visible: bool,
	/// Button text
	text: String,
	/// Button icon
	icon: String,
	/// Button enabled state
	enabled: bool,
	/// Button pressed state
	pressed: bool,
	/// Button callback
	callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Button {
	/**
	 * Creates a new button
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @param text - Button text
	 * @return Button - New button
	 */
	pub fn new(id: String, rect: WidgetRect, text: String) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			visible: true,
			text,
			icon: String::new(),
			enabled: true,
			pressed: false,
			callback: None,
		}
	}
	
	/**
	 * Sets the button text
	 * 
	 * @param text - Button text
	 */
	pub fn set_text(&mut self, text: String) {
		self.text = text;
	}
	
	/**
	 * Gets the button text
	 * 
	 * @return &str - Button text
	 */
	pub fn get_text(&self) -> &str {
		&self.text
	}
	
	/**
	 * Sets the button icon
	 * 
	 * @param icon - Button icon
	 */
	pub fn set_icon(&mut self, icon: String) {
		self.icon = icon;
	}
	
	/**
	 * Gets the button icon
	 * 
	 * @return &str - Button icon
	 */
	pub fn get_icon(&self) -> &str {
		&self.icon
	}
	
	/**
	 * Sets the button enabled state
	 * 
	 * @param enabled - Whether button is enabled
	 */
	pub fn set_enabled(&mut self, enabled: bool) {
		self.enabled = enabled;
	}
	
	/**
	 * Gets the button enabled state
	 * 
	 * @return bool - Whether button is enabled
	 */
	pub fn is_enabled(&self) -> bool {
		self.enabled
	}
	
	/**
	 * Sets the button callback
	 * 
	 * @param callback - Button callback function
	 */
	pub fn set_callback<F>(&mut self, callback: F)
	where
		F: Fn() + Send + Sync + 'static,
	{
		self.callback = Some(Box::new(callback));
	}
	
	/**
	 * Renders the button
	 * 
	 * @return String - Rendered button
	 */
	fn render_button(&self) -> String {
		let mut result = String::new();
		let width = self.rect.width as usize;
		
		// Add border if specified
		if self.style.border_style != super::BorderStyle::None {
			result.push_str(&self.render_border());
		}
		
		// Render button content
		let button_text = if !self.icon.is_empty() {
			format!("{} {}", self.icon, self.text)
		} else {
			self.text.clone()
		};
		
		// Center the text
		let padding = if button_text.len() < width {
			(width - button_text.len()) / 2
		} else {
			0
		};
		
		// Add padding
		for _ in 0..padding {
			result.push(' ');
		}
		
		// Add button state indicator
		if self.pressed {
			result.push_str("[");
		}
		
		result.push_str(&button_text);
		
		if self.pressed {
			result.push_str("]");
		}
		
		// Add remaining padding
		let remaining = width.saturating_sub(padding + button_text.len());
		for _ in 0..remaining {
			result.push(' ');
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
			super::BorderStyle::Single => {
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
				border.push_str(&self.render_button());
				border.push('│');
				border.push('\n');
				
				// Bottom border
				border.push('└');
				for _ in 0..width {
					border.push('─');
				}
				border.push('┘');
				
				border
			}
			super::BorderStyle::Double => {
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
				border.push_str(&self.render_button());
				border.push('║');
				border.push('\n');
				
				// Bottom border
				border.push('╚');
				for _ in 0..width {
					border.push('═');
				}
				border.push('╝');
				
				border
			}
			super::BorderStyle::Rounded => {
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
				border.push_str(&self.render_button());
				border.push('│');
				border.push('\n');
				
				// Bottom border
				border.push('╰');
				for _ in 0..width {
					border.push('─');
				}
				border.push('╯');
				
				border
			}
			super::BorderStyle::Dashed => {
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
				border.push_str(&self.render_button());
				border.push('│');
				border.push('\n');
				
				// Bottom border
				border.push('└');
				for i in 0..width {
					if i % 2 == 0 {
						border.push('─');
					} else {
						border.push(' ');
					}
				}
				border.push('┘');
				
				border
			}
			super::BorderStyle::None => String::new(),
		}
	}
	
	/**
	 * Handles button click
	 * 
	 * @return Result<bool> - Whether click was handled
	 */
	fn handle_click(&mut self) -> Result<bool> {
		if self.enabled && self.visible {
			self.pressed = true;
			
			// Execute callback if available
			if let Some(callback) = &self.callback {
				callback();
			}
			
			return Ok(true);
		}
		
		Ok(false)
	}
}

impl Widget for Button {
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
		
		Ok(self.render_button())
	}
	
	fn handle_event(&mut self, event: WidgetEvent) -> Result<bool> {
		match event {
			WidgetEvent::Click { x, y, button: MouseButton::Left } => {
				// Check if click is within button bounds
				if x >= self.rect.x && x < self.rect.x + self.rect.width &&
				   y >= self.rect.y && y < self.rect.y + self.rect.height {
					return self.handle_click();
				}
			}
			_ => {}
		}
		
		Ok(false)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Reset pressed state after a short delay
		if self.pressed {
			self.pressed = false;
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