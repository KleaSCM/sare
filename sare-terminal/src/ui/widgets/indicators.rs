/**
 * Indicators widget for Sare terminal
 * 
 * This module provides indicator widgets for status display
 * and monitoring including gauges, meters, and status lights.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: indicators.rs
 * Description: Indicator widgets for status display and monitoring
 */

use anyhow::Result;
use super::{Widget, WidgetRect, WidgetStyle, WidgetEvent};

/**
 * Indicator type
 * 
 * インジケータタイプです。
 * インジケータの種類を
 * 定義します。
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndicatorType {
	/// Gauge indicator
	Gauge,
	/// Meter indicator
	Meter,
	/// Status light
	StatusLight,
	/// Progress indicator
	Progress,
}

/**
 * Indicator widget
 * 
 * インジケータウィジェットです。
 * ステータス表示とモニタリングのための
 * インジケータを提供します。
 */
pub struct Indicator {
	/// Widget ID
	id: String,
	/// Widget position and size
	rect: WidgetRect,
	/// Widget style
	style: WidgetStyle,
	/// Widget visibility
	visible: bool,
	/// Indicator type
	indicator_type: IndicatorType,
	/// Current value (0.0 to 1.0)
	value: f32,
	/// Indicator label
	label: String,
	/// Indicator color
	color: u32,
	/// Show value
	show_value: bool,
	/// Show percentage
	show_percentage: bool,
}

impl Indicator {
	/**
	 * Creates a new indicator
	 * 
	 * @param id - Widget ID
	 * @param rect - Widget position and size
	 * @param indicator_type - Indicator type
	 * @return Indicator - New indicator
	 */
	pub fn new(id: String, rect: WidgetRect, indicator_type: IndicatorType) -> Self {
		Self {
			id,
			rect,
			style: WidgetStyle::default(),
			visible: true,
			indicator_type,
			value: 0.0,
			label: String::new(),
			color: 0x00FF00, // Green
			show_value: true,
			show_percentage: true,
		}
	}
	
	/**
	 * Sets the indicator value
	 * 
	 * @param value - Indicator value (0.0 to 1.0)
	 */
	pub fn set_value(&mut self, value: f32) {
		self.value = value.max(0.0).min(1.0);
	}
	
	/**
	 * Gets the indicator value
	 * 
	 * @return f32 - Indicator value
	 */
	pub fn get_value(&self) -> f32 {
		self.value
	}
	
	/**
	 * Sets the indicator label
	 * 
	 * @param label - Indicator label
	 */
	pub fn set_label(&mut self, label: String) {
		self.label = label;
	}
	
	/**
	 * Gets the indicator label
	 * 
	 * @return &str - Indicator label
	 */
	pub fn get_label(&self) -> &str {
		&self.label
	}
	
	/**
	 * Sets the indicator color
	 * 
	 * @param color - Indicator color
	 */
	pub fn set_color(&mut self, color: u32) {
		self.color = color;
	}
	
	/**
	 * Gets the indicator color
	 * 
	 * @return u32 - Indicator color
	 */
	pub fn get_color(&self) -> u32 {
		self.color
	}
	
	/**
	 * Renders the indicator
	 * 
	 * @return String - Rendered indicator
	 */
	fn render_indicator(&self) -> String {
		let mut result = String::new();
		
		// Add label if available
		if !self.label.is_empty() {
			result.push_str(&format!("{}: ", self.label));
		}
		
		// Render based on indicator type
		match self.indicator_type {
			IndicatorType::Gauge => result.push_str(&self.render_gauge()),
			IndicatorType::Meter => result.push_str(&self.render_meter()),
			IndicatorType::StatusLight => result.push_str(&self.render_status_light()),
			IndicatorType::Progress => result.push_str(&self.render_progress()),
		}
		
		// Add value if enabled
		if self.show_value {
			result.push_str(&format!(" ({:.1})", self.value));
		}
		
		// Add percentage if enabled
		if self.show_percentage {
			result.push_str(&format!(" {:.0}%", self.value * 100.0));
		}
		
		result
	}
	
	/**
	 * Renders a gauge indicator
	 * 
	 * @return String - Rendered gauge
	 */
	fn render_gauge(&self) -> String {
		let width = self.rect.width as usize;
		let filled_width = (self.value * width as f32) as usize;
		
		let mut result = String::new();
		result.push('[');
		
		for i in 0..width {
			if i < filled_width {
				result.push('█');
			} else {
				result.push('░');
			}
		}
		
		result.push(']');
		result
	}
	
	/**
	 * Renders a meter indicator
	 * 
	 * @return String - Rendered meter
	 */
	fn render_meter(&self) -> String {
		let width = self.rect.width as usize;
		let filled_width = (self.value * width as f32) as usize;
		
		let mut result = String::new();
		result.push('│');
		
		for i in 0..width {
			if i < filled_width {
				result.push('█');
			} else {
				result.push(' ');
			}
		}
		
		result.push('│');
		result
	}
	
	/**
	 * Renders a status light indicator
	 * 
	 * @return String - Rendered status light
	 */
	fn render_status_light(&self) -> String {
		let status_char = if self.value > 0.5 {
			"🟢" // Green circle
		} else if self.value > 0.25 {
			"🟡" // Yellow circle
		} else {
			"🔴" // Red circle
		};
		
		format!("{}", status_char)
	}
	
	/**
	 * Renders a progress indicator
	 * 
	 * @return String - Rendered progress
	 */
	fn render_progress(&self) -> String {
		let width = self.rect.width as usize;
		let filled_width = (self.value * width as f32) as usize;
		
		let mut result = String::new();
		
		for i in 0..width {
			if i < filled_width {
				result.push('█');
			} else {
				result.push('░');
			}
		}
		
		result
	}
}

impl Widget for Indicator {
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
		
		Ok(self.render_indicator())
	}
	
	fn handle_event(&mut self, _event: WidgetEvent) -> Result<bool> {
		// Indicators don't handle events
		Ok(false)
	}
	
	fn update(&mut self) -> Result<bool> {
		// Indicators don't need regular updates
		Ok(false)
	}
	
	fn is_visible(&self) -> bool {
		self.visible
	}
	
	fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
	}
} 