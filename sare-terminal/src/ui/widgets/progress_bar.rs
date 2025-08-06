/**
 * Progress bar widget for Sare terminal
 * 
 * This module provides progress bar widgets with customizable
 * styles, animations, and real-time updates.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: progress_bar.rs
 * Description: Progress bar widget with animations
 */

use anyhow::Result;
use std::time::{Duration, Instant};
use super::{Widget, WidgetRect, WidgetStyle, WidgetEvent, BorderStyle, FontStyle};

/**
 * Progress bar style
 * 
 * プログレスバーのスタイルです。
 * プログレスバーの見た目を
 * カスタマイズします。
 */
#[derive(Debug, Clone)]
pub struct ProgressBarStyle {
	/// Progress bar character
	pub progress_char: char,
	/// Background character
	pub background_char: char,
	/// Progress color
	pub progress_color: u32,
	/// Background color
	pub background_color: u32,
	/// Show percentage
	pub show_percentage: bool,
	/// Show progress text
	pub show_text: bool,
	/// Animated progress
	pub animated: bool,
	/// Animation speed in milliseconds
	pub animation_speed: u64,
}

impl Default for ProgressBarStyle {
	fn default() -> Self {
		Self {
			progress_char: '█',
			background_char: '░',
			progress_color: 0x00FF00, // Green
			background_color: 0x666666, // Gray
			show_percentage: true,
			show_text: true,
			animated: true,
			animation_speed: 100,
		}
	}
}

/**
 * Progress bar widget
 * 
 * プログレスバーウィジェットです。
 * 進捗状況を視覚的に
 * 表示します。
 */
pub struct ProgressBar {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Progress bar style
	progress_style: ProgressBarStyle,
	/// Current progress (0.0 to 1.0)
	progress: f32,
	/// Progress text
	text: String,
	/// Widget visibility
	visible: bool,
	/// Last update time for animation
	last_update: Instant,
	/// Animation frame
	animation_frame: u32,
}

impl ProgressBar {
	/**
	 * Creates a new progress bar
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @return ProgressBar - New progress bar
	 */
	pub fn new(id: String, rect: WidgetRect) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			progress_style: ProgressBarStyle::default(),
			progress: 0.0,
			text: String::new(),
			visible: true,
			last_update: Instant::now(),
			animation_frame: 0,
		}
	}
	
	/**
	 * Sets the progress value
	 * 
	 * @param progress - Progress value (0.0 to 1.0)
	 */
	pub fn set_progress(&mut self, progress: f32) {
		self.progress = progress.max(0.0).min(1.0);
	}
	
	/**
	 * Gets the current progress value
	 * 
	 * @return f32 - Current progress value
	 */
	pub fn get_progress(&self) -> f32 {
		self.progress
	}
	
	/**
	 * Sets the progress text
	 * 
	 * @param text - Progress text
	 */
	pub fn set_text(&mut self, text: String) {
		self.text = text;
	}
	
	/**
	 * Gets the progress text
	 * 
	 * @return &str - Progress text
	 */
	pub fn get_text(&self) -> &str {
		&self.text
	}
	
	/**
	 * Sets the progress bar style
	 * 
	 * @param style - Progress bar style
	 */
	pub fn set_progress_style(&mut self, style: ProgressBarStyle) {
		self.progress_style = style;
	}
	
	/**
	 * Gets the progress bar style
	 * 
	 * @return &ProgressBarStyle - Progress bar style
	 */
	pub fn get_progress_style(&self) -> &ProgressBarStyle {
		&self.progress_style
	}
	
	/**
	 * Renders the progress bar
	 * 
	 * @return String - Rendered progress bar
	 */
	fn render_progress_bar(&self) -> String {
		let mut result = String::new();
		let width = self.rect.width as usize;
		let progress_width = (self.progress * width as f32) as usize;
		
		// Add border if specified
		if self.style.border_style != BorderStyle::None {
			result.push_str(&self.render_border());
		}
		
		// Render progress bar
		for i in 0..width {
			if i < progress_width {
				// Progress character
				result.push(self.progress_style.progress_char);
			} else {
				// Background character
				result.push(self.progress_style.background_char);
			}
		}
		
		// Add percentage if enabled
		if self.progress_style.show_percentage {
			let percentage = (self.progress * 100.0) as u32;
			result.push_str(&format!(" {}%", percentage));
		}
		
		// Add text if enabled and available
		if self.progress_style.show_text && !self.text.is_empty() {
			result.push_str(&format!(" {}", self.text));
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
			BorderStyle::Single => {
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
				border.push_str(&self.render_progress_bar());
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
			BorderStyle::Double => {
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
				border.push_str(&self.render_progress_bar());
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
			BorderStyle::Rounded => {
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
				border.push_str(&self.render_progress_bar());
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
			BorderStyle::Dashed => {
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
				border.push_str(&self.render_progress_bar());
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
			BorderStyle::None => String::new(),
		}
	}
	
	/**
	 * Updates animation frame
	 * 
	 * @return bool - Whether animation frame changed
	 */
	fn update_animation(&mut self) -> bool {
		if !self.progress_style.animated {
			return false;
		}
		
		let now = Instant::now();
		if now.duration_since(self.last_update).as_millis() >= self.progress_style.animation_speed as u128 {
			self.animation_frame = (self.animation_frame + 1) % 4;
			self.last_update = now;
			return true;
		}
		
		false
	}
}

impl Widget for ProgressBar {
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
		
		Ok(self.render_progress_bar())
	}
	
	fn handle_event(&mut self, _event: WidgetEvent) -> Result<bool> {
		// Progress bars don't handle events
		Ok(false)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Update animation
		self.update_animation();
		
		// Always return true to ensure redraw for animations
		Ok(true)
	}
	
	fn is_visible(&self) -> bool {
		self.visible
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
	}
}

/**
 * Animated progress bar
 * 
 * アニメーションプログレスバーです。
 * アニメーション付きの
 * プログレスバーを提供します。
 */
pub struct AnimatedProgressBar {
	/// Base progress bar
	progress_bar: ProgressBar,
	/// Animation characters
	animation_chars: Vec<char>,
	/// Current animation index
	animation_index: usize,
}

impl AnimatedProgressBar {
	/**
	 * Creates a new animated progress bar
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @return AnimatedProgressBar - New animated progress bar
	 */
	pub fn new(id: String, rect: WidgetRect) -> Self {
		let mut progress_bar = ProgressBar::new(id, rect);
		progress_bar.set_progress_style(ProgressBarStyle {
			progress_char: '█',
			background_char: '░',
			progress_color: 0x00FF00,
			background_color: 0x666666,
			show_percentage: true,
			show_text: true,
			animated: true,
			animation_speed: 100,
		});
		
		Self {
			progress_bar,
			animation_chars: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
			animation_index: 0,
		}
	}
	
	/**
	 * Sets the progress value
	 * 
	 * @param progress - Progress value (0.0 to 1.0)
	 */
	pub fn set_progress(&mut self, progress: f32) {
		self.progress_bar.set_progress(progress);
	}
	
	/**
	 * Gets the current progress value
	 * 
	 * @return f32 - Current progress value
	 */
	pub fn get_progress(&self) -> f32 {
		self.progress_bar.get_progress()
	}
	
	/**
	 * Sets the progress text
	 * 
	 * @param text - Progress text
	 */
	pub fn set_text(&mut self, text: String) {
		self.progress_bar.set_text(text);
	}
	
	/**
	 * Gets the progress text
	 * 
	 * @return &str - Progress text
	 */
	pub fn get_text(&self) -> &str {
		self.progress_bar.get_text()
	}
}

impl Widget for AnimatedProgressBar {
	fn id(&self) -> &str {
		self.progress_bar.id()
	}
	
	fn rect(&self) -> WidgetRect {
		self.progress_bar.rect()
	}
	
	fn set_rect(&mut self, rect: WidgetRect) {
		self.progress_bar.set_rect(rect);
	}
	
	fn style(&self) -> &WidgetStyle {
		self.progress_bar.style()
	}
	
	fn set_style(&mut self, style: WidgetStyle) {
		self.progress_bar.set_style(style);
	}
	
	fn render(&self) -> Result<String> {
		if !self.progress_bar.is_visible() {
			return Ok(String::new());
		}
		
		let mut result = self.progress_bar.render()?;
		
		// Add animated spinner if progress is not complete
		if self.progress_bar.get_progress() < 1.0 {
			let spinner = self.animation_chars[self.animation_index];
			result.push_str(&format!(" {}", spinner));
		}
		
		Ok(result)
	}
	
	fn handle_event(&mut self, event: WidgetEvent) -> Result<bool> {
		self.progress_bar.handle_event(event)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Update animation index
		self.animation_index = (self.animation_index + 1) % self.animation_chars.len();
		
		// Update base progress bar
		self.progress_bar.update()
	}
	
	fn is_visible(&self) -> bool {
		self.progress_bar.is_visible()
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.progress_bar.set_visible(visible);
	}
} 